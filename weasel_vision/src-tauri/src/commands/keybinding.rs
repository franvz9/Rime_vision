use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use ts_rs::TS;

use crate::rime::config::{self, RimeConfig};
use crate::rime::error::RimeError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
pub struct KeyBinding {
    pub when: String,
    pub accept: String,
    #[serde(default)]
    pub send: String,
    #[serde(default)]
    pub toggle: String,
    #[serde(default)]
    pub select: String,
}

#[tauri::command]
pub fn get_keybindings() -> Result<Vec<KeyBinding>, RimeError> {
    let cfg = RimeConfig::detect();
    let dict_value = cfg.load_effective(&cfg.default_path(), &cfg.default_custom_path())?;
    let dict = config::value_as_mapping(&dict_value);

    let mut bindings = Vec::new();
    let key_binder = config::get_mapping(dict, "key_binder");
    let bindings_seq = config::get_sequence(key_binder, "bindings");

    for item in bindings_seq {
        if let Some(mapping) = item.as_mapping() {
            let when = mapping
                .get(Value::String("when".into()))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let accept = mapping
                .get(Value::String("accept".into()))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if !when.is_empty() && !accept.is_empty() {
                bindings.push(KeyBinding {
                    when,
                    accept,
                    send: mapping
                        .get(Value::String("send".into()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    toggle: mapping
                        .get(Value::String("toggle".into()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    select: mapping
                        .get(Value::String("select".into()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
    }

    Ok(bindings)
}

#[tauri::command]
pub fn save_keybindings(bindings: Vec<KeyBinding>) -> Result<(), RimeError> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.default_custom_path(), |patch| {
        let bindings_seq: Vec<Value> = bindings
            .iter()
            .map(|b| {
                let mut item = serde_yaml::Mapping::new();
                item.insert(Value::String("when".into()), Value::String(b.when.clone()));
                item.insert(
                    Value::String("accept".into()),
                    Value::String(b.accept.clone()),
                );
                if !b.send.is_empty() {
                    item.insert(Value::String("send".into()), Value::String(b.send.clone()));
                }
                if !b.toggle.is_empty() {
                    item.insert(
                        Value::String("toggle".into()),
                        Value::String(b.toggle.clone()),
                    );
                }
                if !b.select.is_empty() {
                    item.insert(
                        Value::String("select".into()),
                        Value::String(b.select.clone()),
                    );
                }
                Value::Mapping(item)
            })
            .collect();

        patch.insert(
            Value::String("key_binder/bindings".into()),
            Value::Sequence(bindings_seq),
        );
        remove_nested_patch_value(patch, "key_binder", "bindings");

        Ok(())
    })?;
    Ok(())
}

fn remove_nested_patch_value(patch: &mut serde_yaml::Mapping, section: &str, key: &str) {
    let section_key = Value::String(section.into());
    if let Some(section_value) = patch.get_mut(&section_key) {
        if let Some(section_map) = section_value.as_mapping_mut() {
            section_map.remove(Value::String(key.into()));
            if section_map.is_empty() {
                patch.remove(&section_key);
            }
        }
    }
}
