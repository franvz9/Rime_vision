use serde::{Deserialize, Serialize};

use super::color::RimeColor;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RimeColorScheme {
    pub name: String,

    #[serde(default)]
    pub author: String,

    #[serde(rename = "color_space", default = "default_srgb")]
    pub color_space: String,

    #[serde(rename = "back_color")]
    pub back_color: Option<RimeColor>,

    #[serde(rename = "border_color")]
    pub border_color: Option<RimeColor>,

    #[serde(rename = "text_color")]
    pub text_color: Option<RimeColor>,

    #[serde(rename = "hilited_text_color")]
    pub hilited_text_color: Option<RimeColor>,

    #[serde(rename = "hilited_back_color")]
    pub hilited_back_color: Option<RimeColor>,

    #[serde(rename = "hilited_candidate_back_color")]
    pub hilited_candidate_back_color: Option<RimeColor>,

    #[serde(rename = "candidate_text_color")]
    pub candidate_text_color: Option<RimeColor>,

    #[serde(rename = "hilited_candidate_text_color")]
    pub hilited_candidate_text_color: Option<RimeColor>,

    #[serde(rename = "label_color")]
    pub candidate_label_color: Option<RimeColor>,

    #[serde(rename = "hilited_candidate_label_color")]
    pub hilited_candidate_label_color: Option<RimeColor>,

    #[serde(rename = "comment_text_color")]
    pub comment_text_color: Option<RimeColor>,

    #[serde(rename = "hilited_comment_text_color")]
    pub hilited_comment_text_color: Option<RimeColor>,

    #[serde(rename = "preedit_back_color")]
    pub preedit_back_color: Option<RimeColor>,

    #[serde(rename = "candidate_back_color")]
    pub candidate_back_color: Option<RimeColor>,

    #[serde(default)]
    pub translucency: Option<bool>,

    #[serde(default)]
    pub mutual_exclusive: Option<bool>,

    #[serde(rename = "shadow_size")]
    pub shadow_size: Option<f64>,

    #[serde(rename = "line_spacing")]
    pub line_spacing: Option<f64>,

    pub alpha: Option<f64>,

    pub spacing: Option<f64>,

    #[serde(rename = "candidate_list_layout")]
    pub candidate_list_layout: Option<String>,

    #[serde(rename = "inline_preedit")]
    pub inline_preedit: Option<bool>,

    #[serde(rename = "candidate_format")]
    pub candidate_format: Option<String>,

    #[serde(rename = "corner_radius")]
    pub corner_radius: Option<f64>,

    #[serde(rename = "hilited_corner_radius")]
    pub hilited_corner_radius: Option<f64>,

    #[serde(rename = "border_width")]
    pub border_width: Option<f64>,

    #[serde(rename = "border_height")]
    pub border_height: Option<f64>,

    #[serde(rename = "font_face")]
    pub font_face: Option<String>,

    #[serde(rename = "font_point")]
    pub font_point: Option<f64>,

    #[serde(rename = "label_font_face")]
    pub label_font_face: Option<String>,

    #[serde(rename = "label_font_point")]
    pub label_font_point: Option<f64>,

    #[serde(rename = "comment_font_face")]
    pub comment_font_face: Option<String>,

    #[serde(rename = "comment_font_point")]
    pub comment_font_point: Option<f64>,
}

fn default_srgb() -> String {
    "srgb".to_string()
}

impl Default for RimeColorScheme {
    fn default() -> Self {
        Self {
            name: String::new(),
            author: String::new(),
            color_space: "srgb".to_string(),
            back_color: None,
            border_color: None,
            text_color: None,
            hilited_text_color: None,
            hilited_back_color: None,
            hilited_candidate_back_color: None,
            candidate_text_color: None,
            hilited_candidate_text_color: None,
            candidate_label_color: None,
            hilited_candidate_label_color: None,
            comment_text_color: None,
            hilited_comment_text_color: None,
            preedit_back_color: None,
            candidate_back_color: None,
            translucency: None,
            mutual_exclusive: None,
            shadow_size: None,
            line_spacing: None,
            alpha: None,
            spacing: None,
            candidate_list_layout: None,
            inline_preedit: None,
            candidate_format: None,
            corner_radius: None,
            hilited_corner_radius: None,
            border_width: None,
            border_height: None,
            font_face: None,
            font_point: None,
            label_font_face: None,
            label_font_point: None,
            comment_font_face: None,
            comment_font_point: None,
        }
    }
}

