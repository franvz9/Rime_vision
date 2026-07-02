use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use ts_rs::TS;

use crate::rime::color_scheme::RimeColorScheme;
use crate::rime::config::{self, RimeConfig};
use crate::rime::error::RimeError;
use crate::rime::patch;
use crate::rime::style::RimeStyle;

/// Known color field names used in Rime's preset_color_schemes.
/// These must be written as 0x-prefixed hex strings because Squirrel's
/// color parser (`SquirrelConfig.color(from:inSpace:)`) only accepts `0x` format.
const COLOR_FIELD_NAMES: &[&str] = &[
    "back_color",
    "hilited_back_color",
    "hilited_candidate_back_color",
    "preedit_back_color",
    "candidate_back_color",
    "border_color",
    "text_color",
    "hilited_text_color",
    "candidate_text_color",
    "hilited_candidate_text_color",
    "label_color",
    "hilited_candidate_label_color",
    "comment_text_color",
    "hilited_comment_text_color",
];

/// After serde_yaml serialises the custom YAML, integer colour values that
/// were originally written as `0xEEFFFFFF` in the base config get turned into
/// decimal numbers (`4009754623`).  Squirrel's colour parser only recognises
/// the `0x` hex-prefixed string format, so we post-process the file to convert
/// those decimal values back to hex.
fn fix_decimal_colors(file_path: &std::path::Path) {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: fix_decimal_colors: failed to read {}: {}", file_path.display(), e);
            return;
        }
    };

    let mut changed = false;
    let mut new_content = String::with_capacity(content.len());

    for line in content.lines() {
        let trimmed = line.trim();
        let mut matched = false;

        for field in COLOR_FIELD_NAMES {
            let prefix = format!("{}: ", field);
            if trimmed.starts_with(&prefix) {
                let value_str = &trimmed[prefix.len()..];
                // Only convert plain decimal integers (≥ 7 digits to avoid
                // accidentally touching small legitimate decimal values like 0).
                if value_str.len() >= 7 && value_str.chars().all(|c| c.is_ascii_digit()) {
                    if let Ok(num) = value_str.parse::<u64>() {
                        let hex = format!("0x{:X}", num);
                        let indent = &line[..line.len() - trimmed.len()];
                        new_content.push_str(&format!("{}{}: {}", indent, field, hex));
                        new_content.push('\n');
                        changed = true;
                        matched = true;
                        break;
                    }
                }
            }
        }

        if !matched {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    // Preserve trailing newline state
    if !content.ends_with('\n') {
        new_content.pop(); // remove the extra trailing newline
    }

    if changed {
        if let Err(e) = std::fs::write(file_path, &new_content) {
            eprintln!("Warning: fix_decimal_colors: failed to write {}: {}", file_path.display(), e);
        } else {
            eprintln!("WeaselVision: fix_decimal_colors – converted decimal colour values to hex in {}", file_path.display());
        }
    }
}

/// Ensure all base themes from squirrel.yaml are present in the patch's
/// `preset_color_schemes`.  Rime's patch merge replaces the entire
/// `preset_color_schemes` key, so without this step any theme that only
/// exists in the base config would be lost.
fn ensure_base_themes_in_patch(patch: &mut serde_yaml::Mapping, cfg: &RimeConfig) {
    let base_value = cfg.load_yaml(&cfg.style_path()).unwrap_or_else(|e| {
        eprintln!("Warning: failed to load base config for base themes: {}", e);
        Value::Mapping(serde_yaml::Mapping::new())
    });
    let base_map = config::value_as_mapping(&base_value);
    let base_schemes = config::get_mapping(base_map, "preset_color_schemes");
    if base_schemes.is_empty() {
        return;
    }

    let schemes = patch
        .entry(Value::String("preset_color_schemes".into()))
        .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()));

    if let Some(schemes_map) = schemes.as_mapping_mut() {
        for (key, value) in base_schemes.iter() {
            // Only inject themes that are NOT already in the custom config
            // (custom themes take priority over base themes)
            if !schemes_map.contains_key(key) {
                schemes_map.insert(key.clone(), value.clone());
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn set_style_patch_value(patch: &mut serde_yaml::Mapping, key: &str, value: Value) {
    patch.insert(Value::String(format!("style/{}", key)), value);
    if let Some(style) = patch.get_mut(Value::String("style".into())) {
        if let Some(style_map) = style.as_mapping_mut() {
            style_map.remove(Value::String(key.into()));
            if style_map.is_empty() {
                patch.remove(Value::String("style".into()));
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn set_style_patch_value(patch: &mut serde_yaml::Mapping, key: &str, value: Value) {
    let style_section = patch
        .entry(Value::String("style".into()))
        .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()))
        .as_mapping_mut()
        .expect("style patch entry must be a mapping");
    style_section.insert(Value::String(key.into()), value);
}

#[derive(Serialize, Deserialize, TS)]
pub struct StyleData {
    pub style: RimeStyle,
    pub light_schemes: HashMap<String, RimeColorScheme>,
    pub dark_schemes: HashMap<String, RimeColorScheme>,
    pub selected_light: String,
    pub selected_dark: String,
}

#[tauri::command]
pub fn get_style_data() -> Result<StyleData, RimeError> {
    let cfg = RimeConfig::detect();

    // Gracefully handle missing or malformed base/custom files
    let base_value = cfg.load_yaml(&cfg.style_path()).unwrap_or_else(|e| {
        eprintln!(
            "Warning: failed to load {}: {}, using empty config",
            cfg.style_path().display(),
            e
        );
        Value::Mapping(serde_yaml::Mapping::new())
    });
    let custom_value = cfg.load_yaml(&cfg.style_custom_path()).unwrap_or_else(|e| {
        eprintln!(
            "Warning: failed to load {}: {}, using empty config",
            cfg.style_custom_path().display(),
            e
        );
        Value::Mapping(serde_yaml::Mapping::new())
    });

    let base = config::value_as_mapping(&base_value);
    let merged_value = if let Some(patch_map) = custom_value.get(Value::String("patch".into())) {
        if let Some(pm) = patch_map.as_mapping() {
            Value::Mapping(patch::merge(base, pm))
        } else {
            base_value.clone()
        }
    } else {
        base_value.clone()
    };

    let merged = config::value_as_mapping(&merged_value);
    let style_dict = config::get_mapping(merged, "style");
    let style: RimeStyle = serde_yaml::from_value(Value::Mapping(style_dict.clone()))
        .unwrap_or_else(|e| {
            eprintln!("Warning: failed to parse style YAML, using defaults: {}", e);
            RimeStyle::default()
        });

    let schemes_dict = config::get_mapping(merged, "preset_color_schemes");
    let mut light_schemes = HashMap::new();
    let mut dark_schemes = HashMap::new();

    for (key, value) in schemes_dict {
        if let Some(name) = key.as_str() {
            if let Some(scheme_dict) = value.as_mapping() {
                let scheme = RimeColorScheme::from_dict(name.to_string(), scheme_dict);
                if scheme.is_dark() {
                    dark_schemes.insert(name.to_string(), scheme);
                } else {
                    light_schemes.insert(name.to_string(), scheme);
                }
            }
        }
    }

    let selected_light = style.color_scheme_name.clone();
    let selected_dark = style.color_scheme_dark_name.clone();

    Ok(StyleData {
        style,
        light_schemes,
        dark_schemes,
        selected_light,
        selected_dark,
    })
}

#[tauri::command]
pub fn save_style(new_style: RimeStyle) -> Result<(), RimeError> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.style_custom_path(), |patch| {
        set_style_patch_value(patch, "color_scheme", Value::String(new_style.color_scheme_name));
        set_style_patch_value(
            patch,
            "color_scheme_dark",
            Value::String(new_style.color_scheme_dark_name),
        );
        set_style_patch_value(
            patch,
            "status_message_type",
            Value::String(new_style.status_message_type),
        );
        set_style_patch_value(
            patch,
            "candidate_format",
            Value::String(new_style.candidate_format),
        );
        set_style_patch_value(
            patch,
            "text_orientation",
            Value::String(new_style.text_orientation),
        );
        set_style_patch_value(patch, "inline_preedit", Value::Bool(new_style.inline_preedit));
        set_style_patch_value(patch, "inline_candidate", Value::Bool(new_style.inline_candidate));
        set_style_patch_value(patch, "translucency", Value::Bool(new_style.translucency));
        set_style_patch_value(patch, "mutual_exclusive", Value::Bool(new_style.mutual_exclusive));
        set_style_patch_value(patch, "memorize_size", Value::Bool(new_style.memorize_size));
        set_style_patch_value(patch, "show_paging", Value::Bool(new_style.show_paging));
        #[cfg(target_os = "windows")]
        let horizontal_candidates = new_style.candidate_list_layout == "linear";
        set_style_patch_value(
            patch,
            "candidate_list_layout",
            Value::String(new_style.candidate_list_layout),
        );
        #[cfg(target_os = "windows")]
        set_style_patch_value(patch, "horizontal", Value::Bool(horizontal_candidates));
        set_style_patch_value(patch, "alpha", Value::Number(new_style.alpha.into()));
        set_style_patch_value(
            patch,
            "corner_radius",
            Value::Number(new_style.corner_radius.into()),
        );
        set_style_patch_value(
            patch,
            "hilited_corner_radius",
            Value::Number(new_style.hilited_corner_radius.into()),
        );
        set_style_patch_value(
            patch,
            "border_height",
            Value::Number(new_style.border_height.into()),
        );
        set_style_patch_value(
            patch,
            "border_width",
            Value::Number(new_style.border_width.into()),
        );
        set_style_patch_value(
            patch,
            "line_spacing",
            Value::Number(new_style.line_spacing.into()),
        );
        set_style_patch_value(patch, "spacing", Value::Number(new_style.spacing.into()));
        set_style_patch_value(
            patch,
            "shadow_size",
            Value::Number(new_style.shadow_size.into()),
        );
        set_style_patch_value(
            patch,
            "font_point",
            Value::Number(new_style.font_point.into()),
        );
        set_style_patch_value(
            patch,
            "label_font_point",
            Value::Number(new_style.label_font_point.into()),
        );
        set_style_patch_value(
            patch,
            "comment_font_point",
            Value::Number(new_style.comment_font_point.into()),
        );

        if !new_style.font_face.is_empty() {
            set_style_patch_value(patch, "font_face", Value::String(new_style.font_face));
        }
        if !new_style.label_font_face.is_empty() {
            set_style_patch_value(
                patch,
                "label_font_face",
                Value::String(new_style.label_font_face),
            );
        }
        if !new_style.comment_font_face.is_empty() {
            set_style_patch_value(
                patch,
                "comment_font_face",
                Value::String(new_style.comment_font_face),
            );
        }

        Ok(())
    })?;
    fix_decimal_colors(&cfg.style_custom_path());
    Ok(())
}

#[tauri::command]
pub fn save_selected_schemes(light: String, dark: String) -> Result<(), RimeError> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.style_custom_path(), |patch| {
        set_style_patch_value(patch, "color_scheme", Value::String(light));
        set_style_patch_value(patch, "color_scheme_dark", Value::String(dark));

        Ok(())
    })?;
    fix_decimal_colors(&cfg.style_custom_path());
    Ok(())
}

#[tauri::command]
pub fn save_color_scheme(
    name: String,
    scheme: RimeColorScheme,
    original_name: Option<String>,
) -> Result<(), RimeError> {
    // Validate scheme name to prevent YAML injection
    if name.is_empty() || name.len() > 64 {
        return Err(RimeError::Config("配色方案名称长度无效".into()));
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == ' ')
    {
        return Err(RimeError::Config("配色方案名称包含无效字符".into()));
    }

    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.style_custom_path(), |patch| {
        // Inject base themes first so they are not lost by Rime's key-level override
        ensure_base_themes_in_patch(patch, &cfg);

        let schemes = patch
            .entry(Value::String("preset_color_schemes".into()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()));

        if let Some(schemes_map) = schemes.as_mapping_mut() {
            // If renaming, remove the old key first
            if let Some(ref orig) = original_name {
                if orig != &name {
                    schemes_map.remove(Value::String(orig.clone()));
                }
            }
            let scheme_value = scheme.to_dict();
            schemes_map.insert(Value::String(name), scheme_value);
        }

        Ok(())
    })?;
    fix_decimal_colors(&cfg.style_custom_path());
    Ok(())
}

