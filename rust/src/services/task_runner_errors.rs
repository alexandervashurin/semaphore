//! TaskRunner Errors - обработка ошибок
//!
//! Аналог services/tasks/TaskRunner.go из Go версии (часть 5: ошибки)

use std::sync::Arc;
use tracing::{info, warn, error};

use crate::models::TaskStatus;
use crate::services::task_runner_types::TaskRunner;

impl TaskRunner {
    /// Обрабатывает ошибку
    pub fn prepare_error(&self, err: &str, err_msg: &str) -> String {
        let full_msg = format!("{}: {}", err_msg, err);
        error!("{}", full_msg);
        full_msg
    }
    
    /// Логирует ошибку и устанавливает статус Error
    pub async fn handle_error(&self, err: &str, err_msg: &str) {
        let full_msg = self.prepare_error(err, err_msg);
        self.set_status(TaskStatus::Error).await;
        self.log(&full_msg);
    }
    
    /// Проверяет является ли ошибка фатальной
    pub fn is_error_fatal(&self, err: &str) -> bool {
        let err_lower = err.to_lowercase();
        
        // Список фатальных ошибок
        let fatal_errors = [
            "permission denied",
            "authentication failed",
            "connection refused",
            "no such file",
            "command not found",
        ];
        
        fatal_errors.iter().any(|fatal| err_lower.contains(fatal))
    }
    
    /// Обрабатывает фатальную ошибку
    pub async fn handle_fatal_error(&self, err: &str, err_msg: &str) {
        let full_msg = self.prepare_error(err, err_msg);
        
        if self.is_error_fatal(err) {
            error!("Fatal error: {}", full_msg);
            self.log("Fatal error detected");
        }
        
        self.set_status(TaskStatus::Error).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Task, Template, Inventory, Repository, Environment, Project, TaskStatus};
    use crate::db::sql::SqlStore;
    use crate::services::task_pool::TaskPool;
    use crate::db_lib::AccessKeyInstallerImpl;

    async fn create_test_runner() -> TaskRunner {
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
            playbook: "test.yml".to_string(),
            start: None,
            end: None,
        };
        
        TaskRunner::new(
            task,
            Template::default(),
            Inventory::default(),
            Repository::default(),
            Environment::default(),
            pool,
            key_installer,
            "testuser".to_string(),
        )
    }

    #[test]
    fn test_prepare_error() {
        let runner = create_test_runner().await;
        
        let result = runner.prepare_error("test error", "Failed");
        assert!(result.contains("Failed"));
        assert!(result.contains("test error"));
    }

    #[test]
    fn test_is_error_fatal_true() {
        let runner = create_test_runner().await;
        
        assert!(runner.is_error_fatal("permission denied"));
        assert!(runner.is_error_fatal("authentication failed"));
        assert!(runner.is_error_fatal("connection refused"));
        assert!(runner.is_error_fatal("no such file"));
        assert!(runner.is_error_fatal("command not found"));
    }

    #[test]
    fn test_is_error_fatal_false() {
        let runner = create_test_runner().await;
        
        assert!(!runner.is_error_fatal("minor warning"));
        assert!(!runner.is_error_fatal("temporary issue"));
    }

    #[tokio::test]
    async fn test_handle_error() {
        let runner = create_test_runner().await;
        
        runner.handle_error("test error", "Failed to run").await;
        
        assert_eq!(runner.get_status(), TaskStatus::Error);
    }

    #[tokio::test]
    async fn test_handle_fatal_error() {
        let runner = create_test_runner().await;
        
        runner.handle_fatal_error("permission denied", "Fatal").await;
        
        assert_eq!(runner.get_status(), TaskStatus::Error);
    }
}
