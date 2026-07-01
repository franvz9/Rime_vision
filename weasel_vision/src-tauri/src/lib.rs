mod commands;
mod rime;

use commands::{backup, dict, grammar, keybinding, punctuation, settings, schema, style, sync};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            style::get_style_data,
            style::save_style,
            style::save_selected_schemes,
            style::save_color_scheme,
            style::delete_color_scheme,
            style::import_color_scheme,
            schema::get_schemas,
            schema::save_schemas,
            schema::import_schema,
            keybinding::get_keybindings,
            keybinding::save_keybindings,
            settings::get_general_settings,
            settings::save_general_settings,
            settings::get_rime_user_dir,
            settings::get_config_files,
            settings::open_dir,
            settings::open_rime_dir,
            settings::deploy,
            settings::sync_build_dir,
            settings::reset_config,
            grammar::get_grammar_data,
            grammar::mount_grammar,
            grammar::unmount_grammar,
            grammar::import_grammar,
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
            dict::add_dict_entry,
            dict::create_snapshot,
            dict::delete_snapshot,
            dict::apply_modified_snapshot,
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

#[cfg(test)]
mod export_tests {
    use ts_rs::TS;

    #[test]
    fn export_all_bindings() {
        // commands::backup
        crate::commands::backup::BackupInfo::export().unwrap();
        crate::commands::backup::BackupDetail::export().unwrap();
        crate::commands::backup::BackupFile::export().unwrap();
        crate::commands::backup::FileDiff::export().unwrap();

        // commands::dict
        crate::commands::dict::UserDictInfo::export().unwrap();
        crate::commands::dict::DictEntry::export().unwrap();
        crate::commands::dict::DictEntriesResult::export().unwrap();
        crate::commands::dict::SnapshotInfo::export().unwrap();
        crate::commands::dict::NewDictEntry::export().unwrap();

        // commands::sync
        crate::commands::sync::SyncSettings::export().unwrap();
        crate::commands::sync::SyncStatus::export().unwrap();
        crate::commands::sync::SyncedDevice::export().unwrap();
        crate::commands::sync::SyncResult::export().unwrap();

        // commands::keybinding
        crate::commands::keybinding::KeyBinding::export().unwrap();

        // commands::punctuation
        crate::commands::punctuation::PunctRule::export().unwrap();
        crate::commands::punctuation::PunctuationData::export().unwrap();

        // commands::schema
        crate::commands::schema::SchemaListData::export().unwrap();

        // commands::settings
        crate::commands::settings::GeneralSettings::export().unwrap();
        crate::commands::settings::ConfigFileInfo::export().unwrap();

        // commands::grammar
        crate::commands::grammar::GrammarData::export().unwrap();

        // commands::style
        crate::commands::style::StyleData::export().unwrap();

        // rime modules
        crate::rime::color::RimeColor::export().unwrap();
        crate::rime::color_scheme::RimeColorScheme::export().unwrap();
        crate::rime::style::RimeStyle::export().unwrap();
        crate::rime::schema::RimeSchema::export().unwrap();
        crate::rime::grammar::GrammarModel::export().unwrap();
        crate::rime::grammar::SchemaGrammarConfig::export().unwrap();
    }
}
