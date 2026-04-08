//! Модель раннера

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

fn default_active() -> bool {
    true
}

/// Раннер - исполнитель задач
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Runner {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub project_id: Option<i32>,
    #[serde(default)]
    pub token: String,
    pub name: String,
    #[serde(default = "default_active")]
    pub active: bool,
    #[serde(default)]
    pub last_active: Option<DateTime<Utc>>,

    /// Webhook URL для уведомлений
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,

    /// Максимальное количество параллельных задач
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_parallel_tasks: Option<i32>,

    /// Тег раннера
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Время запроса очистки
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleaning_requested: Option<DateTime<Utc>>,

    /// Время последнего обращения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touched: Option<DateTime<Utc>>,

    /// Дата создания
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_serialization() {
        let runner = Runner {
            id: 1,
            project_id: Some(10),
            token: "token123".to_string(),
            name: "Build Runner".to_string(),
            active: true,
            last_active: Some(Utc::now()),
            webhook: Some("https://example.com/webhook".to_string()),
            max_parallel_tasks: Some(5),
            tag: Some("linux".to_string()),
            cleaning_requested: None,
            touched: Some(Utc::now()),
            created: Some(Utc::now()),
        };
        let json = serde_json::to_string(&runner).unwrap();
        assert!(json.contains("\"name\":\"Build Runner\""));
        assert!(json.contains("\"active\":true"));
        assert!(json.contains("\"tag\":\"linux\""));
    }

    #[test]
    fn test_runner_skip_nulls() {
        let runner = Runner {
            id: 1,
            project_id: None,
            token: "token".to_string(),
            name: "Simple Runner".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        let json = serde_json::to_string(&runner).unwrap();
        assert!(!json.contains("\"webhook\":"));
        assert!(!json.contains("\"tag\":"));
        assert!(!json.contains("\"max_parallel_tasks\":"));
    }

    #[test]
    fn test_runner_default_active() {
        // active defaults to true via default_active()
        let runner = Runner {
            id: 0,
            project_id: None,
            token: String::new(),
            name: "Test".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        assert!(runner.active);
    }

    #[test]
    fn test_runner_clone() {
        let runner = Runner {
            id: 1,
            project_id: Some(10),
            token: "clone-token".to_string(),
            name: "Clone Runner".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: Some(3),
            tag: Some("docker".to_string()),
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        let cloned = runner.clone();
        assert_eq!(cloned.name, runner.name);
        assert_eq!(cloned.token, runner.token);
        assert_eq!(cloned.max_parallel_tasks, runner.max_parallel_tasks);
    }

    #[test]
    fn test_runner_deserialization() {
        let json = r#"{"id":5,"project_id":20,"token":"deser-token","name":"Deser Runner","active":false,"last_active":null,"webhook":null,"max_parallel_tasks":10,"tag":"k8s","cleaning_requested":null,"touched":null,"created":null}"#;
        let runner: Runner = serde_json::from_str(json).unwrap();
        assert_eq!(runner.id, 5);
        assert_eq!(runner.name, "Deser Runner");
        assert!(!runner.active);
        assert_eq!(runner.max_parallel_tasks, Some(10));
        assert_eq!(runner.tag, Some("k8s".to_string()));
    }

    #[test]
    fn test_runner_inactive() {
        let runner = Runner {
            id: 1,
            project_id: None,
            token: "token".to_string(),
            name: "Inactive Runner".to_string(),
            active: false,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        let json = serde_json::to_string(&runner).unwrap();
        assert!(json.contains("\"active\":false"));
    }

    #[test]
    fn test_runner_with_all_optional_fields() {
        let now = Utc::now();
        let runner = Runner {
            id: 1,
            project_id: Some(1),
            token: "full".to_string(),
            name: "Full Runner".to_string(),
            active: true,
            last_active: Some(now),
            webhook: Some("https://hooks.example.com".to_string()),
            max_parallel_tasks: Some(10),
            tag: Some("prod".to_string()),
            cleaning_requested: Some(now),
            touched: Some(now),
            created: Some(now),
        };
        let json = serde_json::to_string(&runner).unwrap();
        assert!(json.contains("\"webhook\":\"https://hooks.example.com\""));
        assert!(json.contains("\"max_parallel_tasks\":10"));
        assert!(json.contains("\"tag\":\"prod\""));
    }
}
