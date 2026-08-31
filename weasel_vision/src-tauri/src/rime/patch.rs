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
            // Split only the top-level slash-separated key into a path.
            // Nested mapping VALUES are literal data (e.g. punctuation keys like
            // '/', '\', "'"), NOT further path specs — re-expanding them would
            // drop keys whose name contains a path separator.
            set_value(&mut result, &split_path(key_str), value.clone());
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
            // Merge literally: the patch value is already an expanded tree whose
            // keys may be literal punctuation (e.g. '/'). Do NOT re-route through
            // `merge`/`expanded_patch` again, which would re-split those keys.
            let merged = merge_literal(base_dict, patch_dict);
            dict.insert(key, Value::Mapping(merged));
            return;
        }
    }
    dict.insert(key, value.clone());
}

/// Resolve a slash-separated path against a *patch* mapping, supporting both
/// accepted Rime patch spellings:
///
///   - slash-form:  literal top-level key `"punctuator/half_shape"` → value
///   - nested-form: `punctuator: { half_shape: { ... } }` → value
///
/// Returns `None` when the path is absent in either form.
pub fn get_patch_path_value<'a>(patch: &'a Mapping, path: &str) -> Option<&'a Value> {
    // slash-form: literal top-level key
    if let Some(v) = patch.get(Value::String(path.to_string())) {
        return Some(v);
    }
    // nested-form: walk each path segment, returning the value at the final
    // segment (the last segment is the key whose value we want, not a mapping
    // to descend into).
    let segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let (last, parents) = segs.split_last()?;
    let mut current = patch;
    for seg in parents {
        match current.get(Value::String(seg.to_string())) {
            Some(Value::Mapping(m)) => current = m,
            _ => return None,
        }
    }
    current.get(Value::String(last.to_string()))
}

/// Deep-merge a patch mapping into a base mapping with NO path expansion.
/// Nested mappings merge recursively; scalar/sequence patch values win.
fn merge_literal(base: &Mapping, patch: &Mapping) -> Mapping {
    let mut result = base.clone();
    for (key, value) in patch {
        match (result.get(key), value) {
            (Some(Value::Mapping(base_map)), Value::Mapping(patch_map)) => {
                let merged = merge_literal(base_map, patch_map);
                result.insert(key.clone(), Value::Mapping(merged));
            }
            _ => {
                result.insert(key.clone(), value.clone());
            }
        }
    }
    result
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

        let val = value_at(&dict, &["style".to_string(), "font_face".to_string()]);
        assert_eq!(val, Some(&Value::String("Arial".into())));
    }

    #[test]
    fn test_merge_preserves_literal_separator_keys() {
        // Regression: patch keys like '/' (punctuation half_shape) must not be
        // dropped when nested under a slash-separated path (e.g.
        // "punctuator/half_shape"). See weasel "标点 / 直接上屏无效".
        let base_yaml = "punctuator:\n  half_shape:\n    '/': ['/', '÷']\n";
        let custom_yaml = "patch:\n  punctuator/half_shape:\n    '/': '/'\n";
        let base: Value = serde_yaml::from_str(base_yaml).unwrap();
        let custom: Value = serde_yaml::from_str(custom_yaml).unwrap();

        let patch_map = custom
            .get(Value::String("patch".into()))
            .and_then(|v| v.as_mapping())
            .unwrap();
        let merged = merge(base.as_mapping().unwrap(), patch_map);

        let half_shape = merged
            .get(Value::String("punctuator".into()))
            .and_then(|v| v.as_mapping())
            .and_then(|p| p.get(Value::String("half_shape".into())))
            .and_then(|v| v.as_mapping())
            .unwrap();
        assert_eq!(
            half_shape.get(Value::String("/".into())).and_then(|v| v.as_str()),
            Some("/")
        );
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
        assert!(dict.get(Value::String("a".into())).is_none());
    }

    #[test]
    fn test_get_patch_path_value_slash_form() {
        // slash-form: literal top-level key "punctuator/half_shape"
        let patch_yaml = "punctuator/half_shape:\n  '/': '/'\n";
        let patch: Mapping = serde_yaml::from_str(patch_yaml).unwrap();
        let v = get_patch_path_value(&patch, "punctuator/half_shape");
        assert!(v.is_some());
        assert!(v.unwrap().as_mapping().is_some());
        // unrelated path -> None
        assert!(get_patch_path_value(&patch, "punctuator/full_shape").is_none());
    }

    #[test]
    fn test_get_patch_path_value_nested_form() {
        // nested-form: punctuator -> half_shape
        let patch_yaml = "punctuator:\n  half_shape:\n    '/': '/'\n";
        let patch: Mapping = serde_yaml::from_str(patch_yaml).unwrap();
        let v = get_patch_path_value(&patch, "punctuator/half_shape");
        assert!(v.is_some());
        let hs = v.unwrap().as_mapping().unwrap();
        assert!(hs.contains_key(Value::String("/".into())));
    }

    #[test]
    fn test_get_patch_path_value_missing() {
        let patch: Mapping = Mapping::new();
        assert!(get_patch_path_value(&patch, "menu/page_size").is_none());
    }
}
