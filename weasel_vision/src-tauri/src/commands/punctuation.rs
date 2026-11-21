//! Tauri IPC 命令薄壳：实际逻辑见 rime-core::service::punctuation。
use rime_core::RimeError;
use rime_core::service::punctuation as svc;

pub use rime_core::service::punctuation::{PunctRule, PunctuationData, PunctuationProfile};

#[tauri::command]
pub fn get_punctuation(scope: Option<String>) -> Result<PunctuationData, RimeError> {
    svc::get_punctuation(scope)
}

#[tauri::command]
pub fn get_punctuation_profile(scope: Option<String>) -> Result<PunctuationProfile, RimeError> {
    svc::get_punctuation_profile(scope)
}

#[tauri::command]
pub fn save_punctuation(
    scope: Option<String>,
    half: Vec<PunctRule>,
    full: Vec<PunctRule>,
) -> Result<(), RimeError> {
    svc::save_punctuation(scope, half, full)
}
