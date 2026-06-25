use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::rime::color_scheme::RimeColorScheme;
use crate::rime::config::{self, RimeConfig};
use crate::rime::patch;
use crate::rime::style::RimeStyle;

#[derive(Serialize, Deserialize)]
pub struct StyleData {
    pub style: RimeStyle,
    pub light_schemes: HashMap<String, RimeColorScheme>,
    pub dark_schemes: HashMap<String, RimeColorScheme>,
    pub selected_light: String,
    pub selected_dark: String,
}

#[tauri::command]
pub fn get_style_data() -> Result<StyleData, String> {
    let cfg = RimeConfig::detect();

    let base_value = cfg
        .load_yaml(&cfg.style_path())
        .map_err(|e| e.to_string())?;
    let custom_value = cfg
        .load_yaml(&cfg.style_custom_path())
        .map_err(|e| e.to_string())?;

    let base = config::value_as_mapping(&base_value);
    let merged_value = if let Some(patch_map) = custom_value.get(&Value::String("patch".into())) {
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
        .unwrap_or_default();

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
pub fn save_style(new_style: RimeStyle) -> Result<(), String> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.style_custom_path(), |patch| {
        let style_section = patch
            .entry(Value::String("style".into()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()))
            .as_mapping_mut()
            .ok_or_else(|| anyhow::anyhow!("style is not a mapping"))?;

        style_section.insert(Value::String("color_scheme".into()), Value::String(new_style.color_scheme_name));
        style_section.insert(Value::String("color_scheme_dark".into()), Value::String(new_style.color_scheme_dark_name));
        style_section.insert(Value::String("status_message_type".into()), Value::String(new_style.status_message_type));
        style_section.insert(Value::String("candidate_format".into()), Value::String(new_style.candidate_format));
        style_section.insert(Value::String("text_orientation".into()), Value::String(new_style.text_orientation));
        style_section.insert(Value::String("inline_preedit".into()), Value::Bool(new_style.inline_preedit));
        style_section.insert(Value::String("inline_candidate".into()), Value::Bool(new_style.inline_candidate));
        style_section.insert(Value::String("translucency".into()), Value::Bool(new_style.translucency));
        style_section.insert(Value::String("mutual_exclusive".into()), Value::Bool(new_style.mutual_exclusive));
        style_section.insert(Value::String("memorize_size".into()), Value::Bool(new_style.memorize_size));
        style_section.insert(Value::String("show_paging".into()), Value::Bool(new_style.show_paging));
        style_section.insert(Value::String("candidate_list_layout".into()), Value::String(new_style.candidate_list_layout));
        style_section.insert(Value::String("alpha".into()), Value::Number(new_style.alpha.into()));
        style_section.insert(Value::String("corner_radius".into()), Value::Number(new_style.corner_radius.into()));
        style_section.insert(Value::String("hilited_corner_radius".into()), Value::Number(new_style.hilited_corner_radius.into()));
        style_section.insert(Value::String("border_height".into()), Value::Number(new_style.border_height.into()));
        style_section.insert(Value::String("border_width".into()), Value::Number(new_style.border_width.into()));
        style_section.insert(Value::String("line_spacing".into()), Value::Number(new_style.line_spacing.into()));
        style_section.insert(Value::String("spacing".into()), Value::Number(new_style.spacing.into()));
        style_section.insert(Value::String("shadow_size".into()), Value::Number(new_style.shadow_size.into()));
        style_section.insert(Value::String("font_point".into()), Value::Number(new_style.font_point.into()));
        style_section.insert(Value::String("label_font_point".into()), Value::Number(new_style.label_font_point.into()));
        style_section.insert(Value::String("comment_font_point".into()), Value::Number(new_style.comment_font_point.into()));

        if !new_style.font_face.is_empty() {
            style_section.insert(Value::String("font_face".into()), Value::String(new_style.font_face));
        }
        if !new_style.label_font_face.is_empty() {
            style_section.insert(Value::String("label_font_face".into()), Value::String(new_style.label_font_face));
        }
        if !new_style.comment_font_face.is_empty() {
            style_section.insert(Value::String("comment_font_face".into()), Value::String(new_style.comment_font_face));
        }

        Ok(())
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_selected_schemes(light: String, dark: String) -> Result<(), String> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.style_custom_path(), |patch| {
        let style_section = patch
            .entry(Value::String("style".into()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()))
            .as_mapping_mut()
            .ok_or_else(|| anyhow::anyhow!("style is not a mapping"))?;

        style_section.insert(
            Value::String("color_scheme".into()),
            Value::String(light),
        );
        style_section.insert(
            Value::String("color_scheme_dark".into()),
            Value::String(dark),
        );

        Ok(())
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_color_scheme(
    name: String,
    scheme: RimeColorScheme,
) -> Result<(), String> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.style_custom_path(), |patch| {
        let schemes = patch
            .entry(Value::String("preset_color_schemes".into()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()));

        if let Some(schemes_map) = schemes.as_mapping_mut() {
            let scheme_value = scheme.to_dict();
            schemes_map.insert(Value::String(name), scheme_value);
        }

        Ok(())
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_color_scheme(name: String) -> Result<(), String> {
    let cfg = RimeConfig::detect();
    cfg.save_patch(&cfg.style_custom_path(), |patch| {
        if let Some(schemes) = patch.get_mut(&Value::String("preset_color_schemes".into())) {
            if let Some(schemes_map) = schemes.as_mapping_mut() {
                schemes_map.remove(&Value::String(name.clone()));
            }
        }

        let style_section = patch
            .entry(Value::String("style".into()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()))
            .as_mapping_mut()
            .ok_or_else(|| anyhow::anyhow!("style is not a mapping"))?;

        if style_section.get(&Value::String("color_scheme".into()))
            .and_then(|v| v.as_str())
            == Some(&name)
        {
            style_section.insert(
                Value::String("color_scheme".into()),
                Value::String("native".into()),
            );
        }
        if style_section.get(&Value::String("color_scheme_dark".into()))
            .and_then(|v| v.as_str())
            == Some(&name)
        {
            style_section.insert(
                Value::String("color_scheme_dark".into()),
                Value::String("native".into()),
            );
        }

        Ok(())
    })
    .map_err(|e| e.to_string())
}
