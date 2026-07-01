use std::path::Path;

/// Validates that a path string is safe for use within a trusted base directory.
///
/// This function checks the overall path string for dangerous patterns but does
/// **not** reject `/` characters, because subdirectory structures are intentionally
/// supported (e.g. `subdir/file.yaml`, `lua/aux_code/filter.lua`).
///
/// # Rejected patterns
/// - Empty paths
/// - Paths longer than 512 bytes
/// - Paths containing `..` (directory traversal)
/// - Paths containing null bytes or control characters (`\0`, `\n`, `\r`, `\t`)
///
/// # Allowed patterns
/// - `/` separators (supports subdirectory paths like `build/info.yaml`)
/// - Dots in file names (e.g. `custom.yaml`, `rime_mint.custom.yaml`)
///
/// # Note
/// If you need to validate a single file name (without `/`), perform an
/// additional check to ensure the input does not contain `/`.
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

/// Maximum file size for read operations (10 MB) to prevent memory exhaustion.
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Read a file to string with a size limit to prevent memory exhaustion.
pub fn read_file_with_limit(path: &Path) -> Result<String, crate::rime::error::RimeError> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(crate::rime::error::RimeError::Io(format!(
            "File too large: {} ({} bytes, max {} bytes)",
            path.display(),
            metadata.len(),
            MAX_FILE_SIZE
        )));
    }
    std::fs::read_to_string(path).map_err(crate::rime::error::RimeError::from)
}

/// Maximum recursion depth for directory traversal to prevent stack overflow
/// from symlink cycles or excessively deep directory trees.
pub const MAX_DIR_DEPTH: u32 = 32;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_path() {
        assert!(validate_safe_path("").is_err());
    }

    #[test]
    fn test_path_traversal() {
        assert!(validate_safe_path("../etc/passwd").is_err());
        assert!(validate_safe_path("foo/../bar").is_err());
    }

    #[test]
    fn test_control_characters() {
        assert!(validate_safe_path("test\0file").is_err());
        assert!(validate_safe_path("test\nfile").is_err());
        assert!(validate_safe_path("test\rfile").is_err());
        assert!(validate_safe_path("test\tfile").is_err());
    }

    #[test]
    fn test_long_path() {
        let long_path = "a".repeat(513);
        assert!(validate_safe_path(&long_path).is_err());
    }

    #[test]
    fn test_valid_paths() {
        assert!(validate_safe_path("normal_file.yaml").is_ok());
        assert!(validate_safe_path("subdir/file.txt").is_ok());
        assert!(validate_safe_path("file-name_with.dots.yaml").is_ok());
    }
}
