use std::path::{Path, PathBuf};

use anyhow::Result;
use parking_lot::Mutex;

#[derive(Debug, PartialEq)]
pub enum WriteResult {
    Unchanged,
    Written,
}

/// Global lock to prevent concurrent writes to the same file
static WRITE_LOCK: Mutex<()> = Mutex::new(());

pub fn write_if_changed(content: &str, path: &Path) -> Result<WriteResult> {
    let _guard = WRITE_LOCK.lock();

    if path.exists() {
        let existing = std::fs::read_to_string(path)?;
        if existing == content {
            return Ok(WriteResult::Unchanged);
        }
        let backup = timestamped_backup(path);
        std::fs::copy(path, &backup)?;
    } else if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Atomic write: write to temp file then rename
    let temp_path = path.with_file_name(format!("{}.tmp", uuid::Uuid::new_v4()));
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
