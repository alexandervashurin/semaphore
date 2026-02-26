//! TaskRunner LocalJob - обработка LocalJob
//!
//! Аналог services/tasks/TaskRunner.go из Go версии (часть 3: LocalJob)

use std::sync::Arc;
use tracing::{info, warn, error};

use crate::models::TaskStatus;
use crate::services::task_runner_types::TaskRunner;
use crate::services::local_job::LocalJob;

impl TaskRunner {
    /// Выполняет LocalJob
    pub async fn execute_local_job(&mut self) -> Result<(), String> {
        info!("Executing LocalJob for task {}", self.task.id);
        
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
        
        // Устанавливаем статус Running
        self.set_status(TaskStatus::Running).await;
        
        // Запускаем задачу
        if let Err(e) = local_job.run(&self.username, self.incoming_version.as_deref(), &self.alias).await {
            let err_msg = format!("LocalJob failed: {}", e);
            error!("{}", err_msg);
            self.set_status(TaskStatus::Error).await;
            return Err(err_msg);
        }
        
        // Успех
        self.set_status(TaskStatus::Success).await;
        
        Ok(())
    }
    
    /// Получает информацию для алерта
    pub fn alert_infos(&self) -> (String, String) {
        let author = self.username.clone();
        let version = self.task.version.clone().unwrap_or_default();
        
        (author, version)
    }
    
    /// Получает цвет алерта
    pub fn alert_color(&self, kind: &str) -> String {
        match self.task.status {
            TaskStatus::Success => match kind {
                "telegram" => "✅".to_string(),
                "slack" => "good".to_string(),
                "teams" => "8BC34A".to_string(),
                _ => "green".to_string(),
            },
            TaskStatus::Error => match kind {
                "telegram" => "❌".to_string(),
                "slack" => "danger".to_string(),
                "teams" => "F44336".to_string(),
                _ => "red".to_string(),
            },
            TaskStatus::Stopped => match kind {
                "telegram" => "⏹️".to_string(),
                "slack" => "warning".to_string(),
                "teams" => "FFC107".to_string(),
                _ => "yellow".to_string(),
            },
            _ => "gray".to_string(),
        }
    }
    
    /// Получает ссылку на задачу
    pub fn task_link(&self) -> String {
        format!(
            "{}/project/{}/tasks/{}",
            crate::config::get_public_host(),
            self.task.project_id,
            self.task.id
        )
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
            version: Some("1.0.0".to_string()),
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
    fn test_alert_infos() {
        let runner = create_test_runner().await;
        
        let (author, version) = runner.alert_infos();
        assert_eq!(author, "testuser");
        assert_eq!(version, "1.0.0");
    }

    #[test]
    fn test_alert_color_success() {
        let mut runner = create_test_runner().await;
        runner.task.status = TaskStatus::Success;
        
        assert_eq!(runner.alert_color("telegram"), "✅");
        assert_eq!(runner.alert_color("slack"), "good");
        assert_eq!(runner.alert_color("teams"), "8BC34A");
    }

    #[test]
    fn test_alert_color_error() {
        let mut runner = create_test_runner().await;
        runner.task.status = TaskStatus::Error;
        
        assert_eq!(runner.alert_color("telegram"), "❌");
        assert_eq!(runner.alert_color("slack"), "danger");
        assert_eq!(runner.alert_color("teams"), "F44336");
    }

    #[test]
    fn test_alert_color_stopped() {
        let mut runner = create_test_runner().await;
        runner.task.status = TaskStatus::Stopped;
        
        assert_eq!(runner.alert_color("telegram"), "⏹️");
        assert_eq!(runner.alert_color("slack"), "warning");
        assert_eq!(runner.alert_color("teams"), "FFC107");
    }

    #[tokio::test]
    async fn test_task_link() {
        let runner = create_test_runner().await;
        
        let link = runner.task_link();
        assert!(link.contains("/project/1/tasks/1"));
    }
}
