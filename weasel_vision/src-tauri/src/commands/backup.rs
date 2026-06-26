use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::rime::config::RimeConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub id: String,
    pub created_at: String,
    pub backup_type: String,
    pub file_count: usize,
    pub total_size: i64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupDetail {
    pub info: BackupInfo,
    pub files: Vec<BackupFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFile {
    pub name: String,
    pub size: i64,
    pub modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file_name: String,
    pub current: Option<String>,
    pub backup: String,
}

fn backups_dir() -> PathBuf {
    let cfg = RimeConfig::detect();
    cfg.user_dir.join("backups")
}

fn auto_backups_dir() -> PathBuf {
    backups_dir().join("auto")
}

fn manual_backups_dir() -> PathBuf {
    backups_dir().join("manual")
}

fn deploy_backups_dir() -> PathBuf {
    backups_dir().join("deploy")
}

fn config_files_to_backup() -> Vec<&'static str> {
    let mut files = vec![
        "default.custom.yaml",
        "installation.yaml",
        "user.yaml",
    ];
    if cfg!(target_os = "macos") {
        files.insert(0, "squirrel.custom.yaml");
    } else if cfg!(target_os = "windows") {
        files.insert(0, "weasel.custom.yaml");
    }
    files
}

fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
}

fn timestamp_iso() -> String {
    chrono::Local::now().to_rfc3339()
}

#[tauri::command]
pub fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let mut all_backups = Vec::new();

    for (backup_type, dir) in [
        ("manual".to_string(), manual_backups_dir()),
        ("deploy".to_string(), deploy_backups_dir()),
    ] {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let manifest_path = entry.path().join("manifest.json");
                    if manifest_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                            if let Ok(info) = serde_json::from_str::<BackupInfo>(&content) {
                                all_backups.push(info);
                            }
                        }
                    } else {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let meta = entry.metadata().ok();
                        all_backups.push(BackupInfo {
                            id: name.clone(),
                            created_at: meta
                                .as_ref()
                                .and_then(|m| m.modified().ok())
                                .map(|t| {
                                    chrono::DateTime::<chrono::Local>::from(t)
                                        .format("%Y-%m-%d %H:%M:%S")
                                        .to_string()
                                })
                                .unwrap_or_default(),
                            backup_type: backup_type.clone(),
                            file_count: count_files(&entry.path()),
                            total_size: dir_size(&entry.path()),
                            note: None,
                        });
                    }
                }
            }
        }
    }

    all_backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(all_backups)
}

#[tauri::command]
pub fn get_backup_detail(backup_id: String) -> Result<BackupDetail, String> {
    let dir = find_backup_dir(&backup_id)?;

    let manifest_path = dir.join("manifest.json");
    let info = if manifest_path.exists() {
        let content = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        serde_json::from_str::<BackupInfo>(&content).map_err(|e| e.to_string())?
    } else {
        BackupInfo {
            id: backup_id.clone(),
            created_at: String::new(),
            backup_type: "unknown".to_string(),
            file_count: count_files(&dir),
            total_size: dir_size(&dir),
            note: None,
        }
    };

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) != Some("json") {
                let meta = std::fs::metadata(&path).ok();
                files.push(BackupFile {
                    name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    size: meta.as_ref().map(|m| m.len() as i64).unwrap_or(0),
                    modified: meta
                        .as_ref()
                        .and_then(|m| m.modified().ok())
                        .map(|t| {
                            chrono::DateTime::<chrono::Local>::from(t)
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string()
                        })
                        .unwrap_or_default(),
                });
            }
        }
    }

    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(BackupDetail { info, files })
}

#[tauri::command]
pub fn create_backup(note: Option<String>) -> Result<BackupInfo, String> {
    let cfg = RimeConfig::detect();
    let ts = timestamp();
    let backup_dir = manual_backups_dir().join(&ts);
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    let mut file_count = 0;
    let mut total_size: i64 = 0;

    for file_name in config_files_to_backup() {
        let src = cfg.user_dir.join(file_name);
        if src.exists() {
            let dst = backup_dir.join(file_name);
            std::fs::copy(&src, &dst).map_err(|e| e.to_string())?;
            let meta = std::fs::metadata(&dst).ok();
            file_count += 1;
            total_size += meta.map(|m| m.len() as i64).unwrap_or(0);
        }
    }

    let info = BackupInfo {
        id: ts,
        created_at: timestamp_iso(),
        backup_type: "manual".to_string(),
        file_count,
        total_size,
        note,
    };

    let manifest = serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?;
    std::fs::write(backup_dir.join("manifest.json"), manifest)
        .map_err(|e| e.to_string())?;

    Ok(info)
}

