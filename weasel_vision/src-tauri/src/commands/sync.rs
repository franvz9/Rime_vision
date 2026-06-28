use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::rime::config::RimeConfig;

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

fn get_string(dict: &Mapping, key: &str) -> Option<String> {
    dict.get(Value::String(key.into()))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[tauri::command]
pub fn get_sync_settings() -> Result<SyncSettings, String> {
    let dict = load_installation_yaml()?;
    let sync_dir = get_string(&dict, "sync_dir");
    let installation_id = get_string(&dict, "installation_id").unwrap_or_else(uuid_simple);

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

#[tauri::command]
pub async fn execute_sync() -> Result<SyncResult, String> {
    let settings = get_sync_settings()?;
    
    // Check if sync directory is configured
    if settings.sync_dir.is_none() {
        return Err("请先在「同步设置」中配置同步目录".to_string());
    }
    
    // Trigger Rime native sync by sending kill -HUP to Squirrel
    // This will:
    // 1. Export LevelDB data to *.userdb/*.txt files
    // 2. Copy those .txt files to sync/<device_id>/*.userdb.txt
    let process_found = crate::rime::deployer::sync().map_err(|e| format!("触发同步失败: {}", e))?;
    
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
    
    let message = if process_found {
        "Rime 原生同步已触发".to_string()
    } else {
        "输入法进程未运行，已记录同步时间，下次启动时将自动同步".to_string()
    };
    
    // Report success - the actual sync was done by Rime/Squirrel
    Ok(SyncResult {
        success: true,
        uploaded: vec![message],
        downloaded: vec![],
        errors: if process_found { vec![] } else { vec!["Squirrel/Weasel 未运行".to_string()] },
    })
}

/// Merge all remote device snapshots for a given dict_id into the local user dictionary.
/// First merges all remote snapshots together, then merges with the local snapshot.
/// If no local snapshot exists, the merged remote data becomes the initial local snapshot.
// Reserved for future sync functionality
/// Merge all remote userdb snapshots into the local LevelDB database
/// 
/// This function is reserved for future multi-device sync merge functionality.
/// Currently not used but kept for planned sync features.
#[allow(dead_code)]
fn merge_all_remotes_into_userdb(
    dict_id: &str,
    device_snapshots: &[(String, PathBuf)], // (device_id, staging_path)
    cfg: &RimeConfig,
) -> Result<(), String> {
    // Merge all remote snapshots into one combined set
    // key = (word, code), value = (full_line, frequency) — preserve full line for max freq entry
    let mut combined: std::collections::HashMap<(String, String), (String, i64)> =
        std::collections::HashMap::new();

    for (_device_id, path) in device_snapshots {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        for line in content.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let word = parts[0].to_string();
                let code = parts[1].to_string();
                let freq = parts[2].parse::<i64>().unwrap_or(0);
                let entry = combined.entry((word, code));
                match entry {
                    std::collections::hash_map::Entry::Occupied(mut e) => {
                        if freq > e.get().1 {
                            e.insert((line.to_string(), freq));
                        }
                    }
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert((line.to_string(), freq));
                    }
                }
            }
        }
    }

    // Try to find existing local snapshot; if none, use merged remote as initial
    let local_snapshot = match find_local_snapshot(dict_id, cfg) {
        Ok(path) => path,
        Err(_) => {
            // No local snapshot — write merged remote data as initial snapshot
            let snapshots_dir = cfg.user_dir.join("user_dictionaries");
            std::fs::create_dir_all(&snapshots_dir).map_err(|e| e.to_string())?;
            let target = snapshots_dir.join(format!("{}.userdb.txt", dict_id));
            let mut lines: Vec<String> = combined.into_values().map(|(line, _)| line).collect();
            lines.sort();
            let output = lines.join("\n") + "\n";
            std::fs::write(&target, output).map_err(|e| e.to_string())?;
            return Ok(());
        }
    };

    // Merge remote combined with local snapshot
    let local_content = std::fs::read_to_string(&local_snapshot).map_err(|e| e.to_string())?;

    // Parse local entries, preserving full lines
    for line in local_content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let word = parts[0].to_string();
            let code = parts[1].to_string();
            let freq = parts[2].parse::<i64>().unwrap_or(0);
            let entry = combined.entry((word, code));
            match entry {
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    if freq > e.get().1 {
                        e.insert((line.to_string(), freq));
                    }
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert((line.to_string(), freq));
                }
            }
        }
    }

    // Write merged result preserving full original lines
    let mut lines: Vec<String> = combined.into_values().map(|(line, _)| line).collect();
    lines.sort();
    let output = lines.join("\n") + "\n";
    std::fs::write(&local_snapshot, output).map_err(|e| e.to_string())?;
    Ok(())
}

/// Find the most recent local snapshot for a dict.
/// Returns the newest matching file by modification time.
fn find_local_snapshot(dict_id: &str, cfg: &RimeConfig) -> Result<PathBuf, String> {
    let mut candidates: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

    // Check user_dictionaries/ first
    let snapshots_dir = cfg.user_dir.join("user_dictionaries");
    if snapshots_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&format!("{}.", dict_id)) && name.ends_with(".txt") {
                    let modified = std::fs::metadata(entry.path())
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    candidates.push((entry.path(), modified));
                }
            }
        }
    }

    // Check userdb dir
    let userdb_dir = cfg.user_dir.join(format!("{}.userdb", dict_id));
    if let Ok(entries) = std::fs::read_dir(&userdb_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".txt") {
                let modified = std::fs::metadata(entry.path())
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                candidates.push((entry.path(), modified));
            }
        }
    }

    candidates
        .into_iter()
        .max_by_key(|(_, t)| *t)
        .map(|(p, _)| p)
        .ok_or_else(|| format!("No local snapshot found for dict '{}'", dict_id))
}

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
