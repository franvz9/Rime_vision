use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::rime::config::{self, RimeConfig};
use crate::rime::schema::RimeSchema;

#[derive(Serialize, Deserialize)]
pub struct SchemaListData {
    pub schemas: Vec<RimeSchema>,
    pub page_size: i64,
}

#[tauri::command]
pub fn get_schemas() -> Result<SchemaListData, String> {
    let cfg = RimeConfig::detect();
    let dict_value = cfg
        .load_effective(&cfg.default_path(), &cfg.default_custom_path())
        .map_err(|e| e.to_string())?;
    let dict = config::value_as_mapping(&dict_value);

    let mut schemas = Vec::new();
    let schema_list = config::get_sequence(dict, "schema_list");
    for item in schema_list {
        if let Some(schema_id) = item.get(&Value::String("schema".into())) {
            if let Some(id) = schema_id.as_str() {
                schemas.push(RimeSchema::new(id.to_string()));
            }
        }
    }

    let page_size = config::get_mapping(&dict, "menu")
        .get(&Value::String("page_size".into()))
        .and_then(|v| v.as_i64())
        .unwrap_or(6);

    Ok(SchemaListData {
        schemas,
        page_size,
    })
}

#[tauri::command]
pub fn save_schemas(schemas: Vec<RimeSchema>, page_size: Option<i64>) -> Result<(), String> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.default_custom_path(), |patch| {
        let schema_list: Vec<Value> = schemas
            .iter()
            .filter(|s| s.enabled)
            .map(|s| {
                let mut item = serde_yaml::Mapping::new();
                item.insert(
                    Value::String("schema".into()),
                    Value::String(s.schema_id.clone()),
                );
                Value::Mapping(item)
            })
            .collect();

        patch.insert(
            Value::String("schema_list".into()),
            Value::Sequence(schema_list),
        );

        if let Some(ps) = page_size {
            let mut menu = patch
                .get(&Value::String("menu".into()))
                .and_then(|v| v.as_mapping())
                .cloned()
                .unwrap_or_default();
            menu.insert(
                Value::String("page_size".into()),
                Value::Number(ps.into()),
            );
            patch.insert(Value::String("menu".into()), Value::Mapping(menu));
        }

        Ok(())
    })
    .map_err(|e| e.to_string())
}
