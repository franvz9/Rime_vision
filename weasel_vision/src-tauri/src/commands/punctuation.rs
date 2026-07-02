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

#[tauri::command]
pub fn get_punctuation() -> Result<PunctuationData, RimeError> {
    let cfg = RimeConfig::detect();
    let dict_value = cfg.load_effective(&cfg.default_path(), &cfg.default_custom_path())?;
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
pub fn save_punctuation(half: Vec<PunctRule>, full: Vec<PunctRule>) -> Result<(), RimeError> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.default_custom_path(), |patch| {
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
