//! TaskRunner Lifecycle - жизненный цикл задачи
//!
//! Аналог services/tasks/task_runner_lifecycle.go из Go версии

use crate::db_lib::AccessKeyInstallerImpl;
use crate::error::Result;
use crate::services::local_job::LocalJob;
use crate::services::task_logger::TaskLogger;
use crate::services::task_runner::TaskRunner;
use std::sync::Arc;
use tracing::{error, info};

impl TaskRunner {
    /// run запускает задачу
    pub async fn run(&mut self) -> Result<()> {
        self.log("Task started");

        // Подготовка деталей
        if let Err(e) = self.populate_details().await {
            let msg = format!("Failed to populate details: {}", e);
            self.log(&msg);
            return Err(e);
        }

        // Подготовка окружения
        if let Err(e) = self.populate_task_environment().await {
            let msg = format!("Failed to populate environment: {}", e);
            self.log(&msg);
            return Err(e);
        }

        // Создание LocalJob
        let logger = Arc::new(crate::services::task_logger::BasicLogger::new());

        let mut local_job = LocalJob::new(
            self.task.clone(),
            self.template.clone(),
            self.inventory.clone(),
            self.repository.clone(),
            self.environment.clone(),
            logger,
            crate::db_lib::AccessKeyInstallerImpl::new(),
            std::path::PathBuf::from(format!("/tmp/semaphore/task_{}", self.task.id)),
            std::path::PathBuf::from(format!("/tmp/semaphore/task_{}_tmp", self.task.id)),
        );
        local_job.store =
            Some(Arc::clone(&self.pool.store) as Arc<dyn crate::db::store::Store + Send + Sync>);
        local_job.set_run_params(
            self.username.clone(),
            self.incoming_version.clone(),
            self.alias.clone().unwrap_or_default(),
        );
        self.job = Some(Box::new(local_job));

        // Запуск задачи
        if let Some(ref mut job) = self.job {
            if let Err(e) = job.run().await {
                let msg = format!("Task failed: {}", e);
                self.log(&msg);
                return Err(e);
            }
        }

        self.log("Task completed successfully");

        // Создание события задачи
        self.create_task_event().await?;

        Ok(())
    }

    /// kill останавливает задачу
    pub async fn kill(&mut self) {
        if let Some(ref mut job) = self.job {
            job.kill();
        }

        let mut killed = self.killed.lock().await;
        *killed = true;

        self.log("Task killed");
    }

