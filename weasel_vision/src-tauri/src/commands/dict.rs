use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::rime::config::RimeConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDictInfo {
    pub dict_id: String,
    pub display_name: String,
    pub schema_ids: Vec<String>,
    pub entry_count: i64,
    pub file_size: i64,
    pub last_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictEntry {
    pub word: String,
    pub code: String,
    pub frequency: i64,
    pub commit_count: i64,
    pub last_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictEntriesResult {
    pub entries: Vec<DictEntry>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_frequency: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub file_name: String,
    pub created_at: String,
    pub size: i64,
    pub snapshot_type: String,
}

fn userdb_dirs() -> Vec<PathBuf> {
    let cfg = RimeConfig::detect();
    let mut dirs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&cfg.user_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".userdb") {
                        dirs.push(path);
                    }
                }
            }
        }
    }

    dirs.sort();
    dirs
}

fn dict_id_from_path(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .replace(".userdb", "")
}

fn user_dict_snapshots_dir() -> PathBuf {
    let cfg = RimeConfig::detect();
    cfg.user_dir.join("user_dictionaries")
}

fn parse_snapshot_line(line: &str) -> Option<DictEntry> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 3 {
        return None;
    }

    let word = parts[0].to_string();
    let code = parts[1].to_string();
    
    // Rime snapshot format: "word\tcode\tc=N d=X.XX t=XXX"
    // Parse the third column to extract commit count (c=N)
    let third_col = parts[2];
    let mut frequency = 0i64;
    let mut commit_count = 0i64;
    
    // Try to parse as simple number first (legacy format)
    if let Ok(freq) = third_col.parse::<i64>() {
        frequency = freq;
        commit_count = freq;
    } else {
        // Parse Rime format: "c=N d=X.XX t=XXX"
        for part in third_col.split_whitespace() {
            if let Some(val_str) = part.strip_prefix("c=") {
                if let Ok(val) = val_str.parse::<i64>() {
                    commit_count = val;
                    frequency = val; // Use commit count as frequency
                }
            }
        }
    }

    if word.is_empty() || code.is_empty() {
        return None;
    }

    Some(DictEntry {
        word,
        code,
        frequency,
        commit_count,
        last_used: String::new(),
    })
}

#[tauri::command]
pub fn list_user_dictionaries() -> Result<Vec<UserDictInfo>, String> {
    let cfg = RimeConfig::detect();
    let mut result = Vec::new();

    for dir in userdb_dirs() {
        let dict_id = dict_id_from_path(&dir);
        let file_size = dir_size(&dir);
        let last_modified = dir_metadata_modified(&dir);

        let schema_ids = find_schemas_for_dict(&cfg, &dict_id);

        let entry_count = count_snapshot_entries(&dict_id);
        // If no snapshot entries found but userdb dir has data, indicate snapshot needed
        // Use i64 cast: -1 means "data exists but not exported as text snapshot"
        let entry_count: i64 = if entry_count == 0 && file_size > 0 {
            -1
        } else {
            entry_count as i64
        };

        result.push(UserDictInfo {
            dict_id: dict_id.clone(),
            display_name: dict_display_name(&dict_id),
            schema_ids,
            entry_count,
            file_size,
            last_modified,
        });
    }

    Ok(result)
}

