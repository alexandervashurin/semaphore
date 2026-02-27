//! Event CRUD Operations
//!
//! Операции с событиями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Event;

impl SqlDb {
    /// Получает события проекта
    pub async fn get_events(&self, project_id: Option<i32>, limit: usize) -> Result<Vec<Event>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let query = if let Some(pid) = project_id {
                    sqlx::query_as::<_, Event>(
                        "SELECT * FROM event WHERE project_id = ? ORDER BY created DESC LIMIT ?"
                    )
                    .bind(pid)
                    .bind(limit as i64)
                } else {
                    sqlx::query_as::<_, Event>(
                        "SELECT * FROM event ORDER BY created DESC LIMIT ?"
                    )
                    .bind(limit as i64)
                };

                let events = query
                    .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(events)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Создаёт событие
    pub async fn create_event(&self, mut event: Event) -> Result<Event> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO event (project_id, user_id, object_id, object_type, description, created)
                     VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(event.project_id)
                .bind(event.user_id)
                .bind(event.object_id)
                .bind(&event.object_type)
                .bind(&event.description)
                .bind(event.created)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                event.id = result.last_insert_rowid() as i32;
                Ok(event)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
}
