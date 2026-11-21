//! Tauri IPC 命令薄壳：实际逻辑见 rime-core::service::grammar。
use rime_core::RimeError;
use rime_core::rime::grammar::SchemaGrammarConfig;
use rime_core::service::grammar as svc;

pub use rime_core::service::grammar::GrammarData;

#[tauri::command]
pub fn get_grammar_data(schema_ids: Vec<String>) -> Result<GrammarData, RimeError> {
    svc::get_grammar_data(schema_ids)
}

#[tauri::command]
pub fn mount_grammar(
    model_filename: String,
    schema_id: String,
    config: SchemaGrammarConfig,
) -> Result<(), RimeError> {
    svc::mount_grammar(model_filename, schema_id, config)
}

#[tauri::command]
pub fn unmount_grammar(schema_id: String) -> Result<(), RimeError> {
    svc::unmount_grammar(schema_id)
}

#[tauri::command]
pub fn import_grammar(file_path: String) -> Result<(), RimeError> {
    svc::import_grammar(file_path)
}
