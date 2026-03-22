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
            let rows = q.bind(limit as i64).fetch_all(self.get_postgres_pool()?).await.map_err(Error::Database)?;
            Ok(rows.into_iter().map(|row| Event {
                id: row.get("id"),
                project_id: row.try_get("project_id").ok(),
                user_id: row.try_get("user_id").ok(),
                object_id: row.try_get("object_id").ok(),
                object_type: row.get("object_type"),
                description: row.get("description"),
                created: row.get("created"),
            }).collect())
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
                .fetch_one(self.get_postgres_pool()?).await.map_err(Error::Database)?;
            event.id = id;
            Ok(event)
    }
}

