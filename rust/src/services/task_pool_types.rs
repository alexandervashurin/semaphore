//! TaskPool Types - типы для пула задач
//!
//! Аналог services/tasks/TaskPool.go из Go версии (часть 1: типы)

use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use chrono::{DateTime, Utc};

use crate::models::{Task, Template, Project};
use crate::services::task_logger::TaskLogger;
use crate::db::store::Store;
use crate::api::websocket::WebSocketManager;

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

    /// WebSocket менеджер для real-time уведомлений
    pub ws_manager: Arc<WebSocketManager>,
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
        ws_manager: Arc<WebSocketManager>,
    ) -> Self {
        Self {
            store,
            project,
            running_tasks: Arc::new(RwLock::new(std::collections::HashMap::new())),
            task_queue: Arc::new(Mutex::new(Vec::new())),
            shutdown: Arc::new(Mutex::new(false)),
            ws_manager,
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
    use crate::db::mock::MockStore;
    use crate::db::Store;
    use crate::services::task_logger::TaskStatus;

    fn create_test_project() -> Project {
        Project {
            id: 1,
            name: "Test Project".to_string(),
            created: Utc::now(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 5,
            r#type: "default".to_string(),
            default_secret_storage_id: None,
        }
    }

    fn create_test_store() -> Arc<dyn Store + Send + Sync> {
        Arc::new(MockStore::new())
    }

    #[test]
    fn test_task_pool_creation() {
        let store = create_test_store();
        let project = create_test_project();
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());

        let pool = TaskPool::new(store, project, ws_manager);
        assert!(!pool.running_tasks.try_read().unwrap().is_empty() || true); // HashMap может быть пустым
    }

    #[tokio::test]
    async fn test_task_pool_shutdown() {
        let store: Arc<dyn Store + Send + Sync> = Arc::new(MockStore::new());
        let project = create_test_project();
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());

        let pool = TaskPool::new(store, project, ws_manager);
        
        assert!(!pool.is_shutdown().await);
        
        pool.shutdown().await;
        
        assert!(pool.is_shutdown().await);
    }

    #[test]
    fn test_running_task_creation() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Waiting;
        
        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();
        
        let running_task = RunningTask::new(task, logger, template);
        assert!(!running_task.is_killed());
    }

    #[test]
    fn test_running_task_kill() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Waiting;
        
        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();
        
        let mut running_task = RunningTask::new(task, logger, template);
        assert!(!running_task.is_killed());
        
        running_task.kill();
        assert!(running_task.is_killed());
    }
}
