//! Модель проекта

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Проект - верхнеуровневая структура в Velum
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    /// Уникальный идентификатор
    pub id: i32,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Название проекта
    pub name: String,

    /// Включить уведомления
    #[serde(default)]
    pub alert: bool,

    /// Chat ID для уведомлений
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_chat: Option<String>,

    /// Максимальное количество параллельных задач
    #[serde(default)]
    pub max_parallel_tasks: i32,

    /// Тип проекта
    #[serde(default)]
    pub r#type: String,

    /// ID хранилища секретов по умолчанию
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_secret_storage_id: Option<i32>,
}

#[cfg(test)]
impl Default for Project {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}

impl Project {
    /// Создаёт новый проект
    pub fn new(name: String) -> Self {
        Self {
            id: 0, // Будет установлен базой данных
            created: Utc::now(),
            name,
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 0,
            r#type: "default".to_string(),
            default_secret_storage_id: None,
        }
    }

    /// Проверяет валидность проекта
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Название проекта не может быть пустым".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_new() {
        let project = Project::new("Test Project".to_string());
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.id, 0);
        assert!(!project.alert);
        assert!(project.alert_chat.is_none());
        assert_eq!(project.max_parallel_tasks, 0);
        assert_eq!(project.r#type, "default");
        assert!(project.default_secret_storage_id.is_none());
    }

    #[test]
    fn test_project_default() {
        let project = Project::default();
        assert_eq!(project.name, "default");
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_project_validate_empty_name() {
        let project = Project::new("".to_string());
        assert!(project.validate().is_err());
    }

    #[test]
    fn test_project_validate_non_empty_name() {
        let project = Project::new("Valid Project".to_string());
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_project_clone() {
        let project = Project::new("Clone Test".to_string());
        let cloned = project.clone();
        assert_eq!(cloned.name, project.name);
        assert_eq!(cloned.alert, project.alert);
        assert_eq!(cloned.max_parallel_tasks, project.max_parallel_tasks);
    }

    #[test]
    fn test_project_with_alerts() {
        let mut project = Project::new("Alert Project".to_string());
        project.alert = true;
        project.alert_chat = Some("chat123".to_string());
        project.max_parallel_tasks = 5;

        assert!(project.alert);
        assert_eq!(project.alert_chat, Some("chat123".to_string()));
        assert_eq!(project.max_parallel_tasks, 5);
    }

    #[test]
    fn test_project_serialization() {
        let project = Project::new("Serialize Test".to_string());
        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("\"name\":\"Serialize Test\""));
        assert!(json.contains("\"alert\":false"));
        assert!(json.contains("\"max_parallel_tasks\":0"));
    }

    #[test]
    fn test_project_deserialization() {
        let json = r#"{"id":5,"created":"2024-01-01T00:00:00Z","name":"Deser Project","alert":true,"alert_chat":"chat456","max_parallel_tasks":10,"type":"ansible","default_secret_storage_id":null}"#;
        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.id, 5);
        assert_eq!(project.name, "Deser Project");
        assert!(project.alert);
        assert_eq!(project.alert_chat, Some("chat456".to_string()));
        assert_eq!(project.max_parallel_tasks, 10);
        assert_eq!(project.r#type, "ansible");
    }

    #[test]
    fn test_project_with_secret_storage() {
        let mut project = Project::new("Vault Project".to_string());
        project.default_secret_storage_id = Some(42);
        assert_eq!(project.default_secret_storage_id, Some(42));
    }

    #[test]
    fn test_project_debug_format() {
        let project = Project::new("Debug Project".to_string());
        let debug_str = format!("{:?}", project);
        assert!(debug_str.contains("Debug Project"));
        assert!(debug_str.contains("Project"));
    }
}
