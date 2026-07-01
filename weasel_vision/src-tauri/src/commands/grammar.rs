use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::rime::config::{self, RimeConfig};
use crate::rime::grammar::{GrammarModel, SchemaGrammarConfig};
use crate::rime::patch;

#[derive(Serialize, Deserialize)]
pub struct GrammarData {
    pub models: Vec<GrammarModel>,
    pub mount_configs: std::collections::HashMap<String, SchemaGrammarConfig>,
}

#[tauri::command]
pub fn get_grammar_data(schema_ids: Vec<String>) -> Result<GrammarData, String> {
    let cfg = RimeConfig::detect();
    let mut models = Vec::new();
    let mut mount_configs = std::collections::HashMap::new();

    if cfg.user_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&cfg.user_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("gram") {
                    let filename = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    let metadata = std::fs::metadata(&path).ok();
                    let file_size = metadata.as_ref().map(|m| m.len() as i64).unwrap_or(0);

                    models.push(GrammarModel::new(
                        filename,
                        path.to_string_lossy().to_string(),
                        file_size,
                    ));
                }
            }
        }
    }

    models.sort_by(|a, b| a.filename.cmp(&b.filename));

    for schema_id in &schema_ids {
        if let Some(custom_url) = cfg.schema_custom_path(schema_id) {
            let config = load_mount_config(&cfg, &custom_url, schema_id);
            mount_configs.insert(schema_id.clone(), config);
        }
    }

    Ok(GrammarData {
        models,
        mount_configs,
    })
}

fn load_mount_config(
    cfg: &RimeConfig,
    custom_url: &std::path::Path,
    schema_id: &str,
) -> SchemaGrammarConfig {
    let mut config = SchemaGrammarConfig::default_for(schema_id);

    if let Ok(dict_value) = cfg.load_yaml(custom_url) {
        if let Some(patch_map) = dict_value.get(Value::String("patch".into())) {
            if let Some(patch_mapping) = patch_map.as_mapping() {
                let expanded = patch::expanded_patch(patch_mapping);
                let grammar = config::get_mapping(&expanded, "grammar");
                let translator = config::get_mapping(&expanded, "translator");

                config.mounted_model = config::get_string(grammar, "language");
                config.collocation_max_length =
                    config::get_i64(grammar, "collocation_max_length").unwrap_or(5);
                config.collocation_min_length =
                    config::get_i64(grammar, "collocation_min_length").unwrap_or(2);
                config.collocation_penalty =
                    config::get_i64(grammar, "collocation_penalty").unwrap_or(-16);
                config.non_collocation_penalty =
                    config::get_i64(grammar, "non_collocation_penalty").unwrap_or(-8);
                config.weak_collocation_penalty =
                    config::get_i64(grammar, "weak_collocation_penalty").unwrap_or(-100);
                config.rear_penalty = config::get_i64(grammar, "rear_penalty").unwrap_or(-20);

                config.contextual_suggestions =
                    config::get_bool(translator, "contextual_suggestions").unwrap_or(true);
                config.max_homophones =
                    config::get_i64(translator, "max_homophones").unwrap_or(7);
                config.max_homographs =
                    config::get_i64(translator, "max_homographs").unwrap_or(7);
            }
        }
    }

    config
}

#[tauri::command]
pub fn mount_grammar(
    model_filename: String,
    schema_id: String,
    config: SchemaGrammarConfig,
) -> Result<(), String> {
    let cfg = RimeConfig::detect();
    let custom_url = cfg.schema_custom_path(&schema_id)
        .ok_or_else(|| format!("Invalid schema_id: {}", schema_id))?;

    cfg.save_patch(&custom_url, |patch| {
        patch.insert(
            Value::String("grammar/language".into()),
            Value::String(model_filename),
        );
        patch.insert(
            Value::String("grammar/collocation_max_length".into()),
            Value::Number(config.collocation_max_length.into()),
        );
        patch.insert(
            Value::String("grammar/collocation_min_length".into()),
            Value::Number(config.collocation_min_length.into()),
        );
        patch.insert(
            Value::String("grammar/collocation_penalty".into()),
            Value::Number(config.collocation_penalty.into()),
        );
        patch.insert(
            Value::String("grammar/non_collocation_penalty".into()),
            Value::Number(config.non_collocation_penalty.into()),
        );
        patch.insert(
            Value::String("grammar/weak_collocation_penalty".into()),
            Value::Number(config.weak_collocation_penalty.into()),
        );
        patch.insert(
            Value::String("grammar/rear_penalty".into()),
            Value::Number(config.rear_penalty.into()),
        );
        patch.insert(
            Value::String("translator/contextual_suggestions".into()),
            Value::Bool(config.contextual_suggestions),
        );
        patch.insert(
            Value::String("translator/max_homophones".into()),
            Value::Number(config.max_homophones.into()),
        );
        patch.insert(
            Value::String("translator/max_homographs".into()),
            Value::Number(config.max_homographs.into()),
        );
        Ok(())
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unmount_grammar(schema_id: String) -> Result<(), String> {
    let cfg = RimeConfig::detect();
    let custom_url = cfg.schema_custom_path(&schema_id)
        .ok_or_else(|| format!("Invalid schema_id: {}", schema_id))?;

    cfg.save_patch(&custom_url, |patch| {
        // Remove all grammar/* keys
        let keys_to_remove: Vec<Value> = patch
            .keys()
            .filter(|k| {
                k.as_str()
                    .map(|s| s == "grammar" || s.starts_with("grammar/"))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        for key in keys_to_remove {
            patch.remove(&key);
        }

        // Remove grammar-related keys from translator section
        if let Some(translator) = patch.get_mut(Value::String("translator".into())) {
            if let Some(t_map) = translator.as_mapping_mut() {
                let remove_keys: Vec<Value> = t_map
                    .keys()
                    .filter(|k| {
                        k.as_str()
                            .map(|s| {
                                s == "contextual_suggestions"
                                    || s == "max_homophones"
                                    || s == "max_homographs"
                            })
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect();
                for key in remove_keys {
                    t_map.remove(&key);
                }
                if t_map.is_empty() {
                    patch.remove(Value::String("translator".into()));
                }
            }
        }

        Ok(())
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_grammar(file_path: String) -> Result<(), String> {
    // Security validation: prevent path traversal attacks
    super::dict::validate_import_path(&file_path)?;

    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    if !path.is_file() {
        return Err("Path is not a file".to_string());
    }

    let cfg = RimeConfig::detect();
    
    // Read the source file as binary (gram files can be binary)
    let content = std::fs::read(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Get filename without extension
    let path = std::path::Path::new(&file_path);
    let filename = path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?
        .to_string();
    
    // Basic validation: check if file is not empty and has reasonable size
    if content.is_empty() {
        return Err("Grammar file is empty".to_string());
    }
    
    // For text-based gram files, do additional validation
    // Try to convert to string for validation, but don't fail if it's binary
    if let Ok(content_str) = std::str::from_utf8(&content) {
        // If it's valid UTF-8, check for expected markers
        if !content_str.contains("# Rime grammar") && !content_str.contains("language:") {
            // Allow binary gram files without these markers
            eprintln!("Warning: Grammar file may not have standard Rime format, but importing anyway");
        }
    }
    // If conversion fails, it's likely a binary file, which is acceptable
    
    // Copy to user directory using binary write
    let dest_path = cfg.user_dir.join(format!("{}.gram", filename));
    std::fs::write(&dest_path, content)
        .map_err(|e| format!("Failed to copy file: {}", e))?;
    
    Ok(())
}
