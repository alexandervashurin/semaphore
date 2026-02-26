//! TaskRunner Logging - логирование задач
//!
//! Аналог services/tasks/TaskRunner.go из Go версии (часть 4: логирование)

use std::sync::Arc;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

use crate::models::{TaskStatus, TaskOutput};
use crate::services::task_runner_types::TaskRunner;

impl TaskRunner {
    /// Логирует сообщение
    pub fn log(&self, msg: &str) {
        info!("Task {}: {}", self.task.id, msg);
        
        // Уведомляем слушателей логов
        for listener in &self.log_listeners {
            listener(Utc::now(), msg.to_string());
        }
    }
    
    /// Логирует с форматированием
    pub fn logf(&self, format: &str, args: &[&str]) {
        let msg = format.replace("{}", "{}");
        let mut formatted = msg;
        for (i, arg) in args.iter().enumerate() {
            formatted = formatted.replacen("{}", arg, 1);
        }
        self.log(&formatted);
    }
    
    /// Устанавливает статус задачи
    pub async fn set_status(&self, status: TaskStatus) {
        info!("Task {} status: {:?}", self.task.id, status);
        
        // Обновляем статус в БД
        if let Err(e) = self.pool.store.update_task_status(
            self.task.project_id,
            self.task.id,
            status,
        ).await {
            error!("Failed to update task status: {}", e);
        }
        
        // Уведомляем слушателей статусов
        for listener in &self.status_listeners {
            listener(status);
        }
    }
    
    /// Получает статус задачи
    pub fn get_status(&self) -> TaskStatus {
        self.task.status.clone()
    }
    
    /// Создаёт вывод задачи
    pub async fn create_task_output(&self, output: &str) -> Result<(), String> {
        let task_output = TaskOutput {
            id: 0,
            task_id: self.task.id,
            project_id: self.task.project_id,
            output: output.to_string(),
            time: Utc::now(),
        };
        
        self.pool.store.create_task_output(task_output)
            .await
            .map_err(|e| format!("Failed to create task output: {}", e))
    }
    
    /// Получает логи задачи
    pub async fn get_task_outputs(&self) -> Result<Vec<TaskOutput>, String> {
        use crate::models::RetrieveQueryParams;
        
        let params = RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: String::new(),
        };
        
        self.pool.store.get_task_outputs(
            self.task.project_id,
            self.task.id,
            &params,
        )
        .await
        .map_err(|e| format!("Failed to get task outputs: {}", e))
    }
    
    /// Ждёт завершения логов
    pub async fn wait_log(&self) {
        // В данной реализации ничего не делаем
        // В production можно реализовать очередь логов
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
    fn test_log() {
        let runner = create_test_runner().await;
        
        // Просто проверяем что метод вызывается без паники
        runner.log("Test message");
    }

    #[test]
    fn test_logf() {
        let runner = create_test_runner().await;
        
        runner.logf("Task {} completed", &["1".to_string()]);
    }

    #[tokio::test]
    async fn test_set_status() {
        let runner = create_test_runner().await;
        
        runner.set_status(TaskStatus::Running).await;
        
        assert_eq!(runner.get_status(), TaskStatus::Running);
    }

    #[test]
    fn test_get_status() {
        let runner = create_test_runner().await;
        
        let status = runner.get_status();
        assert_eq!(status, TaskStatus::Waiting);
    }

    #[tokio::test]
    async fn test_wait_log() {
        let runner = create_test_runner().await;
        
        runner.wait_log().await;
        // Просто проверяем что метод вызывается без паники
    }
}