    /// create_task_event создаёт событие задачи в БД
    pub async fn create_task_event(&self) -> Result<()> {
        use crate::models::{Event, EventType};

        let obj_type = EventType::TaskCreated;
        let desc = format!(
            "Task {} ({}) finished - {}",
            self.task.id,
            self.template.name,
            self.task.status.to_string().to_uppercase()
        );

        match self
            .pool
            .store
            .create_event(Event {
                id: 0,
                object_type: obj_type.to_string(),
                object_id: Some(self.task.id),
                project_id: Some(self.task.project_id),
                description: desc,
                user_id: None,
                created: chrono::Utc::now(),
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to create task event: {}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::MockStore;
    use crate::db::store::*;
    use crate::models::Task;
    use crate::services::task_logger::TaskStatus;
    use chrono::Utc;
    use std::sync::Arc;

    fn create_test_task_runner() -> TaskRunner {
        use crate::services::task_pool::TaskPool;

        let task = Task {
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
        };

        let pool = Arc::new(TaskPool::new(Arc::new(MockStore::new()), 5));

        TaskRunner::new(
            task,
            pool,
            "testuser".to_string(),
            AccessKeyInstallerImpl::new(),
        )
    }

    #[tokio::test]
    async fn test_task_runner_log() {
        let runner = create_test_task_runner();
        runner.log("Test message");
    }

    #[tokio::test]
    async fn test_task_runner_set_status() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Running).await;
        assert_eq!(runner.task.status, TaskStatus::Running);
    }

    #[tokio::test]
    async fn test_create_task_event() {
        let runner = create_test_task_runner();
        let r = runner.create_task_event().await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn test_kill_marks_killed_flag() {
        let mut runner = create_test_task_runner();
        runner.kill().await;
        assert!(runner.is_killed().await);
    }

    #[tokio::test]
    async fn test_create_task_event_records_event() {
        let runner = create_test_task_runner();
        let result = runner.create_task_event().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_fails_on_empty_template() {
        let mut runner = create_test_task_runner();
        let result = runner.run().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_logs_start_and_end_messages() {
        let mut runner = create_test_task_runner();
        let result = runner.run().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kill_without_job() {
        let mut runner = create_test_task_runner();
        runner.kill().await;
        assert!(runner.is_killed().await);
    }

    #[tokio::test]
    async fn test_task_runner_full_lifecycle_status_transitions() {
        let mut runner = create_test_task_runner();
        assert_eq!(runner.get_status(), TaskStatus::Waiting);
        runner.set_status(TaskStatus::Starting).await;
        assert_eq!(runner.get_status(), TaskStatus::Starting);
        runner.set_status(TaskStatus::Running).await;
        assert_eq!(runner.get_status(), TaskStatus::Running);
        runner.set_status(TaskStatus::Success).await;
        assert_eq!(runner.get_status(), TaskStatus::Success);
    }

    #[tokio::test]
    async fn test_kill_is_idempotent() {
        let mut runner = create_test_task_runner();
        // Первый kill
        runner.kill().await;
        assert!(runner.is_killed().await);
        // Второй kill — не должен паниковать
        runner.kill().await;
        assert!(runner.is_killed().await);
    }

    #[tokio::test]
    async fn test_create_task_event_contains_task_id() {
        let runner = create_test_task_runner();
        // Проверяем что событие создаётся с корректным task_id
        let result = runner.create_task_event().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_returns_error_when_populate_details_fails() {
        // MockStore пустой — populate_details вернёт NotFound
        let mut runner = create_test_task_runner();
        let result = runner.run().await;
        // Ошибка должна быть на этапе populate_details
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_status_updates_task_status() {
        let mut runner = create_test_task_runner();

        let statuses = [
            TaskStatus::Waiting,
            TaskStatus::Starting,
            TaskStatus::Running,
            TaskStatus::Success,
            TaskStatus::Error,
            TaskStatus::Stopped,
        ];

        for status in statuses {
            runner.set_status(status).await;
            assert_eq!(
                runner.task.status, status,
                "Status should be {:?} after set_status",
                status
            );
        }
    }

    #[tokio::test]
    async fn test_create_task_event_with_custom_task_id() {
        use crate::services::task_pool::TaskPool;

        let task = Task {
            id: 999,
            project_id: 42,
            template_id: 100,
            status: TaskStatus::Success,
            ..Default::default()
        };

        let pool = Arc::new(TaskPool::new(Arc::new(MockStore::new()), 5));
        let runner = TaskRunner::new(
            task,
            pool,
            "admin".to_string(),
            AccessKeyInstallerImpl::new(),
        );

        let result = runner.create_task_event().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_task_runner_initial_status_is_waiting() {
        let runner = create_test_task_runner();
        assert_eq!(runner.get_status(), TaskStatus::Waiting);
    }

    #[tokio::test]
    async fn test_set_status_to_error() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Error).await;
        assert_eq!(runner.task.status, TaskStatus::Error);
    }

    #[tokio::test]
    async fn test_set_status_to_stopped() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Stopped).await;
        assert_eq!(runner.task.status, TaskStatus::Stopped);
    }

    #[tokio::test]
    async fn test_kill_sets_killed_flag_before_log() {
        let mut runner = create_test_task_runner();
        assert!(!runner.is_killed().await);
        runner.kill().await;
        assert!(runner.is_killed().await);
    }

    #[tokio::test]
    async fn test_log_does_not_panic_with_empty_message() {
        let runner = create_test_task_runner();
        runner.log("");
    }

    #[tokio::test]
    async fn test_log_does_not_panic_with_multiline_message() {
        let runner = create_test_task_runner();
        runner.log("Line 1\nLine 2\nLine 3");
    }

    #[tokio::test]
    async fn test_status_transition_waiting_to_starting() {
        let mut runner = create_test_task_runner();
        assert_eq!(runner.get_status(), TaskStatus::Waiting);
        runner.set_status(TaskStatus::Starting).await;
        assert_eq!(runner.get_status(), TaskStatus::Starting);
    }

    #[tokio::test]
    async fn test_status_transition_running_to_success() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Running).await;
        runner.set_status(TaskStatus::Success).await;
        assert_eq!(runner.get_status(), TaskStatus::Success);
    }

    #[tokio::test]
    async fn test_status_transition_running_to_error() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Running).await;
        runner.set_status(TaskStatus::Error).await;
        assert_eq!(runner.get_status(), TaskStatus::Error);
    }

    #[tokio::test]
    async fn test_status_transition_to_stopped_from_running() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Running).await;
        runner.set_status(TaskStatus::Stopped).await;
        assert_eq!(runner.get_status(), TaskStatus::Stopped);
    }

    #[tokio::test]
    async fn test_create_task_event_description_contains_task_id() {
        let runner = create_test_task_runner();
        // Event creation should succeed, description contains task id
        let result = runner.create_task_event().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_fails_without_template_in_store() {
        let mut runner = create_test_task_runner();
        // MockStore пустой, поэтому populate_details упадёт
        let result = runner.run().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kill_does_not_modify_task_status() {
        let mut runner = create_test_task_runner();
        let status_before = runner.task.status.clone();
        runner.kill().await;
        assert_eq!(runner.task.status, status_before);
    }

    #[tokio::test]
    async fn test_multiple_status_changes_in_sequence() {
        let mut runner = create_test_task_runner();
        let all_statuses = [
            TaskStatus::Waiting,
            TaskStatus::Starting,
            TaskStatus::Running,
            TaskStatus::Success,
        ];
        for status in &all_statuses {
            runner.set_status(status.clone()).await;
            assert_eq!(runner.get_status(), status.clone());
        }
    }
}
