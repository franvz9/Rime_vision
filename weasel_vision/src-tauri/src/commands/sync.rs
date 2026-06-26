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

    let last_sync_time = settings.sync_dir.as_ref().and_then(|d| {
        let sync_dir = std::path::Path::new(d);
        let my_id_dir = sync_dir.join(&settings.installation_id);
        if my_id_dir.exists() {
            std::fs::metadata(&my_id_dir)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    chrono::DateTime::<chrono::Local>::from(t)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
        } else {
            None
        }
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
pub fn execute_sync() -> Result<SyncResult, String> {
    let settings = get_sync_settings()?;
    let sync_dir = match settings.sync_dir {
        Some(d) => d,
        None => return Err("Sync directory not configured".to_string()),
    };

    let sync_path = std::path::PathBuf::from(&sync_dir);
    if !sync_path.exists() {
        return Err(format!("Sync directory '{}' does not exist", sync_dir));
    }

    let cfg = RimeConfig::detect();
    let my_id_dir = sync_path.join(&settings.installation_id);
    std::fs::create_dir_all(&my_id_dir).map_err(|e| e.to_string())?;

    let mut uploaded = Vec::new();
    let mut downloaded = Vec::new();
    let mut errors = Vec::new();

    if settings.sync_config {
        for file_name in &[
            "default.custom.yaml",
            "squirrel.custom.yaml",
            "weasel.custom.yaml",
            "installation.yaml",
        ] {
            let src = cfg.user_dir.join(file_name);
            if src.exists() {
                let dst = my_id_dir.join(file_name);
                match std::fs::copy(&src, &dst) {
                    Ok(_) => uploaded.push(file_name.to_string()),
                    Err(e) => errors.push(format!("Upload {} failed: {}", file_name, e)),
                }
            }
        }
    }

    if settings.sync_user_dict {
        if let Ok(entries) = std::fs::read_dir(&cfg.user_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".userdb") {
                            let dict_id = name.replace(".userdb", "");
                            let snapshot_name = format!("{}.userdb.txt", dict_id);

                            let userdb_dir = cfg.user_dir.join(name);
                            if let Ok(db_files) = std::fs::read_dir(&userdb_dir) {
                                let mut found = false;
                                for db_file in db_files.flatten() {
                                    if found {
                                        break;
                                    }
                                    let db_name = db_file.file_name().to_string_lossy().to_string();
                                    if db_name.ends_with(".txt") {
                                        found = true;
                                        let src = db_file.path();
                                        let dst = my_id_dir.join(&snapshot_name);
                                        match std::fs::copy(&src, &dst) {
                                            Ok(_) => {
                                                uploaded.push(snapshot_name.clone());
                                            }
                                            Err(e) => {
                                                errors.push(format!(
                                                    "Upload {} failed: {}",
                                                    snapshot_name, e
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Download phase: collect snapshots from all other devices using unique per-device filenames
    // Key: dict_id, Value: Vec<(device_id, local_staging_path)>
    let mut remote_snapshots: std::collections::HashMap<String, Vec<(String, PathBuf)>> =
        std::collections::HashMap::new();
    let staging_dir = cfg.user_dir.join("user_dictionaries").join(".sync_staging");
    if let Ok(entries) = std::fs::read_dir(&sync_path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let other_id = entry.file_name().to_string_lossy().to_string();
                if other_id == settings.installation_id {
                    continue;
                }

                if let Ok(other_files) = std::fs::read_dir(entry.path()) {
                    for other_file in other_files.flatten() {
                        let name = other_file.file_name().to_string_lossy().to_string();
                        if name.ends_with(".userdb.txt") {
                            let dict_id = name.replace(".userdb.txt", "");
                            let userdb_dir = cfg.user_dir.join(format!("{}.userdb", dict_id));
                            if userdb_dir.exists() {
                                std::fs::create_dir_all(&staging_dir).ok();
                                // Use per-device filename to avoid overwrite
                                let staging_name = format!("{}_{}", other_id, name);
                                let dst = staging_dir.join(&staging_name);
                                match std::fs::copy(other_file.path(), &dst) {
                                    Ok(_) => {
                                        downloaded.push(format!("{}/{}", other_id, name));
                                        remote_snapshots
                                            .entry(dict_id)
                                            .or_default()
                                            .push((other_id.clone(), dst));
                                    }
                                    Err(e) => {
                                        errors.push(format!(
                                            "Download {}/{} failed: {}",
                                            other_id, name, e
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Merge phase: for each dict_id, merge all remote snapshots together, then merge into local
    for (dict_id, device_snapshots) in &remote_snapshots {
        if let Err(e) = merge_all_remotes_into_userdb(dict_id, device_snapshots, &cfg) {
            errors.push(format!("Merge {} failed: {}", dict_id, e));
        }
    }

    // Cleanup staging directory
    std::fs::remove_dir_all(&staging_dir).ok();

    Ok(SyncResult {
        success: errors.is_empty(),
        uploaded,
        downloaded,
        errors,
    })
}

/// Merge all remote device snapshots for a given dict_id into the local user dictionary.
/// First merges all remote snapshots together, then merges with the local snapshot.
/// If no local snapshot exists, the merged remote data becomes the initial local snapshot.
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
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let rand_part = t % 10000;
    format!("id-{:x}-{:04x}", t / 10000, rand_part)
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
