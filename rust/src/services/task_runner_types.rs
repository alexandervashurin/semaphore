//! TaskRunner Types - типы для запуска задач
//!
//! Аналог services/tasks/TaskRunner.go из Go версии (часть 1: типы)

use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::models::{Task, Template, Inventory, Repository, Environment};
use crate::services::task_logger::TaskLogger;
use crate::db_lib::AccessKeyInstallerImpl;
use crate::services::task_pool::TaskPool;

/// TaskRunner - запускает и выполняет задачи
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
    
    /// Пользователи для уведомлений
    pub users: Vec<i32>,
    
    /// Флаг алерта
    pub alert: bool,
    
    /// Alert chat
    pub alert_chat: Option<String>,
    
    /// Пул задач
    pub pool: Arc<TaskPool>,
    
    /// Установщик ключей
    pub key_installer: AccessKeyInstallerImpl,
    
    /// Имя пользователя
    pub username: String,
    
    /// Входящая версия
    pub incoming_version: Option<String>,
    
    /// Alias
    pub alias: String,
    
    /// Статусы слушатели
    pub status_listeners: Vec<Arc<dyn Fn(crate::models::TaskStatus) + Send + Sync>>,
    
    /// Лог слушатели
    pub log_listeners: Vec<Arc<dyn Fn(DateTime<Utc>, String) + Send + Sync>>,
}

impl TaskRunner {
    /// Создаёт новый TaskRunner
    pub fn new(
        task: Task,
        template: Template,
        inventory: Inventory,
        repository: Repository,
        environment: Environment,
        pool: Arc<TaskPool>,
        key_installer: AccessKeyInstallerImpl,
        username: String,
    ) -> Self {
        Self {
            task,
            template,
            inventory,
            repository,
            environment,
            users: Vec::new(),
            alert: false,
            alert_chat: None,
            pool,
            key_installer,
            username,
            incoming_version: None,
            alias: String::new(),
            status_listeners: Vec::new(),
            log_listeners: Vec::new(),
        }
    }
    
    /// Добавляет слушателя статусов
    pub fn add_status_listener<F>(&mut self, listener: F)
    where
        F: Fn(crate::models::TaskStatus) + Send + Sync + 'static,
    {
        self.status_listeners.push(Arc::new(listener));
    }
    
    /// Добавляет слушателя логов
    pub fn add_log_listener<F>(&mut self, listener: F)
    where
        F: Fn(DateTime<Utc>, String) + Send + Sync + 'static,
    {
        self.log_listeners.push(Arc::new(listener));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TaskStatus, Project};

    fn create_test_task() -> Task {
        Task {
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
        }
    }

    fn create_test_components() -> (Template, Inventory, Repository, Environment, Arc<TaskPool>, AccessKeyInstallerImpl) {
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
        
        let pool = Arc::new(TaskPool::new(store, project));
        let key_installer = AccessKeyInstallerImpl::new();
        
        (
            Template::default(),
            Inventory::default(),
            Repository::default(),
            Environment::default(),
            pool,
            key_installer,
        )
    }

    #[test]
    fn test_task_runner_creation() {
        let task = create_test_task();
        let (template, inventory, repository, environment, pool, key_installer) = create_test_components();
        
        let runner = TaskRunner::new(
            task,
            template,
            inventory,
            repository,
            environment,
            pool,
            key_installer,
            "testuser".to_string(),
        );
        
        assert_eq!(runner.task.id, 1);
        assert_eq!(runner.username, "testuser");
        assert!(runner.users.is_empty());
        assert!(!runner.alert);
    }

    #[test]
    fn test_task_runner_add_status_listener() {
        let task = create_test_task();
        let (template, inventory, repository, environment, pool, key_installer) = create_test_components();
        
        let mut runner = TaskRunner::new(
            task,
            template,
            inventory,
            repository,
            environment,
            pool,
            key_installer,
            "testuser".to_string(),
        );
        
        runner.add_status_listener(|_| {});
        
        assert_eq!(runner.status_listeners.len(), 1);
    }

    #[test]
    fn test_task_runner_add_log_listener() {
        let task = create_test_task();
        let (template, inventory, repository, environment, pool, key_installer) = create_test_components();
        
        let mut runner = TaskRunner::new(
            task,
            template,
            inventory,
            repository,
            environment,
            pool,
            key_installer,
            "testuser".to_string(),
        );
        
        runner.add_log_listener(|_, _| {});
        
        assert_eq!(runner.log_listeners.len(), 1);
    }
}
