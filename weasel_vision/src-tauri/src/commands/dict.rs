//! Tauri IPC 命令薄壳：实际逻辑见 rime-core::service::dict。
use rime_core::RimeError;
use rime_core::service::dict as svc;

pub use rime_core::service::dict::{
    DictEntriesResult, DictEntryKey, NewDictEntry, SnapshotInfo, UserDictInfo,
};

#[tauri::command]
pub fn list_user_dictionaries() -> Result<Vec<UserDictInfo>, RimeError> {
    svc::list_user_dictionaries()
}

#[tauri::command]
pub fn load_user_dict_entries(
    dict_id: String,
    page: usize,
    per_page: usize,
    sort_by: String,
    search: Option<String>,
) -> Result<DictEntriesResult, RimeError> {
    svc::load_user_dict_entries(dict_id, page, per_page, sort_by, search)
}

#[tauri::command]
pub fn list_snapshots(dict_id: String) -> Result<Vec<SnapshotInfo>, RimeError> {
    svc::list_snapshots(dict_id)
}

#[tauri::command]
pub fn delete_entries(
    dict_id: String,
    entries_to_delete: Vec<DictEntryKey>,
) -> Result<usize, RimeError> {
    svc::delete_entries(dict_id, entries_to_delete)
}

#[tauri::command]
pub fn update_entry_frequency(
    dict_id: String,
    word: String,
    code: String,
    new_freq: i64,
) -> Result<(), RimeError> {
    svc::update_entry_frequency(dict_id, word, code, new_freq)
}

#[tauri::command]
pub fn add_dict_entry(dict_id: String, entry: NewDictEntry) -> Result<(), RimeError> {
    svc::add_dict_entry(dict_id, entry)
}

#[tauri::command]
pub async fn create_snapshot(dict_id: String) -> Result<String, RimeError> {
    // 阻塞 I/O 与子进程放到独立线程，与原实现行为一致
    tauri::async_runtime::spawn_blocking(move || svc::create_snapshot(dict_id))
        .await
        .map_err(|e| RimeError::Unknown(format!("create_snapshot task join error: {}", e)))?
}

#[tauri::command]
pub fn delete_snapshot(dict_id: String, file_name: String) -> Result<(), RimeError> {
    svc::delete_snapshot(dict_id, file_name)
}

#[tauri::command]
pub fn apply_modified_snapshot(dict_id: String, file_name: String) -> Result<(), RimeError> {
    svc::apply_modified_snapshot(dict_id, file_name)
}

#[tauri::command]
pub fn batch_delete_low_frequency(dict_id: String, threshold: i64) -> Result<usize, RimeError> {
    svc::batch_delete_low_frequency(dict_id, threshold)
}

#[tauri::command]
pub fn export_user_dict(dict_id: String, output_path: String) -> Result<usize, RimeError> {
    svc::export_user_dict(dict_id, output_path)
}

#[tauri::command]
pub fn clear_user_dict(dict_id: String) -> Result<(), RimeError> {
    svc::clear_user_dict(dict_id)
}
