//! Tauri IPC 命令薄壳：实际逻辑见 rime-core::service::keybinding。
use rime_core::RimeError;
use rime_core::service::keybinding as svc;

pub use rime_core::service::keybinding::KeyBinding;

#[tauri::command]
pub fn get_keybindings(scope: Option<String>) -> Result<Vec<KeyBinding>, RimeError> {
    svc::get_keybindings(scope)
}

#[tauri::command]
pub fn save_keybindings(
    scope: Option<String>,
    bindings: Vec<KeyBinding>,
) -> Result<(), RimeError> {
    svc::save_keybindings(scope, bindings)
}
