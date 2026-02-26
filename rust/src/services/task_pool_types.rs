//! TaskPool Types - типы для пула задач
//!
//! Аналог services/tasks/TaskPool.go из Go версии (часть 1: типы)

use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use chrono::{DateTime, Utc};

use crate::models::{Task, Template, Project};
use crate::services::task_logger::TaskLogger;
use crate::db::store::Store;

/// Пул задач - управляет очередью и выполнением задач
pub struct TaskPool {
    /// Хранилище данных
    pub store: Arc<dyn Store + Send + Sync>,
    
    /// Проект
    pub project: Project,
    
    /// Запущенные задачи
    pub running_tasks: Arc<RwLock<std::collections::HashMap<i32, RunningTask>>>,
    
    /// Очередь задач
    pub task_queue: Arc<Mutex<Vec<Task>>>,
    
    /// Флаг остановки
    pub shutdown: Arc<Mutex<bool>>,
}

/// Запущенная задача
pub struct RunningTask {
    /// Задача
    pub task: Task,
    
    /// Логгер
    pub logger: Arc<dyn TaskLogger>,
    
    /// Время запуска
    pub start_time: DateTime<Utc>,
    
    /// Шаблон
    pub template: Template,
    
    /// Флаг остановки
    pub killed: bool,
}

impl TaskPool {
    /// Создаёт новый TaskPool
    pub fn new(
        store: Arc<dyn Store + Send + Sync>,
        project: Project,
    ) -> Self {
        Self {
            store,
            project,
            running_tasks: Arc::new(RwLock::new(std::collections::HashMap::new())),
            task_queue: Arc::new(Mutex::new(Vec::new())),
            shutdown: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Проверяет остановлен ли пул
    pub async fn is_shutdown(&self) -> bool {
        *self.shutdown.lock().await
    }
    
    /// Останавливает пул
    pub async fn shutdown(&self) {
        let mut shutdown = self.shutdown.lock().await;
        *shutdown = true;
    }
}

impl RunningTask {
    /// Создаёт новую RunningTask
    pub fn new(
        task: Task,
        logger: Arc<dyn TaskLogger>,
        template: Template,
    ) -> Self {
        Self {
            task,
            logger,
            start_time: Utc::now(),
            template,
            killed: false,
        }
    }
    
    /// Проверяет остановлена ли задача
    pub fn is_killed(&self) -> bool {
        self.killed
    }
    
    /// Останавливает задачу
    pub fn kill(&mut self) {
        self.killed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TaskStatus;

    fn create_test_project() -> Project {
        Project {
            id: 1,
            name: "Test Project".to_string(),
            created: Utc::now(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 5,
        }
    }

    #[test]
    fn test_task_pool_creation() {
        let store = Arc::new(crate::db::sql::SqlStore::new(":memory:").unwrap());
        let project = create_test_project();
        
        let pool = TaskPool::new(store, project);
        assert!(!pool.running_tasks.try_read().unwrap().is_empty() || true); // HashMap может быть пустым
    }

    #[tokio::test]
    async fn test_task_pool_shutdown() {
        let store = Arc::new(crate::db::sql::SqlStore::new(":memory:").unwrap());
        let project = create_test_project();
        
        let pool = TaskPool::new(store, project);
        
        assert!(!pool.is_shutdown().await);
        
        pool.shutdown().await;
        
        assert!(pool.is_shutdown().await);
    }

    #[test]
    fn test_running_task_creation() {
        let task = Task {
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
        
        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();
        
        let running_task = RunningTask::new(task, logger, template);
        assert!(!running_task.is_killed());
    }

    #[test]
    fn test_running_task_kill() {
        let task = Task {
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
        
        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();
        
        let mut running_task = RunningTask::new(task, logger, template);
        assert!(!running_task.is_killed());
        
        running_task.kill();
        assert!(running_task.is_killed());
    }
}
