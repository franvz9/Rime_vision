mod commands;
mod rime;

use commands::{backup, dict, grammar, keybinding, punctuation, settings, schema, style, sync};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
            backup::list_backups,
            backup::get_backup_detail,
            backup::create_backup,
            backup::restore_backup,
            backup::compare_backup,
            backup::delete_backup,
            dict::list_user_dictionaries,
            dict::load_user_dict_entries,
            dict::list_snapshots,
            dict::delete_entries,
            dict::update_entry_frequency,
            dict::batch_delete_low_frequency,
            dict::export_user_dict,
            dict::clear_user_dict,
            sync::get_sync_settings,
            sync::save_sync_settings,
            sync::get_sync_status,
            sync::list_synced_devices,
            sync::execute_sync,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