impl RimeColorScheme {
    pub fn from_dict(name: String, dict: &serde_yaml::Mapping) -> Self {
        let mut scheme = Self {
            name,
            ..Default::default()
        };

        if let Some(v) = dict.get("author").and_then(|v| v.as_str()) {
            scheme.author = v.to_string();
        }
        if let Some(v) = dict.get("color_space").and_then(|v| v.as_str()) {
            scheme.color_space = v.to_string();
        }

        scheme.back_color = parse_color(dict, "back_color");
        scheme.border_color = parse_color(dict, "border_color");
        scheme.text_color = parse_color(dict, "text_color");
        scheme.hilited_text_color = parse_color(dict, "hilited_text_color");
        scheme.hilited_back_color = parse_color(dict, "hilited_back_color");
        scheme.hilited_candidate_back_color = parse_color(dict, "hilited_candidate_back_color");
        scheme.candidate_text_color = parse_color(dict, "candidate_text_color");
        scheme.hilited_candidate_text_color = parse_color(dict, "hilited_candidate_text_color");
        scheme.candidate_label_color = parse_color(dict, "label_color");
        scheme.hilited_candidate_label_color = parse_color(dict, "hilited_candidate_label_color");
        scheme.comment_text_color = parse_color(dict, "comment_text_color");
        scheme.hilited_comment_text_color = parse_color(dict, "hilited_comment_text_color");
        scheme.preedit_back_color = parse_color(dict, "preedit_back_color");
        scheme.candidate_back_color = parse_color(dict, "candidate_back_color");

        scheme.translucency = dict.get("translucency").and_then(|v| v.as_bool());
        scheme.mutual_exclusive = dict.get("mutual_exclusive").and_then(|v| v.as_bool());
        scheme.shadow_size = parse_f64(dict, "shadow_size");
        scheme.line_spacing = parse_f64(dict, "line_spacing");
        scheme.alpha = parse_f64(dict, "alpha");
        scheme.spacing = parse_f64(dict, "spacing");
        scheme.candidate_list_layout = parse_string(dict, "candidate_list_layout");
        scheme.inline_preedit = parse_bool(dict, "inline_preedit");
        scheme.candidate_format = parse_string(dict, "candidate_format");
        scheme.corner_radius = parse_f64(dict, "corner_radius");
        scheme.hilited_corner_radius = parse_f64(dict, "hilited_corner_radius");
        scheme.border_width = parse_f64(dict, "border_width");
        scheme.border_height = parse_f64(dict, "border_height");
        scheme.font_face = parse_string(dict, "font_face");
        scheme.font_point = parse_f64(dict, "font_point");
        scheme.label_font_face = parse_string(dict, "label_font_face");
        scheme.label_font_point = parse_f64(dict, "label_font_point");
        scheme.comment_font_face = parse_string(dict, "comment_font_face");
        scheme.comment_font_point = parse_f64(dict, "comment_font_point");

        scheme
    }

    pub fn to_dict(&self) -> serde_yaml::Value {
        let mut dict = serde_yaml::Mapping::new();

        dict.insert(
            serde_yaml::Value::String("name".into()),
            serde_yaml::Value::String(self.name.clone()),
        );

        if !self.author.is_empty() {
            dict.insert(
                serde_yaml::Value::String("author".into()),
                serde_yaml::Value::String(self.author.clone()),
            );
        }
        if self.color_space != "srgb" {
            dict.insert(
                serde_yaml::Value::String("color_space".into()),
                serde_yaml::Value::String(self.color_space.clone()),
            );
        }

        insert_color(&mut dict, "back_color", self.back_color);
        insert_color(&mut dict, "border_color", self.border_color);
        insert_color(&mut dict, "text_color", self.text_color);
        insert_color(&mut dict, "hilited_text_color", self.hilited_text_color);
        insert_color(&mut dict, "hilited_back_color", self.hilited_back_color);
        insert_color(
            &mut dict,
            "hilited_candidate_back_color",
            self.hilited_candidate_back_color,
        );
        insert_color(
            &mut dict,
            "candidate_text_color",
            self.candidate_text_color,
        );
        insert_color(
            &mut dict,
            "hilited_candidate_text_color",
            self.hilited_candidate_text_color,
        );
        insert_color(&mut dict, "label_color", self.candidate_label_color);
        insert_color(
            &mut dict,
            "hilited_candidate_label_color",
            self.hilited_candidate_label_color,
        );
        insert_color(
            &mut dict,
            "comment_text_color",
            self.comment_text_color,
        );
        insert_color(
            &mut dict,
            "hilited_comment_text_color",
            self.hilited_comment_text_color,
        );
        insert_color(&mut dict, "preedit_back_color", self.preedit_back_color);
        insert_color(
            &mut dict,
            "candidate_back_color",
            self.candidate_back_color,
        );

        insert_opt_bool(&mut dict, "translucency", self.translucency);
        insert_opt_bool(&mut dict, "mutual_exclusive", self.mutual_exclusive);
        insert_opt_f64(&mut dict, "shadow_size", self.shadow_size);
        insert_opt_f64(&mut dict, "line_spacing", self.line_spacing);
        insert_opt_f64(&mut dict, "alpha", self.alpha);
        insert_opt_f64(&mut dict, "spacing", self.spacing);
        insert_opt_string(
            &mut dict,
            "candidate_list_layout",
            self.candidate_list_layout.as_deref(),
        );
        insert_opt_bool(&mut dict, "inline_preedit", self.inline_preedit);
        insert_opt_string(
            &mut dict,
            "candidate_format",
            self.candidate_format.as_deref(),
        );
        insert_opt_f64(&mut dict, "corner_radius", self.corner_radius);
        insert_opt_f64(&mut dict, "hilited_corner_radius", self.hilited_corner_radius);
        insert_opt_f64(&mut dict, "border_width", self.border_width);
        insert_opt_f64(&mut dict, "border_height", self.border_height);
        insert_opt_string(&mut dict, "font_face", self.font_face.as_deref());
        insert_opt_f64(&mut dict, "font_point", self.font_point);
        insert_opt_string(&mut dict, "label_font_face", self.label_font_face.as_deref());
        insert_opt_f64(&mut dict, "label_font_point", self.label_font_point);
        insert_opt_string(
            &mut dict,
            "comment_font_face",
            self.comment_font_face.as_deref(),
        );
        insert_opt_f64(&mut dict, "comment_font_point", self.comment_font_point);

        serde_yaml::Value::Mapping(dict)
    }

