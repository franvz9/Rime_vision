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

/// Get files by backup category
fn get_files_by_category(category: &str) -> Result<Vec<PathBuf>, String> {
    let cfg = RimeConfig::detect();
    let mut files = Vec::new();
    
    match category {
        "core" => {
            // Core config files only
            let core_files = [
                "default.yaml", "default.custom.yaml",
                "installation.yaml", "user.yaml",
            ];
            for f in &core_files {
                let path = cfg.user_dir.join(f);
                if path.exists() {
                    files.push(path);
                }
            }
            // Platform-specific
            if cfg!(target_os = "macos") {
                let p = cfg.user_dir.join("squirrel.yaml");
                if p.exists() { files.push(p); }
                let p = cfg.user_dir.join("squirrel.custom.yaml");
                if p.exists() { files.push(p); }
            } else if cfg!(target_os = "windows") {
                let p = cfg.user_dir.join("weasel.yaml");
                if p.exists() { files.push(p); }
                let p = cfg.user_dir.join("weasel.custom.yaml");
                if p.exists() { files.push(p); }
            }
        }
        "schemas" => {
            // Schema definition files (*.schema.yaml)
            if let Ok(entries) = std::fs::read_dir(&cfg.user_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.ends_with(".schema.yaml") || name.ends_with(".schema.yml") {
                                files.push(path);
                            }
                        }
                    }
                }
            }
        }
        "themes" => {
            // Color schemes from custom yaml files
            let theme_files = vec![
                cfg.user_dir.join("squirrel.custom.yaml"),
                cfg.user_dir.join("weasel.custom.yaml"),
                cfg.user_dir.join("style.custom.yaml"),
            ];
            for p in theme_files {
                if p.exists() {
                    files.push(p);
                }
            }
        }
        "dicts" => {
            // User dictionaries (*.userdb.txt)
            let dicts_dir = cfg.user_dir.join("user_dictionaries");
            if dicts_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&dicts_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                if name.ends_with(".userdb.txt") {
                                    files.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }
        "models" => {
            // Grammar model files (*.gram)
            if let Ok(entries) = std::fs::read_dir(&cfg.user_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.ends_with(".gram") {
                                files.push(path);
                            }
                        }
                    }
                }
            }
        }
        "opencc" => {
            // OpenCC data directory
            let opencc_dir = cfg.user_dir.join("opencc");
            if opencc_dir.exists() {
                collect_all_files(&opencc_dir, &mut files)?;
            }
        }
        "lua" => {
            // Lua scripts directory
            let lua_dir = cfg.user_dir.join("lua");
            if lua_dir.exists() {
                collect_all_files(&lua_dir, &mut files)?;
            }
        }
        "full" => {
            // Entire user directory (excluding backups and build)
            collect_user_dir(&cfg.user_dir, &mut files)?;
        }
        _ => return Err(format!("Unknown backup category: {}", category)),
    }
    
    Ok(files)
}

fn collect_all_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                collect_all_files(&path, files)?;
            }
        }
    }
    Ok(())
}

fn collect_user_dir(user_dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let exclude_dirs = ["backups", "build", ".restore_temp"];
    
    if let Ok(entries) = std::fs::read_dir(user_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if exclude_dirs.contains(&dir_name) {
                        continue;
                    }
                }
            }
            
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                collect_all_files(&path, files)?;
            }
        }
    }
    Ok(())
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<usize, String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("Failed to create dir {:?}: {}", dst, e))?;
    let mut count = 0;
    
    if let Ok(entries) = std::fs::read_dir(src) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if src_path.is_dir() {
                count += copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)
                    .map_err(|e| format!("Failed to copy {:?} to {:?}: {}", src_path, dst_path, e))?;
                count += 1;
            }
        }
    }
    Ok(count)
}

/// Copy directory recursively, optionally excluding .gram model files
fn copy_dir_recursive_exclude(src: &Path, dst: &Path, include_models: bool) -> Result<usize, String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("Failed to create dir {:?}: {}", dst, e))?;
    let mut count = 0;

    if let Ok(entries) = std::fs::read_dir(src) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let dst_path = dst.join(&name);

            // Skip .gram model files when not needed
            if !include_models && name.ends_with(".gram") {
                continue;
            }

            if src_path.is_dir() {
                count += copy_dir_recursive_exclude(&src_path, &dst_path, include_models)?;
            } else {
                std::fs::copy(&src_path, &dst_path)
                    .map_err(|e| format!("Failed to copy {:?} to {:?}: {}", src_path, dst_path, e))?;
                count += 1;
            }
        }
    }
    Ok(count)
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
    collect_backup_files_recursive(&dir, &dir, &mut files);

    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(BackupDetail { info, files })
}

