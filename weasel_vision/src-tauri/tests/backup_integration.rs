/// Integration test: backup manifest JSON round-trip and file operations.
///
/// Tests the patterns used by the backup system:
/// - Writing a manifest.json with backup metadata
/// - Reading it back and deserializing
/// - File copy and directory operations
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct BackupManifest {
    id: String,
    created_at: String,
    backup_type: String,
    file_count: usize,
    total_size: i64,
    note: Option<String>,
}

#[test]
fn test_backup_manifest_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let backup_dir = tmp.path().join("20250101-120000.000");
    std::fs::create_dir_all(&backup_dir).unwrap();

    let manifest = BackupManifest {
        id: "20250101-120000.000".to_string(),
        created_at: "2025-01-01T12:00:00+08:00".to_string(),
        backup_type: "manual".to_string(),
        file_count: 42,
        total_size: 1024 * 1024,
        note: Some("Test backup".to_string()),
    };

    let manifest_path = backup_dir.join("manifest.json");
    let json = serde_json::to_string_pretty(&manifest).unwrap();
    std::fs::write(&manifest_path, &json).unwrap();

    // Read back and verify
    let read_back = std::fs::read_to_string(&manifest_path).unwrap();
    let parsed: BackupManifest = serde_json::from_str(&read_back).unwrap();
    assert_eq!(manifest, parsed);
}

#[test]
fn test_backup_manifest_without_note() {
    let manifest = BackupManifest {
        id: "test".to_string(),
        created_at: String::new(),
        backup_type: "deploy".to_string(),
        file_count: 0,
        total_size: 0,
        note: None,
    };

    let json = serde_json::to_string(&manifest).unwrap();
    let parsed: BackupManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(manifest, parsed);
    assert!(parsed.note.is_none());
}

#[test]
fn test_backup_directory_copy_simulation() {
    let tmp = TempDir::new().unwrap();

    // Create source directory with files
    let src = tmp.path().join("source");
    std::fs::create_dir_all(src.join("subdir")).unwrap();
    std::fs::write(src.join("file1.yaml"), "content1").unwrap();
    std::fs::write(src.join("subdir/file2.yaml"), "content2").unwrap();

    // Copy to destination
    let dst = tmp.path().join("backup");
    std::fs::create_dir_all(&dst).unwrap();

    // Recursive copy
    let mut count = 0usize;
    for entry in walkdir(&src) {
        let rel = entry.strip_prefix(&src).unwrap();
        let dst_path = dst.join(rel);
        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::copy(&entry, &dst_path).unwrap();
        count += 1;
    }

    assert_eq!(count, 2);
    assert_eq!(
        std::fs::read_to_string(dst.join("file1.yaml")).unwrap(),
        "content1"
    );
    assert_eq!(
        std::fs::read_to_string(dst.join("subdir/file2.yaml")).unwrap(),
        "content2"
    );
}

/// Simple recursive file walker for test purposes.
fn walkdir(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                files.extend(walkdir(&path));
            }
        }
    }
    files
}

#[test]
fn test_backup_id_validation() {
    // Valid IDs
    assert!(is_valid_backup_id("20250101-120000"));
    assert!(is_valid_backup_id("deploy_20250101"));
    assert!(is_valid_backup_id("abc123"));

    // Invalid IDs
    assert!(!is_valid_backup_id("")); // empty
    assert!(!is_valid_backup_id("../escape")); // path traversal
    assert!(!is_valid_backup_id("has/slash")); // slash
    assert!(!is_valid_backup_id("has space")); // space
    assert!(!is_valid_backup_id(&"a".repeat(65))); // too long
}

fn is_valid_backup_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 64 {
        return false;
    }
    id.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
