//! Session CRUD Operations
//!
//! Операции с сессиями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Session;

impl SqlDb {
    /// Получает сессию по ID
    pub async fn get_session(&self, user_id: i32, session_id: i32) -> Result<Session> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let session = sqlx::query_as::<_, Session>(
                    "SELECT * FROM session WHERE id = ? AND user_id = ?"
                )
                .bind(session_id)
                .bind(user_id)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                session.ok_or(Error::NotFound("Session not found".to_string()))
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Создаёт сессию
    pub async fn create_session(&self, mut session: Session) -> Result<Session> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO session (user_id, created, last_active, ip)
                     VALUES (?, ?, ?, ?)"
                )
                .bind(session.user_id)
                .bind(session.created)
                .bind(session.last_active)
                .bind(&session.ip)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                session.id = result.last_insert_rowid() as i32;
                Ok(session)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Истекает сессию
    pub async fn expire_session(&self, user_id: i32, session_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM session WHERE id = ? AND user_id = ?")
                    .bind(session_id)
                    .bind(user_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
}
