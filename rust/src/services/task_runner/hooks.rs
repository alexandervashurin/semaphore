//! TaskRunner Hooks - hooks для задач
//!
//! Аналог services/tasks/task_runner_hooks.go из Go версии

use crate::error::Result;
use crate::services::task_runner::TaskRunner;

impl TaskRunner {
    /// run_hooks запускает hooks для задачи
    pub async fn run_hooks(&self, event_type: &str) -> Result<()> {
        // Получение hooks из шаблона
        let hooks_list = &self.template.hooks;
        
        if hooks_list.is_empty() {
            return Ok(());
        }
        
        // Запуск hooks для указанного события
        for hook in hooks_list {
            if hook.event == event_type {
                if let Err(e) = self.execute_hook(hook).await {
                    self.log(&format!("Hook failed: {}", e));
                    // Продолжаем выполнение даже если hook не удался
                }
            }
        }
        
        Ok(())
    }

    /// execute_hook выполняет один hook
    async fn execute_hook(&self, hook: &crate::models::Hook) -> Result<()> {
        // TODO: Интеграция с PRO hook executor
        // В зависимости от типа hook:
        // - HTTP запрос
        // - Выполнение скрипта
        // - Отправка уведомления
        
        self.log(&format!("Executing hook: {}", hook.name));
        
        Ok(())
    }

    /// run_before_hooks запускает hooks перед задачей
    pub async fn run_before_hooks(&self) -> Result<()> {
        self.run_hooks("before_task").await
    }

    /// run_after_hooks запускает hooks после задачи
    pub async fn run_after_hooks(&self) -> Result<()> {
        self.run_hooks("after_task").await
    }

    /// run_on_success_hooks запускает hooks при успешном завершении
    pub async fn run_on_success_hooks(&self) -> Result<()> {
        self.run_hooks("on_success").await
    }

    /// run_on_failure_hooks запускает hooks при ошибке
    pub async fn run_on_failure_hooks(&self) -> Result<()> {
        self.run_hooks("on_failure").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::services::task_logger::TaskStatus;
    use crate::models::{Task, Project};
    use crate::services::task_pool::TaskPool;
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::db::MockStore;
    use std::sync::Arc;

    fn create_test_task_runner() -> TaskRunner {
        let task = Task {
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
        };

        let pool = Arc::new(TaskPool::new(
            Project::default(),
            AccessKeyInstallerImpl::new(),
            Arc::new(MockStore::new()),
        ));

        TaskRunner::new(task, pool, "testuser".to_string(), AccessKeyInstallerImpl::new())
    }

    #[tokio::test]
    async fn test_run_hooks_empty() {
        let runner = create_test_task_runner();
        let result = runner.run_hooks("before_task").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_before_hooks() {
        let runner = create_test_task_runner();
        let result = runner.run_before_hooks().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_after_hooks() {
        let runner = create_test_task_runner();
        let result = runner.run_after_hooks().await;
        assert!(result.is_ok());
    }
}
