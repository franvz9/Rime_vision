use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use ts_rs::TS;

use crate::rime::config::{self, RimeConfig};
use crate::rime::error::RimeError;
use crate::rime::patch::get_patch_path_value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
pub struct TranslatorSettings {
    #[serde(default = "default_true")]
    pub enable_encoder: bool,
    #[serde(default = "default_true")]
    pub enable_sentence: bool,
    #[serde(default = "default_true")]
    pub enable_user_dict: bool,
    #[serde(default = "default_true")]
    pub encode_commit_history: bool,
}

fn default_true() -> bool {
    true
}

impl Default for TranslatorSettings {
    fn default() -> Self {
        Self {
            enable_encoder: true,
            enable_sentence: true,
            enable_user_dict: true,
            encode_commit_history: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
pub struct TranslatorProfile {
    pub settings: TranslatorSettings,
    /// custom 文件中是否显式定义了 translator 相关配置（用于标注"自定义"）
    pub is_custom: bool,
}

/// 判断 custom 文件中是否显式定义了 translator 下的某个键（兼容 slash/nested 写法）。
fn is_translator_key_custom(custom: &Value, key: &str) -> bool {
    custom
        .get(Value::String("patch".into()))
        .and_then(|v| v.as_mapping())
        .map(|m| get_patch_path_value(m, &format!("translator/{}", key)).is_some())
        .unwrap_or(false)
}

/// 获取 scope 的翻译器 Profile（生效值 + 是否自定义）。
///
/// - scope = None → 全局（default.yaml + default.custom.yaml）
/// - scope = Some(id) → 方案（schema + custom，含 import_preset 解析）
#[tauri::command]
pub fn get_translator_profile(scope: Option<String>) -> Result<TranslatorProfile, RimeError> {
    let cfg = RimeConfig::detect();
    let dict_value = match scope.as_deref() {
        None => cfg.load_effective(&cfg.default_path(), &cfg.default_custom_path())?,
        Some(id) => cfg.load_schema_effective(id)?,
    };
    let dict = config::value_as_mapping(&dict_value);
    let translator = config::get_mapping(dict, "translator");
    let settings = TranslatorSettings {
        enable_encoder: config::get_bool(translator, "enable_encoder").unwrap_or(true),
        enable_sentence: config::get_bool(translator, "enable_sentence").unwrap_or(true),
        enable_user_dict: config::get_bool(translator, "enable_user_dict").unwrap_or(true),
        encode_commit_history: config::get_bool(translator, "encode_commit_history").unwrap_or(true),
    };

    let custom_path = match scope.as_deref() {
        None => cfg.default_custom_path(),
        Some(id) => cfg
            .schema_custom_path(id)
            .ok_or_else(|| RimeError::InvalidPath(format!("Invalid schema_id: {}", id)))?,
    };
    let custom = cfg.load_yaml(&custom_path).unwrap_or_default();
    let is_custom = ["enable_encoder", "enable_sentence", "enable_user_dict", "encode_commit_history"]
        .iter()
        .any(|k| is_translator_key_custom(&custom, k));

    Ok(TranslatorProfile { settings, is_custom })
}

/// 保存 scope 的翻译器设置（差异写入：仅在需自定义的键上写 patch）。
#[tauri::command]
pub fn save_translator(
    scope: Option<String>,
    settings: TranslatorSettings,
) -> Result<(), RimeError> {
    let cfg = RimeConfig::detect();
    let target_path = match scope.as_deref() {
        None => cfg.default_custom_path(),
        Some(id) => cfg
            .schema_custom_path(id)
            .ok_or_else(|| RimeError::InvalidPath(format!("Invalid schema_id: {}", id)))?,
    };
    cfg.save_patch(&target_path, |patch| {
        patch.insert(
            Value::String("translator/enable_encoder".into()),
            Value::Bool(settings.enable_encoder),
        );
        patch.insert(
            Value::String("translator/enable_sentence".into()),
            Value::Bool(settings.enable_sentence),
        );
        patch.insert(
            Value::String("translator/enable_user_dict".into()),
            Value::Bool(settings.enable_user_dict),
        );
        patch.insert(
            Value::String("translator/encode_commit_history".into()),
            Value::Bool(settings.encode_commit_history),
        );
        Ok(())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rime::config::RimeConfig;
    use tempfile::tempdir;

    fn test_cfg() -> (tempfile::TempDir, RimeConfig) {
        let tmp = tempdir().unwrap();
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
    fn test_translator_defaults_when_missing() {
        let (_tmp, cfg) = test_cfg();
        // 无任何配置 → 全默认 true
        let profile = get_translator_profile_inner(&cfg, None).unwrap();
        assert!(profile.settings.enable_encoder);
        assert!(profile.settings.enable_sentence);
        assert!(profile.settings.enable_user_dict);
        assert!(profile.settings.encode_commit_history);
        assert!(!profile.is_custom);
    }

    #[test]
    fn test_translator_reads_custom_values() {
        let (_tmp, cfg) = test_cfg();
        std::fs::write(
            cfg.user_dir.join("default.yaml"),
            "translator:\n  enable_encoder: true\n  enable_sentence: true\n",
        )
        .unwrap();
        std::fs::write(
            cfg.user_dir.join("default.custom.yaml"),
            "patch:\n  translator:\n    enable_encoder: false\n    enable_user_dict: false\n",
        )
        .unwrap();
        let profile = get_translator_profile_inner(&cfg, None).unwrap();
        assert!(!profile.settings.enable_encoder, "custom 覆盖 enable_encoder");
        assert!(profile.settings.enable_sentence);
        assert!(!profile.settings.enable_user_dict);
        assert!(profile.settings.encode_commit_history);
        assert!(profile.is_custom, "custom 中定义了 translator 键");
    }

    #[test]
    fn test_translator_schema_scope() {
        let (_tmp, cfg) = test_cfg();
        std::fs::write(
            cfg.user_dir.join("tr.schema.yaml"),
            "translator:\n  enable_encoder: true\n  enable_sentence: false\n",
        )
        .unwrap();
        std::fs::write(
            cfg.user_dir.join("tr.custom.yaml"),
            "patch:\n  translator/enable_sentence: true\n",
        )
        .unwrap();
        let profile = get_translator_profile_inner(&cfg, Some("tr".to_string())).unwrap();
        assert!(profile.settings.enable_sentence, "方案 custom 覆盖为 true");
        assert!(profile.settings.enable_encoder);
        assert!(profile.is_custom);
    }

    /// 测试用内部封装（避免 Tauri command 的注入依赖）
    fn get_translator_profile_inner(
        cfg: &RimeConfig,
        scope: Option<String>,
    ) -> Result<TranslatorProfile, RimeError> {
        let dict_value = match scope.as_deref() {
            None => cfg.load_effective(&cfg.default_path(), &cfg.default_custom_path())?,
            Some(id) => cfg.load_schema_effective(id)?,
        };
        let dict = config::value_as_mapping(&dict_value);
        let translator = config::get_mapping(dict, "translator");
        let settings = TranslatorSettings {
            enable_encoder: config::get_bool(translator, "enable_encoder").unwrap_or(true),
            enable_sentence: config::get_bool(translator, "enable_sentence").unwrap_or(true),
            enable_user_dict: config::get_bool(translator, "enable_user_dict").unwrap_or(true),
            encode_commit_history: config::get_bool(translator, "encode_commit_history")
                .unwrap_or(true),
        };

        let custom_path = match scope.as_deref() {
            None => cfg.default_custom_path(),
            Some(id) => cfg
                .schema_custom_path(id)
                .ok_or_else(|| RimeError::InvalidPath(format!("Invalid schema_id: {}", id)))?,
        };
        let custom = cfg.load_yaml(&custom_path).unwrap_or_default();
        let is_custom = [
            "enable_encoder",
            "enable_sentence",
            "enable_user_dict",
            "encode_commit_history",
        ]
        .iter()
        .any(|k| is_translator_key_custom(&custom, k));

        Ok(TranslatorProfile { settings, is_custom })
    }
}
