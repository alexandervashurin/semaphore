//! EventManager - управление событиями

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Event;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

#[async_trait]
impl EventManager for SqlStore {
    async fn get_events(&self, project_id: Option<i32>, limit: usize) -> Result<Vec<Event>> {
        let query = if project_id.is_some() {
            "SELECT * FROM event WHERE project_id = $1 ORDER BY created DESC LIMIT $2"
        } else {
            "SELECT * FROM event ORDER BY created DESC LIMIT $1"
        };
        let mut q = sqlx::query(query);
        if let Some(pid) = project_id {
            q = q.bind(pid);
        }
        let rows = q
            .bind(limit as i64)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(rows
            .into_iter()
            .map(|row| Event {
                id: row.get("id"),
                project_id: row.try_get("project_id").ok(),
                user_id: row.try_get("user_id").ok(),
                object_id: row.try_get("object_id").ok(),
                object_type: row.get("object_type"),
                description: row.get("description"),
                created: row.get("created"),
            })
            .collect())
    }

    async fn create_event(&self, mut event: Event) -> Result<Event> {
        let query = "INSERT INTO event (project_id, user_id, object_id, object_type, description, created) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id";
        let id: i32 = sqlx::query_scalar(query)
            .bind(event.project_id)
            .bind(event.user_id)
            .bind(event.object_id)
            .bind(&event.object_type)
            .bind(&event.description)
            .bind(event.created)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        event.id = id;
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::event::{Event, EventType};
    use chrono::Utc;

    #[test]
    fn test_event_type_display() {
        assert_eq!(EventType::TaskCreated.to_string(), "task_created");
        assert_eq!(EventType::TemplateDeleted.to_string(), "template_deleted");
        assert_eq!(EventType::InventoryUpdated.to_string(), "inventory_updated");
        assert_eq!(
            EventType::EnvironmentDeleted.to_string(),
            "environment_deleted"
        );
        assert_eq!(
            EventType::AccessKeyCreated.to_string(),
            "access_key_created"
        );
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
    }

    #[test]
    fn test_event_null_fields() {
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

    #[test]
    fn test_event_clone() {
        let event = Event {
            id: 1,
            project_id: Some(1),
            user_id: Some(1),
            object_id: None,
            object_type: "repo".to_string(),
            description: "Repo created".to_string(),
            created: Utc::now(),
        };
        let cloned = event.clone();
        assert_eq!(cloned.object_type, event.object_type);
        assert_eq!(cloned.project_id, event.project_id);
    }

    #[test]
    fn test_event_type_all_variants() {
        let variants = [
            (EventType::TaskCreated, "task_created"),
            (EventType::TaskUpdated, "task_updated"),
            (EventType::TaskDeleted, "task_deleted"),
            (EventType::TemplateCreated, "template_created"),
            (EventType::RepositoryCreated, "repository_created"),
            (EventType::EnvironmentCreated, "environment_created"),
            (EventType::IntegrationCreated, "integration_created"),
        ];
        for (variant, expected) in &variants {
            assert_eq!(variant.to_string(), *expected);
        }
    }

    #[test]
    fn test_event_type_equality() {
        assert_eq!(EventType::TaskCreated, EventType::TaskCreated);
        assert_ne!(EventType::TaskCreated, EventType::TaskDeleted);
    }

    #[test]
    fn test_event_deserialization() {
        let json = r#"{"id":10,"project_id":5,"user_id":2,"object_id":null,"object_type":"env","description":"Env updated","created":"2024-01-01T00:00:00Z"}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, 10);
        assert_eq!(event.object_type, "env");
    }
}
