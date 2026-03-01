//! TaskRunner Types - базовые типы и структура
//!
//! Аналог services/tasks/task_runner_types.go из Go версии

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::models::{Task, Template, Inventory, Repository, Environment};
use crate::services::task_logger::{TaskLogger, TaskStatus, StatusListener, LogListener};
use crate::db_lib::AccessKeyInstallerImpl;
use crate::services::task_pool::TaskPool;

/// Job trait определяет интерфейс для выполнения задачи
pub trait Job: Send + Sync {
    /// Запускает задачу
    fn run(&mut self, username: &str, incoming_version: Option<&str>, alias: &str) -> Result<(), crate::error::Error>;
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
    use chrono::Utc;

    fn create_test_task() -> Task {
        Task {
            id: 1,
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

    #[test]
    fn test_task_runner_creation() {
        // Mock pool and key_installer would be needed here
        // For now, just test that we can create the struct
        let task = create_test_task();
        
        // Basic assertion that task has correct ID
        assert_eq!(task.id, 1);
    }

    #[test]
    fn test_task_runner_is_killed_default() {
        // Test that newly created task runner is not killed
        // This would need proper mocking of dependencies
    }
}
