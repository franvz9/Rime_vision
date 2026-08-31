use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use ts_rs::TS;

use crate::rime::config::{self, RimeConfig};
use crate::rime::error::RimeError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
pub struct PunctRule {
    pub key: String,
    #[serde(default)]
    pub commit: String,
    #[serde(default)]
    pub pair: Vec<String>,
    #[serde(default)]
    pub list: Vec<String>,
}

#[derive(Serialize, Deserialize, TS)]
pub struct PunctuationData {
    pub half_shape: Vec<PunctRule>,
    pub full_shape: Vec<PunctRule>,
}

#[derive(Serialize, Deserialize, TS)]
pub struct PunctuationProfile {
    pub half_shape: Vec<PunctRule>,
    pub full_shape: Vec<PunctRule>,
    /// custom 文件中已显式定义的 half_shape/full_shape 键集合（用于前端标注"自定义"）
    pub custom_keys: Vec<String>,
}

/// 读取 scope 对应 custom 文件中显式定义的 punctuator 键集合
pub fn custom_punct_keys(cfg: &RimeConfig, scope: Option<String>) -> Vec<String> {
    let custom_path = match scope.as_deref() {
        None => cfg.default_custom_path(),
        Some(id) => cfg.schema_custom_path(id).unwrap_or_default(),
    };
    let custom = cfg.load_yaml(&custom_path).unwrap_or_default();
    let mut keys = Vec::new();
    if let Some(patch) = custom
        .get(serde_yaml::Value::String("patch".into()))
        .and_then(|v| v.as_mapping())
    {
        for prefix in ["punctuator/half_shape", "punctuator/full_shape"] {
            if let Some(Value::Mapping(m)) = patch.get(serde_yaml::Value::String(prefix.into())) {
                for k in m.keys() {
                    if let Some(s) = k.as_str() {
                        keys.push(s.to_string());
                    }
                }
            }
        }
    }
    keys.sort();
    keys
}

/// 获取 scope 的标点 Profile（生效值 + custom 键集合）
#[tauri::command]
pub fn get_punctuation_profile(scope: Option<String>) -> Result<PunctuationProfile, RimeError> {
    let cfg = RimeConfig::detect();
    let (dict_value, _target) = punct_dict_source(&cfg, scope.clone())?;
    let dict = config::value_as_mapping(&dict_value);
    let punctuator = config::get_mapping(dict, "punctuator");
    let half_shape = parse_punct_dict(config::get_mapping(punctuator, "half_shape"));
    let full_shape = parse_punct_dict(config::get_mapping(punctuator, "full_shape"));
    let custom_keys = custom_punct_keys(&cfg, scope);
    Ok(PunctuationProfile {
        half_shape,
        full_shape,
        custom_keys,
    })
}

/// Resolve the effective config mapping and the target custom path for a scope.
///
/// - `scope = None` → global: `load_effective(default.yaml, default.custom.yaml)`, target = default.custom.yaml
/// - `scope = Some(id)` → schema: `load_schema_effective(id)`, target = <id>.custom.yaml
/// - invalid schema_id → `RimeError::InvalidPath`
fn punct_dict_source(
    cfg: &RimeConfig,
    scope: Option<String>,
) -> Result<(serde_yaml::Value, std::path::PathBuf), RimeError> {
    match scope.as_deref() {
        None => {
            let dict_value = cfg.load_effective(&cfg.default_path(), &cfg.default_custom_path())?;
            Ok((dict_value, cfg.default_custom_path()))
        }
        Some(id) => {
            let dict_value = cfg.load_schema_effective(id)?;
            let custom_path = cfg
                .schema_custom_path(id)
                .ok_or_else(|| RimeError::InvalidPath(format!("Invalid schema_id: {}", id)))?;
            Ok((dict_value, custom_path))
        }
    }
}

#[tauri::command]
pub fn get_punctuation(scope: Option<String>) -> Result<PunctuationData, RimeError> {
    let cfg = RimeConfig::detect();
    let (dict_value, _target) = punct_dict_source(&cfg, scope)?;
    let dict = config::value_as_mapping(&dict_value);

    let punctuator = config::get_mapping(dict, "punctuator");
    let half_shape = parse_punct_dict(config::get_mapping(punctuator, "half_shape"));
    let full_shape = parse_punct_dict(config::get_mapping(punctuator, "full_shape"));

    Ok(PunctuationData {
        half_shape,
        full_shape,
    })
}

