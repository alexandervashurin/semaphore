//! Runner - операции с раннерами в SQL
//!
//! Аналог db/sql/global_runner.go из Go версии

use sqlx::{Executor, FromRow};
use crate::error::{Error, Result};
use crate::models::Runner;
use crate::db::sql::SqlDb;
use chrono::Utc;

impl SqlDb {
    /// Получает раннера по токену
    pub async fn get_runner_by_token(&self, token: &str) -> Result<Runner> {
        let runner = sqlx::query_as::<_, Runner>(
            r#"SELECT * FROM runner WHERE token = ?"#
        )
        .bind(token)
        .fetch_optional(&*self.pool)
        .await?;

        runner.ok_or(Error::NotFound("Runner not found".to_string()))
    }

    /// Получает глобального раннера по ID
    pub async fn get_global_runner(&self, runner_id: i32) -> Result<Runner> {
        let runner = sqlx::query_as::<_, Runner>(
            r#"SELECT * FROM runner WHERE id = ? AND project_id IS NULL"#
        )
        .bind(runner_id)
        .fetch_optional(&*self.pool)
        .await?;

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
            .fetch_all(&*self.pool)
            .await?;

        Ok(runners)
    }

    /// Удаляет глобального раннера
    pub async fn delete_global_runner(&self, runner_id: i32) -> Result<()> {
        let result = sqlx::query(
            r#"DELETE FROM runner WHERE id = ? AND project_id IS NULL"#
        )
        .bind(runner_id)
        .execute(&*self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Global runner not found".to_string()));
        }

        Ok(())
    }

    /// Очищает кэш раннера
    pub async fn clear_runner_cache(&self, runner: &Runner) -> Result<()> {
        if runner.project_id.is_none() {
            sqlx::query(
                r#"UPDATE runner SET cleaning_requested = ? WHERE id = ?"#
            )
            .bind(Utc::now())
            .bind(runner.id)
            .execute(&*self.pool)
            .await?;
        } else {
            sqlx::query(
                r#"UPDATE runner SET cleaning_requested = ? WHERE id = ? AND project_id = ?"#
            )
            .bind(Utc::now())
            .bind(runner.id)
            .bind(runner.project_id.unwrap())
            .execute(&*self.pool)
            .await?;
        }

        Ok(())
    }

    /// Обновляет время активности раннера
    pub async fn touch_runner(&self, runner: &Runner) -> Result<()> {
        if runner.project_id.is_none() {
            sqlx::query(
                r#"UPDATE runner SET touched = ? WHERE id = ?"#
            )
            .bind(Utc::now())
            .bind(runner.id)
            .execute(&*self.pool)
            .await?;
        } else {
            sqlx::query(
                r#"UPDATE runner SET touched = ? WHERE id = ? AND project_id = ?"#
            )
            .bind(Utc::now())
            .bind(runner.id)
            .bind(runner.project_id.unwrap())
            .execute(&*self.pool)
            .await?;
        }

        Ok(())
    }

    /// Обновляет раннера
    pub async fn update_runner(&self, runner: &Runner) -> Result<()> {
        sqlx::query(
            r#"UPDATE runner SET name = ?, active = ?, webhook = ?, max_parallel_tasks = ?, tag = ? WHERE id = ?"#
        )
        .bind(&runner.name)
        .bind(runner.active)
        .bind(&runner.webhook)
        .bind(runner.max_parallel_tasks)
        .bind(&runner.tag)
        .bind(runner.id)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// Создаёт раннера
    pub async fn create_runner(&self, runner: &Runner) -> Result<Runner> {
        let result = sqlx::query(
            r#"INSERT INTO runner (name, active, webhook, max_parallel_tasks, tag, token, project_id) 
               VALUES (?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&runner.name)
        .bind(runner.active)
        .bind(&runner.webhook)
        .bind(runner.max_parallel_tasks)
        .bind(&runner.tag)
        .bind(&runner.token)
        .bind(runner.project_id)
        .execute(&*self.pool)
        .await?;

        let mut new_runner = runner.clone();
        new_runner.id = result.last_insert_rowid() as i32;

        Ok(new_runner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_runner() -> Runner {
        Runner {
            id: 0,
            name: "Test Runner".to_string(),
            active: true,
            webhook: None,
            max_parallel_tasks: 5,
            tag: "test".to_string(),
            token: Uuid::new_v4().to_string(),
            project_id: None,
            cleaning_requested: None,
            touched: None,
            created: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_runner_creation() {
        let runner = create_test_runner();
        assert_eq!(runner.name, "Test Runner");
        assert!(runner.active);
        assert_eq!(runner.max_parallel_tasks, 5);
    }

    #[test]
    fn test_runner_token_generation() {
        let runner = create_test_runner();
        assert!(!runner.token.is_empty());
        assert!(runner.token.len() > 32); // UUID format
    }
}
