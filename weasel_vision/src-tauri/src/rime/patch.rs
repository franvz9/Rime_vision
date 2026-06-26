use serde_yaml::{Mapping, Value};

pub fn split_path(key: &str) -> Vec<String> {
    key.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub fn expanded_patch(patch: &Mapping) -> Mapping {
    let mut result = Mapping::new();

    for (key, value) in patch {
        if let Some(key_str) = key.as_str() {
            if key_str == "__delete__" || key_str == "__append__" {
                continue;
            }
            let expanded = if let Some(nested) = value.as_mapping() {
                Value::Mapping(expanded_patch(nested))
            } else {
                value.clone()
            };
            set_value(&mut result, &split_path(key_str), expanded);
        }
    }

    if let Some(deletions) = patch.get(Value::String("__delete__".into())) {
        result.insert(Value::String("__delete__".into()), deletions.clone());
    }
    if let Some(appends) = patch.get(Value::String("__append__".into())) {
        result.insert(Value::String("__append__".into()), appends.clone());
    }

    result
}

pub fn merge(base: &Mapping, patch: &Mapping) -> Mapping {
    let mut result = base.clone();
    let expanded = expanded_patch(patch);

    if let Some(deletions) = expanded.get(Value::String("__delete__".into())) {
        if let Some(keys) = deletions.as_sequence() {
            for key in keys {
                if let Some(key_str) = key.as_str() {
                    remove_value(&mut result, &split_path(key_str));
                }
            }
        }
    }

    if let Some(appends) = expanded.get(Value::String("__append__".into())) {
        if let Some(append_map) = appends.as_mapping() {
            for (key, value) in append_map {
                if let Some(key_str) = key.as_str() {
                    let path = split_path(key_str);
                    let new_items = if let Some(seq) = value.as_sequence() {
                        seq.clone()
                    } else {
                        vec![value.clone()]
                    };
                    if let Some(existing) = value_at(&result, &path) {
                        if let Some(mut seq) = existing.as_sequence().cloned() {
                            seq.extend(new_items);
                            set_value(&mut result, &path, Value::Sequence(seq));
                        } else {
                            set_value(&mut result, &path, Value::Sequence(new_items));
                        }
                    } else {
                        set_value(&mut result, &path, Value::Sequence(new_items));
                    }
                }
            }
        }
    }

    for (key, value) in &expanded {
        if let Some(key_str) = key.as_str() {
            if key_str == "__delete__" || key_str == "__append__" {
                continue;
            }
            merge_value(value, &mut result, &[key_str]);
        }
    }

    result
}

pub fn value_at<'a>(dict: &'a Mapping, path: &[String]) -> Option<&'a Value> {
    if path.is_empty() {
        return None;
    }
    if path.len() == 1 {
        return dict.get(Value::String(path[0].clone()));
    }
    if let Some(Value::Mapping(nested)) = dict.get(Value::String(path[0].clone())) {
        value_at(nested, &path[1..])
    } else {
        None
    }
}

pub fn set_value(dict: &mut Mapping, path: &[String], value: Value) {
    if path.is_empty() {
        return;
    }
    if path.len() == 1 {
        dict.insert(Value::String(path[0].clone()), value);
        return;
    }
    let key = Value::String(path[0].clone());
    let nested = dict
        .get(&key)
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();
    let mut new_nested = nested;
    set_value(&mut new_nested, &path[1..], value);
    dict.insert(key, Value::Mapping(new_nested));
}

pub fn remove_value(dict: &mut Mapping, path: &[String]) {
    if path.is_empty() {
        return;
    }
    if path.len() == 1 {
        dict.remove(Value::String(path[0].clone()));
        return;
    }
    let key = Value::String(path[0].clone());
    if let Some(Value::Mapping(nested)) = dict.get(&key) {
        let mut new_nested = nested.clone();
        remove_value(&mut new_nested, &path[1..]);
        // Clean up empty mappings to keep the tree tidy
        if new_nested.is_empty() {
            dict.remove(&key);
        } else {
            dict.insert(key, Value::Mapping(new_nested));
        }
    }
}

fn merge_value(value: &Value, dict: &mut Mapping, path: &[&str]) {
    if path.is_empty() {
        return;
    }
    if path.len() > 1 {
        let key = Value::String(path[0].to_string());
        let nested = dict
            .get(&key)
            .and_then(|v| v.as_mapping())
            .cloned()
            .unwrap_or_default();
        let mut new_nested = nested;
        merge_value(value, &mut new_nested, &path[1..]);
        dict.insert(key, Value::Mapping(new_nested));
        return;
    }

    let key = Value::String(path[0].to_string());
    if let Some(patch_dict) = value.as_mapping() {
        if let Some(Value::Mapping(base_dict)) = dict.get(&key) {
            let merged = merge(base_dict, patch_dict);
            dict.insert(key, Value::Mapping(merged));
            return;
        }
    }
    dict.insert(key, value.clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_path() {
        assert_eq!(split_path("style/font_face"), vec!["style", "font_face"]);
        assert_eq!(split_path("simple"), vec!["simple"]);
        assert_eq!(
            split_path("key_binder/bindings"),
            vec!["key_binder", "bindings"]
        );
    }

    #[test]
    fn test_set_and_get() {
        let mut dict = Mapping::new();
        set_value(
            &mut dict,
            &["style".to_string(), "font_face".to_string()],
            Value::String("Arial".into()),
        );

        let val = value_at(
            &dict,
            &["style".to_string(), "font_face".to_string()],
        );
        assert_eq!(val, Some(&Value::String("Arial".into())));
    }

    #[test]
    fn test_remove() {
        let mut dict = Mapping::new();
        set_value(
            &mut dict,
            &["a".to_string(), "b".to_string()],
            Value::Number(1.into()),
        );

        remove_value(&mut dict, &["a".to_string(), "b".to_string()]);
        assert!(dict.get(&Value::String("a".into())).is_none());
    }
}
