use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RimeSchema {
    #[serde(rename = "schema")]
    pub schema_id: String,

    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub version: String,

    #[serde(default)]
    pub dependencies: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl RimeSchema {
    pub fn new(schema_id: String) -> Self {
        Self {
            schema_id,
            enabled: true,
            name: String::new(),
            version: String::new(),
            dependencies: Vec::new(),
        }
    }
}
