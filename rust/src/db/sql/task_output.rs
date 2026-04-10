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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TaskOutput;
    use chrono::Utc;

    #[test]
    fn test_task_output_new() {
        let output = TaskOutput {
            id: 0,
            task_id: 10,
            project_id: 1,
            time: Utc::now(),
            output: "Starting task...".to_string(),
            stage_id: None,
        };
        assert_eq!(output.task_id, 10);
        assert_eq!(output.project_id, 1);
        assert_eq!(output.output, "Starting task...");
        assert!(output.stage_id.is_none());
    }

    #[test]
    fn test_task_output_with_stage_id() {
        let output = TaskOutput {
            id: 1,
            task_id: 5,
            project_id: 2,
            time: Utc::now(),
            output: "Stage output".to_string(),
            stage_id: Some(3),
        };
        assert_eq!(output.stage_id, Some(3));
    }

    #[test]
    fn test_task_output_serialization() {
        let output = TaskOutput {
            id: 1,
            task_id: 10,
            project_id: 1,
            time: Utc::now(),
            output: "Line 1".to_string(),
            stage_id: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"task_id\":10"));
        assert!(json.contains("\"output\":\"Line 1\""));
        assert!(json.contains("\"project_id\":1"));
    }

    #[test]
    fn test_task_output_serialization_skip_null_stage_id() {
        let output = TaskOutput {
            id: 1,
            task_id: 10,
            project_id: 1,
            time: Utc::now(),
            output: "test".to_string(),
            stage_id: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(!json.contains("stage_id"));
    }

    #[test]
    fn test_task_output_deserialization() {
        let json = r#"{"id":5,"task_id":20,"project_id":3,"time":"2024-01-01T00:00:00Z","output":"hello","stage_id":1}"#;
        let output: TaskOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.id, 5);
        assert_eq!(output.task_id, 20);
        assert_eq!(output.project_id, 3);
        assert_eq!(output.output, "hello");
        assert_eq!(output.stage_id, Some(1));
    }

    #[test]
    fn test_task_output_deserialization_without_stage_id() {
        let json = r#"{"id":2,"task_id":15,"project_id":1,"time":"2024-01-01T00:00:00Z","output":"no stage"}"#;
        let output: TaskOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.id, 2);
        assert!(output.stage_id.is_none());
    }

    #[test]
    fn test_task_output_clone() {
        let output = TaskOutput {
            id: 1,
            task_id: 7,
            project_id: 2,
            time: Utc::now(),
            output: "clone output".to_string(),
            stage_id: Some(4),
        };
        let cloned = output.clone();
        assert_eq!(cloned.output, output.output);
        assert_eq!(cloned.stage_id, output.stage_id);
    }

    #[test]
    fn test_task_output_debug_format() {
        let output = TaskOutput {
            id: 1,
            task_id: 1,
            project_id: 1,
            time: Utc::now(),
            output: "debug".to_string(),
            stage_id: None,
        };
        let debug_str = format!("{:?}", output);
        assert!(debug_str.contains("TaskOutput"));
        assert!(debug_str.contains("debug"));
    }

    #[test]
    fn test_task_output_empty_output_string() {
        let output = TaskOutput {
            id: 1,
            task_id: 1,
            project_id: 1,
            time: Utc::now(),
            output: "".to_string(),
            stage_id: None,
        };
        assert!(output.output.is_empty());
    }

    #[test]
    fn test_task_output_multiline_string() {
        let output = TaskOutput {
            id: 1,
            task_id: 1,
            project_id: 1,
            time: Utc::now(),
            output: "line1\nline2\nline3".to_string(),
            stage_id: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("line1\\nline2"));
    }

    #[test]
    fn test_task_stage_type_display() {
        assert_eq!(TaskStageType::Init.to_string(), "init");
        assert_eq!(TaskStageType::Running.to_string(), "running");
        assert_eq!(TaskStageType::PrintResult.to_string(), "print_result");
        assert_eq!(TaskStageType::TerraformPlan.to_string(), "terraform_plan");
    }

    #[test]
    fn test_task_stage_type_serialization() {
        let json = serde_json::to_string(&TaskStageType::Init).unwrap();
        assert_eq!(json, "\"init\"");
    }
}
