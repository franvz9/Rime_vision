use std::path::Path;

/// Calculate total size of a directory recursively
pub fn dir_size(dir: &Path) -> i64 {
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

/// Count files in a directory recursively
pub fn count_files(dir: &Path) -> usize {
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
