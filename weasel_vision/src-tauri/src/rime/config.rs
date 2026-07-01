use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};

use super::backup;
use super::patch;

/// Maximum YAML file size in bytes (10 MB). Larger files are rejected to prevent
/// memory exhaustion from malformed or excessively large configuration files.
const MAX_YAML_SIZE: u64 = 10 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct RimeConfig {
    /// Always points to the Rime user root directory (Rime/ on Windows, ~/Library/Rime on macOS)
    pub user_dir: PathBuf,
    /// Windows only: Rime/build/ directory for compiled configs (default.yaml, weasel.yaml)
    pub build_dir: Option<PathBuf>,
    pub style_file: String,
    pub style_custom: String,
    pub default_yaml: String,
    pub default_custom: String,
}

fn cached_config() -> &'static RimeConfig {
    static CONFIG: OnceLock<RimeConfig> = OnceLock::new();
    CONFIG.get_or_init(RimeConfig::detect_inner)
}

impl RimeConfig {
    /// Detect Rime config paths. Cached after first call — the user directory
    /// is determined by the OS home directory and does not change during a session.
    pub fn detect() -> Self {
        cached_config().clone()
    }

    fn detect_inner() -> Self {
        if cfg!(target_os = "macos") {
            Self {
                user_dir: dirs::home_dir()
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
                    .join("Library/Rime"),
                build_dir: None,
                style_file: "squirrel.yaml".into(),
                style_custom: "squirrel.custom.yaml".into(),
                default_yaml: "default.yaml".into(),
                default_custom: "default.custom.yaml".into(),
            }
        } else if cfg!(target_os = "windows") {
            // On Windows, user_dir always points to the Rime root directory.
            // build_dir (if present) holds compiled configs like default.yaml, weasel.yaml.
            let base_dir = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("Rime");
            
            let build_dir_path = base_dir.join("build");
            let has_build_dir = build_dir_path.exists();
            
            Self {
                user_dir: base_dir,
                build_dir: if has_build_dir {
                    Some(build_dir_path)
                } else {
                    None
                },
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
                build_dir: None,
                style_file: "rime.yaml".into(),
                style_custom: "rime.custom.yaml".into(),
                default_yaml: "default.yaml".into(),
                default_custom: "default.custom.yaml".into(),
            }
        }
    }

    pub fn style_path(&self) -> PathBuf {
        // On Windows, compiled weasel.yaml is in build/
        #[cfg(target_os = "windows")]
        if let Some(ref build) = self.build_dir {
            return build.join(&self.style_file);
        }
        self.user_dir.join(&self.style_file)
    }

    pub fn style_custom_path(&self) -> PathBuf {
        self.user_dir.join(&self.style_custom)
    }

    pub fn default_path(&self) -> PathBuf {
        // On Windows, compiled default.yaml is in build/
        #[cfg(target_os = "windows")]
        if let Some(ref build) = self.build_dir {
            return build.join(&self.default_yaml);
        }
        self.user_dir.join(&self.default_yaml)
    }

    pub fn default_custom_path(&self) -> PathBuf {
        self.user_dir.join(&self.default_custom)
    }

    /// Get path to user.yaml file
    pub fn user_yaml_path(&self) -> PathBuf {
        self.user_dir.join("user.yaml")
    }

    /// Get the custom config path for a given schema_id
    /// Returns None if schema_id contains path traversal characters
    pub fn schema_custom_path(&self, schema_id: &str) -> Option<PathBuf> {
        // Reject schema_id with path traversal characters
        if schema_id.contains('/') || schema_id.contains('\\') || schema_id.contains("..") {
            eprintln!("Warning: schema_id '{}' contains invalid path characters", schema_id);
            return None;
        }
        Some(self.user_dir.join(format!("{}.custom.yaml", schema_id)))
    }

    /// Returns the directory containing compiled/generated configs.
    /// On Windows with build/: returns build/ dir.
    /// Otherwise: returns user_dir (same as user-editable dir).
    pub fn rime_data_dir(&self) -> &Path {
        self.build_dir.as_deref().unwrap_or(&self.user_dir)
    }

    pub fn load_yaml(&self, path: &Path) -> Result<Value> {
        if !path.exists() {
            return Ok(Value::Mapping(Mapping::new()));
        }
        let size = std::fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?
            .len();
        if size > MAX_YAML_SIZE {
            return Err(anyhow::anyhow!(
                "YAML file {} is too large ({} bytes, max {} bytes)",
                path.display(),
                size,
                MAX_YAML_SIZE
            ));
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

        if let Some(patch_map) = custom.get(Value::String("patch".into())) {
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
            .get(Value::String("patch".into()))
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
    dict.get(Value::String(key.into()))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub fn get_bool(dict: &Mapping, key: &str) -> Option<bool> {
    dict.get(Value::String(key.into()))
        .and_then(|v| v.as_bool())
}

pub fn get_i64(dict: &Mapping, key: &str) -> Option<i64> {
    dict.get(Value::String(key.into())).and_then(|v| {
        v.as_i64().or_else(|| v.as_f64().map(|f| f as i64))
    })
}

pub fn get_mapping<'a>(dict: &'a Mapping, key: &str) -> &'a Mapping {
    static EMPTY: std::sync::OnceLock<Mapping> = std::sync::OnceLock::new();
    let empty = EMPTY.get_or_init(Mapping::new);
    dict.get(Value::String(key.into()))
        .and_then(|v| v.as_mapping())
        .unwrap_or(empty)
}

pub fn get_sequence<'a>(dict: &'a Mapping, key: &str) -> &'a Vec<Value> {
    static EMPTY: std::sync::OnceLock<Vec<Value>> = std::sync::OnceLock::new();
    let empty = EMPTY.get_or_init(Vec::new);
    dict.get(Value::String(key.into()))
        .and_then(|v| v.as_sequence())
        .unwrap_or(empty)
}
