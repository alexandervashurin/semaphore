//! View CRUD Operations
//!
//! Операции с представлениями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::View;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_view(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает представления проекта
    pub async fn get_views(&self, project_id: i32) -> Result<Vec<View>> {
        let rows = sqlx::query(
            "SELECT * FROM view WHERE project_id = $1 ORDER BY position ASC, id ASC"
        )
        .bind(project_id)
        .fetch_all(self.pg_pool_view()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| View {
            id: row.get("id"),
            project_id: row.get("project_id"),
            title: row.get("title"),
            position: row.try_get("position").ok().unwrap_or(0),
        }).collect())
    }

    /// Получает представление по ID
    pub async fn get_view(&self, project_id: i32, view_id: i32) -> Result<View> {
        let row = sqlx::query(
            "SELECT * FROM view WHERE id = $1 AND project_id = $2"
        )
        .bind(view_id)
        .bind(project_id)
        .fetch_one(self.pg_pool_view()?)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Представление не найдено".to_string()),
            _ => Error::Database(e),
        })?;

        Ok(View {
            id: row.get("id"),
            project_id: row.get("project_id"),
            title: row.get("title"),
            position: row.try_get("position").ok().unwrap_or(0),
        })
    }

    /// Создаёт представление
    pub async fn create_view(&self, mut view: View) -> Result<View> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO view (project_id, title, position) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(view.project_id)
        .bind(&view.title)
        .bind(view.position)
        .fetch_one(self.pg_pool_view()?)
        .await
        .map_err(Error::Database)?;

        view.id = id;
        Ok(view)
    }

    /// Обновляет представление
    pub async fn update_view(&self, view: View) -> Result<()> {
        sqlx::query(
            "UPDATE view SET title = $1, position = $2 WHERE id = $3 AND project_id = $4"
        )
        .bind(&view.title)
        .bind(view.position)
        .bind(view.id)
        .bind(view.project_id)
        .execute(self.pg_pool_view()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет представление
    pub async fn delete_view(&self, project_id: i32, view_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM view WHERE id = $1 AND project_id = $2")
            .bind(view_id)
            .bind(project_id)
            .execute(self.pg_pool_view()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}
