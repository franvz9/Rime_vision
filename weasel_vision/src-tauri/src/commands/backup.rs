use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::rime::config::RimeConfig;

/// Maximum recursion depth for directory traversal to prevent stack overflow
/// from symlink cycles or excessively deep directory trees.
const MAX_DIR_DEPTH: u32 = 32;

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

/// 判断目录是否应被备份排除
fn is_backup_excluded_dir(name: &str) -> bool {
    let excluded_exact = ["backups", "build", ".restore_temp"];
    excluded_exact.contains(&name) || name.starts_with(".restore_temp")
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
            // Core config files — user-editable files from user_dir
            let user_files = [
                "default.custom.yaml",
                "installation.yaml", "user.yaml",
            ];
            for f in &user_files {
                let path = cfg.user_dir.join(f);
                if path.exists() {
                    files.push(path);
                }
            }
            // Platform-specific
            if cfg!(target_os = "macos") {
                // macOS: all files in same dir
                let p = cfg.user_dir.join("default.yaml");
                if p.exists() { files.push(p); }
                let p = cfg.user_dir.join("squirrel.yaml");
                if p.exists() { files.push(p); }
                let p = cfg.user_dir.join("squirrel.custom.yaml");
                if p.exists() { files.push(p); }
            } else if cfg!(target_os = "windows") {
                // Windows: compiled files from rime_data_dir(), custom files from user_dir
                let data_dir = cfg.rime_data_dir();
                let p = data_dir.join("default.yaml");
                if p.exists() { files.push(p); }
                let p = data_dir.join("weasel.yaml");
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
    collect_all_files_inner(dir, files, 0)
}

fn collect_all_files_inner(dir: &Path, files: &mut Vec<PathBuf>, depth: u32) -> Result<(), String> {
    if depth > MAX_DIR_DEPTH {
        return Ok(());
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Skip symbolic links to prevent infinite loops from symlink cycles
            if path.is_symlink() {
                continue;
            }
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                collect_all_files_inner(&path, files, depth + 1)?;
            }
        }
    }
    Ok(())
}

