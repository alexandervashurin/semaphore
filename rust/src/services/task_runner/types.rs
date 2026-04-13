//! TaskRunner Types - базовые типы и структура
//!
//! Аналог services/tasks/task_runner_types.go из Go версии

use crate::db_lib::AccessKeyInstallerImpl;
use crate::models::{Environment, Inventory, Repository, Task, Template};
use crate::services::task_logger::{LogListener, StatusListener, TaskLogger, TaskStatus};
use crate::services::task_pool::TaskPool;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Job trait определяет интерфейс для выполнения задачи
#[async_trait::async_trait]
pub trait Job: Send + Sync {
    /// Запускает задачу
    async fn run(&mut self) -> Result<(), crate::error::Error>;
    /// Останавливает задачу
    fn kill(&mut self);
    /// Проверяет, убита ли задача
    fn is_killed(&self) -> bool;
}

/// TaskRunner представляет выполняющуюся задачу
pub struct TaskRunner {
    /// Задача
    pub task: Task,
    /// Шаблон
    pub template: Template,
    /// Инвентарь
    pub inventory: Inventory,
    /// Репозиторий
    pub repository: Repository,
    /// Окружение
    pub environment: Environment,

    /// Текущая стадия
    pub current_stage: Option<crate::models::TaskStage>,
    /// Текущий вывод
    pub current_output: Option<crate::models::TaskOutput>,
    /// Текущее состояние
    pub current_state: Option<serde_json::Value>,

    /// Пользователи для уведомлений
    pub users: Vec<i32>,
    /// Флаг алерта
    pub alert: bool,
    /// Alert chat
    pub alert_chat: Option<String>,
    /// Ссылка на пул задач
    pub pool: Arc<TaskPool>,
    /// Установщик ключей
    pub key_installer: AccessKeyInstallerImpl,

    /// Job для выполнения
    pub job: Option<Box<dyn Job>>,

    /// ID раннера
    pub runner_id: i32,
    /// Имя пользователя
    pub username: String,
    /// Входящая версия
    pub incoming_version: Option<String>,

    /// Слушатели статусов
    pub status_listeners: Vec<StatusListener>,
    /// Слушатели логов
    pub log_listeners: Vec<LogListener>,

    /// Alias для запуска (например, для Terraform)
    pub alias: Option<String>,

    /// Флаг остановки
    pub killed: Arc<Mutex<bool>>,
}

impl TaskRunner {
    /// Создаёт новый TaskRunner
    pub fn new(
        task: Task,
        pool: Arc<TaskPool>,
        username: String,
        key_installer: AccessKeyInstallerImpl,
    ) -> Self {
        Self {
            task,
            pool,
            username,
            key_installer,
            template: Template::default(),
            inventory: Inventory::default(),
            repository: Repository::default(),
            environment: Environment::default(),
            current_stage: None,
            current_output: None,
            current_state: None,
            users: Vec::new(),
            alert: false,
            alert_chat: None,
            job: None,
            runner_id: 0,
            incoming_version: None,
            status_listeners: Vec::new(),
            log_listeners: Vec::new(),
            alias: None,
            killed: Arc::new(Mutex::new(false)),
        }
    }

    /// Добавляет слушателя статусов
    pub fn add_status_listener(&mut self, listener: StatusListener) {
        self.status_listeners.push(listener);
    }

    /// Добавляет слушателя логов
    pub fn add_log_listener(&mut self, listener: LogListener) {
        self.log_listeners.push(listener);
    }

    /// Проверяет, убита ли задача
    pub async fn is_killed(&self) -> bool {
        *self.killed.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::mock::MockStore;
    use crate::services::task_pool::TaskPool;
    use chrono::Utc;
    use std::sync::Arc;

    fn create_test_task() -> Task {
        Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            project_id: 1,
            status: TaskStatus::Waiting,
            playbook: None,
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: None,
            start: None,
            end: None,
            message: None,
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            params: None,
        }
    }

    fn create_test_task_pool() -> Arc<TaskPool> {
        let store = Arc::new(MockStore::new());
        Arc::new(TaskPool::new(store, 5))
    }

