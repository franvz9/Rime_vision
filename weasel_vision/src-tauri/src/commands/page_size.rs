//! Tauri IPC 命令薄壳：实际逻辑见 rime-core::service::page_size。
use rime_core::RimeError;
use rime_core::service::page_size as svc;

pub use rime_core::service::page_size::PageSizeDetail;

#[tauri::command]
pub fn get_page_size_detail(scope: Option<String>) -> Result<PageSizeDetail, RimeError> {
    svc::get_page_size_detail(scope)
}

#[tauri::command]
pub fn save_page_size(scope: Option<String>, page_size: i64) -> Result<(), RimeError> {
    svc::save_page_size(scope, page_size)
}
