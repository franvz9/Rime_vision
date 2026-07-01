use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::rime::config::{self, RimeConfig};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralSettings {
    pub page_size: i64,
    pub enable_encoder: bool,
    pub enable_sentence: bool,
    pub enable_user_dict: bool,
    pub encode_commit_history: bool,
    pub switcher_caption: String,
    pub switcher_hotkeys: String,
    pub switcher_fold_options: bool,
    pub switcher_abbreviate_options: bool,
    pub caps_lock_action: String,
    pub shift_left_action: String,
    pub shift_right_action: String,
    pub good_old_caps_lock: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            page_size: 6,
            enable_encoder: true,
            enable_sentence: true,
            enable_user_dict: true,
            encode_commit_history: true,
            switcher_caption: "〔方案切换〕".to_string(),
            switcher_hotkeys: "Control+grave,Control+Shift+grave".to_string(),
            switcher_fold_options: true,
            switcher_abbreviate_options: true,
            caps_lock_action: "commit_code".to_string(),
            shift_left_action: "commit_code".to_string(),
            shift_right_action: "inline_ascii".to_string(),
            good_old_caps_lock: true,
        }
    }
}

#[tauri::command]
pub fn get_general_settings() -> Result<GeneralSettings, String> {
    let cfg = RimeConfig::detect();
    let dict_value = cfg
        .load_effective(&cfg.default_path(), &cfg.default_custom_path())
        .map_err(|e| e.to_string())?;
    let dict = config::value_as_mapping(&dict_value);

    let mut settings = GeneralSettings::default();

    if let Some(menu) = dict.get(Value::String("menu".into())) {
        if let Some(menu_map) = menu.as_mapping() {
            settings.page_size = config::get_i64(menu_map, "page_size").unwrap_or(6);
        }
    }

    if let Some(translator) = dict.get(Value::String("translator".into())) {
        if let Some(t_map) = translator.as_mapping() {
            settings.enable_encoder = config::get_bool(t_map, "enable_encoder").unwrap_or(true);
            settings.enable_sentence = config::get_bool(t_map, "enable_sentence").unwrap_or(true);
            settings.enable_user_dict = config::get_bool(t_map, "enable_user_dict").unwrap_or(true);
            settings.encode_commit_history =
                config::get_bool(t_map, "encode_commit_history").unwrap_or(true);
        }
    }

    if let Some(switcher) = dict.get(Value::String("switcher".into())) {
        if let Some(s_map) = switcher.as_mapping() {
            settings.switcher_caption =
                config::get_string(s_map, "caption").unwrap_or_else(|| "〔方案切换〕".into());
            let hotkeys = config::get_sequence(s_map, "hotkeys");
            settings.switcher_hotkeys = hotkeys
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            settings.switcher_fold_options = config::get_bool(s_map, "fold_options").unwrap_or(true);
            settings.switcher_abbreviate_options =
                config::get_bool(s_map, "abbreviate_options").unwrap_or(true);
        }
    }

    if let Some(composer) = dict.get(Value::String("ascii_composer".into())) {
        if let Some(c_map) = composer.as_mapping() {
            settings.good_old_caps_lock =
                config::get_bool(c_map, "good_old_caps_lock").unwrap_or(true);
            let switch_key = config::get_mapping(c_map, "switch_key");
            settings.caps_lock_action = config::get_string(switch_key, "Caps_Lock")
                .unwrap_or_else(|| "commit_code".into());
            settings.shift_left_action = config::get_string(switch_key, "Shift_L")
                .unwrap_or_else(|| "commit_code".into());
            settings.shift_right_action = config::get_string(switch_key, "Shift_R")
                .unwrap_or_else(|| "inline_ascii".into());
        }
    }

    Ok(settings)
}

