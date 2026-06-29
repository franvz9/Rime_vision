use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GrammarModel {
    pub filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub display_name: String,
    pub formatted_size: String,
}

impl GrammarModel {
    pub fn new(filename: String, file_path: String, file_size: i64) -> Self {
        let display_name = filename.clone();
        let formatted_size = format_bytes(file_size);
        Self {
            filename,
            file_path,
            file_size,
            display_name,
            formatted_size,
        }
    }
}

fn format_bytes(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaGrammarConfig {
    pub schema_id: String,
    pub mounted_model: Option<String>,
    pub collocation_max_length: i64,
    pub collocation_min_length: i64,
    pub collocation_penalty: i64,
    pub non_collocation_penalty: i64,
    pub weak_collocation_penalty: i64,
    pub rear_penalty: i64,
    pub contextual_suggestions: bool,
    pub max_homophones: i64,
    pub max_homographs: i64,
}

impl Default for SchemaGrammarConfig {
    fn default() -> Self {
        Self {
            schema_id: String::new(),
            mounted_model: None,
            collocation_max_length: 5,
            collocation_min_length: 2,
            collocation_penalty: -16,
            non_collocation_penalty: -8,
            weak_collocation_penalty: -100,
            rear_penalty: -20,
            contextual_suggestions: true,
            max_homophones: 7,
            max_homographs: 7,
        }
    }
}

impl SchemaGrammarConfig {
    pub fn default_for(schema_id: &str) -> Self {
        Self {
            schema_id: schema_id.to_string(),
            ..Default::default()
        }
    }
}
