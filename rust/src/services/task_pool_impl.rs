//! TaskPool - пул задач для выполнения
//!
//! Аналог services/tasks/TaskPool.go из Go версии
//! Управляет очередью задач и их выполнением

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, error};

use crate::error::Result;
use crate::models::{Task, Project, TaskStatus};
use crate::services::task_logger::{TaskLogger, TaskStatus, BasicLogger};
use crate::services::local_job::LocalJob;
use crate::db_lib::AccessKeyInstallerImpl;
use crate::db::Store;

/// Задача в пуле
pub struct PoolTask {
    /// Задача
    pub task: Task,
    /// Логгер
    pub logger: Arc<dyn TaskLogger>,
    /// Статус выполнения
    pub running: bool,
}

/// Пул задач
pub struct TaskPool {
    /// Очередь задач
    pub queue: Arc<RwLock<Vec<PoolTask>>>,
    /// Запущенные задачи
    pub running: Arc<RwLock<HashMap<i32, PoolTask>>>,
    /// Проект
    pub project: Project,
    /// Установщик ключей
    pub key_installer: AccessKeyInstallerImpl,
    /// Флаг остановки
    pub shutdown: Arc<Mutex<bool>>,
    /// Хранилище данных
    pub store: Arc<dyn Store + Send + Sync>,
}

impl TaskPool {
    /// Создаёт новый пул задач
    pub fn new(project: Project, key_installer: AccessKeyInstallerImpl, store: Arc<dyn Store + Send + Sync>) -> Self {
        Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(HashMap::new())),
            project,
            key_installer,
            shutdown: Arc::new(Mutex::new(false)),
            store,
        }
    }

    /// Добавляет задачу в очередь
    pub async fn add_task(&self, task: Task) -> Result<()> {
        let logger = Arc::new(BasicLogger::new());
        
        let pool_task = PoolTask {
            task: task.clone(),
            logger: logger.clone(),
            running: false,
        };

        let mut queue = self.queue.write().await;
        queue.push(pool_task);

        // Обновляем статус задачи в БД
        self.store.update_task_status(task.id, TaskStatus::Waiting).await?;

        info!("Task {} added to pool", task.id);
        Ok(())
    }

    /// Запускает следующую задачу из очереди
    pub async fn run_next_task(&self) -> Result<Option<i32>> {
        let mut queue = self.queue.write().await;
        
        if queue.is_empty() {
            return Ok(None);
        }

        let pool_task = queue.remove(0);
        let task_id = pool_task.task.id;

        // Перемещаем задачу в running
        let mut running = self.running.write().await;
        running.insert(task_id, pool_task);

        info!("Task {} started", task_id);
        Ok(Some(task_id))
    }

    /// Завершает задачу
    pub async fn complete_task(&self, task_id: i32) -> Result<()> {
        let mut running = self.running.write().await;
        
        if let Some(pool_task) = running.remove(&task_id) {
            info!("Task {} completed", task_id);
            
            // Обновляем статус задачи в БД
            // TODO: self.store.update_task_status(task_id, pool_task.task.status).await?;
        }

        Ok(())
    }

    /// Останавливает задачу
    pub async fn kill_task(&self, task_id: i32) -> Result<()> {
        let mut running = self.running.write().await;
        
        if let Some(ref mut pool_task) = running.get_mut(&task_id) {
            pool_task.logger.set_status(TaskStatus::Stopped);
            pool_task.logger.log("Task killed by user");
            
            info!("Task {} killed", task_id);
        }

        Ok(())
    }

    /// Получает статус задачи
    pub async fn get_task_status(&self, task_id: i32) -> Option<TaskStatus> {
        let running = self.running.read().await;
        
        if let Some(pool_task) = running.get(&task_id) {
            return Some(pool_task.logger.get_status());
        }

        let queue = self.queue.read().await;
        for pool_task in queue.iter() {
            if pool_task.task.id == task_id {
                return Some(pool_task.logger.get_status());
            }
        }

        None
    }

    /// Запускает обработчик очереди
    pub async fn run_queue_processor(&self) {
        loop {
            // Проверяем флаг остановки
            {
                let shutdown = self.shutdown.lock().await;
                if *shutdown {
                    break;
                }
            }

            // Пытаемся запустить следующую задачу
            match self.run_next_task().await {
                Ok(Some(task_id)) => {
                    // TODO: Запустить задачу в фоне
                    // tokio::spawn(async move {
                    //     self.execute_task(task_id).await
                    // });
                }
                Ok(None) => {
                    // Очередь пуста, ждём
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!("Error running next task: {}", e);
                }
            }
        }
    }

    /// Выполняет задачу
    async fn execute_task(&self, task_id: i32) -> Result<()> {
        info!("Executing task {}", task_id);

        // Получаем полную информацию о задаче из БД
        let task = self.store.get_task(task_id).await?;
        let template = self.store.get_template(task.template_id).await?;
        let inventory = self.store.get_inventory(task.project_id, task.inventory_id.unwrap_or(0)).await?;
        let repository = self.store.get_repository(task.project_id, task.repository_id.unwrap_or(0)).await?;
        let environment = self.store.get_environment(task.project_id, task.environment_id.unwrap_or(0)).await?;

        // Создаём LocalJob
        let mut job = LocalJob::new(
            task,
            template,
            inventory,
            repository,
            environment,
            Arc::new(BasicLogger::new()),
            self.key_installer.clone(),
            std::path::PathBuf::from("/tmp/work"),
            std::path::PathBuf::from("/tmp/tmp"),
        );

        // Запускаем задачу
        if let Err(e) = job.run("system", None, "").await {
            error!("Task {} failed: {}", task_id, e);
            return Err(e);
        }

        self.complete_task(task_id).await?;
        Ok(())
    }

    /// Останавливает пул задач
    pub async fn shutdown(&self) {
        let mut shutdown = self.shutdown.lock().await;
        *shutdown = true;
        
        // Останавливаем все запущенные задачи
        let mut running = self.running.write().await;
        for (task_id, pool_task) in running.iter_mut() {
            pool_task.logger.set_status(TaskStatus::Stopped);
            info!("Task {} stopped during shutdown", task_id);
        }
        running.clear();

        info!("TaskPool shutdown complete");
    }

    /// Получает количество задач в очереди
    pub async fn queue_len(&self) -> usize {
        let queue = self.queue.read().await;
        queue.len()
    }

    /// Получает количество запущенных задач
    pub async fn running_len(&self) -> usize {
        let running = self.running.read().await;
        running.len()
    }
}

