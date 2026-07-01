use serde::Serialize;

/// Structured error type for Rime operations.
///
/// Designed to be serialized and passed to the frontend so the UI can
/// display appropriate error messages and recovery actions.
///
/// TODO(v0.3.0): Migrate all Tauri commands from `Result<T, String>` to
/// `Result<T, RimeError>` for structured error handling.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "code", content = "detail")]
#[allow(dead_code)] // Reserved for v0.3.0 structured error migration
pub enum RimeError {
    /// I/O error (file not found, permission denied, etc.)
    #[error("文件操作失败: {0}")]
    Io(String),

    /// YAML/JSON parse error
    #[error("配置解析失败: {0}")]
    Parse(String),

    /// Invalid configuration or user input
    #[error("配置错误: {0}")]
    Config(String),

    /// Permission denied
    #[error("权限不足: {0}")]
    Permission(String),

    /// Path traversal attempted
    #[error("路径无效: {0}")]
    InvalidPath(String),

    /// Resource not found
    #[error("未找到: {0}")]
    NotFound(String),

    /// Rate limited
    #[error("操作过于频繁: {0}")]
    RateLimited(String),

    /// Unknown / unexpected error
    #[error("未知错误: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for RimeError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => RimeError::NotFound(e.to_string()),
            std::io::ErrorKind::PermissionDenied => RimeError::Permission(e.to_string()),
            _ => RimeError::Io(e.to_string()),
        }
    }
}

impl From<serde_yaml::Error> for RimeError {
    fn from(e: serde_yaml::Error) -> Self {
        RimeError::Parse(e.to_string())
    }
}

impl From<serde_json::Error> for RimeError {
    fn from(e: serde_json::Error) -> Self {
        RimeError::Parse(e.to_string())
    }
}

impl From<anyhow::Error> for RimeError {
    fn from(e: anyhow::Error) -> Self {
        RimeError::Unknown(e.to_string())
    }
}
