use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use ts_rs::TS;

use crate::rime::config::{self, RimeConfig};
use crate::rime::error::RimeError;
use crate::rime::schema::RimeSchema;

#[derive(Serialize, Deserialize, TS)]
pub struct SchemaListData {
    pub schemas: Vec<RimeSchema>,
    pub current_schema: String,
}

#[tauri::command]
pub fn get_schemas() -> Result<SchemaListData, RimeError> {
    let cfg = RimeConfig::detect();

    // Get current schema from user.yaml (var/previously_selected_schema)
    let user_value = cfg.load_yaml(&cfg.user_yaml_path()).unwrap_or_else(|e| {
        eprintln!(
            "Warning: failed to load user.yaml: {}, using empty config",
            e
        );
        Value::Mapping(serde_yaml::Mapping::new())
    });

    // Try to get current schema from user.yaml (var/previously_selected_schema)
    let current_schema_from_user = user_value
        .get(Value::String("var".into()))
        .and_then(|v| v.as_mapping())
        .and_then(|m| m.get(Value::String("previously_selected_schema".into())))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let dict_value = cfg.load_effective(&cfg.default_path(), &cfg.default_custom_path())?;
    let dict = config::value_as_mapping(&dict_value);

    let mut schemas = Vec::new();
    let schema_list = config::get_sequence(dict, "schema_list");
    for item in schema_list {
        if let Some(schema_id) = item.get(Value::String("schema".into())) {
            if let Some(id) = schema_id.as_str() {
                schemas.push(load_schema_meta(&cfg, id.to_string()));
            }
        }
    }

    // Fallback: if user.yaml has no previously_selected_schema, use first schema in list
    let current_schema = current_schema_from_user
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            schemas
                .first()
                .map(|s| s.schema_id.clone())
                .unwrap_or_default()
        });

    Ok(SchemaListData {
        schemas,
        current_schema,
    })
}

#[tauri::command]
pub fn save_schemas(schemas: Vec<RimeSchema>) -> Result<(), RimeError> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.default_custom_path(), |patch| {
        let schema_list: Vec<Value> = schemas
            .iter()
            .filter(|s| s.enabled)
            .map(|s| {
                let mut item = serde_yaml::Mapping::new();
                item.insert(
                    Value::String("schema".into()),
                    Value::String(s.schema_id.clone()),
                );
                Value::Mapping(item)
            })
            .collect();

        patch.insert(
            Value::String("schema_list".into()),
            Value::Sequence(schema_list),
        );

        Ok(())
    })?;
    Ok(())
}

#[tauri::command]
pub fn import_schema(file_path: String) -> Result<(), RimeError> {
    // Validate file path
    super::dict::validate_import_path(&file_path)?;
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Err(RimeError::NotFound(format!(
            "File not found: {}",
            file_path
        )));
    }
    if !path.is_file() {
        return Err(RimeError::InvalidPath("Path is not a file".into()));
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "yaml" && ext != "yml" {
        return Err(RimeError::Config(
            "Invalid file extension: only .yaml and .yml are supported".into(),
        ));
    }

    // Read file with size limit
    let metadata = std::fs::metadata(&file_path)?;
    if metadata.len() > 10 * 1024 * 1024 {
        return Err(RimeError::Io(format!(
            "File too large: {} bytes",
            metadata.len()
        )));
    }
    let content = std::fs::read_to_string(&file_path)?;

    // Parse YAML
    let yaml_value: Value = serde_yaml::from_str(&content)?;

    // Check for schema_id
    if let Some(schema_id) = yaml_value.get(Value::String("schema_id".into())) {
        if let Some(id) = schema_id.as_str() {
            // Validate schema_id to prevent path traversal and injection
            if id.contains("..") || id.contains('/') || id.contains('\\') || id.contains('\0') {
                return Err(RimeError::InvalidPath(
                    "Invalid schema_id: contains invalid characters".into(),
                ));
            }
            if id.is_empty() || id.len() > 64 {
                return Err(RimeError::Config(
                    "Invalid schema_id: length must be 1-64 characters".into(),
                ));
            }
            // Add to schema_list in default.custom.yaml
            let cfg = RimeConfig::detect();
            cfg.save_patch(&cfg.default_custom_path(), |patch| {
                let existing_list = patch
                    .entry(Value::String("schema_list".into()))
                    .or_insert_with(|| Value::Sequence(Vec::new()));

                if let Some(existing_seq) = existing_list.as_sequence_mut() {
                    // Check if already exists
                    let exists = existing_seq.iter().any(|item| {
                        if let Some(map) = item.as_mapping() {
                            map.get(Value::String("schema".into()))
                                .and_then(|v| v.as_str())
                                == Some(id)
                        } else {
                            false
                        }
                    });

                    if !exists {
                        let mut new_item = serde_yaml::Mapping::new();
                        new_item.insert(
                            Value::String("schema".into()),
                            Value::String(id.to_string()),
                        );
                        existing_seq.push(Value::Mapping(new_item));
                    }
                }

                Ok(())
            })?;

            // Copy the schema file to user directory so Rime can use it
            let dest_file = cfg.user_dir.join(format!("{}.schema.yaml", id));
            // Don't overwrite if already exists
            if !dest_file.exists() {
                std::fs::copy(path, &dest_file)?;
            }

            return Ok(());
        }
    }

    Err(RimeError::NotFound("No schema_id found in file".into()))
}

