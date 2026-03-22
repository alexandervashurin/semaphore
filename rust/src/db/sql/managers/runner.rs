//! RunnerManager - управление раннерами

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Runner;
use async_trait::async_trait;
use sqlx::Row;

fn row_to_runner(row: sqlx::postgres::PgRow) -> Runner {
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
        let pool = self.get_postgres_pool()?;
        let rows = if let Some(pid) = project_id {
            sqlx::query("SELECT * FROM runner WHERE project_id = $1 OR project_id IS NULL ORDER BY name")
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

    async fn get_runner(&self, runner_id: i32) -> Result<Runner> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query("SELECT * FROM runner WHERE id = $1")
            .bind(runner_id)
            .fetch_optional(pool)
            .await
            .map_err(Error::Database)?
            .ok_or_else(|| Error::NotFound("Раннер не найден".to_string()))?;
        Ok(row_to_runner(row))
    }

    async fn create_runner(&self, mut runner: Runner) -> Result<Runner> {
        let pool = self.get_postgres_pool()?;
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO runner (project_id, token, name, active, webhook, max_parallel_tasks, tag) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"
        )
        .bind(runner.project_id)
        .bind(&runner.token)
        .bind(&runner.name)
        .bind(runner.active)
        .bind(&runner.webhook)
        .bind(runner.max_parallel_tasks)
        .bind(&runner.tag)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

        runner.id = id;
        Ok(runner)
    }

    async fn update_runner(&self, runner: Runner) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query(
            "UPDATE runner SET name = $1, active = $2, webhook = $3, max_parallel_tasks = $4, tag = $5 WHERE id = $6"
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

    async fn delete_runner(&self, runner_id: i32) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query("DELETE FROM runner WHERE id = $1")
            .bind(runner_id)
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}
