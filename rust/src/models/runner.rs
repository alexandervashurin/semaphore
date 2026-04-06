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
}
