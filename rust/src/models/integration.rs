//! Модель интеграции

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Интеграция - вебхук для внешних систем
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Integration {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub project_id: i32,
    pub name: String,
    pub template_id: i32,
    /// Метод аутентификации: "none", "hmac", "token"
    #[serde(default)]
    pub auth_method: String,
    /// Заголовок HTTP для проверки токена/подписи
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    /// ID ключа (secret) для HMAC/token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_secret_id: Option<i32>,
}

/// Извлекаемое значение интеграции
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationExtractValue {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub integration_id: i32,
    #[serde(default)]
    pub project_id: i32,
    pub name: String,
    pub value_source: String,
    pub body_data_type: String,
    pub key: Option<String>,
    pub variable: Option<String>,
    pub value_name: String,
    pub value_type: String,
}

/// Матчер интеграции
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationMatcher {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub integration_id: i32,
    #[serde(default)]
    pub project_id: i32,
    pub name: String,
    pub body_data_type: String,
    pub key: Option<String>,
    pub matcher_type: String,
    pub matcher_value: String,
    pub method: String,
}

/// Псевдоним интеграции
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationAlias {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub integration_id: i32,
    #[serde(default)]
    pub project_id: i32,
    pub alias: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_serialization() {
        let integration = Integration {
            id: 1,
            project_id: 10,
            name: "Slack Webhook".to_string(),
            template_id: 5,
            auth_method: "hmac".to_string(),
            auth_header: Some("X-Signature".to_string()),
            auth_secret_id: Some(3),
        };
        let json = serde_json::to_string(&integration).unwrap();
        assert!(json.contains("\"name\":\"Slack Webhook\""));
        assert!(json.contains("\"auth_method\":\"hmac\""));
        assert!(json.contains("\"auth_header\":\"X-Signature\""));
    }

    #[test]
    fn test_integration_default_values() {
        let integration = Integration {
            id: 0,
            project_id: 0,
            name: String::new(),
            template_id: 0,
            auth_method: String::new(),
            auth_header: None,
            auth_secret_id: None,
        };
        let json = serde_json::to_string(&integration).unwrap();
        assert!(!json.contains("auth_header"));
        assert!(!json.contains("auth_secret_id"));
    }

    #[test]
    fn test_integration_extract_value_serialization() {
        let extract = IntegrationExtractValue {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Deploy URL".to_string(),
            value_source: "body".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.url".to_string()),
            variable: Some("DEPLOY_URL".to_string()),
            value_name: "url".to_string(),
            value_type: "string".to_string(),
        };
        let json = serde_json::to_string(&extract).unwrap();
        assert!(json.contains("\"name\":\"Deploy URL\""));
        assert!(json.contains("\"key\":\"$.url\""));
    }

    #[test]
    fn test_integration_matcher_serialization() {
        let matcher = IntegrationMatcher {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Task Started".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.event".to_string()),
            matcher_type: "equals".to_string(),
            matcher_value: "task_started".to_string(),
            method: "POST".to_string(),
        };
        let json = serde_json::to_string(&matcher).unwrap();
        assert!(json.contains("\"name\":\"Task Started\""));
        assert!(json.contains("\"method\":\"POST\""));
    }

    #[test]
    fn test_integration_alias_serialization() {
        let alias = IntegrationAlias {
            id: 1,
            integration_id: 10,
            project_id: 5,
            alias: "webhook-alias-123".to_string(),
        };
        let json = serde_json::to_string(&alias).unwrap();
        assert!(json.contains("\"alias\":\"webhook-alias-123\""));
        assert!(json.contains("\"integration_id\":10"));
    }

    #[test]
    fn test_integration_clone() {
        let integration = Integration {
            id: 1,
            project_id: 10,
            name: "Clone Test".to_string(),
            template_id: 5,
            auth_method: "token".to_string(),
            auth_header: Some("Authorization".to_string()),
            auth_secret_id: Some(1),
        };
        let cloned = integration.clone();
        assert_eq!(cloned.name, integration.name);
        assert_eq!(cloned.auth_method, integration.auth_method);
    }

    #[test]
    fn test_integration_extract_value_clone() {
        let extract = IntegrationExtractValue {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Clone Extract".to_string(),
            value_source: "body".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.key".to_string()),
            variable: Some("VAR".to_string()),
            value_name: "key".to_string(),
            value_type: "string".to_string(),
        };
        let cloned = extract.clone();
        assert_eq!(cloned.name, extract.name);
        assert_eq!(cloned.key, extract.key);
    }

    #[test]
    fn test_integration_matcher_clone() {
        let matcher = IntegrationMatcher {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Clone Matcher".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.event".to_string()),
            matcher_type: "equals".to_string(),
            matcher_value: "task_started".to_string(),
            method: "POST".to_string(),
        };
        let cloned = matcher.clone();
        assert_eq!(cloned.name, matcher.name);
        assert_eq!(cloned.matcher_type, matcher.matcher_type);
    }

    #[test]
    fn test_integration_extract_value_default_values() {
        let extract = IntegrationExtractValue {
            id: 0,
            integration_id: 0,
            project_id: 0,
            name: String::new(),
            value_source: String::new(),
            body_data_type: String::new(),
            key: None,
            variable: None,
            value_name: String::new(),
            value_type: String::new(),
        };
        let json = serde_json::to_string(&extract).unwrap();
        assert!(json.contains("\"key\":null"));
        assert!(json.contains("\"variable\":null"));
    }
}
