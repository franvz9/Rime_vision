//! Tauri IPC 命令薄壳：实际逻辑见 rime-core::service::translator。
use rime_core::RimeError;
use rime_core::service::translator as svc;

pub use rime_core::service::translator::{TranslatorPatch, TranslatorProfile};

#[tauri::command]
pub fn get_translator_profile(scope: Option<String>) -> Result<TranslatorProfile, RimeError> {
    svc::get_translator_profile(scope)
}

#[tauri::command]
pub fn save_translator(scope: Option<String>, patch: TranslatorPatch) -> Result<(), RimeError> {
    svc::save_translator(scope, patch)
}

#[tauri::command]
pub fn reset_translator(scope: Option<String>) -> Result<(), RimeError> {
    svc::reset_translator(scope)
}
