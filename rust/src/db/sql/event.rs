//! Event CRUD Operations
//!
//! Операции с событиями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Event;
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
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
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
