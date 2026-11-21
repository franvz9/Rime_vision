//! Tauri IPC 命令薄壳：实际逻辑见 rime-core::service::backup。
use rime_core::RimeError;
use rime_core::service::backup as svc;

pub use rime_core::service::backup::{BackupDetail, BackupInfo, FileDiff};

#[tauri::command]
pub fn list_backups() -> Result<Vec<BackupInfo>, RimeError> {
    svc::list_backups()
}

#[tauri::command]
pub fn get_backup_detail(backup_id: String) -> Result<BackupDetail, RimeError> {
    svc::get_backup_detail(backup_id)
}

#[tauri::command]
pub fn create_backup(
    categories: Vec<String>,
    note: Option<String>,
) -> Result<BackupInfo, RimeError> {
    svc::create_backup(categories, note)
}

#[tauri::command]
pub fn restore_backup(
    backup_id: String,
    restore_files: Vec<String>,
    confirm: bool,
) -> Result<(), RimeError> {
    svc::restore_backup(backup_id, restore_files, confirm)
}

#[tauri::command]
pub fn compare_backup(backup_id: String, file_name: String) -> Result<FileDiff, RimeError> {
    svc::compare_backup(backup_id, file_name)
}

#[tauri::command]
pub fn delete_backup(backup_id: String) -> Result<(), RimeError> {
    svc::delete_backup(backup_id)
}
