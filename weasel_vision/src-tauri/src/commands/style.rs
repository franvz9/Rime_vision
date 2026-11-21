//! Tauri IPC 命令薄壳：实际逻辑见 rime-core::service::style。
use rime_core::RimeError;
use rime_core::rime::color_scheme::RimeColorScheme;
use rime_core::rime::style::RimeStyle;
use rime_core::service::style as svc;

pub use rime_core::service::style::StyleData;

#[tauri::command]
pub fn get_style_data() -> Result<StyleData, RimeError> {
    svc::get_style_data()
}

#[tauri::command]
pub fn save_style(new_style: RimeStyle) -> Result<(), RimeError> {
    svc::save_style(new_style)
}

#[tauri::command]
pub fn save_selected_schemes(light: String, dark: String) -> Result<(), RimeError> {
    svc::save_selected_schemes(light, dark)
}

#[tauri::command]
pub fn save_color_scheme(
    name: String,
    scheme: RimeColorScheme,
    original_name: Option<String>,
) -> Result<(), RimeError> {
    svc::save_color_scheme(name, scheme, original_name)
}

#[tauri::command]
pub fn delete_color_scheme(name: String) -> Result<(), RimeError> {
    svc::delete_color_scheme(name)
}

#[tauri::command]
pub fn import_color_scheme(file_path: String) -> Result<(), RimeError> {
    svc::import_color_scheme(file_path)
}
