//! Tauri IPC 命令薄壳：实际逻辑见 rime-core::service::settings。
use rime_core::RimeError;
use rime_core::service::settings as svc;

pub use rime_core::service::settings::{ConfigFileInfo, GeneralSettings, PendingDelete};

#[tauri::command]
pub fn get_general_settings() -> Result<GeneralSettings, RimeError> {
    svc::get_general_settings()
}

#[tauri::command]
pub fn save_general_settings(settings: GeneralSettings) -> Result<(), RimeError> {
    svc::save_general_settings(settings)
}

#[tauri::command]
pub fn get_rime_user_dir() -> Result<String, RimeError> {
    svc::get_rime_user_dir()
}

#[tauri::command]
pub fn get_config_files() -> Result<Vec<ConfigFileInfo>, RimeError> {
    svc::get_config_files()
}

#[tauri::command]
pub fn open_dir(path: String) -> Result<(), RimeError> {
    svc::open_dir(path)
}

#[tauri::command]
pub fn open_rime_dir() -> Result<(), RimeError> {
    svc::open_rime_dir()
}

#[tauri::command]
pub async fn deploy(pending_deletes: Option<Vec<PendingDelete>>) -> Result<(), RimeError> {
    // 阻塞 I/O 与子进程放到独立线程，与原实现行为一致
    tauri::async_runtime::spawn_blocking(move || svc::deploy(pending_deletes))
        .await
        .map_err(|e| RimeError::Unknown(format!("deploy task join error: {}", e)))?
}

#[tauri::command]
pub fn sync_build_dir() -> Result<(), RimeError> {
    svc::sync_build_dir()
}

#[tauri::command]
pub fn reset_config() -> Result<(), RimeError> {
    svc::reset_config()
}
