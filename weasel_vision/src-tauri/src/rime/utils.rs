use std::path::Path;

/// Unified path validation: rejects empty, excessively long, path-traversal (`..`),
/// null bytes, and control characters. Used by both backup and dict modules.
pub fn validate_safe_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("路径为空".to_string());
    }
    if path.len() > 512 {
        return Err("路径过长".to_string());
    }
    if path.contains("..") {
        return Err("路径包含路径遍历字符".to_string());
    }
    if path.contains('\0') || path.contains('\n') || path.contains('\r') || path.contains('\t') {
        return Err("路径包含控制字符".to_string());
    }
    Ok(())
}

/// Maximum recursion depth for directory traversal to prevent stack overflow
/// from symlink cycles or excessively deep directory trees.
const MAX_DIR_DEPTH: u32 = 32;

/// Calculate total size of a directory recursively
pub fn dir_size(dir: &Path) -> i64 {
    let mut size = 0i64;
    dir_size_recursive(dir, &mut size, 0);
    size
}

fn dir_size_recursive(dir: &Path, size: &mut i64, depth: u32) {
    if depth > MAX_DIR_DEPTH {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Skip symbolic links to prevent infinite loops from symlink cycles
            if path.is_symlink() {
                continue;
            }
            if path.is_file() {
                if let Ok(m) = std::fs::metadata(&path) {
                    *size += m.len() as i64;
                }
            } else if path.is_dir() {
                dir_size_recursive(&path, size, depth + 1);
            }
        }
    }
}

/// Count files in a directory recursively
pub fn count_files(dir: &Path) -> usize {
    let mut count = 0;
    count_files_recursive(dir, &mut count, 0);
    count
}

fn count_files_recursive(dir: &Path, count: &mut usize, depth: u32) {
    if depth > MAX_DIR_DEPTH {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Skip symbolic links to prevent infinite loops from symlink cycles
            if path.is_symlink() {
                continue;
            }
            if path.is_file() {
                *count += 1;
            } else if path.is_dir() {
                count_files_recursive(&path, count, depth + 1);
            }
        }
    }
}
