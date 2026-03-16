//! TaskPool Runner - запуск и выполнение задач
//!
//! Аналог services/tasks/TaskPool.go из Go версии (часть 3: runner)

use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, error};

use crate::models::{Task, Inventory, Repository, Environment};
use crate::services::task_logger::TaskStatus;
use crate::services::task_pool_types::{TaskPool, RunningTask};
use crate::services::task_logger::{TaskLogger, BasicLogger};
use crate::services::local_job::LocalJob;
use crate::db_lib::AccessKeyInstallerImpl;

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
    
    /// Выполняет задачу через LocalJob
    pub async fn execute_task(&self, task: Task) -> Result<(), String> {
        // Обновляем статус на Running
        self.update_task_status(task.id, TaskStatus::Running).await?;

        // Получаем шаблон
        let template = self.store.get_template(task.project_id, task.template_id)
            .await
            .map_err(|e| format!("Failed to get template: {}", e))?;

        // Получаем инвентарь, репозиторий, окружение
        let inventory_id = task.inventory_id.or(template.inventory_id);
        let inventory = match inventory_id {
            Some(id) => self.store.get_inventory(task.project_id, id)
                .await
                .map_err(|e| format!("Failed to get inventory: {}", e))?,
            None => Inventory::default(),
        };

        let repository_id = task.repository_id.or(template.repository_id);
        let repository = match repository_id {
            Some(id) => self.store.get_repository(task.project_id, id)
                .await
                .map_err(|e| format!("Failed to get repository: {}", e))?,
            None => Repository::default(),
        };

        let environment_id = task.environment_id.or(template.environment_id);
        let environment = match environment_id {
            Some(id) => self.store.get_environment(task.project_id, id)
                .await
                .map_err(|e| format!("Failed to get environment: {}", e))?,
            None => Environment::default(),
        };

        // Получаем логгер из running_task
        let logger = {
            let running = self.running_tasks.read().await;
            running.get(&task.id)
                .map(|rt| rt.logger.clone())
                .unwrap_or_else(|| Arc::new(BasicLogger::new()))
        };

        // Создаём рабочие директории
        let work_dir = std::env::temp_dir().join(format!("semaphore_task_{}_{}", task.project_id, task.id));
        let tmp_dir = work_dir.join("tmp");
        if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
            self.update_task_status(task.id, TaskStatus::Error).await.ok();
            let mut running = self.running_tasks.write().await;
            running.remove(&task.id);
            return Err(format!("Failed to create work dir: {}", e));
        }

        let key_installer = AccessKeyInstallerImpl::new();
        let mut job = LocalJob::new(
            task.clone(),
            template,
            inventory,
            repository,
            environment,
            logger,
            key_installer,
            work_dir.clone(),
            tmp_dir.clone(),
        );

        job.store = Some(self.store.clone());
        job.set_run_params("runner".to_string(), None, "default".to_string());

        let result = job.run("runner", None, "default").await;

        // Удаляем из запущенных
        {
            let mut running = self.running_tasks.write().await;
            running.remove(&task.id);
        }

        match result {
            Ok(()) => {
                job.cleanup();
                self.update_task_status(task.id, TaskStatus::Success).await?;
                info!("Task {} completed", task.id);
                Ok(())
            }
            Err(e) => {
                job.cleanup();
                error!("Task {} failed: {}", task.id, e);
                self.update_task_status(task.id, TaskStatus::Error).await?;
                Err(e.to_string())
            }
        }
    }
    
    /// Останавливает задачу
    pub async fn kill_task(&self, task_id: i32) -> Result<(), String> {
        let mut running = self.running_tasks.write().await;

        if let Some(running_task) = running.get_mut(&task_id) {
            running_task.kill();
            info!("Task {} killed", task_id);

            // Удаляем из запущенных
            running.remove(&task_id);
        } else {
            return Err(format!("Task {} not found", task_id));
        }

        drop(running);

        // Обновляем статус на Stopped
        self.update_task_status(task_id, TaskStatus::Stopped).await?;

        Ok(())
    }
    
    /// Получает запущенную задачу
    pub async fn get_running_task(&self, task_id: i32) -> Option<RunningTask> {
        let running = self.running_tasks.read().await;
        running.get(&task_id).map(|rt| RunningTask {
            task: rt.task.clone(),
            logger: rt.logger.clone(),
            start_time: rt.start_time,
            template: rt.template.clone(),
            killed: rt.killed,
        })
    }

    /// Получает все запущенные задачи
    pub async fn get_running_tasks(&self) -> std::collections::HashMap<i32, RunningTask> {
        let running = self.running_tasks.read().await;
        running.iter().map(|(k, v)| (*k, RunningTask {
            task: v.task.clone(),
            logger: v.logger.clone(),
            start_time: v.start_time,
            template: v.template.clone(),
            killed: v.killed,
        })).collect()
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
    use crate::db::mock::MockStore;
    use chrono::Utc;

    async fn create_test_pool() -> TaskPool {
        let store = Arc::new(MockStore::new());
        let project = Project {
            id: 1,
            name: "Test Project".to_string(),
            created: Utc::now(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 5,
            r#type: "default".to_string(),
            default_secret_storage_id: None,
        };
        
        TaskPool::new(store, project)
    }

    #[tokio::test]
    async fn test_kill_task() {
        let pool = create_test_pool().await;
        
        // Добавляем задачу в запущенные
        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Running;
        task.message = Some("Test task".to_string());
        
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
        
        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Waiting;
        task.message = Some("Test task".to_string());
        
        let result = pool.run_task(task).await;
        assert!(result.is_err());
    }
}
