//! Ошибки Kubernetes модуля

use thiserror::Error;

/// Ошибки Kubernetes операций
#[derive(Error, Debug)]
pub enum KubeError {
    #[error("Kubernetes API error: {0}")]
    ApiError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("YAML parse error: {0}")]
    YamlError(String),

    #[error("JSON parse error: {0}")]
    JsonError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("RBAC error: {0}")]
    RBACError(String),
}

impl From<kube::Error> for KubeError {
    fn from(err: kube::Error) -> Self {
        KubeError::ApiError(err.to_string())
    }
}

impl From<serde_yaml::Error> for KubeError {
    fn from(err: serde_yaml::Error) -> Self {
        KubeError::YamlError(err.to_string())
    }
}

impl From<serde_json::Error> for KubeError {
    fn from(err: serde_json::Error) -> Self {
        KubeError::JsonError(err.to_string())
    }
}

/// Тип результата для Kubernetes операций
pub type KubeResult<T> = Result<T, KubeError>;
