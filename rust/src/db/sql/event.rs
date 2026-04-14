//! Event CRUD Operations
//!
//! Операции с событиями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{Event, EventType};
use sqlx::Row;

impl SqlDb {
    fn pg_pool_event(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает события проекта
    pub async fn get_events(&self, project_id: Option<i32>, limit: usize) -> Result<Vec<Event>> {
        let rows = if let Some(pid) = project_id {
            sqlx::query("SELECT * FROM event WHERE project_id = $1 ORDER BY created DESC LIMIT $2")
                .bind(pid)
                .bind(limit as i64)
                .fetch_all(self.pg_pool_event()?)
                .await
                .map_err(Error::Database)?
        } else {
            sqlx::query("SELECT * FROM event ORDER BY created DESC LIMIT $1")
                .bind(limit as i64)
                .fetch_all(self.pg_pool_event()?)
                .await
                .map_err(Error::Database)?
        };

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

    /// Создаёт событие
    pub async fn create_event(&self, mut event: Event) -> Result<Event> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO event (project_id, user_id, object_id, object_type, description, created) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(event.project_id)
        .bind(event.user_id)
        .bind(event.object_id)
        .bind(&event.object_type)
        .bind(&event.description)
        .bind(event.created)
        .fetch_one(self.pg_pool_event()?)
        .await
        .map_err(Error::Database)?;

        event.id = id;
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_event_type_task_created_display() {
        assert_eq!(EventType::TaskCreated.to_string(), "task_created");
    }

    #[test]
    fn test_event_type_task_deleted_display() {
        assert_eq!(EventType::TaskDeleted.to_string(), "task_deleted");
    }

    #[test]
    fn test_event_type_serialization() {
        let json = serde_json::to_string(&EventType::TaskCreated).unwrap();
        assert_eq!(json, "\"task_created\"");
    }

    #[test]
    fn test_event_type_deserialization() {
        let event_type: EventType = serde_json::from_str("\"task_created\"").unwrap();
        assert_eq!(event_type, EventType::TaskCreated);
    }

    #[test]
    fn test_event_serialization() {
        let event = Event {
            id: 1,
            project_id: Some(10),
            user_id: Some(5),
            object_id: Some(100),
            object_type: "task".to_string(),
            description: "Task created".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"object_type\":\"task\""));
        assert!(json.contains("\"description\":\"Task created\""));
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

    #[test]
    fn test_event_deserialization() {
        let json = r#"{"id":5,"project_id":20,"user_id":3,"object_id":100,"object_type":"template","description":"Template updated","created":"2024-01-01T00:00:00Z"}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, 5);
        assert_eq!(event.project_id, Some(20));
        assert_eq!(event.object_type, "template");
        assert_eq!(event.description, "Template updated");
    }

    #[test]
    fn test_event_clone() {
        let event = Event {
            id: 1,
            project_id: Some(1),
            user_id: Some(1),
            object_id: Some(1),
            object_type: "clone".to_string(),
            description: "Clone event".to_string(),
            created: Utc::now(),
        };
        let cloned = event.clone();
        assert_eq!(cloned.description, event.description);
        assert_eq!(cloned.object_type, event.object_type);
    }

    #[test]
    fn test_event_debug_format() {
        let event = Event {
            id: 42,
            project_id: None,
            user_id: None,
            object_id: None,
            object_type: "debug".to_string(),
            description: "Debug event".to_string(),
            created: Utc::now(),
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Event"));
        assert!(debug_str.contains("Debug event"));
    }

    #[test]
    fn test_event_type_equality() {
        assert_eq!(EventType::TaskCreated, EventType::TaskCreated);
        assert_ne!(EventType::TaskCreated, EventType::TaskDeleted);
    }

    #[test]
    fn test_event_type_all_variants_serialize() {
        let types = vec![
            EventType::TaskCreated,
            EventType::TaskUpdated,
            EventType::TaskDeleted,
            EventType::TemplateCreated,
            EventType::InventoryUpdated,
            EventType::RepositoryDeleted,
            EventType::EnvironmentCreated,
            EventType::AccessKeyUpdated,
            EventType::IntegrationDeleted,
            EventType::ScheduleCreated,
            EventType::UserJoined,
            EventType::ProjectUpdated,
            EventType::Other,
        ];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            assert!(json.starts_with('"') && json.ends_with('"'));
        }
    }
}
