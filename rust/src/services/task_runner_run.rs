//! TaskRunner Run - запуск задачи
//!
//! Аналог services/tasks/TaskRunner.go из Go версии (часть 2: запуск)

use std::sync::Arc;
use tracing::{info, warn, error};

use crate::models::TaskStatus;
use crate::services::task_runner_types::TaskRunner;
use crate::services::local_job::LocalJob;

impl TaskRunner {
    /// Запускает задачу
    pub async fn run(&mut self) -> Result<(), String> {
        info!("Starting task {}", self.task.id);
        
        // Устанавливаем статус Starting
        self.set_status(TaskStatus::Starting).await;
        
        // Подготовка
        if let Err(e) = self.prepare_run().await {
            let err_msg = format!("Failed to prepare run: {}", e);
            error!("{}", err_msg);
            self.set_status(TaskStatus::Error).await;
            return Err(err_msg);
        }
        
        // Checkout репозитория
        if let Err(e) = self.checkout_repository().await {
            let err_msg = format!("Failed to checkout repository: {}", e);
            error!("{}", err_msg);
            self.set_status(TaskStatus::Error).await;
            return Err(err_msg);
        }
        
        // Создаём LocalJob
        let mut local_job = LocalJob::new(
            self.task.clone(),
            self.template.clone(),
            self.inventory.clone(),
            self.repository.clone(),
            self.environment.clone(),
            Arc::new(self.create_logger()),
            self.key_installer.clone(),
            std::path::PathBuf::from("/tmp/work"),
            std::path::PathBuf::from("/tmp/tmp"),
        );
        
        // Запускаем задачу
        if let Err(e) = local_job.run(&self.username, self.incoming_version.as_deref(), &self.alias).await {
            let err_msg = format!("Failed to run task: {}", e);
            error!("{}", err_msg);
            self.set_status(TaskStatus::Error).await;
            return Err(err_msg);
        }
        
        // Успех
        self.set_status(TaskStatus::Success).await;
        info!("Task {} completed successfully", self.task.id);
        
        Ok(())
    }
    
    /// Подготовка к запуску
    async fn prepare_run(&self) -> Result<(), String> {
        info!("Preparing to run task {}", self.task.id);
        
        // Проверяем что все необходимые данные присутствуют
        if self.template.playbook.is_empty() {
            return Err("Template playbook is empty".to_string());
        }
        
        Ok(())
    }
    
    /// Checkout репозитория
    async fn checkout_repository(&self) -> Result<(), String> {
        use crate::db_lib::GitRepository;
        
        info!("Checking out repository {}", self.repository.id);
        
        // Создаём GitRepository
        let git_repo = GitRepository::new(
            &self.repository,
            std::path::PathBuf::from("/tmp/repo"),
            Arc::new(self.create_logger()),
            self.key_installer.clone(),
        );
        
        // Клонируем или pull
        git_repo.clone_or_pull().await
            .map_err(|e| format!("Failed to clone/pull repository: {}", e))?;
        
        // Checkout если указан commit
        if let Some(ref commit_hash) = self.task.commit_hash {
            git_repo.checkout(commit_hash).await
                .map_err(|e| format!("Failed to checkout commit: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Создаёт логгер
    fn create_logger(&self) -> impl crate::services::task_logger::TaskLogger {
        crate::services::task_logger::BasicLogger::new()
    }
    
    /// Устанавливает статус задачи
    async fn set_status(&self, status: TaskStatus) {
        // Обновляем статус в БД
        if let Err(e) = self.pool.store.update_task_status(
            self.task.project_id,
            self.task.id,
            status,
        ).await {
            error!("Failed to update task status: {}", e);
        }
        
        // Уведомляем слушателей
        for listener in &self.status_listeners {
            listener(status);
        }
    }
    
    /// Логирует сообщение
    fn log(&self, msg: &str) {
        info!("Task {}: {}", self.task.id, msg);
        
        // Уведомляем слушателей логов
        for listener in &self.log_listeners {
            listener(Utc::now(), msg.to_string());
        }
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

    #[tokio::test]
    async fn test_prepare_run() {
        let mut runner = create_test_runner().await;
        
        // Template playbook уже установлен в "test.yml"
        let result = runner.prepare_run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prepare_run_empty_playbook() {
        let mut runner = create_test_runner().await;
        runner.template.playbook = String::new();
        
        let result = runner.prepare_run().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("playbook"));
    }

    #[test]
    fn test_create_logger() {
        use crate::services::task_logger::TaskLogger;
        
        let runner = create_test_runner().await;
        let logger = runner.create_logger();
        
        // Просто проверяем что логгер создаётся
        logger.log("Test message");
    }
}