#[tauri::command]
pub fn create_backup(categories: Vec<String>, note: Option<String>) -> Result<BackupInfo, String> {
    let cfg = RimeConfig::detect();
    let ts = timestamp();
    let backup_dir = manual_backups_dir().join(&ts);
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    let mut file_count = 0;
    
    // If no categories specified, use full backup (backward compatibility)
    let categories_to_use = if categories.is_empty() {
        vec!["full".to_string()]
    } else {
        categories
    };

    // Check if this is a full backup
    let is_full = categories_to_use.iter().any(|c| c == "full");
    
    if is_full {
        // For full backup: directly copy entire user directory structure
        let exclude_dirs = ["backups", "build", ".restore_temp"];
        
        // On Windows, when using build/ directory as user_dir, we need to copy from parent Rime dir
        #[cfg(target_os = "windows")]
        let actual_backup_source = if cfg.user_dir.ends_with("build") && cfg.user_dir.parent().map_or(false, |p| p.ends_with("Rime")) {
            cfg.user_dir.parent().unwrap().to_path_buf()
        } else {
            cfg.user_dir.clone()
        };
        
        #[cfg(not(target_os = "windows"))]
        let actual_backup_source = cfg.user_dir.clone();
        
        if let Ok(entries) = std::fs::read_dir(&actual_backup_source) {
            for entry in entries.flatten() {
                let src_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Skip excluded directories
                if src_path.is_dir() && exclude_dirs.contains(&name.as_str()) {
                    continue;
                }
                
                // Skip locked LevelDB files on Windows (os error 32)
                #[cfg(target_os = "windows")]
                if name.ends_with(".log") || name == "LOCK" || name.starts_with("MANIFEST-") || name == "CURRENT" {
                    eprintln!("Skipping locked file: {}", name);
                    continue;
                }
                
                let dst_path = backup_dir.join(&name);
                if src_path.is_dir() {
                    // Copy entire directory recursively
                    file_count += copy_dir_recursive(&src_path, &dst_path)?;
                } else {
                    // Copy single file with retry for locked files
                    match std::fs::copy(&src_path, &dst_path) {
                        Ok(_) => file_count += 1,
                        Err(e) => {
                            // Skip locked files on Windows
                            if e.raw_os_error() == Some(32) {
                                eprintln!("Skipping locked file: {}", src_path.display());
                                continue;
                            }
                            return Err(e.to_string());
                        }
                    }
                }
            }
        }
    } else {
        // For category-based backup: collect and copy files
        for category in &categories_to_use {
            let files = get_files_by_category(category)?;
            for src_path in files {
                let rel_path = src_path.strip_prefix(&cfg.user_dir)
                    .map_err(|_| format!("Failed to get relative path for {:?}", src_path))?;
                
                if let Some(parent) = rel_path.parent() {
                    let dst_parent = backup_dir.join(parent);
                    std::fs::create_dir_all(&dst_parent).map_err(|e| e.to_string())?;
                }
                
                let dst = backup_dir.join(rel_path);
                std::fs::copy(&src_path, &dst).map_err(|e| e.to_string())?;
                file_count += 1;
            }
        }
    }

    // Calculate total size
    let total_size = dir_size(&backup_dir);

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

    // Check if this is a full restore (restore_files is empty or contains all files)
    let is_full_restore = restore_files.is_empty();

    if is_full_restore {
        // Full restore: copy entire backup directory back to user directory
        // First, auto-backup current user directory
        let auto_backup_dir = auto_backups_dir().join(format!("pre_restore_{}", timestamp()));
        std::fs::create_dir_all(&auto_backup_dir).map_err(|e| e.to_string())?;
        
        let exclude_dirs = ["backups", "build", ".restore_temp"];
        if let Ok(entries) = std::fs::read_dir(&cfg.user_dir) {
            for entry in entries.flatten() {
                let src_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                if src_path.is_dir() && exclude_dirs.contains(&name.as_str()) {
                    continue;
                }
                
                let dst_path = auto_backup_dir.join(&name);
                if src_path.is_dir() {
                    copy_dir_recursive(&src_path, &dst_path)
                        .map_err(|e| format!("自动备份目录失败 {:?}: {}", src_path, e))?;
                } else {
                    std::fs::copy(&src_path, &dst_path)
                        .map_err(|e| format!("自动备份文件失败 {:?}: {}", src_path, e))?;
                }
            }
        }
        
        // Now copy backup files back to user directory
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let src_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Skip manifest.json
                if name == "manifest.json" {
                    continue;
                }
                
                let dst_path = cfg.user_dir.join(&name);
                if src_path.is_dir() {
                    // Remove existing directory and copy backup
                    if dst_path.exists() {
                        std::fs::remove_dir_all(&dst_path)
                            .map_err(|e| format!("删除已有目录失败 {:?}: {}", dst_path, e))?;
                    }
                    copy_dir_recursive(&src_path, &dst_path)?;
                } else {
                    std::fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
                }
            }
        }
        
        return Ok(());
    }

    // Partial restore: file-by-file approach
    let files = restore_files;

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
        // Create parent directory for subdirectory files
        if let Some(parent) = staged.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
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

        // Ensure parent directory exists for subdirectory files
        if let Some(parent) = dst.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Auto-backup current file before overwriting
        if dst.exists() {
            // Use relative path for backup name to avoid collisions
            let safe_name = file_name.replace('/', "_").replace('\\', "_");
            let backup_path =
                auto_backups_dir().join(format!("{}.{}.bak", safe_name, timestamp()));
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
    if name.is_empty() || name.len() > 512 {
        return Err("Invalid file name".to_string());
    }
    // Block path traversal but allow relative paths with /
    if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
        return Err("File name contains invalid characters".to_string());
    }
    Ok(())
}

