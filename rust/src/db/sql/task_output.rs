//! Task Output - операции с выводами задач
//!
//! Аналог db/sql/task.go из Go версии (часть 2: TaskOutput)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает выводы задачи
    pub async fn get_task_outputs(&self, project_id: i32, task_id: i32, params: &RetrieveQueryParams) -> Result<Vec<TaskOutput>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let mut query = String::from(
                    "SELECT * FROM task_output WHERE task_id = ? AND project_id = ?"
                );
                
                // Добавляем лимит и оффсет
                query.push_str(&format!(" LIMIT {} OFFSET {}", params.count, params.offset));
                
                let outputs = sqlx::query_as::<_, TaskOutput>(&query)
                    .bind(task_id)
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                Ok(outputs)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт вывод задачи
    pub async fn create_task_output(&self, mut output: TaskOutput) -> Result<TaskOutput> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO task_output (task_id, project_id, output, time) VALUES (?, ?, ?, ?)"
                )
                .bind(output.task_id)
                .bind(output.project_id)
                .bind(&output.output)
                .bind(output.time)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                output.id = result.last_insert_rowid() as i32;
                Ok(output)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
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
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM task_output WHERE task_id = ? AND project_id = ?")
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
    
    /// Получает количество выводов задачи
    pub async fn get_task_output_count(&self, project_id: i32, task_id: i32) -> Result<usize> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query("SELECT COUNT(*) FROM task_output WHERE task_id = ? AND project_id = ?")
                    .bind(task_id)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                let count: i64 = result.get(0);
                Ok(count as usize)
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
        let temp_db = env::temp_dir().join("test_task_output.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();
        
        // Создаём таблицу task_output
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task_output (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                output TEXT NOT NULL,
                time DATETIME NOT NULL
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();
        
        db
    }

    #[tokio::test]
    async fn test_create_and_get_task_output() {
        let db = create_test_db().await;
        
        let output = TaskOutput {
            id: 0,
            task_id: 1,
            project_id: 1,
            output: "Test output line 1".to_string(),
            time: Utc::now(),
        };
        
        let created = db.create_task_output(output.clone()).await.unwrap();
        assert!(created.id > 0);
        
        let params = RetrieveQueryParams {
            offset: 0,
            count: 10,
            filter: None,
        };
        
        let outputs = db.get_task_outputs(1, 1, &params).await.unwrap();
        assert!(outputs.len() >= 1);
        assert_eq!(outputs[0].output, "Test output line 1");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_create_task_output_batch() {
        let db = create_test_db().await;
        
        let outputs = vec![
            TaskOutput {
                id: 0,
                task_id: 1,
                project_id: 1,
                output: "Output 1".to_string(),
                time: Utc::now(),
            },
            TaskOutput {
                id: 0,
                task_id: 1,
                project_id: 1,
                output: "Output 2".to_string(),
                time: Utc::now(),
            },
            TaskOutput {
                id: 0,
                task_id: 1,
                project_id: 1,
                output: "Output 3".to_string(),
                time: Utc::now(),
            },
        ];
        
        db.create_task_output_batch(outputs).await.unwrap();
        
        let params = RetrieveQueryParams {
            offset: 0,
            count: 10,
            filter: None,
        };
        
        let outputs = db.get_task_outputs(1, 1, &params).await.unwrap();
        assert_eq!(outputs.len(), 3);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_task_output() {
        let db = create_test_db().await;
        
        let output = TaskOutput {
            id: 0,
            task_id: 1,
            project_id: 1,
            output: "Test output".to_string(),
            time: Utc::now(),
        };
        
        db.create_task_output(output).await.unwrap();
        
        db.delete_task_output(1, 1).await.unwrap();
        
        let params = RetrieveQueryParams {
            offset: 0,
            count: 10,
            filter: None,
        };
        
        let outputs = db.get_task_outputs(1, 1, &params).await.unwrap();
        assert!(outputs.is_empty());
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_get_task_output_count() {
        let db = create_test_db().await;
        
        let outputs = vec![
            TaskOutput {
                id: 0,
                task_id: 1,
                project_id: 1,
                output: "Output 1".to_string(),
                time: Utc::now(),
            },
            TaskOutput {
                id: 0,
                task_id: 1,
                project_id: 1,
                output: "Output 2".to_string(),
                time: Utc::now(),
            },
        ];
        
        for output in outputs {
            db.create_task_output(output).await.unwrap();
        }
        
        let count = db.get_task_output_count(1, 1).await.unwrap();
        assert_eq!(count, 2);
        
        // Cleanup
        let _ = db.close().await;
    }
}
