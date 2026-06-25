use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};

use super::backup;
use super::patch;

#[derive(Debug, Clone)]
pub struct RimeConfig {
    pub user_dir: PathBuf,
    pub style_file: String,
    pub style_custom: String,
    pub default_yaml: String,
    pub default_custom: String,
}

impl RimeConfig {
    pub fn detect() -> Self {
        if cfg!(target_os = "macos") {
            Self {
                user_dir: dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("~"))
                    .join("Library/Rime"),
                style_file: "squirrel.yaml".into(),
                style_custom: "squirrel.custom.yaml".into(),
                default_yaml: "default.yaml".into(),
                default_custom: "default.custom.yaml".into(),
            }
        } else if cfg!(target_os = "windows") {
            Self {
                user_dir: dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("Rime"),
                style_file: "weasel.yaml".into(),
                style_custom: "weasel.custom.yaml".into(),
                default_yaml: "default.yaml".into(),
                default_custom: "default.custom.yaml".into(),
            }
        } else {
            Self {
                user_dir: dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("Rime"),
                style_file: "rime.yaml".into(),
                style_custom: "rime.custom.yaml".into(),
                default_yaml: "default.yaml".into(),
                default_custom: "default.custom.yaml".into(),
            }
        }
    }

    pub fn style_path(&self) -> PathBuf {
        self.user_dir.join(&self.style_file)
    }

    pub fn style_custom_path(&self) -> PathBuf {
        self.user_dir.join(&self.style_custom)
    }

    pub fn default_path(&self) -> PathBuf {
        self.user_dir.join(&self.default_yaml)
    }

    pub fn default_custom_path(&self) -> PathBuf {
        self.user_dir.join(&self.default_custom)
    }

    pub fn schema_custom_path(&self, schema_id: &str) -> PathBuf {
        self.user_dir.join(format!("{}.custom.yaml", schema_id))
    }

    pub fn load_yaml(&self, path: &Path) -> Result<Value> {
        if !path.exists() {
            return Ok(Value::Mapping(Mapping::new()));
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        parse_yaml(&content)
    }

    pub fn load_effective(&self, base_path: &Path, custom_path: &Path) -> Result<Value> {
        let base = self.load_yaml(base_path)?;
        let custom = match self.load_yaml(custom_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "Warning: failed to parse {}: {}, using base config only",
                    custom_path.display(),
                    e
                );
                return Ok(base);
            }
        };

        if let Some(patch_map) = custom.get(&Value::String("patch".into())) {
            if let Some(patch_mapping) = patch_map.as_mapping() {
                if let Some(base_mapping) = base.as_mapping() {
                    return Ok(Value::Mapping(patch::merge(base_mapping, patch_mapping)));
                }
            }
        }

        Ok(base)
    }

    pub fn save_patch<F>(&self, custom_path: &Path, mutate: F) -> Result<()>
    where
        F: FnOnce(&mut Mapping) -> Result<()>,
    {
        let existing = self.load_yaml(custom_path)?;
        let mut patch_map = existing
            .get(&Value::String("patch".into()))
            .and_then(|v| v.as_mapping())
            .cloned()
            .unwrap_or_default();

        mutate(&mut patch_map)?;

        let mut root = Mapping::new();
        root.insert(
            Value::String("patch".into()),
            Value::Mapping(patch_map),
        );

        let content = serde_yaml::to_string(&root)?;
        backup::write_if_changed(&content, custom_path)?;
        Ok(())
    }
}

pub fn parse_yaml(text: &str) -> Result<Value> {
    let value: Value = serde_yaml::from_str(text)?;
    Ok(normalize(value))
}

#[allow(dead_code)]
pub fn dump_yaml(value: &Value) -> Result<String> {
    Ok(serde_yaml::to_string(value)?)
}

fn normalize(value: Value) -> Value {
    match value {
        Value::Mapping(map) => {
            let mut result = Mapping::new();
            for (k, v) in map {
                let key = match k {
                    Value::String(s) => s,
                    other => format!("{:?}", other),
                };
                result.insert(Value::String(key), normalize(v));
            }
            Value::Mapping(result)
        }
        Value::Sequence(seq) => {
            Value::Sequence(seq.into_iter().map(normalize).collect())
        }
        other => other,
    }
}

pub fn value_as_mapping(value: &Value) -> &Mapping {
    static EMPTY: std::sync::OnceLock<Mapping> = std::sync::OnceLock::new();
    let empty = EMPTY.get_or_init(Mapping::new);
    value.as_mapping().unwrap_or(empty)
}

pub fn get_string(dict: &Mapping, key: &str) -> Option<String> {
    dict.get(&Value::String(key.into()))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub fn get_bool(dict: &Mapping, key: &str) -> Option<bool> {
    dict.get(&Value::String(key.into()))
        .and_then(|v| v.as_bool())
}

#[allow(dead_code)]
pub fn get_f64(dict: &Mapping, key: &str) -> Option<f64> {
    dict.get(&Value::String(key.into())).and_then(|v| {
        v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))
    })
}

pub fn get_i64(dict: &Mapping, key: &str) -> Option<i64> {
    dict.get(&Value::String(key.into())).and_then(|v| {
        v.as_i64().or_else(|| v.as_f64().map(|f| f as i64))
    })
}

pub fn get_mapping<'a>(dict: &'a Mapping, key: &str) -> &'a Mapping {
    static EMPTY: std::sync::OnceLock<Mapping> = std::sync::OnceLock::new();
    let empty = EMPTY.get_or_init(Mapping::new);
    dict.get(&Value::String(key.into()))
        .and_then(|v| v.as_mapping())
        .unwrap_or(empty)
}

pub fn get_sequence<'a>(dict: &'a Mapping, key: &str) -> &'a Vec<Value> {
    static EMPTY: std::sync::OnceLock<Vec<Value>> = std::sync::OnceLock::new();
    let empty = EMPTY.get_or_init(Vec::new);
    dict.get(&Value::String(key.into()))
        .and_then(|v| v.as_sequence())
        .unwrap_or(empty)
}

#[allow(dead_code)]
pub fn set_string(dict: &mut Mapping, key: &str, value: &str) {
    dict.insert(
        Value::String(key.into()),
        Value::String(value.into()),
    );
}

#[allow(dead_code)]
pub fn set_bool(dict: &mut Mapping, key: &str, value: bool) {
    dict.insert(Value::String(key.into()), Value::Bool(value));
}

#[allow(dead_code)]
pub fn set_f64(dict: &mut Mapping, key: &str, value: f64) {
    dict.insert(
        Value::String(key.into()),
        Value::Number(value.into()),
    );
}

#[allow(dead_code)]
pub fn set_i64(dict: &mut Mapping, key: &str, value: i64) {
    dict.insert(
        Value::String(key.into()),
        Value::Number(value.into()),
    );
}
