//! Task CRUD - операции с задачами
//!
//! Аналог db/sql/task.go из Go версии (часть 1: CRUD)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use crate::services::task_logger::TaskStatus;
use sqlx::Row;

impl SqlDb {
    /// Получает задачи проекта
    pub async fn get_tasks(&self, project_id: i32, template_id: Option<i32>) -> Result<Vec<TaskWithTpl>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let mut query = String::from(
                    "SELECT t.*, tpl.name as template_name 
                     FROM task t 
                     LEFT JOIN template tpl ON t.template_id = tpl.id 
                     WHERE t.project_id = ?"
                );
                
                if let Some(tpl_id) = template_id {
                    query.push_str(" AND t.template_id = ?");
                    
                    let tasks = sqlx::query_as::<_, TaskWithTpl>(&query)
                        .bind(project_id)
                        .bind(tpl_id)
                        .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                        .await
                        .map_err(|e| Error::Database(e))?;
                    
                    Ok(tasks)
                } else {
                    let tasks = sqlx::query_as::<_, TaskWithTpl>(&query)
                        .bind(project_id)
                        .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                        .await
                        .map_err(|e| Error::Database(e))?;
                    
                    Ok(tasks)
                }
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Получает задачу по ID
    pub async fn get_task(&self, project_id: i32, task_id: i32) -> Result<Task> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let task = sqlx::query_as::<_, Task>(
                    "SELECT * FROM task WHERE project_id = ? AND id = ?"
                )
                .bind(project_id)
                .bind(task_id)
                .fetch_one(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(task)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт новую задачу
    pub async fn create_task(&self, mut task: Task) -> Result<Task> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO task (
                        project_id, template_id, status, message, 
                        commit_hash, commit_message, version,
                        inventory_id, repository_id, environment_id,
                        arguments, params, playbook, start, end
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(task.project_id)
                .bind(task.template_id)
                .bind(task.status.to_string())
                .bind(&task.message)
                .bind(&task.commit_hash)
                .bind(&task.commit_message)
                .bind(&task.version)
                .bind(task.inventory_id)
                .bind(task.repository_id)
                .bind(task.environment_id)
                .bind(&task.arguments)
                .bind(&task.params)
                .bind(&task.playbook)
                .bind(task.start)
                .bind(task.end)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                task.id = result.last_insert_rowid() as i32;
                Ok(task)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Обновляет задачу
    pub async fn update_task(&self, task: Task) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE task SET 
                        template_id = ?, status = ?, message = ?,
                        commit_hash = ?, commit_message = ?, version = ?,
                        inventory_id = ?, repository_id = ?, environment_id = ?,
                        arguments = ?, params = ?, playbook = ?, start = ?, end = ?
                    WHERE id = ? AND project_id = ?"
                )
                .bind(task.template_id)
                .bind(task.status.to_string())
                .bind(&task.message)
                .bind(&task.commit_hash)
                .bind(&task.commit_message)
                .bind(&task.version)
                .bind(task.inventory_id)
                .bind(task.repository_id)
                .bind(task.environment_id)
                .bind(&task.arguments)
                .bind(&task.params)
                .bind(&task.playbook)
                .bind(task.start)
                .bind(task.end)
                .bind(task.id)
                .bind(task.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Обновляет статус задачи
    pub async fn update_task_status(&self, project_id: i32, task_id: i32, status: TaskStatus) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("UPDATE task SET status = ? WHERE id = ? AND project_id = ?")
                    .bind(status.to_string())
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
    
    /// Удаляет задачу
    pub async fn delete_task(&self, project_id: i32, task_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM task WHERE id = ? AND project_id = ?")
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
        let temp_db = env::temp_dir().join("test_task.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();
        
        // Создаём таблицу task
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                template_id INTEGER,
                status TEXT NOT NULL,
                message TEXT,
                commit_hash TEXT,
                commit_message TEXT,
                version TEXT,
                inventory_id INTEGER,
                repository_id INTEGER,
                environment_id INTEGER,
                arguments TEXT,
                params TEXT,
                playbook TEXT,
                start DATETIME,
                end DATETIME
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();
        
        // Создаём таблицу template для join
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS template (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();
        
        db
    }

    #[tokio::test]
    async fn test_create_and_get_task() {
        let db = create_test_db().await;
        
        let task = Task {
            id: 0,
            project_id: 1,
            template_id: 1,
            status: TaskStatus::Waiting,
            message: "Test task".to_string(),
            commit_hash: None,
            commit_message: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            arguments: None,
            params: String::new(),
            playbook: String::new(),
            start: None,
            end: None,
        };
        
        let created = db.create_task(task.clone()).await.unwrap();
        assert!(created.id > 0);
        
        let retrieved = db.get_task(1, created.id).await.unwrap();
        assert_eq!(retrieved.message, "Test task");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_get_tasks() {
        let db = create_test_db().await;
        
        // Создаём несколько задач
        for i in 0..5 {
            let task = Task {
                id: 0,
                project_id: 1,
                template_id: 1,
                status: TaskStatus::Waiting,
                message: format!("Task {}", i),
                commit_hash: None,
                commit_message: None,
                version: None,
                inventory_id: None,
                repository_id: None,
                environment_id: None,
                arguments: None,
                params: String::new(),
                playbook: String::new(),
                start: None,
                end: None,
            };
            db.create_task(task).await.unwrap();
        }
        
        let tasks = db.get_tasks(1, None).await.unwrap();
        assert!(tasks.len() >= 5);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let db = create_test_db().await;
        
        let task = Task {
            id: 0,
            project_id: 1,
            template_id: 1,
            status: TaskStatus::Waiting,
            message: "Test task".to_string(),
            commit_hash: None,
            commit_message: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            arguments: None,
            params: String::new(),
            playbook: String::new(),
            start: None,
            end: None,
        };
        
        let created = db.create_task(task).await.unwrap();
        
        db.update_task_status(1, created.id, TaskStatus::Running).await.unwrap();
        
        let retrieved = db.get_task(1, created.id).await.unwrap();
        assert_eq!(retrieved.status, TaskStatus::Running);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_task() {
        let db = create_test_db().await;
        
        let task = Task {
            id: 0,
            project_id: 1,
            template_id: 1,
            status: TaskStatus::Waiting,
            message: "Test task".to_string(),
            commit_hash: None,
            commit_message: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            arguments: None,
            params: String::new(),
            playbook: String::new(),
            start: None,
            end: None,
        };
        
        let created = db.create_task(task).await.unwrap();
        
        db.delete_task(1, created.id).await.unwrap();
        
        let result = db.get_task(1, created.id).await;
        assert!(result.is_err());
        
        // Cleanup
        let _ = db.close().await;
    }
}