    #[tokio::test]
    async fn test_task_runner_creation_initializes_defaults() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "testuser".to_string(), key_installer);

        // Проверяем что базовые поля инициализированы
        assert_eq!(runner.task.id, 1);
        assert_eq!(runner.username, "testuser");
        assert_eq!(runner.runner_id, 0);
        assert_eq!(runner.status_listeners.len(), 0);
        assert_eq!(runner.log_listeners.len(), 0);
        assert!(runner.job.is_none());
        assert!(runner.alias.is_none());
        assert!(runner.incoming_version.is_none());
        assert_eq!(runner.users.len(), 0);
        assert!(!runner.alert);
        assert!(runner.alert_chat.is_none());

        // Default значения для сущностей
        assert_eq!(runner.template.id, 0);
        assert_eq!(runner.inventory.id, 0);
        assert_eq!(runner.repository.id, 0);
        assert_eq!(runner.environment.id, 0);

        // Stage и output — None
        assert!(runner.current_stage.is_none());
        assert!(runner.current_output.is_none());
        assert!(runner.current_state.is_none());
    }

    #[tokio::test]
    async fn test_task_runner_with_custom_username() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(
            task,
            pool,
            "admin".to_string(),
            key_installer,
        );

        assert_eq!(runner.username, "admin");
    }

    #[tokio::test]
    async fn test_task_runner_is_killed_returns_false_initially() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);

        assert!(!runner.is_killed().await);
    }

    #[tokio::test]
    async fn test_task_runner_is_killed_returns_true_after_set() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);

        // Устанавливаем флаг killed
        *runner.killed.lock().await = true;

        assert!(runner.is_killed().await);
    }

    #[tokio::test]
    async fn test_task_runner_add_status_listener() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let mut runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);

        // Добавляем слушателя (Box<dyn Fn(TaskStatus) + Send>)
        runner.add_status_listener(Box::new(|_status| {}));

        assert_eq!(runner.status_listeners.len(), 1);
    }

    #[tokio::test]
    async fn test_task_runner_add_log_listener() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let mut runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);

        // Добавляем слушателя логов (Box<dyn Fn(DateTime<Utc>, String) + Send + Sync>)
        runner.add_log_listener(Box::new(|_time, _log| {}));

        assert_eq!(runner.log_listeners.len(), 1);
    }

    #[tokio::test]
    async fn test_task_runner_add_multiple_listeners() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let mut runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);

        // Добавляем несколько слушателей
        runner.add_status_listener(Box::new(|_status| {}));
        runner.add_status_listener(Box::new(|_status| {}));
        runner.add_log_listener(Box::new(|_time, _log| {}));
        runner.add_log_listener(Box::new(|_time, _log| {}));
        runner.add_log_listener(Box::new(|_time, _log| {}));

        assert_eq!(runner.status_listeners.len(), 2);
        assert_eq!(runner.log_listeners.len(), 3);
    }

    #[tokio::test]
    async fn test_task_runner_killed_flag_is_shared_via_arc() {
        let task1 = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner1 = TaskRunner::new(task1, pool.clone(), "user1".to_string(), key_installer);

        // Создаём второй TaskRunner с тем же killed Arc (через клонирование)
        let task2 = create_test_task();
        let task2 = Task { id: 2, ..task2 };
        let killed_clone = runner1.killed.clone();

        // Проверяем что можно создать runner с общим killed
        let runner2_killed = killed_clone.clone();
        *runner2_killed.lock().await = true;

        // runner1 тоже видит изменение
        assert!(runner1.is_killed().await);
    }

    #[tokio::test]
    async fn test_task_runner_username_stored_correctly() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "myuser".to_string(), key_installer);
        assert_eq!(runner.username, "myuser");
    }

    #[tokio::test]
    async fn test_task_runner_pool_reference() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool.clone(), "user".to_string(), key_installer);
        // Pool is stored as Arc
        assert!(Arc::strong_count(&pool) >= 2);
    }

    #[tokio::test]
    async fn test_task_runner_default_values_comprehensive() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);

        assert!(runner.current_stage.is_none());
        assert!(runner.current_output.is_none());
        assert!(runner.current_state.is_none());
        assert!(runner.users.is_empty());
        assert!(!runner.alert);
        assert!(runner.alert_chat.is_none());
        assert!(runner.job.is_none());
        assert_eq!(runner.runner_id, 0);
        assert!(runner.incoming_version.is_none());
        assert!(runner.alias.is_none());
    }

    #[tokio::test]
    async fn test_task_runner_status_listeners_initially_empty() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);
        assert!(runner.status_listeners.is_empty());
    }

    #[tokio::test]
    async fn test_task_runner_log_listeners_initially_empty() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);
        assert!(runner.log_listeners.is_empty());
    }

    #[tokio::test]
    async fn test_task_runner_is_killed_false_initially() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);
        assert!(!runner.is_killed().await);
    }

    #[tokio::test]
    async fn test_task_runner_killed_flag_can_be_set() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);
        *runner.killed.lock().await = true;
        assert!(runner.is_killed().await);
    }

    #[tokio::test]
    async fn test_task_runner_killed_flag_can_be_unset() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);
        *runner.killed.lock().await = true;
        assert!(runner.is_killed().await);

        *runner.killed.lock().await = false;
        assert!(!runner.is_killed().await);
    }

    #[tokio::test]
    async fn test_task_runner_add_status_listener_increases_count() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let mut runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);
        assert_eq!(runner.status_listeners.len(), 0);

        runner.add_status_listener(Box::new(|_| {}));
        assert_eq!(runner.status_listeners.len(), 1);

        runner.add_status_listener(Box::new(|_| {}));
        assert_eq!(runner.status_listeners.len(), 2);
    }

    #[tokio::test]
    async fn test_task_runner_add_log_listener_increases_count() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let mut runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);
        assert_eq!(runner.log_listeners.len(), 0);

        runner.add_log_listener(Box::new(|_, _| {}));
        assert_eq!(runner.log_listeners.len(), 1);

        runner.add_log_listener(Box::new(|_, _| {}));
        assert_eq!(runner.log_listeners.len(), 2);
    }

    #[tokio::test]
    async fn test_task_runner_with_alias() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);
        assert!(runner.alias.is_none());
    }

    #[tokio::test]
    async fn test_task_runner_with_incoming_version() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);
        assert!(runner.incoming_version.is_none());
    }

    #[tokio::test]
    async fn test_task_runner_task_field_accessible() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task.clone(), pool, "user".to_string(), key_installer);
        assert_eq!(runner.task.id, task.id);
        assert_eq!(runner.task.project_id, task.project_id);
        assert_eq!(runner.task.template_id, task.template_id);
    }

    #[tokio::test]
    async fn test_task_runner_key_installer_stored() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);
        // key_installer is stored - just verify runner was created
        assert_eq!(runner.task.id, 1);
    }

    #[tokio::test]
    async fn test_task_runner_concurrent_is_killed_reads() {
        let task = create_test_task();
        let pool = create_test_task_pool();
        let key_installer = AccessKeyInstallerImpl::new();

        let runner = TaskRunner::new(task, pool, "user".to_string(), key_installer);

        let (r1, r2, r3) = tokio::join!(
            runner.is_killed(),
            runner.is_killed(),
            runner.is_killed(),
        );
        assert!(!r1);
        assert!(!r2);
        assert!(!r3);
    }
}
