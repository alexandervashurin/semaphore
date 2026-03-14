//! ViewManager - управление представлениями

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::db::sql::SqlDialect;
use crate::error::{Error, Result};
use crate::models::View;
use async_trait::async_trait;

#[async_trait]
impl ViewManager for SqlStore {
    async fn get_views(&self, _project_id: i32) -> Result<Vec<View>> { Ok(vec![]) }
    async fn get_view(&self, _project_id: i32, _view_id: i32) -> Result<View> { Err(Error::NotFound("Представление не найдено".to_string())) }
    async fn create_view(&self, _view: View) -> Result<View> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_view(&self, _view: View) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_view(&self, _project_id: i32, _view_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }

    async fn set_view_positions(&self, project_id: i32, positions: Vec<(i32, i32)>) -> Result<()> {
        // positions: Vec<(view_id, position)>
        for (view_id, position) in positions {
            match self.get_dialect() {
                SqlDialect::SQLite => {
                    let query = "UPDATE view SET position = ? WHERE id = ? AND project_id = ?";
                    sqlx::query(query)
                        .bind(position)
                        .bind(view_id)
                        .bind(project_id)
                        .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                        .await
                        .map_err(|e| Error::Database(e))?;
                }
                SqlDialect::PostgreSQL => {
                    let query = "UPDATE view SET position = $1 WHERE id = $2 AND project_id = $3";
                    sqlx::query(query)
                        .bind(position)
                        .bind(view_id)
                        .bind(project_id)
                        .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                        .await
                        .map_err(|e| Error::Database(e))?;
                }
                SqlDialect::MySQL => {
                    let query = "UPDATE `view` SET position = ? WHERE id = ? AND project_id = ?";
                    sqlx::query(query)
                        .bind(position)
                        .bind(view_id)
                        .bind(project_id)
                        .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                        .await
                        .map_err(|e| Error::Database(e))?;
                }
            }
        }
        Ok(())
    }
}