#[tauri::command]
pub fn load_user_dict_entries(
    dict_id: String,
    page: usize,
    per_page: usize,
    sort_by: String,
    search: Option<String>,
) -> Result<DictEntriesResult, String> {
    validate_dict_id(&dict_id)?;
    let snapshot = match find_snapshot(&dict_id) {
        Ok(s) => s,
        Err(_) => {
            // No snapshot found - return empty result
            return Ok(DictEntriesResult {
                entries: Vec::new(),
                total: 0,
                page,
                per_page,
                total_frequency: 0,
            });
        }
    };
    let content = std::fs::read_to_string(&snapshot).map_err(|e| e.to_string())?;

    let mut entries: Vec<DictEntry> = content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter_map(parse_snapshot_line)
        .collect();

    if let Some(ref query) = search {
        let q = query.to_lowercase();
        entries.retain(|e| {
            e.word.to_lowercase().contains(&q) || e.code.to_lowercase().contains(&q)
        });
    }

    let total = entries.len();
    let total_frequency: i64 = entries.iter().map(|e| e.frequency).sum();

    match sort_by.as_str() {
        "frequency_desc" => entries.sort_by_key(|a| std::cmp::Reverse(a.frequency)),
        "frequency_asc" => entries.sort_by_key(|a| a.frequency),
        "word_asc" => entries.sort_by(|a, b| a.word.cmp(&b.word)),
        "word_desc" => entries.sort_by(|a, b| b.word.cmp(&a.word)),
        "code_asc" => entries.sort_by(|a, b| a.code.cmp(&b.code)),
        _ => entries.sort_by_key(|a| std::cmp::Reverse(a.frequency)),
    }

    let start = page * per_page;
    let paged: Vec<DictEntry> = entries.into_iter().skip(start).take(per_page).collect();

    Ok(DictEntriesResult {
        entries: paged,
        total,
        page,
        per_page,
        total_frequency,
    })
}

