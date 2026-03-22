//! Task Stage - операции со стадиями задач
//!
//! Аналог db/sql/task.go из Go версии (часть 3: TaskStage)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_task_stage(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает стадии задачи
    pub async fn get_task_stages(&self, project_id: i32, task_id: i32) -> Result<Vec<TaskStage>> {
        let rows = sqlx::query(
            "SELECT * FROM task_stage WHERE task_id = $1 AND project_id = $2 ORDER BY id"
        )
        .bind(task_id)
        .bind(project_id)
        .fetch_all(self.pg_pool_task_stage()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| {
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
        }).collect())
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
             VALUES ($1, $2, $3, $4, $5) RETURNING id"
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
    pub async fn get_task_stage_result(&self, project_id: i32, task_id: i32, stage_id: i32) -> Result<Option<TaskStageResult>> {
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
    pub async fn upsert_task_stage_result(&self, mut result: TaskStageResult) -> Result<TaskStageResult> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO task_stage_result (stage_id, task_id, project_id, result) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (stage_id, task_id, project_id) \
             DO UPDATE SET result = EXCLUDED.result \
             RETURNING id"
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
    pub async fn delete_task_stage_result(&self, project_id: i32, task_id: i32, stage_id: i32) -> Result<()> {
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
    use chrono::Utc;

    struct TestDb {
        db: SqlDb,
        _temp: tempfile::NamedTempFile,
    }

    async fn create_test_db() -> TestDb {
        let (db_path, temp) = crate::db::sql::init::test_sqlite_url();

        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();

        // Создаём таблицу task_stage
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task_stage (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                type TEXT NOT NULL,
                start DATETIME,
                end DATETIME
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();

        // Создаём таблицу task_stage_result
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task_stage_result (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                stage_id INTEGER NOT NULL,
                task_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                result TEXT
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();

        TestDb { db, _temp: temp }
    }

    #[tokio::test]
    async fn test_create_and_get_task_stage() {
        let TestDb { db, _temp } = create_test_db().await;

        let stage = TaskStage {
            id: 0,
            task_id: 1,
            project_id: 1,
            r#type: TaskStageType::Init,
            start: Some(Utc::now()),
            end: None,
        };

        let created = db.create_task_stage(stage.clone()).await.unwrap();
        assert!(created.id > 0);

        let stages = db.get_task_stages(1, 1).await.unwrap();
        assert!(stages.len() >= 1);

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_task_stage() {
        let TestDb { db, _temp } = create_test_db().await;

        let stage = TaskStage {
            id: 0,
            task_id: 1,
            project_id: 1,
            r#type: TaskStageType::Init,
            start: Some(Utc::now()),
            end: None,
        };

        let created = db.create_task_stage(stage).await.unwrap();

        let mut updated = created.clone();
        updated.end = Some(Utc::now());

        db.update_task_stage(updated).await.unwrap();

        let stages = db.get_task_stages(1, 1).await.unwrap();
        assert!(stages[0].end.is_some());

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_upsert_task_stage_result() {
        let TestDb { db, _temp } = create_test_db().await;

        let result = TaskStageResult {
            id: 0,
            stage_id: 1,
            task_id: 1,
            project_id: 1,
            result: "Success".to_string(),
        };

        let created = db.upsert_task_stage_result(result.clone()).await.unwrap();
        assert!(created.id > 0);

        let retrieved = db.get_task_stage_result(1, 1, 1).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().result, "Success".to_string());

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_task_stage_result() {
        let TestDb { db, _temp } = create_test_db().await;

        let result = TaskStageResult {
            id: 0,
            stage_id: 1,
            task_id: 1,
            project_id: 1,
            result: "Success".to_string(),
        };

        db.upsert_task_stage_result(result).await.unwrap();

        db.delete_task_stage_result(1, 1, 1).await.unwrap();

        let retrieved = db.get_task_stage_result(1, 1, 1).await.unwrap();
        assert!(retrieved.is_none());

        // Cleanup
        let _ = db.close().await;
    }
}
