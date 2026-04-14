//! Task Stage - операции со стадиями задач
//!
//! Аналог db/sql/task.go из Go версии (часть 3: TaskStage)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use chrono::Utc;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_task_stage(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает стадии задачи
    pub async fn get_task_stages(&self, project_id: i32, task_id: i32) -> Result<Vec<TaskStage>> {
        let rows = sqlx::query(
            "SELECT * FROM task_stage WHERE task_id = $1 AND project_id = $2 ORDER BY id",
        )
        .bind(task_id)
        .bind(project_id)
        .fetch_all(self.pg_pool_task_stage()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let type_str: String = row.try_get("type").ok().unwrap_or_default();
                let stage_type = match type_str.as_str() {
                    "terraform_plan" => TaskStageType::TerraformPlan,
                    "running" => TaskStageType::Running,
                    "print_result" => TaskStageType::PrintResult,
                    _ => TaskStageType::Init,
                };
                TaskStage {
                    id: row.get("id"),
                    task_id: row.get("task_id"),
                    project_id: row.get("project_id"),
                    start: row.try_get("start").ok().flatten(),
                    end: row.try_get("end").ok().flatten(),
                    r#type: stage_type,
                }
            })
            .collect())
    }

    /// Создаёт стадию задачи
    pub async fn create_task_stage(&self, mut stage: TaskStage) -> Result<TaskStage> {
        let type_str = match &stage.r#type {
            TaskStageType::Init => "init",
            TaskStageType::TerraformPlan => "terraform_plan",
            TaskStageType::Running => "running",
            TaskStageType::PrintResult => "print_result",
        };

        let id: i32 = sqlx::query_scalar(
            "INSERT INTO task_stage (task_id, project_id, type, start, end) \
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(stage.task_id)
        .bind(stage.project_id)
        .bind(type_str)
        .bind(stage.start)
        .bind(stage.end)
        .fetch_one(self.pg_pool_task_stage()?)
        .await
        .map_err(Error::Database)?;

        stage.id = id;
        Ok(stage)
    }

    /// Обновляет стадию задачи
    pub async fn update_task_stage(&self, stage: TaskStage) -> Result<()> {
        let type_str = match &stage.r#type {
            TaskStageType::Init => "init",
            TaskStageType::TerraformPlan => "terraform_plan",
            TaskStageType::Running => "running",
            TaskStageType::PrintResult => "print_result",
        };

        sqlx::query(
            "UPDATE task_stage SET type = $1, start = $2, end = $3 WHERE id = $4 AND task_id = $5 AND project_id = $6"
        )
        .bind(type_str)
        .bind(stage.start)
        .bind(stage.end)
        .bind(stage.id)
        .bind(stage.task_id)
        .bind(stage.project_id)
        .execute(self.pg_pool_task_stage()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Получает результат стадии задачи
    pub async fn get_task_stage_result(
        &self,
        project_id: i32,
        task_id: i32,
        stage_id: i32,
    ) -> Result<Option<TaskStageResult>> {
        let row = sqlx::query(
            "SELECT * FROM task_stage_result WHERE stage_id = $1 AND task_id = $2 AND project_id = $3"
        )
        .bind(stage_id)
        .bind(task_id)
        .bind(project_id)
        .fetch_optional(self.pg_pool_task_stage()?)
        .await
        .map_err(Error::Database)?;

        if let Some(row) = row {
            Ok(Some(TaskStageResult {
                id: row.get("id"),
                stage_id: row.get("stage_id"),
                task_id: row.get("task_id"),
                project_id: row.get("project_id"),
                result: row.try_get("result").ok().unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Создаёт или обновляет результат стадии
    pub async fn upsert_task_stage_result(
        &self,
        mut result: TaskStageResult,
    ) -> Result<TaskStageResult> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO task_stage_result (stage_id, task_id, project_id, result) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (stage_id, task_id, project_id) \
             DO UPDATE SET result = EXCLUDED.result \
             RETURNING id",
        )
        .bind(result.stage_id)
        .bind(result.task_id)
        .bind(result.project_id)
        .bind(&result.result)
        .fetch_one(self.pg_pool_task_stage()?)
        .await
        .map_err(Error::Database)?;

        result.id = id;
        Ok(result)
    }

    /// Удаляет результат стадии
    pub async fn delete_task_stage_result(
        &self,
        project_id: i32,
        task_id: i32,
        stage_id: i32,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM task_stage_result WHERE stage_id = $1 AND task_id = $2 AND project_id = $3"
        )
        .bind(stage_id)
        .bind(task_id)
        .bind(project_id)
        .execute(self.pg_pool_task_stage()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_stage_type_display() {
        assert_eq!(TaskStageType::Init.to_string(), "init");
        assert_eq!(TaskStageType::TerraformPlan.to_string(), "terraform_plan");
        assert_eq!(TaskStageType::Running.to_string(), "running");
        assert_eq!(TaskStageType::PrintResult.to_string(), "print_result");
    }

    #[test]
    fn test_task_stage_type_equality() {
        assert_eq!(TaskStageType::Init, TaskStageType::Init);
        assert_ne!(TaskStageType::Running, TaskStageType::Init);
    }

    #[test]
    fn test_task_stage_struct() {
        let stage = TaskStage {
            id: 1,
            task_id: 10,
            project_id: 5,
            start: Some(Utc::now()),
            end: None,
            r#type: TaskStageType::Running,
        };
        assert_eq!(stage.id, 1);
        assert_eq!(stage.task_id, 10);
        assert_eq!(stage.r#type, TaskStageType::Running);
        assert!(stage.end.is_none());
    }

    #[test]
    fn test_task_stage_clone() {
        let stage = TaskStage {
            id: 1,
            task_id: 10,
            project_id: 5,
            start: None,
            end: None,
            r#type: TaskStageType::Init,
        };
        let cloned = stage.clone();
        assert_eq!(cloned.id, stage.id);
        assert_eq!(cloned.r#type, stage.r#type);
    }

    #[test]
    fn test_task_stage_result_struct() {
        let result = TaskStageResult {
            id: 1,
            stage_id: 5,
            task_id: 10,
            project_id: 3,
            result: "Task output result".to_string(),
        };
        assert_eq!(result.id, 1);
        assert_eq!(result.stage_id, 5);
        assert_eq!(result.result, "Task output result");
    }

    #[test]
    fn test_task_stage_result_clone() {
        let result = TaskStageResult {
            id: 1,
            stage_id: 5,
            task_id: 10,
            project_id: 3,
            result: "output".to_string(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.id, result.id);
        assert_eq!(cloned.result, result.result);
    }

    #[test]
    fn test_task_stage_with_result_struct() {
        let stage = TaskStage {
            id: 1,
            task_id: 10,
            project_id: 5,
            start: Some(Utc::now()),
            end: Some(Utc::now()),
            r#type: TaskStageType::PrintResult,
        };
        let with_result = TaskStageWithResult {
            stage,
            start_output: Some("Started".to_string()),
            end_output: Some("Finished".to_string()),
        };
        assert_eq!(with_result.stage.r#type, TaskStageType::PrintResult);
        assert!(with_result.start_output.is_some());
        assert!(with_result.end_output.is_some());
    }

    #[test]
    fn test_task_stage_type_serialize() {
        let stage_type = TaskStageType::TerraformPlan;
        let json = serde_json::to_string(&stage_type).unwrap();
        assert!(json.contains("terraform_plan"));
    }

    #[test]
    fn test_task_stage_type_deserialize() {
        let json = "\"running\"";
        let stage_type: TaskStageType = serde_json::from_str(json).unwrap();
        assert_eq!(stage_type, TaskStageType::Running);
    }

    #[test]
    fn test_task_stage_serialization() {
        let stage = TaskStage {
            id: 1,
            task_id: 10,
            project_id: 5,
            start: None,
            end: None,
            r#type: TaskStageType::Init,
        };
        let json = serde_json::to_string(&stage).unwrap();
        assert!(json.contains("\"type\":\"init\""));
        assert!(json.contains("\"task_id\":10"));
    }
}