#[tauri::command]
pub fn save_punctuation(
    scope: Option<String>,
    half: Vec<PunctRule>,
    full: Vec<PunctRule>,
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
            Value::String("punctuator/half_shape".into()),
            Value::Mapping(serialize_punct_rules(&half)),
        );
        patch.insert(
            Value::String("punctuator/full_shape".into()),
            Value::Mapping(serialize_punct_rules(&full)),
        );
        remove_nested_patch_value(patch, "punctuator", "half_shape");
        remove_nested_patch_value(patch, "punctuator", "full_shape");
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

fn parse_punct_dict(dict: &serde_yaml::Mapping) -> Vec<PunctRule> {
    let mut rules = Vec::new();

    for (key, value) in dict {
        if let Some(key_str) = key.as_str() {
            let mut rule = PunctRule {
                key: key_str.to_string(),
                commit: String::new(),
                pair: Vec::new(),
                list: Vec::new(),
            };

            if let Some(mapping) = value.as_mapping() {
                if let Some(commit) = mapping.get(Value::String("commit".into())) {
                    if let Some(s) = commit.as_str() {
                        rule.commit = s.to_string();
                    }
                }
                if let Some(pair) = mapping.get(Value::String("pair".into())) {
                    if let Some(seq) = pair.as_sequence() {
                        rule.pair = seq
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                    }
                }
                if let Some(list) = mapping.get(Value::String("list".into())) {
                    if let Some(seq) = list.as_sequence() {
                        rule.list = seq
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                    }
                }
            } else if let Some(seq) = value.as_sequence() {
                rule.list = seq
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            } else if let Some(s) = value.as_str() {
                rule.commit = s.to_string();
            }

            rules.push(rule);
        }
    }

    rules.sort_by(|a, b| a.key.cmp(&b.key));
    rules
}

