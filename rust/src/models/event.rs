//! Модель события

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Тип события
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    TaskCreated,
    TaskUpdated,
    TaskDeleted,
    TemplateCreated,
    TemplateUpdated,
    TemplateDeleted,
    InventoryCreated,
    InventoryUpdated,
    InventoryDeleted,
    RepositoryCreated,
    RepositoryUpdated,
    RepositoryDeleted,
    EnvironmentCreated,
    EnvironmentUpdated,
    EnvironmentDeleted,
    AccessKeyCreated,
    AccessKeyUpdated,
    AccessKeyDeleted,
    IntegrationCreated,
    IntegrationUpdated,
    IntegrationDeleted,
    ScheduleCreated,
    ScheduleUpdated,
    ScheduleDeleted,
    UserJoined,
    UserLeft,
    UserUpdated,
    ProjectUpdated,
    Other,
}

/// Событие системы
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: i32,
    pub project_id: Option<i32>,
    pub user_id: Option<i32>,
    pub object_id: Option<i32>,
    pub object_type: String,
    pub description: String,
    pub created: DateTime<Utc>,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::TaskCreated => write!(f, "task_created"),
            EventType::TaskUpdated => write!(f, "task_updated"),
            EventType::TaskDeleted => write!(f, "task_deleted"),
            EventType::TemplateCreated => write!(f, "template_created"),
            EventType::TemplateUpdated => write!(f, "template_updated"),
            EventType::TemplateDeleted => write!(f, "template_deleted"),
            EventType::InventoryCreated => write!(f, "inventory_created"),
            EventType::InventoryUpdated => write!(f, "inventory_updated"),
            EventType::InventoryDeleted => write!(f, "inventory_deleted"),
            EventType::RepositoryCreated => write!(f, "repository_created"),
            EventType::RepositoryUpdated => write!(f, "repository_updated"),
            EventType::RepositoryDeleted => write!(f, "repository_deleted"),
            EventType::EnvironmentCreated => write!(f, "environment_created"),
            EventType::EnvironmentUpdated => write!(f, "environment_updated"),
            EventType::EnvironmentDeleted => write!(f, "environment_deleted"),
            EventType::AccessKeyCreated => write!(f, "access_key_created"),
            EventType::AccessKeyUpdated => write!(f, "access_key_updated"),
            EventType::AccessKeyDeleted => write!(f, "access_key_deleted"),
            EventType::IntegrationCreated => write!(f, "integration_created"),
            EventType::IntegrationUpdated => write!(f, "integration_updated"),
            EventType::IntegrationDeleted => write!(f, "integration_deleted"),
            _ => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_display() {
        assert_eq!(EventType::TaskCreated.to_string(), "task_created");
        assert_eq!(EventType::TemplateDeleted.to_string(), "template_deleted");
        assert_eq!(EventType::InventoryUpdated.to_string(), "inventory_updated");
        assert_eq!(EventType::EnvironmentDeleted.to_string(), "environment_deleted");
        assert_eq!(EventType::AccessKeyCreated.to_string(), "access_key_created");
    }

    #[test]
    fn test_event_type_serialization() {
        let json = serde_json::to_string(&EventType::TaskCreated).unwrap();
        assert_eq!(json, "\"task_created\"");
    }

    #[test]
    fn test_event_type_unknown() {
        assert_eq!(EventType::Other.to_string(), "unknown");
        assert_eq!(EventType::ScheduleCreated.to_string(), "unknown");
        assert_eq!(EventType::UserJoined.to_string(), "unknown");
    }

    #[test]
    fn test_event_serialization() {
        let event = Event {
            id: 1,
            project_id: Some(10),
            user_id: Some(5),
            object_id: Some(100),
            object_type: "task".to_string(),
            description: "Task started".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"object_type\":\"task\""));
        assert!(json.contains("\"description\":\"Task started\""));
        assert!(json.contains("\"project_id\":10"));
    }

    #[test]
    fn test_event_serialization_null_fields() {
        let event = Event {
            id: 1,
            project_id: None,
            user_id: None,
            object_id: None,
            object_type: "system".to_string(),
            description: "System event".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"project_id\":null"));
        assert!(json.contains("\"user_id\":null"));
    }
}
