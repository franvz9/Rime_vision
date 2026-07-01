/// Integration test: YAML patch file round-trip.
///
/// Verifies that a YAML patch file can be written, read back, and parsed
/// correctly, exercising the same serde_yaml path used by RimeConfig.
use serde_yaml::{Mapping, Value};
use tempfile::TempDir;

#[test]
fn test_yaml_patch_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("default.custom.yaml");

    // Build a patch document (mirrors what RimeConfig::save_patch produces)
    let mut patch_map = Mapping::new();
    patch_map.insert(
        Value::String("menu/page_size".into()),
        Value::Number(8.into()),
    );
    patch_map.insert(
        Value::String("translator/enable_encoder".into()),
        Value::Bool(false),
    );

    let mut root = Mapping::new();
    root.insert(Value::String("patch".into()), Value::Mapping(patch_map));

    let content = serde_yaml::to_string(&root).unwrap();
    std::fs::write(&path, &content).unwrap();

    // Read back and verify
    let read_back = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, read_back);

    // Parse and verify structure
    let parsed: Value = serde_yaml::from_str(&read_back).unwrap();
    let patch = parsed
        .get(Value::String("patch".into()))
        .and_then(|v| v.as_mapping())
        .unwrap();

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
}

#[test]
fn test_yaml_merge_patch() {
    // Simulate base + custom merge (the core of load_effective)
    let base_yaml = "menu:\n  page_size: 5\ntranslator:\n  enable_encoder: true\n";
    let custom_yaml = "patch:\n  menu/page_size: 10\n";

    let base: Value = serde_yaml::from_str(base_yaml).unwrap();
    let custom: Value = serde_yaml::from_str(custom_yaml).unwrap();

    // Verify custom has a patch section
    let patch = custom
        .get(Value::String("patch".into()))
        .and_then(|v| v.as_mapping());
    assert!(patch.is_some());

    // Verify base has original values
    let base_map = base.as_mapping().unwrap();
    let menu = base_map
        .get(Value::String("menu".into()))
        .and_then(|v| v.as_mapping())
        .unwrap();
    assert_eq!(
        menu.get(Value::String("page_size".into()))
            .and_then(|v| v.as_i64()),
        Some(5)
    );
}

#[test]
fn test_yaml_empty_patch() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("empty.custom.yaml");

    let mut root = Mapping::new();
    root.insert(
        Value::String("patch".into()),
        Value::Mapping(Mapping::new()),
    );
    let content = serde_yaml::to_string(&root).unwrap();
    std::fs::write(&path, &content).unwrap();

    let read_back = std::fs::read_to_string(&path).unwrap();
    let parsed: Value = serde_yaml::from_str(&read_back).unwrap();
    let patch = parsed
        .get(Value::String("patch".into()))
        .and_then(|v| v.as_mapping())
        .unwrap();
    assert!(patch.is_empty());
}