impl Drop for TaskPool {
    fn drop(&mut self) {
        // Асинхронная очистка не возможна в Drop,
        // поэтому вызывающая сторона должна явно вызвать shutdown()
        info!("TaskPool dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::db::MemoryDB;

    fn create_test_task(id: i32) -> Task {
        Task {
            id,
            created: Utc::now(),
            template_id: 1,
            status: TaskStatus::Waiting,
            message: String::new(),
            commit_hash: None,
            commit_message: None,
            version: None,
            project_id: 1,
            ..Default::default()
        }
    }

    fn create_test_pool() -> TaskPool {
        let project = Project::default();
        let key_installer = AccessKeyInstallerImpl::new();
        let store = Arc::new(MemoryDB::new());
        TaskPool::new(project, key_installer, store)
    }

    #[tokio::test]
    async fn test_task_pool_creation() {
        let pool = create_test_pool();
        assert_eq!(pool.queue_len().await, 0);
        assert_eq!(pool.running_len().await, 0);
    }

    #[tokio::test]
    async fn test_add_task() {
        let pool = create_test_pool();
        let task = create_test_task(1);
        
        pool.add_task(task).await.unwrap();
        assert_eq!(pool.queue_len().await, 1);
    }

    #[tokio::test]
    async fn test_run_next_task() {
        let pool = create_test_pool();
        let task = create_test_task(1);
        
        pool.add_task(task).await.unwrap();
        assert_eq!(pool.queue_len().await, 1);
        
        let result = pool.run_next_task().await.unwrap();
        assert_eq!(result, Some(1));
        assert_eq!(pool.queue_len().await, 0);
        assert_eq!(pool.running_len().await, 1);
    }

    #[tokio::test]
    async fn test_complete_task() {
        let pool = create_test_pool();
        let task = create_test_task(1);
        
        pool.add_task(task).await.unwrap();
        pool.run_next_task().await.unwrap();
        
        pool.complete_task(1).await.unwrap();
        assert_eq!(pool.running_len().await, 0);
    }

    #[tokio::test]
    async fn test_get_task_status() {
        let pool = create_test_pool();
        let task = create_test_task(1);
        
        pool.add_task(task).await.unwrap();
        
        let status = pool.get_task_status(1).await;
        assert!(status.is_some());
    }
}
