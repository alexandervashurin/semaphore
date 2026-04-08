//! Модель Hook - хуки для задач

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Тип хука
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
    /// HTTP запрос
    Http,
    /// Bash скрипт
    Bash,
    /// Python скрипт
    Python,
}

/// Хук для задачи
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Hook {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// ID шаблона
    pub template_id: i32,

    /// Название хука
    pub name: String,

    /// Тип хука
    pub r#type: HookType,

    /// URL (для HTTP)
    pub url: Option<String>,

    /// Скрипт (для Bash/Python)
    pub script: Option<String>,

    /// Метод HTTP (GET, POST, etc.)
    pub http_method: Option<String>,

    /// Тело запроса (для HTTP)
    pub http_body: Option<String>,

    /// Таймаут в секундах
    pub timeout_secs: Option<i32>,
}

impl Hook {
    /// Создаёт новый хук
    pub fn new(project_id: i32, template_id: i32, name: String, hook_type: HookType) -> Self {
        Self {
            id: 0,
            project_id,
            template_id,
            name,
            r#type: hook_type,
            url: None,
            script: None,
            http_method: None,
            http_body: None,
            timeout_secs: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_type_serialization() {
        assert_eq!(serde_json::to_string(&HookType::Http).unwrap(), "\"http\"");
        assert_eq!(serde_json::to_string(&HookType::Bash).unwrap(), "\"bash\"");
        assert_eq!(serde_json::to_string(&HookType::Python).unwrap(), "\"python\"");
    }

    #[test]
    fn test_hook_new_http() {
        let hook = Hook::new(10, 5, "Notify Slack".to_string(), HookType::Http);
        assert_eq!(hook.id, 0);
        assert_eq!(hook.project_id, 10);
        assert_eq!(hook.template_id, 5);
        assert_eq!(hook.name, "Notify Slack");
        assert_eq!(hook.r#type, HookType::Http);
        assert!(hook.url.is_none());
    }

    #[test]
    fn test_hook_new_bash() {
        let hook = Hook::new(1, 1, "Cleanup".to_string(), HookType::Bash);
        assert_eq!(hook.r#type, HookType::Bash);
        assert!(hook.script.is_none());
    }

    #[test]
    fn test_hook_serialization() {
        let hook = Hook {
            id: 1,
            project_id: 10,
            template_id: 5,
            name: "Webhook".to_string(),
            r#type: HookType::Http,
            url: Some("https://hooks.slack.com/xxx".to_string()),
            script: None,
            http_method: Some("POST".to_string()),
            http_body: Some(r#"{"text":"done"}"#.to_string()),
            timeout_secs: Some(30),
        };
        let json = serde_json::to_string(&hook).unwrap();
        assert!(json.contains("\"name\":\"Webhook\""));
        assert!(json.contains("\"type\":\"http\""));
        assert!(json.contains("\"http_method\":\"POST\""));
        assert!(json.contains("\"timeout_secs\":30"));
    }

    #[test]
    fn test_hook_serialization_skip_nulls() {
        let hook = Hook::new(1, 1, "Simple".to_string(), HookType::Bash);
        let json = serde_json::to_string(&hook).unwrap();
        // Hook struct doesn't have skip_serializing_if on Option fields
        // so null values are serialized as "url":null
        assert!(json.contains("\"url\":null"));
        assert!(json.contains("\"script\":null"));
        assert!(json.contains("\"http_method\":null"));
        assert!(json.contains("\"http_body\":null"));
        assert!(json.contains("\"timeout_secs\":null"));
    }

    #[test]
    fn test_hook_clone() {
        let hook = Hook {
            id: 1,
            project_id: 10,
            template_id: 5,
            name: "Clone Test".to_string(),
            r#type: HookType::Http,
            url: Some("https://example.com".to_string()),
            script: None,
            http_method: Some("GET".to_string()),
            http_body: None,
            timeout_secs: Some(10),
        };
        let cloned = hook.clone();
        assert_eq!(cloned.name, hook.name);
        assert_eq!(cloned.url, hook.url);
        assert_eq!(cloned.r#type, hook.r#type);
    }

    #[test]
    fn test_hook_type_serialization_display() {
        // HookType uses serde(rename_all = "snake_case") for serialization
        assert_eq!(serde_json::to_string(&HookType::Http).unwrap(), "\"http\"");
        assert_eq!(serde_json::to_string(&HookType::Bash).unwrap(), "\"bash\"");
        assert_eq!(serde_json::to_string(&HookType::Python).unwrap(), "\"python\"");
    }

    #[test]
    fn test_hook_new_python() {
        let hook = Hook::new(1, 1, "Process Data".to_string(), HookType::Python);
        assert_eq!(hook.r#type, HookType::Python);
        assert_eq!(hook.id, 0);
    }

    #[test]
    fn test_hook_deserialization() {
        let json = r#"{"id":5,"project_id":20,"template_id":10,"name":"Test Hook","type":"http","url":"https://test.com","script":null,"http_method":"POST","http_body":"{}","timeout_secs":5}"#;
        let hook: Hook = serde_json::from_str(json).unwrap();
        assert_eq!(hook.id, 5);
        assert_eq!(hook.r#type, HookType::Http);
        assert_eq!(hook.url, Some("https://test.com".to_string()));
    }
}
