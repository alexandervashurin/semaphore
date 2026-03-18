//! RunnerManager - управление раннерами

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Runner;
use async_trait::async_trait;
use sqlx::Row;

fn row_to_runner(row: sqlx::sqlite::SqliteRow) -> Runner {
    Runner {
        id: row.get("id"),
        project_id: row.try_get("project_id").ok().flatten(),
        token: row.try_get("token").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        active: row.try_get::<bool, _>("active").unwrap_or(true),
        last_active: row.try_get("last_active").ok().flatten(),
        webhook: row.try_get("webhook").ok().flatten(),
        max_parallel_tasks: row.try_get("max_parallel_tasks").ok().flatten(),
        tag: row.try_get("tag").ok().flatten(),
        cleaning_requested: None,
        touched: row.try_get("last_active").ok().flatten(),
        created: row.try_get("created").ok().flatten(),
    }
}

#[async_trait]
impl RunnerManager for SqlStore {
    async fn get_runners(&self, project_id: Option<i32>) -> Result<Vec<Runner>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;
                let rows = if let Some(pid) = project_id {
                    sqlx::query("SELECT * FROM runner WHERE project_id = ? OR project_id IS NULL ORDER BY name")
                        .bind(pid)
                        .fetch_all(pool)
                        .await
                        .map_err(Error::Database)?
                } else {
                    sqlx::query("SELECT * FROM runner ORDER BY name")
                        .fetch_all(pool)
                        .await
                        .map_err(Error::Database)?
                };
                Ok(rows.into_iter().map(row_to_runner).collect())
            }
            _ => Ok(vec![]),
        }
    }

    async fn get_runner(&self, runner_id: i32) -> Result<Runner> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;
                let row = sqlx::query("SELECT * FROM runner WHERE id = ?")
                    .bind(runner_id)
                    .fetch_optional(pool)
                    .await
                    .map_err(Error::Database)?
                    .ok_or_else(|| Error::NotFound("Раннер не найден".to_string()))?;
                Ok(row_to_runner(row))
            }
            _ => Err(Error::NotFound("Раннер не найден".to_string())),
        }
    }

    async fn create_runner(&self, mut runner: Runner) -> Result<Runner> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;
                let result = sqlx::query(
                    "INSERT INTO runner (project_id, token, name, active, webhook, max_parallel_tasks, tag) VALUES (?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(runner.project_id)
                .bind(&runner.token)
                .bind(&runner.name)
                .bind(runner.active)
                .bind(&runner.webhook)
                .bind(runner.max_parallel_tasks)
                .bind(&runner.tag)
                .execute(pool)
                .await
                .map_err(Error::Database)?;

                runner.id = result.last_insert_rowid() as i32;
                Ok(runner)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string())),
        }
    }

    async fn update_runner(&self, runner: Runner) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;
                sqlx::query(
                    "UPDATE runner SET name = ?, active = ?, webhook = ?, max_parallel_tasks = ?, tag = ? WHERE id = ?"
                )
                .bind(&runner.name)
                .bind(runner.active)
                .bind(&runner.webhook)
                .bind(runner.max_parallel_tasks)
                .bind(&runner.tag)
                .bind(runner.id)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string())),
        }
    }

    async fn delete_runner(&self, runner_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;
                sqlx::query("DELETE FROM runner WHERE id = ?")
                    .bind(runner_id)
                    .execute(pool)
                    .await
                    .map_err(Error::Database)?;
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string())),
        }
    }
}
