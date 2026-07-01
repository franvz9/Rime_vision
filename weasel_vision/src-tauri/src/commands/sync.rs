use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::rime::config::{self, RimeConfig};

type Mapping = serde_yaml::Mapping;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub sync_dir: Option<String>,
    pub installation_id: String,
    pub sync_user_dict: bool,
    pub sync_config: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub configured: bool,
    pub last_sync_time: Option<String>,
    pub sync_dir_exists: bool,
    pub current_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedDevice {
    pub id: String,
    pub last_sync: String,
    pub synced_dicts: Vec<String>,
    pub synced_configs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub uploaded: Vec<String>,
    pub downloaded: Vec<String>,
    pub errors: Vec<String>,
}

fn installation_yaml_path() -> PathBuf {
    let cfg = RimeConfig::detect();
    cfg.user_dir.join("installation.yaml")
}

fn load_installation_yaml() -> Result<Mapping, String> {
    let path = installation_yaml_path();
    if !path.exists() {
        return Ok(Mapping::new());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let value: Value = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;
    Ok(value
        .as_mapping()
        .cloned()
        .unwrap_or_default())
}

fn save_installation_yaml(dict: &Mapping) -> Result<(), String> {
    let path = installation_yaml_path();
    let content = serde_yaml::to_string(dict).map_err(|e| e.to_string())?;
    crate::rime::backup::write_if_changed(&content, &path)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_sync_settings() -> Result<SyncSettings, String> {
    let dict = load_installation_yaml()?;
    let sync_dir = config::get_string(&dict, "sync_dir");
    let installation_id = config::get_string(&dict, "installation_id").unwrap_or_else(uuid_simple);

    Ok(SyncSettings {
        sync_dir,
        installation_id,
        sync_user_dict: true,
        sync_config: true,
    })
}

#[tauri::command]
pub fn save_sync_settings(settings: SyncSettings) -> Result<(), String> {
    validate_installation_id(&settings.installation_id)?;
    let mut dict = load_installation_yaml()?;

    if let Some(ref dir) = settings.sync_dir {
        dict.insert(
            Value::String("sync_dir".into()),
            Value::String(dir.clone()),
        );
    } else {
        dict.remove(Value::String("sync_dir".into()));
    }

    dict.insert(
        Value::String("installation_id".into()),
        Value::String(settings.installation_id),
    );

    save_installation_yaml(&dict)
}

#[tauri::command]
pub fn get_sync_status() -> Result<SyncStatus, String> {
    let settings = get_sync_settings()?;
    let sync_dir_exists = settings
        .sync_dir
        .as_ref()
        .map(|d| std::path::Path::new(d).exists())
        .unwrap_or(false);

    // Check for a recorded last_sync_time first (written by execute_sync)
    let last_sync_time = settings.sync_dir.as_ref().and_then(|d| {
        let marker = std::path::Path::new(d).join(&settings.installation_id).join(".last_sync");
        std::fs::read_to_string(&marker).ok().and_then(|s| {
            let s = s.trim();
            if s.is_empty() { None } else { Some(s.to_string()) }
        })
    });

    Ok(SyncStatus {
        configured: settings.sync_dir.is_some(),
        last_sync_time,
        sync_dir_exists,
        current_id: settings.installation_id,
    })
}

#[tauri::command]
pub fn list_synced_devices() -> Result<Vec<SyncedDevice>, String> {
    let settings = get_sync_settings()?;
    let mut devices = Vec::new();

    let sync_dir = match settings.sync_dir {
        Some(d) => std::path::PathBuf::from(d),
        None => return Ok(devices),
    };

    if !sync_dir.exists() {
        return Ok(devices);
    }

    if let Ok(entries) = std::fs::read_dir(&sync_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let id = entry.file_name().to_string_lossy().to_string();
                if id == settings.installation_id {
                    continue;
                }

                let mut synced_dicts = Vec::new();
                let mut synced_configs = Vec::new();

                if let Ok(files) = std::fs::read_dir(entry.path()) {
                    for file in files.flatten() {
                        let name = file.file_name().to_string_lossy().to_string();
                        if name.ends_with(".userdb.txt") {
                            synced_dicts.push(name.replace(".userdb.txt", ""));
                        } else if name.ends_with(".yaml") {
                            synced_configs.push(name);
                        }
                    }
                }

                let last_sync = std::fs::metadata(entry.path())
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        chrono::DateTime::<chrono::Local>::from(t)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_default();

                devices.push(SyncedDevice {
                    id,
                    last_sync,
                    synced_dicts,
                    synced_configs,
                });
            }
        }
    }

    devices.sort_by(|a, b| b.last_sync.cmp(&a.last_sync));
    Ok(devices)
}

/// Minimum interval between sync operations (3 seconds) to prevent
/// rapid repeated syncs that could overwhelm the input method process.
static LAST_SYNC: Mutex<Option<Instant>> = Mutex::new(None);
const SYNC_THROTTLE_SECS: u64 = 3;

#[tauri::command]
pub async fn execute_sync() -> Result<SyncResult, String> {
    // Rate limiting: prevent rapid repeated syncs
    {
        let mut last = LAST_SYNC.lock().map_err(|e| e.to_string())?;
        if let Some(t) = *last {
            if t.elapsed().as_secs() < SYNC_THROTTLE_SECS {
                return Err(format!("操作过于频繁，请 {} 秒后再试", SYNC_THROTTLE_SECS));
            }
        }
        *last = Some(Instant::now());
    }
    
    let settings = get_sync_settings()?;
    
    // Check if sync directory is configured
    if settings.sync_dir.is_none() {
        return Err("请先在「同步设置」中配置同步目录".to_string());
    }
    
    // Trigger Rime native sync by sending kill -HUP to Squirrel
    let process_found = crate::rime::deployer::sync().map_err(|e| format!("触发同步失败: {}", e))?;

    if !process_found {
        return Ok(SyncResult {
            success: false,
            uploaded: vec![],
            downloaded: vec![],
            errors: vec!["输入法进程未运行，无法执行同步。请确保 Rime 输入法正在运行。".to_string()],
        });
    }

    // Wait for Squirrel to complete the sync (minimum 2s)
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Record the sync timestamp so get_sync_status can read it
    if let Some(ref sync_dir) = settings.sync_dir {
        let device_dir = std::path::Path::new(sync_dir).join(&settings.installation_id);
        if let Err(e) = std::fs::create_dir_all(&device_dir) {
            eprintln!("Warning: failed to create sync device dir: {}", e);
        } else {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            if let Err(e) = std::fs::write(device_dir.join(".last_sync"), &now) {
                eprintln!("Warning: failed to write .last_sync: {}", e);
            } else {
                eprintln!("Recorded sync time: {} at {:?}", now, device_dir.join(".last_sync"));
            }
        }
    }

    // Report success - the actual sync was done by Rime/Squirrel
    Ok(SyncResult {
        success: true,
        uploaded: vec!["Rime 原生同步已触发".to_string()],
        downloaded: vec![],
        errors: vec![],
    })
}

/// Merge all remote device snapshots for a given dict_id into the local user dictionary.
fn uuid_simple() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn validate_installation_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("Invalid installation ID".to_string());
    }
    if id.chars().any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_') {
        return Err("Installation ID contains invalid characters".to_string());
    }
    Ok(())
}