#[tauri::command]
pub fn save_general_settings(settings: GeneralSettings) -> Result<(), String> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.default_custom_path(), |patch| {
        let menu = patch
            .entry(Value::String("menu".into()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()))
            .as_mapping_mut()
            .ok_or_else(|| anyhow::anyhow!("menu is not a mapping"))?;
        menu.insert(
            Value::String("page_size".into()),
            Value::Number(settings.page_size.into()),
        );

        let translator = patch
            .entry(Value::String("translator".into()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()))
            .as_mapping_mut()
            .ok_or_else(|| anyhow::anyhow!("translator is not a mapping"))?;
        translator.insert(
            Value::String("enable_encoder".into()),
            Value::Bool(settings.enable_encoder),
        );
        translator.insert(
            Value::String("enable_sentence".into()),
            Value::Bool(settings.enable_sentence),
        );
        translator.insert(
            Value::String("enable_user_dict".into()),
            Value::Bool(settings.enable_user_dict),
        );
        translator.insert(
            Value::String("encode_commit_history".into()),
            Value::Bool(settings.encode_commit_history),
        );

        let hotkeys: Vec<Value> = settings
            .switcher_hotkeys
            .split(',')
            .map(|s| Value::String(s.trim().to_string()))
            .collect();
        let switcher = patch
            .entry(Value::String("switcher".into()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()))
            .as_mapping_mut()
            .ok_or_else(|| anyhow::anyhow!("switcher is not a mapping"))?;
        switcher.insert(
            Value::String("caption".into()),
            Value::String(settings.switcher_caption),
        );
        switcher.insert(
            Value::String("hotkeys".into()),
            Value::Sequence(hotkeys),
        );
        switcher.insert(
            Value::String("fold_options".into()),
            Value::Bool(settings.switcher_fold_options),
        );
        switcher.insert(
            Value::String("abbreviate_options".into()),
            Value::Bool(settings.switcher_abbreviate_options),
        );

        let ascii_composer = patch
            .entry(Value::String("ascii_composer".into()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()))
            .as_mapping_mut()
            .ok_or_else(|| anyhow::anyhow!("ascii_composer is not a mapping"))?;
        ascii_composer.insert(
            Value::String("good_old_caps_lock".into()),
            Value::Bool(settings.good_old_caps_lock),
        );

        let switch_key = ascii_composer
            .entry(Value::String("switch_key".into()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()))
            .as_mapping_mut()
            .ok_or_else(|| anyhow::anyhow!("switch_key is not a mapping"))?;
        switch_key.insert(
            Value::String("Caps_Lock".into()),
            Value::String(settings.caps_lock_action),
        );
        switch_key.insert(
            Value::String("Shift_L".into()),
            Value::String(settings.shift_left_action),
        );
        switch_key.insert(
            Value::String("Shift_R".into()),
            Value::String(settings.shift_right_action),
        );

        Ok(())
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_rime_user_dir() -> Result<String, String> {
    let cfg = RimeConfig::detect();
    Ok(cfg.user_dir.to_string_lossy().to_string())
}

#[derive(Serialize)]
pub struct ConfigFileInfo {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub is_main: bool,
}

#[tauri::command]
pub fn get_config_files() -> Result<Vec<ConfigFileInfo>, String> {
    let cfg = RimeConfig::detect();
    let files = vec![
        ConfigFileInfo {
            name: cfg.style_file.clone(),
            path: cfg.style_path().to_string_lossy().to_string(),
            exists: cfg.style_path().exists(),
            is_main: true,
        },
        ConfigFileInfo {
            name: cfg.style_custom.clone(),
            path: cfg.style_custom_path().to_string_lossy().to_string(),
            exists: cfg.style_custom_path().exists(),
            is_main: false,
        },
        ConfigFileInfo {
            name: cfg.default_yaml.clone(),
            path: cfg.default_path().to_string_lossy().to_string(),
            exists: cfg.default_path().exists(),
            is_main: true,
        },
        ConfigFileInfo {
            name: cfg.default_custom.clone(),
            path: cfg.default_custom_path().to_string_lossy().to_string(),
            exists: cfg.default_custom_path().exists(),
            is_main: false,
        },
    ];
    Ok(files)
}

#[tauri::command]
pub fn sync() -> Result<(), String> {
    crate::rime::deployer::sync().map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_dir(path: String) -> Result<(), String> {
    // Validate path to prevent command injection and path traversal
    if path.is_empty() {
        return Err("路径不能为空".to_string());
    }
    // Reject paths starting with '-' (could be interpreted as command flags)
    if path.starts_with('-') {
        return Err("无效的路径格式".to_string());
    }
    // Block path traversal and shell metacharacters
    if path.contains("..") {
        return Err("路径包含无效字符".to_string());
    }
    // Validate path contains only safe characters (no shell metacharacters)
    // Note: parentheses are valid in macOS paths, so they are allowed
    if path.contains('\0') || path.contains('\n') || path.contains('\r') || path.contains('\t')
        || path.contains(';') || path.contains('|') || path.contains('&')
        || path.contains('$') || path.contains('`')
        || path.contains('<') || path.contains('>')
    {
        return Err("路径包含无效字符".to_string());
    }
    
    let path_obj = std::path::Path::new(&path);
    if !path_obj.exists() {
        return Err(format!("目录不存在: {}", path));
    }
    if !path_obj.is_dir() {
        return Err("路径不是目录".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_rime_dir() -> Result<(), String> {
    let cfg = RimeConfig::detect();
    let dir = cfg.user_dir.to_string_lossy().to_string();
    open_dir(dir)
}

#[tauri::command]
pub fn reset_config() -> Result<(), String> {
    let cfg = RimeConfig::detect();
    let style_custom = cfg.style_custom_path();
    let default_custom = cfg.default_custom_path();

    // Backup before deleting, so user can recover
    if style_custom.exists() || default_custom.exists() {
        if let Err(e) = super::backup::create_deploy_backup(false) {
            eprintln!("Warning: backup before reset failed: {}", e);
        }
    }

    if style_custom.exists() {
        std::fs::remove_file(&style_custom).map_err(|e| e.to_string())?;
    }
    if default_custom.exists() {
        std::fs::remove_file(&default_custom).map_err(|e| e.to_string())?;
    }

    // Trigger deploy so Rime picks up the changes
    if let Err(e) = crate::rime::deployer::deploy() {
        eprintln!("Warning: deploy after reset failed: {}", e);
    }

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct PendingDelete {
    pub delete_type: String, // "theme", "schema", "model"
    pub identifier: String,  // theme name, schema file name, or model filename
}

#[tauri::command]
pub fn deploy(pending_deletes: Option<Vec<PendingDelete>>) -> Result<(), String> {
    // Determine if any model deletion is planned
    let has_model_delete = pending_deletes.as_ref()
        .map_or(false, |deletes| deletes.iter().any(|d| d.delete_type == "model"));

    // Backup before deploy: include models only if model deletion is planned
    if let Err(e) = super::backup::create_deploy_backup(has_model_delete) {
        eprintln!("Warning: deploy backup failed: {}", e);
    }

    // Execute pending deletes after backup
    if let Some(deletes) = pending_deletes {
        let cfg = RimeConfig::detect();
        for delete in &deletes {
            // Validate identifier to prevent path traversal
            if delete.identifier.contains("..") || delete.identifier.contains('/') || delete.identifier.contains('\\') {
                eprintln!("Warning: skipping delete with invalid identifier: {}", delete.identifier);
                continue;
            }
            match delete.delete_type.as_str() {
                "theme" => {
                    // Remove theme from squirrel.custom.yaml
                    let _ = super::style::delete_color_scheme(delete.identifier.clone());
                }
                "schema" => {
                    // Delete schema file from user directory
                    let file_path = cfg.user_dir.join(&delete.identifier);
                    if file_path.exists() {
                        if let Err(e) = std::fs::remove_file(&file_path) {
                            eprintln!("Warning: failed to delete schema {}: {}", delete.identifier, e);
                        }
                    }
                    // Also remove from schema_list in default.custom.yaml
                    let _ = remove_schema_from_list(&cfg, &delete.identifier);
                }
                "model" => {
                    // Delete grammar model file from user directory
                    let file_path = cfg.user_dir.join(&delete.identifier);
                    if file_path.exists() {
                        if let Err(e) = std::fs::remove_file(&file_path) {
                            eprintln!("Warning: failed to delete model {}: {}", delete.identifier, e);
                        }
                    }
                }
                _ => {
                    eprintln!("Warning: unknown delete type: {}", delete.delete_type);
                }
            }
        }
    }

    crate::rime::deployer::deploy().map_err(|e| e.to_string())
}

fn remove_schema_from_list(cfg: &RimeConfig, schema_filename: &str) -> Result<(), String> {
    // Extract schema_id from filename (e.g., "my_schema.schema.yaml" -> "my_schema")
    let schema_id = schema_filename
        .strip_suffix(".schema.yaml")
        .or_else(|| schema_filename.strip_suffix(".schema.yml"))
        .unwrap_or(schema_filename);

    cfg.save_patch(&cfg.default_custom_path(), |patch| {
        if let Some(schema_list) = patch.get_mut(Value::String("schema_list".into())) {
            if let Some(list) = schema_list.as_sequence_mut() {
                list.retain(|item| {
                    if let Some(map) = item.as_mapping() {
                        let id = map
                            .get(Value::String("schema".into()))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        id != schema_id
                    } else {
                        true
                    }
                });
            }
        }
        Ok(())
    }).map_err(|e| e.to_string())
}