#[tauri::command]
pub fn delete_color_scheme(name: String) -> Result<(), RimeError> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.style_custom_path(), |patch| {
        // Inject base themes first so they survive Rime's key-level override
        ensure_base_themes_in_patch(patch, &cfg);

        if let Some(schemes) = patch.get_mut(Value::String("preset_color_schemes".into())) {
            if let Some(schemes_map) = schemes.as_mapping_mut() {
                schemes_map.remove(Value::String(name.clone()));
            }
        }

        let light_key = Value::String("style/color_scheme".into());
        let dark_key = Value::String("style/color_scheme_dark".into());
        let should_reset_light = patch.get(&light_key).and_then(|v| v.as_str()) == Some(&name)
            || patch
                .get(Value::String("style".into()))
                .and_then(|v| v.as_mapping())
                .and_then(|m| m.get(Value::String("color_scheme".into())))
                .and_then(|v| v.as_str())
                == Some(&name);
        let should_reset_dark = patch.get(&dark_key).and_then(|v| v.as_str()) == Some(&name)
            || patch
                .get(Value::String("style".into()))
                .and_then(|v| v.as_mapping())
                .and_then(|m| m.get(Value::String("color_scheme_dark".into())))
                .and_then(|v| v.as_str())
                == Some(&name);

        if should_reset_light {
            set_style_patch_value(patch, "color_scheme", Value::String("native".into()));
        }
        if should_reset_dark {
            set_style_patch_value(patch, "color_scheme_dark", Value::String("native".into()));
        }

        Ok(())
    })?;
    fix_decimal_colors(&cfg.style_custom_path());
    Ok(())
}

