use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RimeStyle {
    #[serde(rename = "color_scheme", default = "default_native")]
    pub color_scheme_name: String,

    #[serde(rename = "color_scheme_dark", default = "default_native")]
    pub color_scheme_dark_name: String,

    #[serde(rename = "status_message_type", default = "default_mix")]
    pub status_message_type: String,

    #[serde(rename = "candidate_format", default = "default_candidate_format")]
    pub candidate_format: String,

    #[serde(rename = "text_orientation", default = "default_horizontal")]
    pub text_orientation: String,

    #[serde(rename = "inline_preedit", default = "default_true")]
    pub inline_preedit: bool,

    #[serde(rename = "inline_candidate", default)]
    pub inline_candidate: bool,

    #[serde(rename = "translucency", default)]
    pub translucency: bool,

    #[serde(rename = "mutual_exclusive", default)]
    pub mutual_exclusive: bool,

    #[serde(rename = "memorize_size", default = "default_true")]
    pub memorize_size: bool,

    #[serde(rename = "show_paging", default)]
    pub show_paging: bool,

    #[serde(rename = "candidate_list_layout", default = "default_stacked")]
    pub candidate_list_layout: String,

    #[serde(rename = "alpha", default = "default_1_0")]
    pub alpha: f64,

    #[serde(rename = "corner_radius", default = "default_10")]
    pub corner_radius: f64,

    #[serde(rename = "hilited_corner_radius", default)]
    pub hilited_corner_radius: f64,

    #[serde(rename = "border_height", default)]
    pub border_height: f64,

    #[serde(rename = "border_width", default)]
    pub border_width: f64,

    #[serde(rename = "line_spacing", default = "default_5")]
    pub line_spacing: f64,

    #[serde(rename = "spacing", default = "default_10")]
    pub spacing: f64,

    #[serde(rename = "shadow_size", default)]
    pub shadow_size: f64,

    #[serde(rename = "font_face", default = "platform_default_font")]
    pub font_face: String,

    #[serde(rename = "font_point", default = "default_16")]
    pub font_point: f64,

    #[serde(rename = "label_font_face", default = "platform_label_font")]
    pub label_font_face: String,

    #[serde(rename = "label_font_point", default = "default_16")]
    pub label_font_point: f64,

    #[serde(rename = "comment_font_face", default = "platform_default_font")]
    pub comment_font_face: String,

    #[serde(rename = "comment_font_point", default = "default_14")]
    pub comment_font_point: f64,
}

fn default_native() -> String {
    "native".to_string()
}
fn default_mix() -> String {
    "mix".to_string()
}
fn default_candidate_format() -> String {
    "[label]. [candidate] [comment]".to_string()
}
fn default_horizontal() -> String {
    "horizontal".to_string()
}
fn default_stacked() -> String {
    "stacked".to_string()
}
fn default_true() -> bool {
    true
}
fn default_1_0() -> f64 {
    1.0
}
fn default_10() -> f64 {
    10.0
}
fn default_5() -> f64 {
    5.0
}
fn default_16() -> f64 {
    16.0
}
fn default_14() -> f64 {
    14.0
}

impl Default for RimeStyle {
    fn default() -> Self {
        Self {
            color_scheme_name: "native".to_string(),
            color_scheme_dark_name: "native".to_string(),
            status_message_type: "mix".to_string(),
            candidate_format: "[label]. [candidate] [comment]".to_string(),
            text_orientation: "horizontal".to_string(),
            inline_preedit: true,
            inline_candidate: false,
            translucency: false,
            mutual_exclusive: false,
            memorize_size: true,
            show_paging: false,
            candidate_list_layout: "stacked".to_string(),
            alpha: 1.0,
            corner_radius: 10.0,
            hilited_corner_radius: 0.0,
            border_height: 0.0,
            border_width: 0.0,
            line_spacing: 5.0,
            spacing: 10.0,
            shadow_size: 0.0,
            font_face: platform_default_font(),
            font_point: 16.0,
            label_font_face: platform_label_font(),
            label_font_point: 16.0,
            comment_font_face: platform_default_font(),
            comment_font_point: 14.0,
        }
    }
}

fn platform_default_font() -> String {
    if cfg!(target_os = "macos") {
        "PingFang SC".to_string()
    } else if cfg!(target_os = "windows") {
        "Microsoft YaHei".to_string()
    } else {
        "Noto Sans CJK SC".to_string()
    }
}

fn platform_label_font() -> String {
    if cfg!(target_os = "macos") {
        "Lucida Grande".to_string()
    } else if cfg!(target_os = "windows") {
        "Segoe UI".to_string()
    } else {
        "Noto Sans".to_string()
    }
}
