use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::rime::config::{self, RimeConfig};
use crate::rime::schema::RimeSchema;

#[derive(Serialize, Deserialize)]
pub struct SchemaListData {
    pub schemas: Vec<RimeSchema>,
    pub current_schema: String,
}

#[tauri::command]
pub fn get_schemas() -> Result<SchemaListData, String> {
    let cfg = RimeConfig::detect();
    
    // Get current schema from user.yaml (var/previously_selected_schema)
    let user_value = cfg
        .load_yaml(&cfg.user_yaml_path())
        .unwrap_or_else(|e| {
            eprintln!("Warning: failed to load user.yaml: {}, using empty config", e);
            Value::Mapping(serde_yaml::Mapping::new())
        });
    
    // Try to get current schema from user.yaml (var/previously_selected_schema)
    let current_schema_from_user = user_value
        .get(Value::String("var".into()))
        .and_then(|v| v.as_mapping())
        .and_then(|m| m.get(Value::String("previously_selected_schema".into())))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let dict_value = cfg
        .load_effective(&cfg.default_path(), &cfg.default_custom_path())
        .map_err(|e| e.to_string())?;
    let dict = config::value_as_mapping(&dict_value);

    let mut schemas = Vec::new();
    let schema_list = config::get_sequence(dict, "schema_list");
    for item in schema_list {
        if let Some(schema_id) = item.get(Value::String("schema".into())) {
            if let Some(id) = schema_id.as_str() {
                schemas.push(RimeSchema::new(id.to_string()));
            }
        }
    }

    // Fallback: if user.yaml has no previously_selected_schema, use first schema in list
    let current_schema = current_schema_from_user
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            schemas.first().map(|s| s.schema_id.clone()).unwrap_or_default()
        });

    Ok(SchemaListData {
        schemas,
        current_schema,
    })
}

#[tauri::command]
pub fn save_schemas(schemas: Vec<RimeSchema>) -> Result<(), String> {
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
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_schema(file_path: String) -> Result<(), String> {
    // Validate file path
    super::dict::validate_import_path(&file_path)?;
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    if !path.is_file() {
        return Err("Path is not a file".to_string());
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "yaml" && ext != "yml" {
        return Err("Invalid file extension: only .yaml and .yml are supported".to_string());
    }
    
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Parse YAML
    let yaml_value: Value = serde_yaml::from_str(&content)
        .map_err(|e| format!("Invalid YAML format: {}", e))?;
    
    // Check for schema_id
    if let Some(schema_id) = yaml_value.get(Value::String("schema_id".into())) {
        if let Some(id) = schema_id.as_str() {
            // Validate schema_id to prevent path traversal and injection
            if id.contains("..") || id.contains('/') || id.contains('\\') || id.contains('\0') {
                return Err("Invalid schema_id: contains invalid characters".to_string());
            }
            if id.is_empty() || id.len() > 64 {
                return Err("Invalid schema_id: length must be 1-64 characters".to_string());
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
            }).map_err(|e| e.to_string())?;
            
            // Copy the schema file to user directory so Rime can use it
            let dest_file = cfg.user_dir.join(format!("{}.schema.yaml", id));
            // Don't overwrite if already exists
            if !dest_file.exists() {
                std::fs::copy(&path, &dest_file)
                    .map_err(|e| format!("Failed to copy schema file: {}", e))?;
            }
            
            return Ok(());
        }
    }
    
    Err("No schema_id found in file".to_string())
}