#[tauri::command]
pub fn import_color_scheme(file_path: String) -> Result<(), RimeError> {
    // Validate file path
    super::dict::validate_import_path(&file_path)?;
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Err(RimeError::NotFound(format!(
            "File not found: {}",
            file_path
        )));
    }
    if !path.is_file() {
        return Err(RimeError::InvalidPath("Path is not a file".into()));
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "yaml" && ext != "yml" {
        return Err(RimeError::Config(
            "Invalid file extension: only .yaml and .yml are supported".into(),
        ));
    }
    // Read the YAML file (with size limit)
    let metadata = std::fs::metadata(&file_path)?;
    if metadata.len() > 10 * 1024 * 1024 {
        return Err(RimeError::Io(format!(
            "File too large: {} bytes",
            metadata.len()
        )));
    }
    let content = std::fs::read_to_string(&file_path)?;

    // Parse YAML
    let yaml_value: Value = serde_yaml::from_str(&content)?;

    // Check for preset_color_schemes
    if let Some(schemes) = yaml_value.get(Value::String("preset_color_schemes".into())) {
        if let Some(schemes_map) = schemes.as_mapping() {
            // Merge into custom config
            let cfg = RimeConfig::detect();
            cfg.save_patch(&cfg.style_custom_path(), |patch| {
                // Inject base themes first
                ensure_base_themes_in_patch(patch, &cfg);

                let existing_schemes = patch
                    .entry(Value::String("preset_color_schemes".into()))
                    .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()));

                if let Some(existing_map) = existing_schemes.as_mapping_mut() {
                    for (key, value) in schemes_map {
                        existing_map.insert(key.clone(), value.clone());
                    }
                }

                Ok(())
            })?;
            fix_decimal_colors(&cfg.style_custom_path());

            return Ok(());
        }
    }

    Err(RimeError::NotFound("No color schemes found in file".into()))
}

