mod commands;
mod rime;

use commands::{grammar, keybinding, punctuation, settings, schema, style};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            style::get_style_data,
            style::save_style,
            style::save_selected_schemes,
            style::save_color_scheme,
            style::delete_color_scheme,
            schema::get_schemas,
            schema::save_schemas,
            keybinding::get_keybindings,
            keybinding::save_keybindings,
            settings::get_general_settings,
            settings::save_general_settings,
            settings::get_rime_user_dir,
            settings::get_config_files,
            settings::deploy,
            settings::sync,
            settings::reset_config,
            grammar::get_grammar_data,
            grammar::mount_grammar,
            grammar::unmount_grammar,
            punctuation::get_punctuation,
            punctuation::save_punctuation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
