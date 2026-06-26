use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::rime::config::RimeConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDictInfo {
    pub dict_id: String,
    pub display_name: String,
    pub schema_ids: Vec<String>,
    pub entry_count: usize,
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
    let frequency = parts[2].parse::<i64>().unwrap_or(0);
    let commit_count = parts.get(3).and_then(|s| s.parse::<i64>().ok()).unwrap_or(frequency);

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
    let snapshot = find_snapshot(&dict_id)?;
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

    let snapshots_dir = user_dict_snapshots_dir();
    if snapshots_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&format!("{}.", dict_id)) && name.ends_with(".txt") {
                    let meta = std::fs::metadata(entry.path()).ok();
                    snapshots.push(SnapshotInfo {
                        file_name: name,
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
                }
            }
        }
    }

    let userdb_dir = cfg.user_dir.join(format!("{}.userdb", dict_id));
    if let Ok(entries) = std::fs::read_dir(&userdb_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".txt") {
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
    let mut candidates: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

    let snapshots_dir = user_dict_snapshots_dir();
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
        .ok_or_else(|| format!("No snapshot found for dict '{}'", dict_id))
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
                    // Preserve extra fields (commit_count, etc.) by only replacing frequency
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() > 3 {
                        return format!(
                            "{}\t{}\t{}{}",
                            entry.word,
                            entry.code,
                            new_freq,
                            parts[3..].iter().map(|p| format!("\t{}", p)).collect::<String>()
                        );
                    }
                    return format!("{}\t{}\t{}", entry.word, entry.code, new_freq);
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

#[tauri::command]
pub fn clear_user_dict(dict_id: String) -> Result<(), String> {
    validate_dict_id(&dict_id)?;
    let cfg = RimeConfig::detect();
    let userdb_dir = cfg.user_dir.join(format!("{}.userdb", dict_id));

    if !userdb_dir.exists() {
        return Err(format!("Dictionary '{}' not found", dict_id));
    }

    for entry in std::fs::read_dir(&userdb_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !name.starts_with('.') {
                    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
                }
            }
        }
    }

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