#[tauri::command]
pub fn list_snapshots(dict_id: String) -> Result<Vec<SnapshotInfo>, String> {
    validate_dict_id(&dict_id)?;
    let cfg = RimeConfig::detect();
    let mut snapshots = Vec::new();
    let mut seen_files = std::collections::HashSet::new();

    // Priority 1: Check sync directory (Rime native sync output)
    if let Ok(installation_content) = std::fs::read_to_string(cfg.user_dir.join("installation.yaml")) {
        if let Ok(installation) = serde_yaml::from_str::<serde_yaml::Value>(&installation_content) {
            if let Some(sync_dir) = installation.get("sync_dir").and_then(|v| v.as_str()) {
                if let Some(device_id) = installation.get("installation_id").and_then(|v| v.as_str()) {
                    let sync_device_dir = std::path::PathBuf::from(sync_dir).join(device_id);
                    if sync_device_dir.exists() {
                        if let Ok(entries) = std::fs::read_dir(&sync_device_dir) {
                            for entry in entries.flatten() {
                                let name = entry.file_name().to_string_lossy().to_string();
                                if name == format!("{}.userdb.txt", dict_id) {
                                    let meta = std::fs::metadata(entry.path()).ok();
                                    snapshots.push(SnapshotInfo {
                                        file_name: name.clone(),
                                        created_at: meta
                                            .as_ref()
                                            .and_then(|m| m.modified().ok())
                                            .map(|t| {
                                                chrono::DateTime::<chrono::Local>::from(t)
                                                    .format("%Y-%m-%d %H:%M:%S")
                                                    .to_string()
                                            })
                                            .unwrap_or_default(),
                                        size: meta.as_ref().map(|m| m.len() as i64).unwrap_or(0),
                                        snapshot_type: "sync".to_string(),
                                    });
                                    seen_files.insert(name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Priority 2: Check user_dictionaries/ directory (legacy location)
    let snapshots_dir = user_dict_snapshots_dir();
    if snapshots_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&format!("{}.", dict_id)) && name.ends_with(".txt") && !seen_files.contains(&name) {
                    let meta = std::fs::metadata(entry.path()).ok();
                    snapshots.push(SnapshotInfo {
                        file_name: name.clone(),
                        created_at: meta
                            .as_ref()
                            .and_then(|m| m.modified().ok())
                            .map(|t| {
                                chrono::DateTime::<chrono::Local>::from(t)
                                    .format("%Y-%m-%d %H:%M:%S")
                                    .to_string()
                            })
                            .unwrap_or_default(),
                        size: meta.as_ref().map(|m| m.len() as i64).unwrap_or(0),
                        snapshot_type: "manual".to_string(),
                    });
                    seen_files.insert(name);
                }
            }
        }
    }

    // Priority 3: Check *.userdb/ directory
    let userdb_dir = cfg.user_dir.join(format!("{}.userdb", dict_id));
    if let Ok(entries) = std::fs::read_dir(&userdb_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".txt") && !seen_files.contains(&name) {
                let meta = std::fs::metadata(entry.path()).ok();
                snapshots.push(SnapshotInfo {
                    file_name: format!("{}/{}", dict_id, name),
                    created_at: meta
                        .as_ref()
                        .and_then(|m| m.modified().ok())
                        .map(|t| {
                            chrono::DateTime::<chrono::Local>::from(t)
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string()
                        })
                        .unwrap_or_default(),
                    size: meta.as_ref().map(|m| m.len() as i64).unwrap_or(0),
                    snapshot_type: "auto".to_string(),
                });
            }
        }
    }

    snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(snapshots)
}

/// Find the most recent snapshot for a dict.
/// Collects all matching files and returns the one with the latest modification time.
fn find_snapshot(dict_id: &str) -> Result<PathBuf, String> {
    let cfg = RimeConfig::detect();
    
    // Priority 1: Check sync directory (Rime native sync output)
    // Path: ~/Library/Rime/sync/<device_id>/<dict_id>.userdb.txt
    // IMPORTANT: Scan ALL device folders in sync dir, not just current device,
    // because other devices' synced data is stored in their respective folders.
    if let Ok(installation_content) = std::fs::read_to_string(cfg.user_dir.join("installation.yaml")) {
        if let Ok(installation) = serde_yaml::from_str::<serde_yaml::Value>(&installation_content) {
            if let Some(sync_dir) = installation.get("sync_dir").and_then(|v| v.as_str()) {
                let sync_base = std::path::PathBuf::from(sync_dir);
                if sync_base.exists() {
                    // Scan all device folders to find the most recent snapshot
                    let mut candidates: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
                    
                    if let Ok(entries) = std::fs::read_dir(&sync_base) {
                        for entry in entries.flatten() {
                            if entry.path().is_dir() {
                                let snapshot_path = entry.path().join(format!("{}.userdb.txt", dict_id));
                                if snapshot_path.exists() {
                                    if let Ok(meta) = std::fs::metadata(&snapshot_path) {
                                        if let Ok(modified) = meta.modified() {
                                            candidates.push((snapshot_path, modified));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Sort by modification time (most recent first)
                    candidates.sort_by_key(|(_, m)| std::cmp::Reverse(*m));
                    
                    if let Some((latest_path, _)) = candidates.first() {
                        return Ok(latest_path.clone());
                    }
                }
            }
        }
    }
    
    // Priority 2: Check user_dictionaries/ directory (legacy location)
    let snapshots_dir = user_dict_snapshots_dir();
    if snapshots_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
            let mut candidates: Vec<_> = entries
                .flatten()
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.starts_with(&format!("{}.", dict_id)) && name.ends_with(".txt")
                })
                .collect();
            
            // Sort by modification time (most recent first)
            candidates.sort_by_key(|e| {
                std::cmp::Reverse(
                    e.metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                )
            });
            
            if let Some(latest) = candidates.first() {
                return Ok(latest.path());
            }
        }
    }

    // Priority 3: Check *.userdb/ directory (Rime may export here before copying to sync dir)
    let userdb_dir = cfg.user_dir.join(format!("{}.userdb", dict_id));
    if let Ok(entries) = std::fs::read_dir(&userdb_dir) {
        let mut candidates: Vec<_> = entries
            .flatten()
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.ends_with(".txt")
            })
            .collect();
        
        // Sort by modification time (most recent first)
        candidates.sort_by_key(|e| {
            std::cmp::Reverse(
                e.metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            )
        });
        
        if let Some(latest) = candidates.first() {
            return Ok(latest.path());
        }
    }

    Err(format!("No snapshot found for dict '{}'", dict_id))
}

fn find_schemas_for_dict(cfg: &RimeConfig, dict_id: &str) -> Vec<String> {
    let mut schemas = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&cfg.user_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".schema.yaml") {
                    let schema_id = name.trim_end_matches(".schema.yaml").to_string();
                    schemas.push(schema_id);
                }
            }
        }
    }

    schemas.retain(|s| {
        s == dict_id
            || s.starts_with(&format!("{}.", dict_id))
            || dict_id.starts_with(&format!("{}.", s))
    });
    schemas
}