fn collect_user_dir(user_dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if let Ok(entries) = std::fs::read_dir(user_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Skip symbolic links to prevent infinite loops from symlink cycles
            if path.is_symlink() {
                continue;
            }
            
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if is_backup_excluded_dir(dir_name) {
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

/// Unified recursive directory copy with an optional per-file filter.
/// `skip_file` receives the file/dir name and returns true to skip it.
fn copy_dir_filtered(
    src: &Path,
    dst: &Path,
    skip_file: &dyn Fn(&str) -> bool,
) -> Result<usize, String> {
    copy_dir_filtered_inner(src, dst, skip_file, 0)
}

fn copy_dir_filtered_inner(
    src: &Path,
    dst: &Path,
    skip_file: &dyn Fn(&str) -> bool,
    depth: u32,
) -> Result<usize, String> {
    if depth > MAX_DIR_DEPTH {
        return Ok(0);
    }
    std::fs::create_dir_all(dst).map_err(|e| format!("Failed to create dir {:?}: {}", dst, e))?;
    let mut count = 0;

    if let Ok(entries) = std::fs::read_dir(src) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip symbolic links to prevent infinite loops from symlink cycles
            if src_path.is_symlink() {
                continue;
            }

            // Validate file name to prevent path traversal
            if let Err(e) = validate_file_name(&name) {
                eprintln!("Skipping invalid entry: {} - {}", name, e);
                continue;
            }

            // Apply caller-supplied filter
            if skip_file(&name) {
                continue;
            }

            let dst_path = dst.join(&name);

            // Skip locked LevelDB files on Windows (os error 32)
            #[cfg(target_os = "windows")]
            {
                if name.ends_with(".log") || name == "LOCK" || name.starts_with("MANIFEST-") || name == "CURRENT" {
                    eprintln!("Skipping locked file: {}", name);
                    continue;
                }
            }

            if src_path.is_dir() {
                count += copy_dir_filtered_inner(&src_path, &dst_path, skip_file, depth + 1)?;
            } else {
                match std::fs::copy(&src_path, &dst_path) {
                    Ok(_) => count += 1,
                    Err(e) => {
                        // Skip locked files on Windows
                        #[cfg(target_os = "windows")]
                        if e.raw_os_error() == Some(32) {
                            eprintln!("Skipping locked file: {:?}", src_path);
                            continue;
                        }
                        return Err(format!("Failed to copy {:?} to {:?}: {}", src_path, dst_path, e));
                    }
                }
            }
        }
    }
    Ok(count)
}

/// Recursively copy a directory (copy everything)
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<usize, String> {
    copy_dir_filtered(src, dst, &|_| false)
}

/// Copy directory recursively, optionally excluding .gram model files
fn copy_dir_recursive_exclude(src: &Path, dst: &Path, include_models: bool) -> Result<usize, String> {
    copy_dir_filtered(src, dst, &|name: &str| !include_models && name.ends_with(".gram"))
}

fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S%.3f").to_string()
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
        // user_dir is always the correct Rime root directory
        let actual_backup_source = cfg.user_dir.clone();
        
        if let Ok(entries) = std::fs::read_dir(&actual_backup_source) {
            for entry in entries.flatten() {
                let src_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Skip excluded directories
                if src_path.is_dir() && is_backup_excluded_dir(&name) {
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

    if restore_files.is_empty() {
        restore_full(&cfg, &dir)
    } else {
        restore_partial(&cfg, &dir, restore_files)
    }
}

/// Full restore: copy entire backup directory back to user directory.
/// Strategy: stage backup files, then atomically swap with current files.
fn restore_full(cfg: &RimeConfig, backup_dir: &Path) -> Result<(), String> {
    // Create auto-backup of current state
    let auto_backup_dir = auto_backups_dir().join(format!("pre_restore_{}", timestamp()));
    std::fs::create_dir_all(&auto_backup_dir).map_err(|e| e.to_string())?;

    if let Ok(entries) = std::fs::read_dir(&cfg.user_dir) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if src_path.is_dir() && is_backup_excluded_dir(&name) {
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

    // Phase 1: Copy backup to a staging directory
    let restore_id = uuid::Uuid::new_v4().to_string();
    let staging_dir = cfg.user_dir.join(format!(".restore_temp_new_{}", restore_id));
    std::fs::create_dir_all(&staging_dir).map_err(|e| e.to_string())?;

    let mut staged_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(backup_dir) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name == "manifest.json" {
                continue;
            }

            if let Err(e) = validate_file_name(&name) {
                eprintln!("Skipping invalid backup entry: {} - {}", name, e);
                continue;
            }

            let staged_path = staging_dir.join(&name);
            if src_path.is_dir() {
                copy_dir_recursive(&src_path, &staged_path)?;
            } else {
                std::fs::copy(&src_path, &staged_path).map_err(|e| e.to_string())?;
            }
            staged_files.push((name, src_path.is_dir()));
        }
    }

    // Phase 2: Atomically replace current files with staged files.
    let bak_suffix = format!(".restore_bak_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0"));

    for (name, is_dir) in &staged_files {
        let staged_path = staging_dir.join(name);
        let dst_path = cfg.user_dir.join(name);

        if *is_dir {
            if dst_path.exists() {
                let bak_path = cfg.user_dir.join(format!("{}{}", name, bak_suffix));
                if bak_path.exists() {
                    let _ = std::fs::remove_dir_all(&bak_path);
                }
                std::fs::rename(&dst_path, &bak_path)
                    .map_err(|e| format!("Failed to back up directory {:?}: {}", dst_path, e))?;
                if let Err(e) = std::fs::rename(&staged_path, &dst_path) {
                    let _ = std::fs::rename(&bak_path, &dst_path);
                    return Err(format!("Failed to restore directory {}: {}", name, e));
                }
                let _ = std::fs::remove_dir_all(&bak_path);
            } else {
                std::fs::rename(&staged_path, &dst_path)
                    .map_err(|e| format!("Failed to restore new directory {}: {}", name, e))?;
            }
        } else {
            if std::fs::rename(&staged_path, &dst_path).is_err() {
                if dst_path.exists() {
                    let bak = dst_path.with_extension(format!("{}.{}", dst_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default(), bak_suffix));
                    let _ = std::fs::copy(&dst_path, &bak);
                    let _ = std::fs::remove_file(&dst_path);
                }
                std::fs::copy(&staged_path, &dst_path)
                    .map_err(|e| format!("Failed to restore {}: {}", name, e))?;
            }
        }
    }

    // Cleanup staging directory
    let _ = std::fs::remove_dir_all(&staging_dir);
    Ok(())
}

/// Partial restore: file-by-file approach with staged copies.
fn restore_partial(cfg: &RimeConfig, backup_dir: &Path, files: Vec<String>) -> Result<(), String> {
    let restore_id = uuid::Uuid::new_v4().to_string();
    let temp_dir = cfg.user_dir.join(format!(".restore_temp_{}", restore_id));
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let mut files_to_restore: Vec<(String, PathBuf)> = Vec::new();

    for file_name in &files {
        validate_file_name(file_name).inspect_err(|_| {
            let _ = std::fs::remove_dir_all(&temp_dir);
        })?;
        let src = backup_dir.join(file_name);
        if !src.exists() {
            continue;
        }
        let staged = temp_dir.join(file_name);
        if let Some(parent) = staged.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::copy(&src, &staged).map_err(|e| {
            let _ = std::fs::remove_dir_all(&temp_dir);
            format!("Failed to stage {}: {}", file_name, e)
        })?;
        files_to_restore.push((file_name.clone(), staged));
    }

    let mut restore_errors = Vec::new();

    for (file_name, staged_path) in &files_to_restore {
        let dst = cfg.user_dir.join(file_name);

        if let Some(parent) = dst.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if dst.exists() {
            let safe_name = file_name.replace(['/', '\\'], "_");
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
    crate::rime::utils::validate_safe_path(name)?;
    // Block absolute paths and single dot (file-name-specific rules)
    if name.starts_with('/') || name.starts_with('\\') || name == "." {
        return Err("文件名无效".to_string());
    }
    Ok(())
}

fn count_files(dir: &Path) -> usize {
    crate::rime::utils::count_files(dir)
}

fn dir_size(dir: &Path) -> i64 {
    crate::rime::utils::dir_size(dir)
}

/// Recursively collect files from backup dir, storing relative paths
fn collect_backup_files_recursive(base_dir: &Path, current_dir: &Path, files: &mut Vec<BackupFile>) {
    collect_backup_files_recursive_inner(base_dir, current_dir, files, 0);
}

fn collect_backup_files_recursive_inner(base_dir: &Path, current_dir: &Path, files: &mut Vec<BackupFile>, depth: u32) {
    if depth > MAX_DIR_DEPTH {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Skip symbolic links to prevent infinite loops from symlink cycles
            if path.is_symlink() {
                continue;
            }
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
                collect_backup_files_recursive_inner(base_dir, &path, files, depth + 1);
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
    if let Ok(entries) = std::fs::read_dir(&cfg.user_dir) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if src_path.is_dir() && is_backup_excluded_dir(&name) {
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

/// Remove oldest deploy backups when count exceeds `max_count`.
/// Safety: only removes directories that are direct children of the deploy backups dir.
fn cleanup_old_deploy_backups(max_count: usize) {
    let dir = deploy_backups_dir();
    if !dir.exists() {
        return;
    }

    let mut backups: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Safety: only consider direct children of deploy_backups_dir
            if path.is_dir() && path.parent() == Some(&dir) {
                let name = entry.file_name().to_string_lossy().to_string();
                backups.push((name, path));
            }
        }
    }

    // Sort by name (timestamp) descending (newest first)
    backups.sort_by(|a, b| b.0.cmp(&a.0));

    // Remove backups beyond the limit
    for (name, path) in backups.iter().skip(max_count) {
        // Double-check: path must still be under deploy_backups_dir
        if path.parent() != Some(&dir) {
            eprintln!("Warning: refusing to remove backup outside deploy dir: {:?}", path);
            continue;
        }
        if let Err(e) = std::fs::remove_dir_all(path) {
            eprintln!("Warning: failed to remove old deploy backup {}: {}", name, e);
        } else {
            eprintln!("Removed old deploy backup: {}", name);
        }
    }
}
