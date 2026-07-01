use std::path::{Path, PathBuf};

use anyhow::Result;
use parking_lot::Mutex;

#[derive(Debug, PartialEq)]
pub enum WriteResult {
    Unchanged,
    Written,
}

/// Global lock to prevent concurrent writes to the same file.
///
/// This uses `parking_lot::Mutex` (a synchronous lock) because all file I/O
/// operations it protects are inherently blocking. For Tauri async commands
/// that need to use these functions, use `tokio::task::spawn_blocking` or
/// the provided `write_atomic_async` / `write_if_changed_async` wrappers.
static WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Write content to a file atomically (temp + rename) with global lock.
/// Use this when you know the content has changed and don't need the read-compare step.
pub fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let _guard = WRITE_LOCK.lock();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Atomic write: write to temp file then rename
    let temp_name = format!(".{}.tmp", uuid::Uuid::new_v4());
    let temp_path = path.parent().unwrap_or(Path::new(".")).join(&temp_name);
    std::fs::write(&temp_path, content)?;
    // Try direct rename first (works on Windows 10 1607+ and Unix)
    // If it fails (older Windows), fall back to remove + rename
    if std::fs::rename(&temp_path, path).is_err() {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        std::fs::rename(&temp_path, path)?;
    }
    Ok(())
}

pub fn write_if_changed(content: &str, path: &Path) -> Result<WriteResult> {
    let _guard = WRITE_LOCK.lock();

    if path.exists() {
        let existing = std::fs::read_to_string(path)?;
        if existing == content {
            return Ok(WriteResult::Unchanged);
        }
        let backup = timestamped_backup(path);
        std::fs::copy(path, &backup)?;
        cleanup_timestamped_backups(path);
    } else if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Atomic write: write to temp file then rename
    let temp_name = format!(".{}.tmp", uuid::Uuid::new_v4());
    let temp_path = path.parent().unwrap_or(Path::new(".")).join(&temp_name);
    std::fs::write(&temp_path, content)?;
    // Try direct rename first (works on Windows 10 1607+ and Unix)
    // If it fails (older Windows), fall back to remove + rename
    if std::fs::rename(&temp_path, path).is_err() {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        std::fs::rename(&temp_path, path)?;
    }
    Ok(WriteResult::Written)
}

/// Async wrapper for write_if_changed — use this from Tauri async commands
/// to avoid blocking the Tokio runtime thread.
#[allow(dead_code)] // Reserved for future async command migration
pub async fn write_if_changed_async(content: String, path: PathBuf) -> Result<WriteResult> {
    tokio::task::spawn_blocking(move || write_if_changed(&content, &path))
        .await
        .map_err(|e| anyhow::anyhow!("spawn_blocking failed: {}", e))?
}

/// Maximum number of timestamped backup files to retain per original file.
const MAX_TIMESTAMPED_BACKUPS: usize = 10;

fn timestamped_backup(path: &Path) -> PathBuf {
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S%.3f");
    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    path.with_file_name(format!("{}.{}{}", stem, ts, ext))
}

/// Remove oldest timestamped backups when the count exceeds `MAX_TIMESTAMPED_BACKUPS`.
/// Backup file names contain timestamps so lexicographic order = chronological order.
fn cleanup_timestamped_backups(original_path: &Path) {
    let Some(parent) = original_path.parent() else { return };
    let stem = original_path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = original_path.extension().unwrap_or_default().to_string_lossy();

    // Collect timestamped backup files matching the pattern: <stem>.<timestamp>.<ext>
    let mut backups: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Pattern: stem.YYYYMMDD-HHMMSSmmm.ext
            if name.starts_with(&format!("{}.", stem))
                && (ext.is_empty() || name.ends_with(&format!(".{}", ext)))
                && name.len() > stem.len() + 1 // has timestamp suffix
            {
                let between = if ext.is_empty() {
                    &name[stem.len() + 1..]
                } else {
                    &name[stem.len() + 1..name.len() - ext.len() - 1]
                };
                // Verify it looks like a timestamp (starts with digit, contains dash)
                if between.starts_with(|c: char| c.is_ascii_digit()) && between.contains('-') {
                    backups.push((name, entry.path()));
                }
            }
        }
    }

    // Sort by name descending (newest first)
    backups.sort_by(|a, b| b.0.cmp(&a.0));

    // Remove oldest backups beyond the limit
    for (name, path) in backups.iter().skip(MAX_TIMESTAMPED_BACKUPS) {
        if let Err(e) = std::fs::remove_file(path) {
            eprintln!("Warning: failed to remove old backup {}: {}", name, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_write_if_changed_new_file() {
        let dir = std::env::temp_dir().join("weasel_vision_test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_new.yaml");

        let result = write_if_changed("hello", &path).unwrap();
        assert_eq!(result, WriteResult::Written);
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_write_if_changed_unchanged() {
        let dir = std::env::temp_dir().join("weasel_vision_test2");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_unchanged.yaml");

        fs::write(&path, "hello").unwrap();
        let result = write_if_changed("hello", &path).unwrap();
        assert_eq!(result, WriteResult::Unchanged);

        fs::remove_dir_all(&dir).unwrap();
    }
}