    pub fn is_dark(&self) -> bool {
        if self.name.to_lowercase().contains("dark") {
            return true;
        }
        if let Some(back) = self.back_color {
            let luminance =
                (0.299 * back.r as f64 + 0.587 * back.g as f64 + 0.114 * back.b as f64) / 255.0;
            return luminance < 0.4;
        }
        false
    }
}

fn parse_color(dict: &serde_yaml::Mapping, key: &str) -> Option<RimeColor> {
    let value = dict.get(serde_yaml::Value::String(key.into()))?;
    // Handle string format: "0xBBGGRR" or "0xAABBGGRR"
    if let Some(s) = value.as_str() {
        return RimeColor::from_hex(s);
    }
    // Handle integer format: 0xBBGGRR (serde_yaml parses unquoted 0x... as integers)
    if let Some(i) = value.as_i64() {
        if (0..=0xFFFFFF).contains(&i) {
            // 6-digit: 0xBBGGRR -> pad with alpha=FF
            let hex = format!("0xFF{:06X}", i as u32);
            return RimeColor::from_hex(&hex);
        }
        if i > 0xFFFFFF && i <= 0xFFFFFFFF {
            // 8-digit: 0xAABBGGRR
            let hex = format!("0x{:08X}", i as u32);
            return RimeColor::from_hex(&hex);
        }
    }
    None
}

fn parse_string(dict: &serde_yaml::Mapping, key: &str) -> Option<String> {
    dict.get(serde_yaml::Value::String(key.into()))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn parse_bool(dict: &serde_yaml::Mapping, key: &str) -> Option<bool> {
    dict.get(serde_yaml::Value::String(key.into()))
        .and_then(|v| v.as_bool())
}

fn parse_f64(dict: &serde_yaml::Mapping, key: &str) -> Option<f64> {
    dict.get(serde_yaml::Value::String(key.into()))
        .and_then(|v| {
            if let Some(f) = v.as_f64() {
                Some(f)
            } else {
                v.as_i64().map(|i| i as f64)
            }
        })
}

fn insert_color(dict: &mut serde_yaml::Mapping, key: &str, color: Option<RimeColor>) {
    if let Some(c) = color {
        dict.insert(
            serde_yaml::Value::String(key.into()),
            serde_yaml::Value::String(c.to_hex()),
        );
    }
}

fn insert_opt_bool(dict: &mut serde_yaml::Mapping, key: &str, val: Option<bool>) {
    if let Some(v) = val {
        dict.insert(
            serde_yaml::Value::String(key.into()),
            serde_yaml::Value::Bool(v),
        );
    }
}

fn insert_opt_f64(dict: &mut serde_yaml::Mapping, key: &str, val: Option<f64>) {
    if let Some(v) = val {
        dict.insert(
            serde_yaml::Value::String(key.into()),
            serde_yaml::Value::Number(v.into()),
        );
    }
}

fn insert_opt_string(dict: &mut serde_yaml::Mapping, key: &str, val: Option<&str>) {
    if let Some(v) = val {
        dict.insert(
            serde_yaml::Value::String(key.into()),
            serde_yaml::Value::String(v.into()),
        );
    }
}