fn serialize_punct_rules(rules: &[PunctRule]) -> serde_yaml::Mapping {
    let mut dict = serde_yaml::Mapping::new();

    for rule in rules {
        let mut inner = serde_yaml::Mapping::new();
        let mut has_content = false;

        if !rule.commit.is_empty() {
            inner.insert(
                Value::String("commit".into()),
                Value::String(rule.commit.clone()),
            );
            has_content = true;
        }
        if !rule.pair.is_empty() {
            let pair_values: Vec<Value> =
                rule.pair.iter().map(|s| Value::String(s.clone())).collect();
            inner.insert(Value::String("pair".into()), Value::Sequence(pair_values));
            has_content = true;
        }
        if !rule.list.is_empty() {
            let list_values: Vec<Value> =
                rule.list.iter().map(|s| Value::String(s.clone())).collect();
            inner.insert(Value::String("list".into()), Value::Sequence(list_values));
            has_content = true;
        }

        if has_content {
            // If only commit exists and no pair/list, write as plain string
            if inner.len() == 1 && inner.contains_key(&Value::String("commit".into())) {
                dict.insert(
                    Value::String(rule.key.clone()),
                    Value::String(rule.commit.clone()),
                );
            } else {
                dict.insert(Value::String(rule.key.clone()), Value::Mapping(inner));
            }
        }
    }

    dict
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
    fn test_get_punctuation_scope_schema() {
        let (_tmp, cfg) = test_cfg();
        std::fs::write(
            cfg.user_dir.join("sc_schema.schema.yaml"),
            "punctuator:\n  half_shape:\n    '/': ['/', '÷']\n",
        )
        .unwrap();
        // 方案 custom patch：/ 直接上屏
        std::fs::write(
            cfg.user_dir.join("sc_schema.custom.yaml"),
            "patch:\n  punctuator/half_shape:\n    '/': '/'\n",
        )
        .unwrap();

        let eff = cfg.load_schema_effective("sc_schema").unwrap();
        let dict = config::value_as_mapping(&eff);
        let punct = config::get_mapping(dict, "punctuator");
        let hs = config::get_mapping(punct, "half_shape");
        let rules = parse_punct_dict(hs);
        // / 应为直接上屏 commit
        let slash = rules.iter().find(|r| r.key == "/").unwrap();
        assert_eq!(slash.commit, "/");
        assert!(slash.list.is_empty());
    }

    #[test]
    fn test_save_punctuation_writes_schema_custom() {
        let (_tmp, cfg) = test_cfg();
        // 模拟 save 到方案 custom：/ 直接上屏 + , 默认（不写）
        let half = vec![PunctRule {
            key: "/".into(),
            commit: "/".into(),
            pair: vec![],
            list: vec![],
        }];
        let full: Vec<PunctRule> = vec![];
        let custom_path = cfg.schema_custom_path("sc_schema2").unwrap();
        cfg.save_patch(&custom_path, |patch| {
            patch.insert(
                serde_yaml::Value::String("punctuator/half_shape".into()),
                serde_yaml::Value::Mapping(serialize_punct_rules(&half)),
            );
            patch.insert(
                serde_yaml::Value::String("punctuator/full_shape".into()),
                serde_yaml::Value::Mapping(serialize_punct_rules(&full)),
            );
            Ok(())
        })
        .unwrap();

        let content = std::fs::read_to_string(&custom_path).unwrap();
        assert!(content.contains("punctuator/half_shape"));
        assert!(content.contains("/:"));
        assert!(content.contains("/"));
    }
    #[test]
    fn test_punct_dict_source_scope_none_uses_default() {
        let (_tmp, cfg) = test_cfg();
        std::fs::write(
            cfg.user_dir.join("default.yaml"),
            "punctuator:\n  half_shape:\n    '/': ['/', '÷']\n",
        )
        .unwrap();
        let (dict, target) = punct_dict_source(&cfg, None).unwrap();
        assert_eq!(target, cfg.default_custom_path());
        let hs = config::get_mapping(config::get_mapping(config::value_as_mapping(&dict), "punctuator"), "half_shape");
        let slash = hs
            .get(Value::String("/".into()))
            .and_then(|v| v.as_sequence())
            .unwrap();
        assert_eq!(slash.len(), 2);
    }

    #[test]
    fn test_punct_dict_source_scope_schema_uses_schema_custom() {
        let (_tmp, cfg) = test_cfg();
        std::fs::write(
            cfg.user_dir.join("sc_a.schema.yaml"),
            "punctuator:\n  half_shape:\n    '/': ['/', '÷']\n",
        )
        .unwrap();
        std::fs::write(
            cfg.user_dir.join("sc_a.custom.yaml"),
            "patch:\n  punctuator/half_shape:\n    '/': '/'\n",
        )
        .unwrap();
        let (dict, target) = punct_dict_source(&cfg, Some("sc_a".to_string())).unwrap();
        assert_eq!(target, cfg.user_dir.join("sc_a.custom.yaml"));
        let hs = config::get_mapping(config::get_mapping(config::value_as_mapping(&dict), "punctuator"), "half_shape");
        let slash = hs
            .get(Value::String("/".into()))
            .and_then(|v| v.as_str())
            .unwrap();
        assert_eq!(slash, "/");
    }

    #[test]
    fn test_punct_dict_source_scope_invalid_id_rejected() {
        let (_tmp, cfg) = test_cfg();
        let err = punct_dict_source(&cfg, Some("a/b".to_string())).unwrap_err();
        assert!(matches!(err, RimeError::InvalidPath(_)));
    }

    #[test]
    fn test_custom_punct_keys() {
        let (_tmp, cfg) = test_cfg();
        // 方案 custom 显式定义 / 和 . 两个 half_shape 键
        std::fs::write(
            cfg.user_dir.join("ck_schema.custom.yaml"),
            "patch:\n  punctuator/half_shape:\n    '/': '/'\n    '.': '。'\n",
        )
        .unwrap();
        let keys = custom_punct_keys(&cfg, Some("ck_schema".to_string()));
        assert!(keys.contains(&"/".to_string()));
        assert!(keys.contains(&".".to_string()));
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_custom_punct_keys_global_empty() {
        let (_tmp, cfg) = test_cfg();
        // 无 custom 文件 → 空
        assert!(custom_punct_keys(&cfg, None).is_empty());
    }
}
