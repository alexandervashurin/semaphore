//! Runner - операции с раннерами в SQL (PostgreSQL)

use crate::error::{Error, Result};
use crate::models::Runner;
use crate::db::sql::types::SqlDb;
use chrono::Utc;

impl SqlDb {
    fn runner_pg_pool(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает раннера по токену
    pub async fn get_runner_by_token(&self, token: &str) -> Result<Runner> {
        let runner = sqlx::query_as::<_, Runner>(
            "SELECT * FROM runner WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(self.runner_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        runner.ok_or(Error::NotFound("Runner not found".to_string()))
    }

    /// Получает глобального раннера по ID
    pub async fn get_global_runner(&self, runner_id: i32) -> Result<Runner> {
        let runner = sqlx::query_as::<_, Runner>(
            "SELECT * FROM runner WHERE id = $1 AND project_id IS NULL"
        )
        .bind(runner_id)
        .fetch_optional(self.runner_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        runner.ok_or(Error::NotFound("Global runner not found".to_string()))
    }

    /// Получает всех раннеров
    pub async fn get_all_runners(&self, active_only: bool, global_only: bool) -> Result<Vec<Runner>> {
        let mut query = String::from("SELECT * FROM runner WHERE 1=1");

        if global_only {
            query.push_str(" AND project_id IS NULL");
        }

        if active_only {
            query.push_str(" AND active = TRUE");
        }

        let runners = sqlx::query_as::<_, Runner>(&query)
            .fetch_all(self.runner_pg_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(runners)
    }

    /// Удаляет глобального раннера
    pub async fn delete_global_runner(&self, runner_id: i32) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM runner WHERE id = $1 AND project_id IS NULL"
        )
        .bind(runner_id)
        .execute(self.runner_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Global runner not found".to_string()));
        }

        Ok(())
    }

    /// Очищает кэш раннера
    pub async fn clear_runner_cache(&self, runner: &Runner) -> Result<()> {
        let pool = self.runner_pg_pool()?;
        if let Some(project_id) = runner.project_id {
            sqlx::query("UPDATE runner SET cleaning_requested = $1 WHERE id = $2 AND project_id = $3")
                .bind(Utc::now())
                .bind(runner.id)
                .bind(project_id)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
        } else {
            sqlx::query("UPDATE runner SET cleaning_requested = $1 WHERE id = $2")
                .bind(Utc::now())
                .bind(runner.id)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
        }

        Ok(())
    }

    /// Обновляет время активности раннера
    pub async fn touch_runner(&self, runner: &Runner) -> Result<()> {
        let pool = self.runner_pg_pool()?;
        if let Some(project_id) = runner.project_id {
            sqlx::query("UPDATE runner SET touched = $1 WHERE id = $2 AND project_id = $3")
                .bind(Utc::now())
                .bind(runner.id)
                .bind(project_id)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
        } else {
            sqlx::query("UPDATE runner SET touched = $1 WHERE id = $2")
                .bind(Utc::now())
                .bind(runner.id)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
        }

        Ok(())
    }

    /// Обновляет раннера
    pub async fn update_runner_record(&self, runner: &Runner) -> Result<()> {
        sqlx::query(
            "UPDATE runner SET name = $1, active = $2, webhook = $3, max_parallel_tasks = $4, tag = $5 WHERE id = $6"
        )
        .bind(&runner.name)
        .bind(runner.active)
        .bind(&runner.webhook)
        .bind(runner.max_parallel_tasks)
        .bind(&runner.tag)
        .bind(runner.id)
        .execute(self.runner_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    /// Создаёт раннера
    pub async fn create_runner_record(&self, runner: &Runner) -> Result<Runner> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO runner (name, active, webhook, max_parallel_tasks, tag, token, project_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"
        )
        .bind(&runner.name)
        .bind(runner.active)
        .bind(&runner.webhook)
        .bind(runner.max_parallel_tasks)
        .bind(&runner.tag)
        .bind(&runner.token)
        .bind(runner.project_id)
        .fetch_one(self.runner_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        let mut new_runner = runner.clone();
        new_runner.id = id;

        Ok(new_runner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_runner() -> Runner {
        Runner {
            last_active: None,
            id: 0,
            name: "Test Runner".to_string(),
            active: true,
            webhook: None,
            max_parallel_tasks: Some(5),
            tag: Some("test".to_string()),
            token: Uuid::new_v4().to_string(),
            project_id: None,
            cleaning_requested: None,
            touched: None,
            created: Some(chrono::Utc::now()),
        }
    }

    #[test]
    fn test_runner_creation() {
        let runner = create_test_runner();
        assert_eq!(runner.name, "Test Runner");
        assert!(runner.active);
        assert_eq!(runner.max_parallel_tasks, Some(5));
    }

    #[test]
    fn test_runner_token_generation() {
        let runner = create_test_runner();
        assert!(!runner.token.is_empty());
        assert!(runner.token.len() > 32);
    }
}
