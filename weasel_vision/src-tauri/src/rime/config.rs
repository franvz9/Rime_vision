use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};
#[cfg(target_os = "windows")]
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

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

#[cfg(target_os = "windows")]
fn resolve_windows_user_dir() -> PathBuf {
    if let Some(path) = windows_registry_user_dir() {
        return path;
    }
    if let Some(path) = windows_env_user_dir() {
        return path;
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Rime")
}

#[cfg(target_os = "windows")]
fn windows_registry_user_dir() -> Option<PathBuf> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey("Software\\Rime\\Weasel").ok()?;
    let value: String = key.get_value("RimeUserDir").ok()?;
    let path = PathBuf::from(value.trim());
    if path.as_os_str().is_empty() {
        return None;
    }
    Some(path)
}

#[cfg(target_os = "windows")]
fn windows_env_user_dir() -> Option<PathBuf> {
    let value = std::env::var_os("RIME_USER_DIR")?;
    let path = PathBuf::from(value);
    if path.as_os_str().is_empty() {
        return None;
    }
    Some(path)
}

impl RimeConfig {
    /// Detect Rime config paths. Cached after first call — the user directory
    /// is determined by the OS home directory and does not change during a session.
    pub fn detect() -> Self {
        cached_config().clone()
    }

    fn detect_inner() -> Self {
        #[cfg(target_os = "macos")]
        {
            return Self {
                user_dir: dirs::home_dir()
                    .unwrap_or_else(|| {
                        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                    })
                    .join("Library/Rime"),
                build_dir: None,
                style_file: "squirrel.yaml".into(),
                style_custom: "squirrel.custom.yaml".into(),
                default_yaml: "default.yaml".into(),
                default_custom: "default.custom.yaml".into(),
            };
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, user_dir points to the actual Rime user root directory.
            // Priority: registry custom path > RIME_USER_DIR > default %APPDATA%\Rime.
            let base_dir = resolve_windows_user_dir();
            let build_dir_path = base_dir.join("build");
            let has_build_dir = build_dir_path.exists();

            return Self {
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
            };
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
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
        // On Windows, compiled weasel.yaml is in build/. Check dynamically because
        // build/ can be created after the cached config is initialized.
        #[cfg(target_os = "windows")]
        if let Some(build) = self.current_build_dir() {
            return build.join(&self.style_file);
        }
        self.user_dir.join(&self.style_file)
    }

    pub fn style_custom_path(&self) -> PathBuf {
        self.user_dir.join(&self.style_custom)
    }

    pub fn default_path(&self) -> PathBuf {
        // On Windows, compiled default.yaml is in build/. Check dynamically because
        // build/ can be created after the cached config is initialized.
        #[cfg(target_os = "windows")]
        if let Some(build) = self.current_build_dir() {
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
        if !Self::valid_schema_id(schema_id) {
            eprintln!(
                "Warning: schema_id '{}' contains invalid path characters",
                schema_id
            );
            return None;
        }
        Some(self.user_dir.join(format!("{}.custom.yaml", schema_id)))
    }

    /// Get path to <schema>.schema.yaml
    /// Returns None if schema_id contains path traversal characters
    pub fn schema_path(&self, schema_id: &str) -> Option<PathBuf> {
        if !Self::valid_schema_id(schema_id) {
            eprintln!(
                "Warning: schema_id '{}' contains invalid path characters",
                schema_id
            );
            return None;
        }
        Some(self.user_dir.join(format!("{}.schema.yaml", schema_id)))
    }

    /// Validate a schema_id used to build a file path.
    /// Rejects path separators, traversal sequences, and NUL/control chars.
    fn valid_schema_id(schema_id: &str) -> bool {
        !schema_id.contains('/')
            && !schema_id.contains('\\')
            && !schema_id.contains("..")
            && !schema_id.contains('\0')
            && !schema_id.contains('\n')
            && !schema_id.contains('\r')
            && !schema_id.contains('\t')
            && !schema_id.is_empty()
    }

    /// Merge <schema>.schema.yaml with <schema>.custom.yaml patch.
    /// Missing schema file -> empty mapping (so callers can distinguish
    /// "no per-schema config" from "error").
    pub fn load_schema_effective(&self, schema_id: &str) -> Result<Value, crate::rime::error::RimeError> {
        let Some(schema_path) = self.schema_path(schema_id) else {
            return Err(crate::rime::error::RimeError::InvalidPath(format!(
                "Invalid schema_id: {}",
                schema_id
            )));
        };
        let Some(custom_path) = self.schema_custom_path(schema_id) else {
            return Err(crate::rime::error::RimeError::InvalidPath(format!(
                "Invalid schema_id: {}",
                schema_id
            )));
        };
        // 1. 读取方案 schema 源文件
        let mut base = self.load_yaml(&schema_path)?;
        // 2. 解析 import_preset 引用（如 punctuator.import_preset: symbols），
        //    使方案配置能读到预设引用的完整内容（符号集、默认按键绑定等）。
        base = self.resolve_import_presets(base)?;
        // 3. 应用方案 custom patch
        self.merge_custom_patch(base, &custom_path)
            .map_err(crate::rime::error::RimeError::from)
    }

    /// 解析配置顶层各 section 的 `import_preset` 引用。
    ///
    /// Rime 方案常用 `punctuator: { import_preset: symbols }`、
    /// `key_binder: { import_preset: default }` 等方式复用预设内容，而这些
    /// 预设引用的完整内容并不在 schema 文件本身里。此方法会找到对应预设
    /// 文件，把其同名的 section 作为基础，再用当前 schema 的 section 覆盖合并，
    /// 从而让 effective 配置包含完整内容。
    ///
    /// 合并规则：
    /// - Mapping 值：深度合并（预设为基础，当前定义覆盖）
    /// - Sequence 值（如 key_binder.bindings）：追加（预设 + 当前）
    /// - 其它值：当前定义覆盖
    fn resolve_import_presets(&self, value: Value) -> Result<Value> {
        let map = match value {
            Value::Mapping(m) => m,
            other => return Ok(other),
        };
        // 缓存已加载的预设文件，避免重复读取
        let mut preset_cache: std::collections::HashMap<String, Value> =
            std::collections::HashMap::new();
        let mut new_map = serde_yaml::Mapping::new();
        for (key, value) in map {
            let mapped = match value {
                Value::Mapping(section) => {
                    let preset_name = section
                        .get(Value::String("import_preset".into()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    if let Some(preset_name) = preset_name {
                        if let Some(preset_section) = self
                            .load_preset_section(&preset_name, &key, &mut preset_cache)?
                        {
                            if let Some(pm) = preset_section.as_mapping() {
                                Value::Mapping(merge_preset_section(pm, &section, true))
                            } else {
                                Value::Mapping(section)
                            }
                        } else {
                            Value::Mapping(section)
                        }
                    } else {
                        Value::Mapping(section)
                    }
                }
                other => other,
            };
            new_map.insert(key, mapped);
        }
        Ok(Value::Mapping(new_map))
    }

    /// 加载预设文件中某个顶层 section 的内容。
    /// 返回 `None` 表示该预设文件或 section 不存在（静默跳过）。
    fn load_preset_section<'a>(
        &self,
        preset_name: &str,
        section_key: &Value,
        cache: &'a mut std::collections::HashMap<String, Value>,
    ) -> Result<Option<&'a Value>> {
        if !cache.contains_key(preset_name) {
            let path = match preset_name {
                "default" => self.default_path(),
                name => self.user_dir.join(format!("{}.yaml", name)),
            };
            let loaded = self.load_yaml(&path)?;
            cache.insert(preset_name.to_string(), loaded);
        }
        Ok(cache.get(preset_name).and_then(|doc| doc.get(section_key)))
    }

    /// 将 custom 文件的 patch 合并到 base 配置上。
    fn merge_custom_patch(&self, base: Value, custom_path: &Path) -> Result<Value> {
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

    /// Returns the directory containing compiled/generated configs.
    /// On Windows with build/: returns build/ dir.
    /// Otherwise: returns user_dir (same as user-editable dir).
    pub fn rime_data_dir(&self) -> PathBuf {
        self.current_build_dir()
            .unwrap_or_else(|| self.user_dir.clone())
    }

    fn current_build_dir(&self) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Some(build_dir) = &self.build_dir {
                if build_dir.exists() {
                    return Some(build_dir.clone());
                }
            }
            let build_dir = self.user_dir.join("build");
            if build_dir.exists() {
                return Some(build_dir);
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            if let Some(build_dir) = &self.build_dir {
                return Some(build_dir.clone());
            }
        }
        None
    }

    /// 动态检测 build_dir（deploy 后调用，获取最新状态）
    /// 由于 OnceLock 缓存不可变，此方法绕过缓存重新检测
    pub fn detect_build_dir(rime_dir: &Path) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let build_dir_path = rime_dir.join("build");
            if build_dir_path.exists() {
                return Some(build_dir_path);
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = rime_dir;
        }
        None
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
        eprintln!("WeaselVision: save_patch called for {:?}", custom_path);
        let existing = self.load_yaml(custom_path)?;
        let mut patch_map = existing
            .get(Value::String("patch".into()))
            .and_then(|v| v.as_mapping())
            .cloned()
            .unwrap_or_default();

        mutate(&mut patch_map)?;

        let mut root = Mapping::new();
        root.insert(Value::String("patch".into()), Value::Mapping(patch_map));

        let content = serde_yaml::to_string(&root)?;
        eprintln!("WeaselVision: YAML serialized successfully, length={}", content.len());
        backup::write_if_changed(&content, custom_path)?;
        eprintln!("WeaselVision: File written successfully");
        Ok(())
    }
}

/// 合并预设 section 与当前定义 section（模拟 Rime import_preset 语义）。
///
/// `preset` 为基础，`override_map`（当前 schema 定义）覆盖：
/// - Mapping 值：深度合并
/// - Sequence 值：
///   - 顶层 section 直接子键（如 `key_binder.bindings`）：追加（预设 + 当前）
///   - 嵌套层级（如 `punctuator.half_shape` 内的某个键值）：覆盖
/// - 其它值：当前定义覆盖
fn merge_preset_section(
    preset: &serde_yaml::Mapping,
    override_map: &serde_yaml::Mapping,
    top: bool,
) -> serde_yaml::Mapping {
    let mut result = preset.clone();
    for (k, v) in override_map {
        match (result.get(k), v) {
            (Some(Value::Mapping(pm)), Value::Mapping(om)) => {
                result.insert(k.clone(), Value::Mapping(merge_preset_section(pm, om, false)));
            }
            (Some(Value::Sequence(ps)), Value::Sequence(os)) if top => {
                let mut merged = ps.clone();
                merged.extend(os.clone());
                result.insert(k.clone(), Value::Sequence(merged));
            }
            _ => {
                result.insert(k.clone(), v.clone());
            }
        }
    }
    result
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
        Value::Sequence(seq) => Value::Sequence(seq.into_iter().map(normalize).collect()),
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
    dict.get(Value::String(key.into()))
        .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_config() -> (tempfile::TempDir, RimeConfig) {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = RimeConfig {
            user_dir: tmp.path().to_path_buf(),
            build_dir: None,
            style_file: "squirrel.yaml".into(),
            style_custom: "squirrel.custom.yaml".into(),
            default_yaml: "default.yaml".into(),
            default_custom: "default.custom.yaml".into(),
        };
        (tmp, cfg)
    }

    #[test]
    fn test_save_patch_and_load_roundtrip() {
        let (_tmp, cfg) = temp_config();
        let custom_path = cfg.default_custom_path();

        // Save a patch
        cfg.save_patch(&custom_path, |patch| {
            patch.insert(
                Value::String("menu/page_size".into()),
                Value::Number(8.into()),
            );
            patch.insert(
                Value::String("translator/enable_encoder".into()),
                Value::Bool(false),
            );
            Ok(())
        })
        .unwrap();

        // Load it back
        let loaded = cfg.load_yaml(&custom_path).unwrap();
        let patch = loaded
            .get(Value::String("patch".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();

        // Patch stores slash-separated keys literally (expanded_patch expands them on read)
        assert_eq!(
            patch
                .get(Value::String("menu/page_size".into()))
                .and_then(|v| v.as_i64()),
            Some(8)
        );
        assert_eq!(
            patch
                .get(Value::String("translator/enable_encoder".into()))
                .and_then(|v| v.as_bool()),
            Some(false)
        );

        // Verify load_effective expands them correctly
        // Write a base config so load_effective has something to merge
        let base_yaml = "menu:\n  page_size: 5\n";
        std::fs::write(cfg.default_path(), base_yaml).unwrap();
        let effective = cfg
            .load_effective(&cfg.default_path(), &custom_path)
            .unwrap();
        let dict = effective.as_mapping().unwrap();
        let menu = dict
            .get(Value::String("menu".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        assert_eq!(
            menu.get(Value::String("page_size".into()))
                .and_then(|v| v.as_i64()),
            Some(8)
        );
    }

    #[test]
    fn test_load_effective_merges_base_and_custom() {
        let (_tmp, cfg) = temp_config();

        // Write base config
        let base_yaml = "menu:\n  page_size: 5\ntranslator:\n  enable_encoder: true\n";
        std::fs::write(cfg.default_path(), base_yaml).unwrap();

        // Write custom config that overrides page_size
        let custom_yaml = "patch:\n  menu/page_size: 10\n";
        std::fs::write(cfg.default_custom_path(), custom_yaml).unwrap();

        let effective = cfg
            .load_effective(&cfg.default_path(), &cfg.default_custom_path())
            .unwrap();
        let dict = effective.as_mapping().unwrap();

        let menu = dict
            .get(Value::String("menu".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        assert_eq!(
            menu.get(Value::String("page_size".into()))
                .and_then(|v| v.as_i64()),
            Some(10)
        );

        // Translator should still be present from base
        let translator = dict
            .get(Value::String("translator".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        assert_eq!(
            translator
                .get(Value::String("enable_encoder".into()))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn test_load_yaml_missing_file() {
        let (_tmp, cfg) = temp_config();
        let result = cfg
            .load_yaml(&cfg.user_dir.join("nonexistent.yaml"))
            .unwrap();
        assert!(result.as_mapping().unwrap().is_empty());
    }

    #[test]
    fn test_schema_path_rejects_traversal() {
        let (_tmp, cfg) = temp_config();
        assert!(cfg.schema_path("../evil").is_none());
        assert!(cfg.schema_path("a/b").is_none());
        assert!(cfg.schema_path("a\\b").is_none());
        assert!(cfg.schema_path("evil\0x").is_none());
        assert!(cfg.schema_path("evil\n").is_none());
        assert!(cfg.schema_path("").is_none());
        assert!(cfg.schema_custom_path("evil\0x").is_none());
    }

    #[test]
    fn test_load_schema_effective_merges_custom() {
        let (_tmp, cfg) = temp_config();
        // schema 基础文件
        std::fs::write(
            cfg.user_dir.join("test_schema.schema.yaml"),
            "punctuator:\n  half_shape:\n    '/': ['/', '÷']\n",
        )
        .unwrap();
        // 方案 custom patch
        std::fs::write(
            cfg.user_dir.join("test_schema.custom.yaml"),
            "patch:\n  punctuator/half_shape:\n    '/': '/'\n",
        )
        .unwrap();

        let eff = cfg
            .load_schema_effective("test_schema")
            .expect("load should succeed");
        let dict = eff.as_mapping().unwrap();
        let punct = dict
            .get(serde_yaml::Value::String("punctuator".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        let hs = punct
            .get(serde_yaml::Value::String("half_shape".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        // custom patch 覆盖 schema: '/' -> '/'
        assert_eq!(
            hs.get(serde_yaml::Value::String("/".into()))
                .and_then(|v| v.as_str()),
            Some("/")
        );
    }

    #[test]
    fn test_load_schema_effective_missing_schema_file() {
        let (_tmp, cfg) = temp_config();
        // schema 文件不存在时，应返回空映射而不是报错
        let eff = cfg.load_schema_effective("nonexistent_schema").unwrap();
        assert!(eff.as_mapping().unwrap().is_empty());
    }

    #[test]
    fn test_load_schema_effective_resolves_import_preset_symbols() {
        let (_tmp, cfg) = temp_config();
        // 模拟 symbols.yaml 预设：含完整 half_shape/full_shape
        std::fs::write(
            cfg.user_dir.join("symbols.yaml"),
            "punctuator:\n  half_shape:\n    '!': '！'\n    '?': '？'\n    '.': '。'\n    '*': ['*', '＊', '・']\n  full_shape:\n    '!': '！'\n    '?': '？'\n",
        )
        .unwrap();
        // 方案 schema 用 import_preset: symbols，并内联覆盖部分键
        std::fs::write(
            cfg.user_dir.join("ip_schema.schema.yaml"),
            "punctuator:\n  import_preset: symbols\n  half_shape:\n    '/': '/'\n    '*': ['*', '×', '※']\n",
        )
        .unwrap();

        let eff = cfg.load_schema_effective("ip_schema").unwrap();
        let dict = eff.as_mapping().unwrap();
        let punct = dict
            .get(serde_yaml::Value::String("punctuator".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        let hs = punct
            .get(serde_yaml::Value::String("half_shape".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        // preset 的键应被解析进来
        assert_eq!(
            hs.get(serde_yaml::Value::String("!".into())).and_then(|v| v.as_str()),
            Some("！")
        );
        assert_eq!(
            hs.get(serde_yaml::Value::String("?".into())).and_then(|v| v.as_str()),
            Some("？")
        );
        // schema 内联覆盖的键也应保留
        assert_eq!(
            hs.get(serde_yaml::Value::String("/".into())).and_then(|v| v.as_str()),
            Some("/")
        );
        // 嵌套 sequence 键：schema 覆盖 preset，而非追加（避免重复）
        let star = hs
            .get(serde_yaml::Value::String("*".into()))
            .and_then(|v| v.as_sequence())
            .unwrap();
        assert_eq!(star.len(), 3, "嵌套 sequence 应为覆盖而非追加");
        assert_eq!(star, &vec![serde_yaml::Value::String("*".into()), serde_yaml::Value::String("×".into()), serde_yaml::Value::String("※".into())]);
        // full_shape 也从 preset 解析
        let fs = punct
            .get(serde_yaml::Value::String("full_shape".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        assert_eq!(fs.len(), 2);
    }

    #[test]
    fn test_load_schema_effective_resolves_import_preset_default_bindings() {
        let (_tmp, cfg) = temp_config();
        // 模拟 default.yaml 预设：key_binder 含默认 bindings
        std::fs::write(
            cfg.user_dir.join("default.yaml"),
            "key_binder:\n  bindings:\n    - {when: composing, accept: Control+p, send: Up}\n    - {when: composing, accept: Control+n, send: Down}\n",
        )
        .unwrap();
        // 方案 schema 用 import_preset: default，并追加自定义绑定
        std::fs::write(
            cfg.user_dir.join("ipkb.schema.yaml"),
            "key_binder:\n  import_preset: default\n  bindings:\n    - {when: always, accept: F4, send: Escape}\n",
        )
        .unwrap();

        let eff = cfg.load_schema_effective("ipkb").unwrap();
        let dict = eff.as_mapping().unwrap();
        let kb = dict
            .get(serde_yaml::Value::String("key_binder".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        let seq = kb
            .get(serde_yaml::Value::String("bindings".into()))
            .and_then(|v| v.as_sequence())
            .unwrap();
        // 预设 bindings（2 个）+ 方案 bindings（1 个）= 3 个（追加）
        assert_eq!(seq.len(), 3);
    }

    #[test]
    fn test_load_schema_effective_without_preset_unchanged() {
        let (_tmp, cfg) = temp_config();
        // 无 import_preset 的方案，行为不变
        std::fs::write(
            cfg.user_dir.join("plain.schema.yaml"),
            "punctuator:\n  half_shape:\n    '/': '/'\n",
        )
        .unwrap();
        let eff = cfg.load_schema_effective("plain").unwrap();
        let dict = eff.as_mapping().unwrap();
        let punct = dict
            .get(serde_yaml::Value::String("punctuator".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        let hs = punct
            .get(serde_yaml::Value::String("half_shape".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        assert_eq!(hs.len(), 1);
        assert!(hs.contains_key(serde_yaml::Value::String("/".into())));
    }

    #[test]
    fn test_parse_yaml_normalizes_keys() {
        let yaml = "foo: bar\nbaz: 42\n";
        let value = parse_yaml(yaml).unwrap();
        let dict = value.as_mapping().unwrap();
        assert_eq!(
            dict.get(Value::String("foo".into()))
                .and_then(|v| v.as_str()),
            Some("bar")
        );
        assert_eq!(
            dict.get(Value::String("baz".into()))
                .and_then(|v| v.as_i64()),
            Some(42)
        );
    }
}