fn validate_dict_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("Invalid dictionary ID".to_string());
    }
    if id.chars().any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-') {
        return Err("Dictionary ID contains invalid characters".to_string());
    }
    Ok(())
}

fn dict_display_name(dict_id: &str) -> String {
    match dict_id {
        "luna_pinyin" => "朙月拼音".to_string(),
        "terra_pinyin" => "地球拼音".to_string(),
        "double_pinyin" => "自然双拼".to_string(),
        "luna_pinyin_simp" => "朙月拼音·简化字".to_string(),
        "emoji" => "Emoji".to_string(),
        _ => dict_id.to_string(),
    }
}

fn count_snapshot_entries(dict_id: &str) -> usize {
    find_snapshot(dict_id)
        .ok()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|content| {
            content
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
                .filter(|line| line.contains('\t'))
                .count()
        })
        .unwrap_or(0)
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

fn dir_metadata_modified(dir: &Path) -> String {
    std::fs::metadata(dir)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| {
            chrono::DateTime::<chrono::Local>::from(t)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_default()
}

/// Delete entries from user dictionary snapshot
///
/// **Important**: This function directly modifies the `.userdb.txt` snapshot file.
/// Changes may be overwritten by Rime's next sync operation.
/// For persistent changes, the correct workflow is:
/// 1. Modify the LevelDB database directly (not implemented in this tool)
/// 2. Trigger Rime sync to generate new snapshot
///
/// Current implementation is suitable for:
/// - Quick cleanup operations where temporary changes are acceptable
/// - Testing and development purposes
/// - Cases where user understands changes may not persist
#[tauri::command]
pub fn delete_entries(dict_id: String, entries_to_delete: Vec<DictEntryKey>) -> Result<usize, String> {
    validate_dict_id(&dict_id)?;
    let snapshot = find_snapshot(&dict_id)?;
    let content = std::fs::read_to_string(&snapshot).map_err(|e| e.to_string())?;

    let delete_set: std::collections::HashSet<(String, String)> = entries_to_delete
        .iter()
        .map(|e| (e.word.clone(), e.code.clone()))
        .collect();

    let mut deleted = 0;
    let new_lines: Vec<String> = content
        .lines()
        .filter(|line| {
            if line.starts_with('#') || line.trim().is_empty() {
                return true;
            }
            if let Some(entry) = parse_snapshot_line(line) {
                if delete_set.contains(&(entry.word, entry.code)) {
                    deleted += 1;
                    return false;
                }
            }
            true
        })
        .map(|s| s.to_string())
        .collect();

    std::fs::write(&snapshot, new_lines.join("\n") + "\n").map_err(|e| e.to_string())?;
    Ok(deleted)
}

/// Update entry frequency in user dictionary snapshot
///
/// **Important**: This function directly modifies the `.userdb.txt` snapshot file.
/// Changes may be overwritten by Rime's next sync operation.
/// See `delete_entries` documentation for details on data persistence.
#[tauri::command]
pub fn update_entry_frequency(
    dict_id: String,
    word: String,
    code: String,
    new_freq: i64,
) -> Result<(), String> {
    validate_dict_id(&dict_id)?;
    let snapshot = find_snapshot(&dict_id)?;
    let content = std::fs::read_to_string(&snapshot).map_err(|e| e.to_string())?;

    let new_lines: Vec<String> = content
        .lines()
        .map(|line| {
            if line.starts_with('#') || line.trim().is_empty() {
                return line.to_string();
            }
            if let Some(entry) = parse_snapshot_line(line) {
                if entry.word == word && entry.code == code {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 3 {
                        // Preserve extra fields (d=X.XX t=XXX) in column 2,
                        // only update the c=N part
                        let third_col = parts[2];
                        let mut new_fields: Vec<String> = Vec::new();
                        let mut c_updated = false;
                        for part in third_col.split_whitespace() {
                            if let Some(_rest) = part.strip_prefix("c=") {
                                new_fields.push(format!("c={}", new_freq));
                                c_updated = true;
                            } else {
                                new_fields.push(part.to_string());
                            }
                        }
                        if !c_updated {
                            // Simple number format (legacy), just replace
                            new_fields = vec![format!("c={}", new_freq)];
                        }
                        let new_third = new_fields.join(" ");
                        if parts.len() > 3 {
                            return format!(
                                "{}\t{}\t{}{}",
                                entry.word,
                                entry.code,
                                new_third,
                                parts[3..].iter().map(|p| format!("\t{}", p)).collect::<String>()
                            );
                        }
                        return format!("{}\t{}\t{}", entry.word, entry.code, new_third);
                    }
                    return format!("{}\t{}\tc={}", entry.word, entry.code, new_freq);
                }
            }
            line.to_string()
        })
        .collect();

    std::fs::write(&snapshot, new_lines.join("\n") + "\n").map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictEntryKey {
    pub word: String,
    pub code: String,
}

#[tauri::command]
pub fn batch_delete_low_frequency(dict_id: String, threshold: i64) -> Result<usize, String> {
    validate_dict_id(&dict_id)?;
    let snapshot = find_snapshot(&dict_id)?;
    let content = std::fs::read_to_string(&snapshot).map_err(|e| e.to_string())?;

    let mut deleted = 0;
    let new_lines: Vec<String> = content
        .lines()
        .filter(|line| {
            if line.starts_with('#') || line.trim().is_empty() {
                return true;
            }
            if let Some(entry) = parse_snapshot_line(line) {
                if entry.frequency < threshold {
                    deleted += 1;
                    return false;
                }
            }
            true
        })
        .map(|s| s.to_string())
        .collect();

    std::fs::write(&snapshot, new_lines.join("\n") + "\n").map_err(|e| e.to_string())?;
    Ok(deleted)
}

#[tauri::command]
pub fn export_user_dict(dict_id: String, output_path: String) -> Result<usize, String> {
    validate_dict_id(&dict_id)?;
    let output = std::path::PathBuf::from(&output_path);
    // Validate output path
    if output_path.is_empty() {
        return Err("Output path is empty".to_string());
    }
    if output_path.contains("..") {
        return Err("Output path contains path traversal".to_string());
    }
    // Ensure output path is under user's home directory
    if let Some(home) = dirs::home_dir() {
        let canonical_output = output.canonicalize().unwrap_or_else(|_| output.clone());
        let canonical_home = home.canonicalize().unwrap_or(home);
        if !canonical_output.starts_with(&canonical_home) {
            return Err("Output path must be within user home directory".to_string());
        }
    }
    if output.exists() && !output.is_file() {
        return Err("Output path is not a file".to_string());
    }
    let snapshot = find_snapshot(&dict_id)?;
    let content = std::fs::read_to_string(&snapshot).map_err(|e| e.to_string())?;

    let mut count = 0;
    let mut lines = Vec::new();

    for line in content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if let Some(entry) = parse_snapshot_line(line) {
            lines.push(format!("{}\t{}\t{}", entry.word, entry.code, entry.frequency));
            count += 1;
        }
    }

    std::fs::write(&output_path, lines.join("\n") + "\n").map_err(|e| e.to_string())?;
    Ok(count)
}

/// Clear all entries from a user dictionary
///
/// This removes the entire LevelDB directory and recreates it empty,
/// which is safer than deleting individual files (which would corrupt LevelDB).
/// Also removes associated snapshot files.
///
/// **Note**: For changes to take effect, Rime should be redeployed after this operation.
#[tauri::command]
pub fn clear_user_dict(dict_id: String) -> Result<(), String> {
    validate_dict_id(&dict_id)?;
    let cfg = RimeConfig::detect();
    let userdb_dir = cfg.user_dir.join(format!("{}.userdb", dict_id));

    if !userdb_dir.exists() {
        return Err(format!("Dictionary '{}' not found", dict_id));
    }

    // Remove the entire .userdb directory and recreate it empty
    // This is safer than deleting individual files, which would corrupt LevelDB
    std::fs::remove_dir_all(&userdb_dir).map_err(|e| format!("Failed to remove database: {}", e))?;
    std::fs::create_dir_all(&userdb_dir).map_err(|e| format!("Failed to recreate directory: {}", e))?;

    // Also remove snapshot files so entry counts and browsing reflect the cleared state
    let snapshots_dir = cfg.user_dir.join("user_dictionaries");
    if snapshots_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&format!("{}.", dict_id)) && name.ends_with(".txt") {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDictEntry {
    pub word: String,
    pub code: String,
    pub frequency: i64,
}

#[tauri::command]
pub fn add_dict_entry(
    dict_id: String,
    entry: NewDictEntry,
) -> Result<(), String> {
    validate_dict_id(&dict_id)?;
    let snapshot = find_snapshot(&dict_id)?;
    let content = std::fs::read_to_string(&snapshot).unwrap_or_default();

    // Check if entry already exists
    for line in content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if let Some(existing) = parse_snapshot_line(line) {
            if existing.word == entry.word && existing.code == entry.code {
                return Err(format!("Entry '{}' with code '{}' already exists", entry.word, entry.code));
            }
        }
    }

    // Append new entry
    let new_line = format!("{}\t{}\t{}", entry.word, entry.code, entry.frequency);
    let new_content = if content.is_empty() {
        new_line + "\n"
    } else {
        content.trim_end().to_string() + "\n" + &new_line + "\n"
    };

    std::fs::write(&snapshot, new_content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn create_snapshot(dict_id: String) -> Result<String, String> {
    validate_dict_id(&dict_id)?;
    let cfg = RimeConfig::detect();
    
    // Check if userdb directory exists
    let userdb_dir = cfg.user_dir.join(format!("{}.userdb", dict_id));
    if !userdb_dir.exists() {
        return Err(format!("Dictionary '{}' not found", dict_id));
    }
    
    // Check if snapshot already exists (maybe from previous sync)
    if let Ok(existing) = find_snapshot(&dict_id) {
        return Ok(existing.to_string_lossy().to_string());
    }
    
    // Check if sync directory is configured in installation.yaml
    let installation_path = cfg.user_dir.join("installation.yaml");
    let sync_configured = if installation_path.exists() {
        let content = std::fs::read_to_string(&installation_path).unwrap_or_default();
        let value: serde_yaml::Value = serde_yaml::from_str(&content).ok()
            .unwrap_or(serde_yaml::Value::Null);
        value.get("sync_dir").and_then(|v| v.as_str()).is_some()
    } else {
        false
    };
    
    if !sync_configured {
        return Err("SYNC_NOT_CONFIGURED".to_string());
    }
    
    // Sync directory is configured — trigger Rime sync
    // On macOS this sends kill -HUP to Squirrel, which triggers async sync.
    // We need to wait a bit for Squirrel to generate the snapshot.
    crate::rime::deployer::sync().map_err(|e| format!("触发同步失败: {}", e))?;
    
    // Wait for Squirrel to process the sync and generate snapshot files
    // Poll for up to 5 seconds
    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if let Ok(snapshot_path) = find_snapshot(&dict_id) {
            return Ok(snapshot_path.to_string_lossy().to_string());
        }
    }
    
    Err("同步已完成，但未找到快照文件。请确保 Rime 输入法正在运行，然后重试。".to_string())
}

#[tauri::command]
pub fn delete_snapshot(dict_id: String, file_name: String) -> Result<(), String> {
    validate_dict_id(&dict_id)?;
    
    // Validate file_name to prevent path traversal
    if file_name.contains("..") || file_name.starts_with('/') || file_name.starts_with('\\') {
        return Err("Invalid snapshot file name".to_string());
    }
    
    // Determine snapshot path based on file_name format
    let cfg = RimeConfig::detect();
    let snapshot_path = if file_name.contains('/') || file_name.contains('\\') {
        // Format: "dict_id/filename.txt" - from userdb directory
        cfg.user_dir.join(&file_name)
    } else {
        // Format: "filename.txt" - from user_dictionaries directory
        cfg.user_dir.join("user_dictionaries").join(&file_name)
    };
    
    // Ensure resolved path is still under user_dir
    if let Ok(canonical) = snapshot_path.canonicalize() {
        let canonical_user_dir = cfg.user_dir.canonicalize().unwrap_or_else(|_| cfg.user_dir.clone());
        if !canonical.starts_with(&canonical_user_dir) {
            return Err("Snapshot path is outside user directory".to_string());
        }
    }
    
    if !snapshot_path.exists() {
        return Err(format!("Snapshot '{}' not found", file_name));
    }
    
    std::fs::remove_file(&snapshot_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Apply modified snapshot back to Rime user dictionary
/// This copies the .txt file to user_dictionaries/ and triggers deploy
#[tauri::command]
pub fn apply_modified_snapshot(dict_id: String, file_name: String) -> Result<(), String> {
    validate_dict_id(&dict_id)?;
    
    // Validate file_name to prevent path traversal
    if file_name.contains("..") {
        return Err("Invalid file name: path traversal not allowed".to_string());
    }
    
    let cfg = RimeConfig::detect();
    
    // Determine snapshot path based on file_name format
    let snapshot_path = if file_name.contains('/') || file_name.contains('\\') {
        // Format: "dict_id/filename.txt" - from userdb directory
        cfg.user_dir.join(&file_name)
    } else {
        // Format: "filename.txt" - from user_dictionaries directory
        cfg.user_dir.join("user_dictionaries").join(&file_name)
    };
    
    // Ensure resolved path is still under user_dir
    if let Ok(canonical) = snapshot_path.canonicalize() {
        let canonical_user_dir = cfg.user_dir.canonicalize().unwrap_or_else(|_| cfg.user_dir.clone());
        if !canonical.starts_with(&canonical_user_dir) {
            return Err("Invalid snapshot path".to_string());
        }
    }
    
    if !snapshot_path.exists() {
        return Err(format!("Snapshot '{}' not found", snapshot_path.display()));
    }
    
    // Create user_dictionaries directory if it doesn't exist
    let user_dicts_dir = cfg.user_dir.join("user_dictionaries");
    std::fs::create_dir_all(&user_dicts_dir).map_err(|e| e.to_string())?;
    
    // Copy snapshot to user_dictionaries/<dict_id>.userdb.txt
    let target_path = user_dicts_dir.join(format!("{}.userdb.txt", dict_id));
    std::fs::copy(&snapshot_path, &target_path).map_err(|e| e.to_string())?;
    
    // Trigger Rime deploy to reload the dictionary
    crate::rime::deployer::deploy().map_err(|e| format!("Deploy failed: {}", e))?;
    
    Ok(())
}