fn count_files(dir: &Path) -> usize {
    let mut count = 0;
    count_files_recursive(dir, &mut count);
    count
}

fn count_files_recursive(dir: &Path, count: &mut usize) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                *count += 1;
            } else if path.is_dir() {
                count_files_recursive(&path, count);
            }
        }
    }
}

fn dir_size(dir: &Path) -> i64 {
    let mut size = 0i64;
    dir_size_recursive(dir, &mut size);
    size
}

fn dir_size_recursive(dir: &Path, size: &mut i64) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(m) = std::fs::metadata(&path) {
                    *size += m.len() as i64;
                }
            } else if path.is_dir() {
                dir_size_recursive(&path, size);
            }
        }
    }
}

/// Recursively collect files from backup dir, storing relative paths
fn collect_backup_files_recursive(base_dir: &Path, current_dir: &Path, files: &mut Vec<BackupFile>) {
    if let Ok(entries) = std::fs::read_dir(current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                // Skip manifest.json
                if path.extension().and_then(|e| e.to_str()) == Some("json")
                    && path.file_name().and_then(|n| n.to_str()) == Some("manifest.json")
                {
                    continue;
                }
                let rel_path = path.strip_prefix(base_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                let meta = std::fs::metadata(&path).ok();
                files.push(BackupFile {
                    name: rel_path,
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
            } else if path.is_dir() {
                collect_backup_files_recursive(base_dir, &path, files);
            }
        }
    }
}

pub fn create_deploy_backup(include_models: bool) -> Result<BackupInfo, String> {
    let cfg = RimeConfig::detect();
    let ts = timestamp();
    let backup_dir = deploy_backups_dir().join(&ts);
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    let mut file_count = 0;

    // Copy user directory, optionally excluding large model files (.gram)
    // When no model deletion is planned, skip .gram files to save space
    let exclude_dirs = ["backups", "build", ".restore_temp"];
    if let Ok(entries) = std::fs::read_dir(&cfg.user_dir) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if src_path.is_dir() && exclude_dirs.contains(&name.as_str()) {
                continue;
            }

            // Skip .gram model files when not needed
            if !include_models && name.ends_with(".gram") {
                continue;
            }

            let dst_path = backup_dir.join(&name);
            if src_path.is_dir() {
                file_count += copy_dir_recursive_exclude(&src_path, &dst_path, include_models)?;
            } else {
                std::fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
                file_count += 1;
            }
        }
    }

    let total_size = dir_size(&backup_dir);

    let backup_note = if include_models {
        "部署前自动备份（含模型）"
    } else {
        "部署前自动备份（不含模型）"
    };

    let info = BackupInfo {
        id: ts,
        created_at: timestamp_iso(),
        backup_type: "deploy".to_string(),
        file_count,
        total_size,
        note: Some(backup_note.to_string()),
    };

    let manifest = serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?;
    std::fs::write(backup_dir.join("manifest.json"), manifest)
        .map_err(|e| e.to_string())?;

    // Roll out old deploy backups if exceeding limit (max 10)
    cleanup_old_deploy_backups(10);

    Ok(info)
}

/// Remove oldest deploy backups when count exceeds `max_count`
fn cleanup_old_deploy_backups(max_count: usize) {
    let dir = deploy_backups_dir();
    if !dir.exists() {
        return;
    }

    let mut backups: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                backups.push((name, entry.path()));
            }
        }
    }

    // Sort by name (timestamp) descending (newest first)
    backups.sort_by(|a, b| b.0.cmp(&a.0));

    // Remove backups beyond the limit
    for (_, path) in backups.iter().skip(max_count) {
        let _ = std::fs::remove_dir_all(path);
    }
}
