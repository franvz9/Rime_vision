//! Tauri IPC 命令薄壳：实际逻辑见 rime-core::service::sync。
use rime_core::RimeError;
use rime_core::service::sync as svc;

pub use rime_core::service::sync::{SyncResult, SyncSettings, SyncStatus, SyncedDevice};

#[tauri::command]
pub fn get_sync_settings() -> Result<SyncSettings, RimeError> {
    svc::get_sync_settings()
}

#[tauri::command]
pub fn save_sync_settings(settings: SyncSettings) -> Result<(), RimeError> {
    svc::save_sync_settings(settings)
}

#[tauri::command]
pub fn get_sync_status() -> Result<SyncStatus, RimeError> {
    svc::get_sync_status()
}

#[tauri::command]
pub fn list_synced_devices() -> Result<Vec<SyncedDevice>, RimeError> {
    svc::list_synced_devices()
}

#[tauri::command]
pub async fn execute_sync() -> Result<SyncResult, RimeError> {
    // 阻塞等待（≥2s）+ 子进程放到独立线程，与原实现行为一致
    tauri::async_runtime::spawn_blocking(svc::execute_sync)
        .await
        .map_err(|e| RimeError::Unknown(format!("execute_sync task join error: {}", e)))?
}
