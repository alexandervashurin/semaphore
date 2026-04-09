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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kube_error_api() {
        let err = KubeError::ApiError("API failed".to_string());
        assert!(err.to_string().contains("API error"));
    }

    #[test]
    fn test_kube_error_not_found() {
        let err = KubeError::NotFound("pod not found".to_string());
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_kube_error_validation() {
        let err = KubeError::ValidationError("invalid spec".to_string());
        assert!(err.to_string().contains("Validation"));
    }

    #[test]
    fn test_kube_error_connection() {
        let err = KubeError::ConnectionError("timeout".to_string());
        assert!(err.to_string().contains("Connection"));
    }

    #[test]
    fn test_kube_error_config() {
        let err = KubeError::ConfigError("missing config".to_string());
        assert!(err.to_string().contains("Config"));
    }

    #[test]
    fn test_kube_error_yaml() {
        let err = KubeError::YamlError("parse error".to_string());
        assert!(err.to_string().contains("YAML"));
    }

    #[test]
    fn test_kube_error_json() {
        let err = KubeError::JsonError("invalid json".to_string());
        assert!(err.to_string().contains("JSON"));
    }

    #[test]
    fn test_kube_error_websocket() {
        let err = KubeError::WebSocketError("connection lost".to_string());
        assert!(err.to_string().contains("WebSocket"));
    }

    #[test]
    fn test_kube_error_rbac() {
        let err = KubeError::RBACError("forbidden".to_string());
        assert!(err.to_string().contains("RBAC"));
    }
}
