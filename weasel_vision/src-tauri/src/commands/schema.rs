//! Tauri IPC 命令薄壳：实际逻辑见 rime-core::service::schema。
use rime_core::RimeError;
use rime_core::rime::schema::RimeSchema;
use rime_core::service::schema as svc;

pub use rime_core::service::schema::SchemaListData;

#[tauri::command]
pub fn get_schemas() -> Result<SchemaListData, RimeError> {
    svc::get_schemas()
}

#[tauri::command]
pub fn save_schemas(schemas: Vec<RimeSchema>) -> Result<(), RimeError> {
    svc::save_schemas(schemas)
}

#[tauri::command]
pub fn import_schema(file_path: String) -> Result<(), RimeError> {
    svc::import_schema(file_path)
}

#[tauri::command]
pub fn set_current_schema(schema_id: String) -> Result<(), RimeError> {
    svc::set_current_schema(schema_id)
}
