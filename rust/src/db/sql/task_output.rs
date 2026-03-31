//! Task Output - операции с выводами задач
//!
//! Аналог db/sql/task.go из Go версии (часть 2: TaskOutput)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_task_output(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает выводы задачи
    pub async fn get_task_outputs(
        &self,
        project_id: i32,
        task_id: i32,
        params: &RetrieveQueryParams,
    ) -> Result<Vec<TaskOutput>> {
        let limit = params.count.unwrap_or(100) as i64;
        let offset = params.offset as i64;

        let rows = sqlx::query(
            "SELECT * FROM task_output WHERE task_id = $1 AND project_id = $2 \
             ORDER BY time ASC LIMIT $3 OFFSET $4",
        )
        .bind(task_id)
        .bind(project_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pg_pool_task_output()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| TaskOutput {
                id: row.get("id"),
                task_id: row.get("task_id"),
                project_id: row.get("project_id"),
                time: row.get("time"),
                output: row.get("output"),
                stage_id: row.try_get("stage_id").ok().flatten(),
            })
            .collect())
    }

    /// Создаёт вывод задачи
    pub async fn create_task_output(&self, mut output: TaskOutput) -> Result<TaskOutput> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO task_output (task_id, project_id, time, output, stage_id) \
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(output.task_id)
        .bind(output.project_id)
        .bind(output.time)
        .bind(&output.output)
        .bind(output.stage_id)
        .fetch_one(self.pg_pool_task_output()?)
        .await
        .map_err(Error::Database)?;

        output.id = id;
        Ok(output)
    }

    /// Создаёт несколько выводов задачи (batch)
    pub async fn create_task_output_batch(&self, outputs: Vec<TaskOutput>) -> Result<()> {
        for output in outputs {
            self.create_task_output(output).await?;
        }
        Ok(())
    }

    /// Удаляет выводы задачи
    pub async fn delete_task_output(&self, project_id: i32, task_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM task_output WHERE task_id = $1 AND project_id = $2")
            .bind(task_id)
            .bind(project_id)
            .execute(self.pg_pool_task_output()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    /// Получает количество выводов задачи
    pub async fn get_task_output_count(&self, project_id: i32, task_id: i32) -> Result<usize> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM task_output WHERE task_id = $1 AND project_id = $2",
        )
        .bind(task_id)
        .bind(project_id)
        .fetch_one(self.pg_pool_task_output()?)
        .await
        .map_err(Error::Database)?;

        Ok(count as usize)
    }
}
