use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::rime::config::{self, RimeConfig};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PunctRule {
    pub key: String,
    #[serde(default)]
    pub commit: String,
    #[serde(default)]
    pub pair: Vec<String>,
    #[serde(default)]
    pub list: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PunctuationData {
    pub half_shape: Vec<PunctRule>,
    pub full_shape: Vec<PunctRule>,
}

#[tauri::command]
pub fn get_punctuation() -> Result<PunctuationData, String> {
    let cfg = RimeConfig::detect();
    let dict_value = cfg
        .load_effective(&cfg.default_path(), &cfg.default_custom_path())
        .map_err(|e| e.to_string())?;
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
pub fn save_punctuation(half: Vec<PunctRule>, full: Vec<PunctRule>) -> Result<(), String> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.default_custom_path(), |patch| {
        let punctuator = patch
            .entry(Value::String("punctuator".into()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()))
            .as_mapping_mut()
            .ok_or_else(|| anyhow::anyhow!("punctuator is not a mapping"))?;
        punctuator.insert(
            Value::String("half_shape".into()),
            Value::Mapping(serialize_punct_rules(&half)),
        );
        punctuator.insert(
            Value::String("full_shape".into()),
            Value::Mapping(serialize_punct_rules(&full)),
        );
        Ok(())
    })
    .map_err(|e| e.to_string())
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
                if let Some(commit) = mapping.get(&Value::String("commit".into())) {
                    if let Some(s) = commit.as_str() {
                        rule.commit = s.to_string();
                    }
                }
                if let Some(pair) = mapping.get(&Value::String("pair".into())) {
                    if let Some(seq) = pair.as_sequence() {
                        rule.pair = seq
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
        if !rule.commit.is_empty() {
            let mut inner = serde_yaml::Mapping::new();
            inner.insert(
                Value::String("commit".into()),
                Value::String(rule.commit.clone()),
            );
            dict.insert(
                Value::String(rule.key.clone()),
                Value::Mapping(inner),
            );
        } else if !rule.pair.is_empty() {
            let pair_values: Vec<Value> = rule
                .pair
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect();
            let mut inner = serde_yaml::Mapping::new();
            inner.insert(
                Value::String("pair".into()),
                Value::Sequence(pair_values),
            );
            dict.insert(
                Value::String(rule.key.clone()),
                Value::Mapping(inner),
            );
        } else if !rule.list.is_empty() {
            let list_values: Vec<Value> = rule
                .list
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect();
            dict.insert(
                Value::String(rule.key.clone()),
                Value::Sequence(list_values),
            );
        }
    }

    dict
}
