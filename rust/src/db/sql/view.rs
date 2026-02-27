//! View CRUD Operations
//!
//! Операции с представлениями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::View;

impl SqlDb {
    /// Получает представления проекта
    pub async fn get_views(&self, project_id: i32) -> Result<Vec<View>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let views = sqlx::query_as::<_, View>(
                    "SELECT * FROM view WHERE project_id = ? ORDER BY position, name"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(views)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает представление по ID
    pub async fn get_view(&self, project_id: i32, view_id: i32) -> Result<View> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let view = sqlx::query_as::<_, View>(
                    "SELECT * FROM view WHERE id = ? AND project_id = ?"
                )
                .bind(view_id)
                .bind(project_id)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                view.ok_or(Error::NotFound("View not found".to_string()))
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Создаёт представление
    pub async fn create_view(&self, mut view: View) -> Result<View> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO view (project_id, name, position)
                     VALUES (?, ?, ?)"
                )
                .bind(view.project_id)
                .bind(&view.name)
                .bind(view.position)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                view.id = result.last_insert_rowid() as i32;
                Ok(view)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Обновляет представление
    pub async fn update_view(&self, view: View) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE view SET name = ?, position = ?
                     WHERE id = ? AND project_id = ?"
                )
                .bind(&view.name)
                .bind(view.position)
                .bind(view.id)
                .bind(view.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Удаляет представление
    pub async fn delete_view(&self, project_id: i32, view_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM view WHERE id = ? AND project_id = ?")
                    .bind(view_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
}