#[tauri::command]
pub fn restore_backup(backup_id: String, restore_files: Vec<String>) -> Result<(), String> {
    let cfg = RimeConfig::detect();
    let dir = find_backup_dir(&backup_id)?;

    let files = if restore_files.is_empty() {
        config_files_to_backup()
            .into_iter()
            .map(String::from)
            .collect()
    } else {
        restore_files
    };

    // Phase 1: Validate and stage all files into a temp directory first.
    // This ensures we don't partially overwrite files if any source is missing or unreadable.
    let temp_dir = cfg.user_dir.join(".restore_temp");
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let mut files_to_restore: Vec<(String, PathBuf)> = Vec::new(); // (file_name, staged_path)

    for file_name in &files {
        validate_file_name(file_name).inspect_err(|_| {
            let _ = std::fs::remove_dir_all(&temp_dir);
        })?;
        let src = dir.join(file_name);
        if !src.exists() {
            continue;
        }
        let staged = temp_dir.join(file_name);
        std::fs::copy(&src, &staged).map_err(|e| {
            let _ = std::fs::remove_dir_all(&temp_dir);
            format!("Failed to stage {}: {}", file_name, e)
        })?;
        files_to_restore.push((file_name.clone(), staged));
    }

    // Phase 2: Auto-backup current files, then replace from staged copies.
    // Since all source files are already in temp, a failure here only affects
    // files already processed — but originals of unprocessed files remain intact.
    let mut restore_errors = Vec::new();

    for (file_name, staged_path) in &files_to_restore {
        let dst = cfg.user_dir.join(file_name);

        // Auto-backup current file before overwriting
        if dst.exists() {
            let backup_path =
                auto_backups_dir().join(format!("{}.{}.bak", file_name, timestamp()));
            if let Err(e) = std::fs::create_dir_all(
                backup_path.parent().unwrap_or(Path::new(".")),
            ) {
                restore_errors.push(format!("Auto-backup dir for {}: {}", file_name, e));
                continue;
            }
            if let Err(e) = std::fs::copy(&dst, &backup_path) {
                restore_errors.push(format!("Auto-backup {}: {}", file_name, e));
                continue;
            }
        }

        if let Err(e) = std::fs::copy(staged_path, &dst) {
            restore_errors.push(format!("Restore {}: {}", file_name, e));
        }
    }

    // Cleanup temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    if !restore_errors.is_empty() {
        return Err(format!(
            "Restore completed with errors: {}",
            restore_errors.join("; ")
        ));
    }

    Ok(())
}

#[tauri::command]
pub fn compare_backup(backup_id: String, file_name: String) -> Result<FileDiff, String> {
    validate_file_name(&file_name)?;
    let cfg = RimeConfig::detect();
    let dir = find_backup_dir(&backup_id)?;

    let backup_path = dir.join(&file_name);
    let backup_content = if backup_path.exists() {
        std::fs::read_to_string(&backup_path).map_err(|e| e.to_string())?
    } else {
        return Err(format!("File {} not found in backup", file_name));
    };

    let current_path = cfg.user_dir.join(&file_name);
    let current_content = if current_path.exists() {
        Some(std::fs::read_to_string(&current_path).map_err(|e| e.to_string())?)
    } else {
        None
    };

    Ok(FileDiff {
        file_name,
        current: current_content,
        backup: backup_content,
    })
}

#[tauri::command]
pub fn delete_backup(backup_id: String) -> Result<(), String> {
    let dir = find_backup_dir(&backup_id)?;
    std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(())
}

fn find_backup_dir(backup_id: &str) -> Result<PathBuf, String> {
    validate_backup_id(backup_id)?;
    for dir in [manual_backups_dir(), deploy_backups_dir()] {
        let path = dir.join(backup_id);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(format!("Backup '{}' not found", backup_id))
}

fn validate_backup_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("Invalid backup ID".to_string());
    }
    if id.chars().any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_') {
        return Err("Backup ID contains invalid characters".to_string());
    }
    Ok(())
}

fn validate_file_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 255 {
        return Err("Invalid file name".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("File name contains invalid characters".to_string());
    }
    Ok(())
}

fn count_files(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().is_file())
                .count()
        })
        .unwrap_or(0)
}

fn dir_size(dir: &Path) -> i64 {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| std::fs::metadata(e.path()).ok())
                .map(|m| m.len() as i64)
                .sum()
        })
        .unwrap_or(0)
}

pub fn create_deploy_backup() -> Result<BackupInfo, String> {
    let cfg = RimeConfig::detect();
    let ts = timestamp();
    let backup_dir = deploy_backups_dir().join(&ts);
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    let mut file_count = 0;
    let mut total_size: i64 = 0;

    for file_name in config_files_to_backup() {
        let src = cfg.user_dir.join(file_name);
        if src.exists() {
            let dst = backup_dir.join(file_name);
            std::fs::copy(&src, &dst).map_err(|e| e.to_string())?;
            let meta = std::fs::metadata(&dst).ok();
            file_count += 1;
            total_size += meta.map(|m| m.len() as i64).unwrap_or(0);
        }
    }

    let info = BackupInfo {
        id: ts,
        created_at: timestamp_iso(),
        backup_type: "deploy".to_string(),
        file_count,
        total_size,
        note: Some("部署前自动备份".to_string()),
    };

    let manifest = serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?;
    std::fs::write(backup_dir.join("manifest.json"), manifest)
        .map_err(|e| e.to_string())?;

    Ok(info)
}
