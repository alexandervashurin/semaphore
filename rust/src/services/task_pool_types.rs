//! TaskPool Types - типы для пула задач
//!
//! Аналог services/tasks/TaskPool.go из Go версии (часть 1: типы)

use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::api::websocket::WebSocketManager;
use crate::db::store::Store;
use crate::models::{Project, Task, Template};
use crate::services::task_logger::TaskLogger;

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
    pub fn new(task: Task, logger: Arc<dyn TaskLogger>, template: Template) -> Self {
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
        assert_eq!(pool.project.id, 1); // Проверяем, что пул создан с правильным проектом
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

    #[tokio::test]
    async fn test_task_pool_initial_shutdown_flag() {
        let store = create_test_store();
        let project = create_test_project();
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());

        let pool = TaskPool::new(store, project, ws_manager);
        assert!(!pool.is_shutdown().await);
    }

    #[tokio::test]
    async fn test_task_pool_shutdown_idempotent() {
        let store = create_test_store();
        let project = create_test_project();
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());

        let pool = TaskPool::new(store, project, ws_manager);

        // Многократный shutdown не должен вызывать проблем
        pool.shutdown().await;
        pool.shutdown().await;
        pool.shutdown().await;

        assert!(pool.is_shutdown().await);
    }

    #[test]
    fn test_running_task_has_start_time() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Waiting;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let running_task = RunningTask::new(task, logger, template);
        // start_time должен быть установлен
        assert!(running_task.start_time <= Utc::now());
    }

    #[test]
    fn test_running_task_template() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Waiting;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let running_task = RunningTask::new(task, logger, template.clone());
        assert_eq!(running_task.template.id, template.id);
    }

    #[test]
    fn test_running_task_with_running_status() {
        let mut task = Task::default();
        task.id = 42;
        task.project_id = 5;
        task.template_id = 10;
        task.status = TaskStatus::Running;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let running = RunningTask::new(task, logger, template);
        assert!(!running.killed);
        assert_eq!(running.task.status, TaskStatus::Running);
        assert_eq!(running.task.id, 42);
    }

    #[test]
    fn test_running_task_kill_does_not_affect_task_status() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Running;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let mut running = RunningTask::new(task.clone(), logger, template);
        let original_status = running.task.status;

        running.kill();

        // Статус внутри task не должен измениться при kill
        assert_eq!(running.task.status, original_status);
        // Только killed флаг меняется
        assert!(running.killed);
    }

    #[test]
    fn test_running_task_template_clone_independence() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let mut template = Template::default();
        template.name = "original".to_string();

        let running = RunningTask::new(task, logger, template.clone());
        assert_eq!(running.template.name, "original");

        // Изменение оригинального template не влияет на running
        template.name = "modified".to_string();
        assert_eq!(running.template.name, "original");
    }

    #[test]
    fn test_task_pool_project_reference() {
        let store = create_test_store();
        let mut project = create_test_project();
        project.name = "My Project".to_string();
        project.max_parallel_tasks = 10;
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());

        let pool = TaskPool::new(store, project, ws_manager);

        assert_eq!(pool.project.name, "My Project");
        assert_eq!(pool.project.max_parallel_tasks, 10);
    }

    #[test]
    fn test_running_task_with_success_status() {
        let mut task = Task::default();
        task.id = 100;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Success;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let running = RunningTask::new(task, logger, template);
        assert_eq!(running.task.status, TaskStatus::Success);
        assert!(!running.killed);
    }

    #[test]
    fn test_running_task_fields_are_set_correctly() {
        let mut task = Task::default();
        task.id = 777;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Waiting;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let running = RunningTask::new(task, logger, template);
        // Проверяем что все поля корректно установлены
        assert_eq!(running.task.id, 777);
        assert_eq!(running.task.status, TaskStatus::Waiting);
        assert!(!running.killed);
    }

    #[test]
    fn test_running_task_logger_is_arc_shared() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let running = RunningTask::new(task, logger.clone(), template);

        // Arc::strong_count должен быть >= 2 (original + running)
        assert!(Arc::strong_count(&logger) >= 2);
        assert_eq!(
            Arc::strong_count(&running.logger),
            Arc::strong_count(&logger)
        );
    }

    #[test]
    fn test_running_task_with_error_status() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Error;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let mut running = RunningTask::new(task, logger, template);
        assert_eq!(running.task.status, TaskStatus::Error);
        assert!(!running.killed);

        running.kill();
        assert!(running.is_killed());
    }

    #[test]
    fn test_running_task_with_custom_template() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 5;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let mut template = Template::default();
        template.id = 5;
        template.name = "Deploy Template".to_string();
        template.playbook = "deploy.yml".to_string();

        let running = RunningTask::new(task, logger, template);
        assert_eq!(running.template.id, 5);
        assert_eq!(running.template.name, "Deploy Template");
        assert_eq!(running.template.playbook, "deploy.yml");
    }

    #[test]
    fn test_task_pool_has_running_tasks_map() {
        let store = create_test_store();
        let project = create_test_project();
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());
        let pool = TaskPool::new(store, project, ws_manager);
        // running_tasks should be an empty HashMap wrapped in Arc<RwLock>
        assert!(pool.running_tasks.try_read().is_ok());
    }

    #[test]
    fn test_task_pool_has_task_queue_vec() {
        let store = create_test_store();
        let project = create_test_project();
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());
        let pool = TaskPool::new(store, project, ws_manager);
        assert!(pool.task_queue.try_lock().is_ok());
    }

    #[test]
    fn test_task_pool_shutdown_flag_initial_state() {
        let store = create_test_store();
        let project = create_test_project();
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());
        let pool = TaskPool::new(store, project, ws_manager);
        // shutdown should be false initially
        assert!(!*pool.shutdown.try_lock().unwrap());
    }

    #[tokio::test]
    async fn test_task_pool_shutdown_sets_flag() {
        let store = create_test_store();
        let project = create_test_project();
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());
        let pool = TaskPool::new(store, project, ws_manager);
        assert!(!pool.is_shutdown().await);
        pool.shutdown().await;
        assert!(pool.is_shutdown().await);
    }

    #[tokio::test]
    async fn test_task_pool_is_shutdown_concurrent_access() {
        let store = create_test_store();
        let project = create_test_project();
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());
        let pool = TaskPool::new(store, project, ws_manager);

        // Multiple concurrent reads should work fine
        let (r1, r2, r3) =
            tokio::join!(pool.is_shutdown(), pool.is_shutdown(), pool.is_shutdown(),);
        assert!(!r1);
        assert!(!r2);
        assert!(!r3);
    }

    #[test]
    fn test_running_task_new_with_error_status() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Error;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let running = RunningTask::new(task, logger, template);
        assert_eq!(running.task.status, TaskStatus::Error);
        assert!(!running.killed);
    }

    #[test]
    fn test_running_task_new_with_stopped_status() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Stopped;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let running = RunningTask::new(task, logger, template);
        assert_eq!(running.task.status, TaskStatus::Stopped);
    }

    #[test]
    fn test_running_task_kill_is_idempotent() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let mut running = RunningTask::new(task, logger, template);
        running.kill();
        running.kill();
        running.kill();
        assert!(running.killed);
        assert!(running.is_killed());
    }

    #[tokio::test]
    async fn test_task_pool_project_max_parallel_tasks() {
        let store = create_test_store();
        let mut project = create_test_project();
        project.max_parallel_tasks = 20;
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());
        let pool = TaskPool::new(store, project, ws_manager);
        assert_eq!(pool.project.max_parallel_tasks, 20);
    }

    #[test]
    fn test_running_task_start_time_accuracy() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let before = Utc::now();
        let running = RunningTask::new(task, logger, template);
        let after = Utc::now();

        assert!(running.start_time >= before);
        assert!(running.start_time <= after);
    }

    #[test]
    fn test_task_pool_ws_manager_is_arc() {
        let store = create_test_store();
        let project = create_test_project();
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());
        let pool = TaskPool::new(store, project, ws_manager.clone());
        assert!(Arc::strong_count(&ws_manager) >= 2);
    }

    #[test]
    fn test_running_task_with_full_template() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 5;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let mut template = Template::default();
        template.id = 5;
        template.name = "Full Template".to_string();
        template.playbook = "full.yml".to_string();
        template.project_id = 1;

        let running = RunningTask::new(task, logger, template);
        assert_eq!(running.template.id, 5);
        assert_eq!(running.template.name, "Full Template");
        assert_eq!(running.template.playbook, "full.yml");
    }

    #[test]
    fn test_running_task_fields_accessible() {
        let mut task = Task::default();
        task.id = 42;
        task.project_id = 1;
        task.template_id = 1;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();

        let running = RunningTask::new(task, logger, template);
        assert_eq!(running.task.id, 42);
        assert!(!running.killed);
    }

    #[tokio::test]
    async fn test_task_pool_store_reference() {
        let store = create_test_store();
        let project = create_test_project();
        let ws_manager = Arc::new(crate::api::websocket::WebSocketManager::new());
        let pool = TaskPool::new(store.clone(), project, ws_manager);
        // pool.store should be accessible
        assert!(Arc::strong_count(&store) >= 2);
    }

    #[tokio::test]
    async fn test_running_task_arc_logger_shared() {
        let mut task = Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;

        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());
        let template = Template::default();
        let count_before = Arc::strong_count(&logger);

        let running = RunningTask::new(task, logger.clone(), template);

        assert!(Arc::strong_count(&logger) > count_before);
        assert!(Arc::strong_count(&running.logger) == Arc::strong_count(&logger));
    }
}