/// 读取方案 schema 文件的元信息（名称、版本、作者、简介、依赖等），
/// 用于方案信息预览。读取失败时返回仅含 schema_id 的默认结构。
fn load_schema_meta(cfg: &RimeConfig, schema_id: String) -> RimeSchema {
    let mut schema = RimeSchema::new(schema_id.clone());
    let Some(path) = cfg.schema_path(&schema_id) else {
        return schema;
    };
    let Ok(value) = cfg.load_yaml(&path) else {
        return schema;
    };
    let dict = config::value_as_mapping(&value);
    let meta = config::get_mapping(dict, "schema");
    schema.name = config::get_string(meta, "name").unwrap_or_default();
    schema.version = config::get_string(meta, "version").unwrap_or_default();
    schema.author = config::get_string(meta, "author").unwrap_or_default();
    schema.description = config::get_string(meta, "description").unwrap_or_default();
    if let Some(deps) = meta
        .get(Value::String("dependencies".into()))
        .and_then(|v| v.as_sequence())
    {
        schema.dependencies = deps
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }
    schema
}

/// 切换"使用中"的方案：写入 user.yaml 的 var/previously_selected_schema。
/// 保留 user.yaml 中其它内容（如 schema_access_time、last_build_time 等）。
#[tauri::command]
pub fn set_current_schema(schema_id: String) -> Result<(), RimeError> {
    let cfg = RimeConfig::detect();
    set_current_schema_with_cfg(&cfg, &schema_id)
}

fn set_current_schema_with_cfg(cfg: &RimeConfig, schema_id: &str) -> Result<(), RimeError> {
    // 复用统一校验（拒绝路径遍历/换行/空串等）
    if !RimeConfig::valid_schema_id(schema_id) {
        return Err(RimeError::InvalidPath("Invalid schema_id".into()));
    }
    let user_path = cfg.user_yaml_path();

    // 读取现有 user.yaml（若存在），更新 var.previously_selected_schema
    let mut user_value = cfg.load_yaml(&user_path)?;
    let dict = user_value
        .as_mapping_mut()
        .ok_or_else(|| RimeError::Config("user.yaml is not a mapping".into()))?;
    let var = dict
        .entry(Value::String("var".into()))
        .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()));
    let var_map = var
        .as_mapping_mut()
        .ok_or_else(|| RimeError::Config("user.yaml var is not a mapping".into()))?;
    var_map.insert(
        Value::String("previously_selected_schema".into()),
        Value::String(schema_id.to_string()),
    );

    let content = serde_yaml::to_string(&user_value)?;
    crate::rime::backup::write_if_changed(&content, &user_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_cfg() -> (tempfile::TempDir, RimeConfig) {
        let tmp = tempdir().unwrap();
        let cfg = RimeConfig {
            user_dir: tmp.path().to_path_buf(),
            build_dir: None,
            style_file: "squirrel.yaml".into(),
            style_custom: "squirrel.custom.yaml".into(),
            default_yaml: "default.yaml".into(),
            default_custom: "default.custom.yaml".into(),
        };
        (tmp, cfg)
    }

    #[test]
    fn test_load_schema_meta_reads_fields() {
        let (_tmp, cfg) = test_cfg();
        std::fs::write(
            cfg.user_dir.join("demo.schema.yaml"),
            "schema:\n  schema_id: demo\n  name: 演示方案\n  version: '1.0'\n  author: tester\n  description: |\n    这是一个演示方案。\n  dependencies:\n    - radical_pinyin\n    - melt_eng\n",
        )
        .unwrap();
        let meta = load_schema_meta(&cfg, "demo".to_string());
        assert_eq!(meta.name, "演示方案");
        assert_eq!(meta.version, "1.0");
        assert_eq!(meta.author, "tester");
        assert!(meta.description.contains("演示方案"));
        assert_eq!(meta.dependencies.len(), 2);
        assert!(meta.dependencies.contains(&"melt_eng".to_string()));
    }

    #[test]
    fn test_load_schema_meta_missing_falls_back() {
        let (_tmp, cfg) = test_cfg();
        let meta = load_schema_meta(&cfg, "nonexistent".to_string());
        assert_eq!(meta.schema_id, "nonexistent");
        assert!(meta.name.is_empty());
    }

    #[test]
    fn test_set_current_schema_writes_user_yaml() {
        let (_tmp, cfg) = test_cfg();
        // 预置 user.yaml
        std::fs::write(
            cfg.user_dir.join("user.yaml"),
            "var:\n  last_build_time: 123\n  previously_selected_schema: old_schema\n",
        )
        .unwrap();
        set_current_schema_with_cfg(&cfg, "new_schema").unwrap();

        let user = cfg.load_yaml(&cfg.user_yaml_path()).unwrap();
        let dict = user.as_mapping().unwrap();
        let var = dict
            .get(Value::String("var".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        // 新方案已写入
        assert_eq!(
            var.get(Value::String("previously_selected_schema".into()))
                .and_then(|v| v.as_str()),
            Some("new_schema")
        );
        // 其它字段保留
        assert!(var.contains_key(Value::String("last_build_time".into())));
    }

    #[test]
    fn test_set_current_schema_rejects_invalid() {
        let (_tmp, cfg) = test_cfg();
        std::fs::write(
            cfg.user_dir.join("user.yaml"),
            "var:\n  previously_selected_schema: old\n",
        )
        .unwrap();
        let err = set_current_schema_with_cfg(&cfg, "../evil").unwrap_err();
        assert!(matches!(err, RimeError::InvalidPath(_)));
    }
}
