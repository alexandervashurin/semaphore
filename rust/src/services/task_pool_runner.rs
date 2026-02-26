//! TaskPool Runner - запуск и выполнение задач
//!
//! Аналог services/tasks/TaskPool.go из Go версии (часть 3: runner)

use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, error};

use crate::models::{Task, TaskStatus};
use crate::services::task_pool_types::{TaskPool, RunningTask};
use crate::services::task_logger::{TaskLogger, BasicLogger};

impl TaskPool {
    /// Запускает задачу
    pub async fn run_task(&self, task: Task) -> Result<(), String> {
        if self.is_shutdown().await {
            return Err("TaskPool is shutdown".to_string());
        }
        
        // Получаем шаблон для задачи
        let template = self.store.get_template(task.project_id, task.template_id)
            .await
            .map_err(|e| format!("Failed to get template: {}", e))?;
        
        // Создаём логгер
        let logger = Arc::new(BasicLogger::new());
        
        // Создаём RunningTask
        let running_task = RunningTask::new(task.clone(), logger, template);
        
        // Добавляем в запущенные
        {
            let mut running = self.running_tasks.write().await;
            running.insert(task.id, running_task);
        }
        
        info!("Task {} started", task.id);
        
        // Запускаем выполнение в фоне
        let task_clone = task.clone();
        let pool_clone = Arc::new(self.clone());
        
        tokio::spawn(async move {
            if let Err(e) = pool_clone.execute_task(task_clone).await {
                error!("Task {} failed: {}", task.id, e);
            }
        });
        
        Ok(())
    }
    
    /// Выполняет задачу
    pub async fn execute_task(&self, task: Task) -> Result<(), String> {
        // Обновляем статус на Running
        self.update_task_status(task.id, TaskStatus::Running).await?;
        
        // TODO: Здесь будет логика выполнения задачи
        // LocalJob::run() или аналогичная логика
        
        // Симуляция выполнения
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Обновляем статус на Success
        self.update_task_status(task.id, TaskStatus::Success).await?;
        
        // Удаляем из запущенных
        {
            let mut running = self.running_tasks.write().await;
            running.remove(&task.id);
        }
        
        info!("Task {} completed", task.id);
        
        Ok(())
    }
    
    /// Останавливает задачу
    pub async fn kill_task(&self, task_id: i32) -> Result<(), String> {
        let mut running = self.running_tasks.write().await;
        
        if let Some(running_task) = running.get_mut(&task_id) {
            running_task.kill();
            info!("Task {} killed", task_id);
            
            // Обновляем статус на Stopped
            drop(running);
            self.update_task_status(task_id, TaskStatus::Stopped).await?;
            
            // Удаляем из запущенных
            running.remove(&task_id);
            
            return Ok(());
        }
        
        Err(format!("Task {} not found", task_id))
    }
    
    /// Получает запущенную задачу
    pub async fn get_running_task(&self, task_id: i32) -> Option<RunningTask> {
        let running = self.running_tasks.read().await;
        running.get(&task_id).cloned()
    }
    
    /// Получает все запущенные задачи
    pub async fn get_running_tasks(&self) -> std::collections::HashMap<i32, RunningTask> {
        let running = self.running_tasks.read().await;
        running.clone()
    }
    
    /// Обновляет статус задачи
    async fn update_task_status(&self, task_id: i32, status: TaskStatus) -> Result<(), String> {
        self.store.update_task_status(self.project.id, task_id, status)
            .await
            .map_err(|e| format!("Failed to update task status: {}", e))?;
        
        info!("Task {} status updated to {:?}", task_id, status);
        
        Ok(())
    }
    
    /// Обрабатывает очередь задач
    pub async fn process_queue(&self) {
        while !self.is_shutdown().await {
            // Проверяем количество запущенных задач
            let running_count = self.running_tasks.read().await.len();
            let max_parallel = self.project.max_parallel_tasks as usize;
            
            if running_count >= max_parallel {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }
            
            // Получаем задачу из очереди
            if let Some(task) = self.get_next_task().await {
                // Запускаем задачу
                if let Err(e) = self.run_task(task).await {
                    error!("Failed to run task: {}", e);
                }
            } else {
                // Очередь пуста, ждём
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
        
        info!("TaskPool queue processor stopped");
    }
}

// Clone реализация для TaskPool
impl Clone for TaskPool {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            project: self.project.clone(),
            running_tasks: self.running_tasks.clone(),
            task_queue: self.task_queue.clone(),
            shutdown: self.shutdown.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Project;
    use chrono::Utc;

    async fn create_test_pool() -> TaskPool {
        use crate::db::sql::SqlStore;
        
        let store = Arc::new(SqlStore::new(":memory:").unwrap());
        let project = Project {
            id: 1,
            name: "Test Project".to_string(),
            created: Utc::now(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 5,
        };
        
        TaskPool::new(store, project)
    }

    #[tokio::test]
    async fn test_kill_task() {
        let pool = create_test_pool().await;
        
        // Добавляем задачу в запущенные
        let task = crate::models::Task {
            id: 1,
            project_id: 1,
            template_id: 1,
            status: TaskStatus::Running,
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
        
        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let running_task = RunningTask::new(task.clone(), logger, template);
        
        {
            let mut running = pool.running_tasks.write().await;
            running.insert(1, running_task);
        }
        
        // Останавливаем задачу
        let result = pool.kill_task(1).await;
        assert!(result.is_ok());
        
        // Проверяем что задача удалена
        let running = pool.get_running_task(1).await;
        assert!(running.is_none());
    }

    #[tokio::test]
    async fn test_kill_nonexistent_task() {
        let pool = create_test_pool().await;
        
        let result = pool.kill_task(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_running_tasks() {
        let pool = create_test_pool().await;
        
        let tasks = pool.get_running_tasks().await;
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn test_run_task_after_shutdown() {
        let pool = create_test_pool().await;
        
        pool.shutdown().await;
        
        let task = crate::models::Task {
            id: 1,
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
        
        let result = pool.run_task(task).await;
        assert!(result.is_err());
    }
}
