mod commands;
// 核心逻辑在 rime-core（workspace 成员）：commands/* 仅是 #[tauri::command] 委托薄壳。
use commands::{backup, dict, grammar, keybinding, page_size, punctuation, schema, settings, style, sync, translator};

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
            schema::set_current_schema,
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
            punctuation::get_punctuation_profile,
            punctuation::save_punctuation,
            page_size::get_page_size_detail,
            page_size::save_page_size,
            translator::get_translator_profile,
            translator::save_translator,
            translator::reset_translator,
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
        // service::backup
        rime_core::service::backup::BackupInfo::export().unwrap();
        rime_core::service::backup::BackupDetail::export().unwrap();
        rime_core::service::backup::BackupFile::export().unwrap();
        rime_core::service::backup::FileDiff::export().unwrap();

        // service::dict
        rime_core::service::dict::UserDictInfo::export().unwrap();
        rime_core::service::dict::DictEntry::export().unwrap();
        rime_core::service::dict::DictEntriesResult::export().unwrap();
        rime_core::service::dict::SnapshotInfo::export().unwrap();
        rime_core::service::dict::NewDictEntry::export().unwrap();

        // service::sync
        rime_core::service::sync::SyncSettings::export().unwrap();
        rime_core::service::sync::SyncStatus::export().unwrap();
        rime_core::service::sync::SyncedDevice::export().unwrap();
        rime_core::service::sync::SyncResult::export().unwrap();

        // service::translator
        rime_core::service::translator::TranslatorSettings::export().unwrap();
        rime_core::service::translator::TranslatorProfile::export().unwrap();
        rime_core::service::translator::TranslatorPatch::export().unwrap();

        // service::keybinding
        rime_core::service::keybinding::KeyBinding::export().unwrap();

        // service::punctuation
        rime_core::service::punctuation::PunctRule::export().unwrap();
        rime_core::service::punctuation::PunctuationData::export().unwrap();
        rime_core::service::punctuation::PunctuationProfile::export().unwrap();

        // service::page_size
        rime_core::service::page_size::PageSizeDetail::export().unwrap();

        // service::schema
        rime_core::service::schema::SchemaListData::export().unwrap();

        // service::settings
        rime_core::service::settings::GeneralSettings::export().unwrap();
        rime_core::service::settings::ConfigFileInfo::export().unwrap();

        // service::grammar
        rime_core::service::grammar::GrammarData::export().unwrap();

        // service::style
        rime_core::service::style::StyleData::export().unwrap();

        // rime modules
        rime_core::rime::color::RimeColor::export().unwrap();
        rime_core::rime::color_scheme::RimeColorScheme::export().unwrap();
        rime_core::rime::style::RimeStyle::export().unwrap();
        rime_core::rime::schema::RimeSchema::export().unwrap();
        rime_core::rime::grammar::GrammarModel::export().unwrap();
        rime_core::rime::grammar::SchemaGrammarConfig::export().unwrap();
    }
}
