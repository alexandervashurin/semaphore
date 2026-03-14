//! EventManager - управление событиями

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::db::sql::SqlDialect;
use crate::error::{Error, Result};
use crate::models::Event;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

#[async_trait]
impl EventManager for SqlStore {
    async fn get_events(&self, project_id: Option<i32>, limit: usize) -> Result<Vec<Event>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = if project_id.is_some() {
                    "SELECT * FROM event WHERE project_id = ? ORDER BY created DESC LIMIT ?"
                } else {
                    "SELECT * FROM event ORDER BY created DESC LIMIT ?"
                };
                let mut q = sqlx::query(query);
                if let Some(pid) = project_id {
                    q = q.bind(pid);
                }
                let rows = q.bind(limit as i64).fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
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
            SqlDialect::PostgreSQL => {
                let query = if project_id.is_some() {
                    "SELECT * FROM event WHERE project_id = $1 ORDER BY created DESC LIMIT $2"
                } else {
                    "SELECT * FROM event ORDER BY created DESC LIMIT $1"
                };
                let mut q = sqlx::query(query);
                if let Some(pid) = project_id {
                    q = q.bind(pid);
                }
                let rows = q.bind(limit as i64).fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
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
            SqlDialect::MySQL => {
                let query = if project_id.is_some() {
                    "SELECT * FROM `event` WHERE project_id = ? ORDER BY created DESC LIMIT ?"
                } else {
                    "SELECT * FROM `event` ORDER BY created DESC LIMIT ?"
                };
                let mut q = sqlx::query(query);
                if let Some(pid) = project_id {
                    q = q.bind(pid);
                }
                let rows = q.bind(limit as i64).fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
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
        }
    }

    async fn create_event(&self, mut event: Event) -> Result<Event> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO event (project_id, user_id, object_id, object_type, description, created) VALUES (?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(&event.project_id)
                    .bind(&event.user_id)
                    .bind(&event.object_id)
                    .bind(&event.object_type)
                    .bind(&event.description)
                    .bind(event.created)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                event.id = id;
                Ok(event)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO event (project_id, user_id, object_id, object_type, description, created) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(&event.project_id)
                    .bind(&event.user_id)
                    .bind(&event.object_id)
                    .bind(&event.object_type)
                    .bind(&event.description)
                    .bind(event.created)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                event.id = id;
                Ok(event)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `event` (project_id, user_id, object_id, object_type, description, created) VALUES (?, ?, ?, ?, ?, ?)";
                let result = sqlx::query(query)
                    .bind(&event.project_id)
                    .bind(&event.user_id)
                    .bind(&event.object_id)
                    .bind(&event.object_type)
                    .bind(&event.description)
                    .bind(event.created)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                event.id = result.last_insert_id() as i32;
                Ok(event)
            }
        }
    }
}

