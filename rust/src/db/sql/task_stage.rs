//! Task Stage - операции со стадиями задач
//!
//! Аналог db/sql/task.go из Go версии (часть 3: TaskStage)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает стадии задачи
    pub async fn get_task_stages(&self, project_id: i32, task_id: i32) -> Result<Vec<TaskStage>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let stages = sqlx::query_as::<_, TaskStage>(
                    "SELECT * FROM task_stage WHERE task_id = ? AND project_id = ?"
                )
                .bind(task_id)
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(stages)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт стадию задачи
    pub async fn create_task_stage(&self, mut stage: TaskStage) -> Result<TaskStage> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO task_stage (task_id, project_id, type, start, end) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(stage.task_id)
                .bind(stage.project_id)
                .bind(stage.stage_type.to_string())
                .bind(stage.start)
                .bind(stage.end)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                stage.id = result.last_insert_rowid() as i32;
                Ok(stage)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Обновляет стадию задачи
    pub async fn update_task_stage(&self, stage: TaskStage) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE task_stage SET type = ?, start = ?, end = ? WHERE id = ? AND task_id = ? AND project_id = ?"
                )
                .bind(stage.stage_type.to_string())
                .bind(stage.start)
                .bind(stage.end)
                .bind(stage.id)
                .bind(stage.task_id)
                .bind(stage.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Получает результат стадии задачи
    pub async fn get_task_stage_result(&self, project_id: i32, task_id: i32, stage_id: i32) -> Result<Option<TaskStageResult>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "SELECT * FROM task_stage_result WHERE stage_id = ? AND task_id = ? AND project_id = ?"
                )
                .bind(stage_id)
                .bind(task_id)
                .bind(project_id)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                if let Some(row) = result {
                    let stage_result = TaskStageResult {
                        id: row.get(0),
                        stage_id: row.get(1),
                        task_id: row.get(2),
                        project_id: row.get(3),
                        result: row.get(4),
                    };
                    Ok(Some(stage_result))
                } else {
                    Ok(None)
                }
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт или обновляет результат стадии
    pub async fn upsert_task_stage_result(&self, mut result: TaskStageResult) -> Result<TaskStageResult> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let db_result = sqlx::query(
                    "INSERT OR REPLACE INTO task_stage_result (stage_id, task_id, project_id, result) VALUES (?, ?, ?, ?)"
                )
                .bind(result.stage_id)
                .bind(result.task_id)
                .bind(result.project_id)
                .bind(&result.result)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                result.id = db_result.last_insert_rowid() as i32;
                Ok(result)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Удаляет результат стадии
    pub async fn delete_task_stage_result(&self, project_id: i32, task_id: i32, stage_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM task_stage_result WHERE stage_id = ? AND task_id = ? AND project_id = ?")
                    .bind(stage_id)
                    .bind(task_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use chrono::Utc;

    async fn create_test_db() -> SqlDb {
        let temp_db = env::temp_dir().join("test_task_stage.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
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
        
        db
    }

    #[tokio::test]
    async fn test_create_and_get_task_stage() {
        let db = create_test_db().await;
        
        let stage = TaskStage {
            id: 0,
            task_id: 1,
            project_id: 1,
            stage_type: TaskStageType::InstallRoles,
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
        let db = create_test_db().await;
        
        let stage = TaskStage {
            id: 0,
            task_id: 1,
            project_id: 1,
            stage_type: TaskStageType::InstallRoles,
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
        let db = create_test_db().await;
        
        let result = TaskStageResult {
            id: 0,
            stage_id: 1,
            task_id: 1,
            project_id: 1,
            result: Some("Success".to_string()),
        };
        
        let created = db.upsert_task_stage_result(result.clone()).await.unwrap();
        assert!(created.id > 0);
        
        let retrieved = db.get_task_stage_result(1, 1, 1).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().result, Some("Success".to_string()));
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_task_stage_result() {
        let db = create_test_db().await;
        
        let result = TaskStageResult {
            id: 0,
            stage_id: 1,
            task_id: 1,
            project_id: 1,
            result: Some("Success".to_string()),
        };
        
        db.upsert_task_stage_result(result).await.unwrap();
        
        db.delete_task_stage_result(1, 1, 1).await.unwrap();
        
        let retrieved = db.get_task_stage_result(1, 1, 1).await.unwrap();
        assert!(retrieved.is_none());
        
        // Cleanup
        let _ = db.close().await;
    }
}
