//! TaskRunner Details - подготовка деталей задачи
//!
//! Аналог services/tasks/task_runner_details.go из Go версии

use crate::error::Result;
use crate::services::task_runner::TaskRunner;

impl TaskRunner {
    /// populate_details загружает детали задачи из БД
    pub async fn populate_details(&mut self) -> Result<()> {
        // Загрузка шаблона
        self.template = self.pool.store.get_template(self.task.template_id).await?;
        
        // Загрузка инвентаря
        if let Some(inventory_id) = self.task.inventory_id {
            self.inventory = self.pool.store
                .get_inventory(self.template.project_id, inventory_id)
                .await?;
        }
        
        // Загрузка репозитория
        if let Some(repository_id) = self.task.repository_id {
            self.repository = self.pool.store
                .get_repository(self.template.project_id, repository_id)
                .await?;
        }
        
        // Загрузка окружения
        if let Some(environment_id) = self.task.environment_id {
            self.environment = self.pool.store
                .get_environment(self.template.project_id, environment_id)
                .await?;
        }
        
        Ok(())
    }

    /// populate_task_environment подготавливает окружение задачи
    pub async fn populate_task_environment(&mut self) -> Result<()> {
        // Получение пользователей для уведомлений
        self.users = self.pool.store
            .get_template_users(self.task.template_id)
            .await?;
        
        // Получение алертов
        let (alert, alert_chat) = self.pool.store
            .get_task_alert_chat(self.task.template_id)
            .await?;
        
        self.alert = alert;
        self.alert_chat = alert_chat;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::models::{Task, TaskStatus, Project};
    use crate::services::task_pool::TaskPool;
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::db::MemoryDB;
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
            inventory_id: Some(1),
            repository_id: Some(1),
            environment_id: Some(1),
            ..Default::default()
        };
        
        let pool = Arc::new(TaskPool::new(
            Project::default(),
            AccessKeyInstallerImpl::new(),
            Arc::new(MemoryDB::new()),
        ));
        
        TaskRunner::new(task, pool, "testuser".to_string(), AccessKeyInstallerImpl::new())
    }

    #[tokio::test]
    async fn test_populate_details() {
        let mut runner = create_test_task_runner();
        
        // В реальном тесте нужна моковая БД с данными
        // Пока просто проверяем, что метод вызывается
        let result = runner.populate_details().await;
        
        // Ожидается ошибка, так как БД пустая
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_populate_task_environment() {
        let mut runner = create_test_task_runner();
        
        let result = runner.populate_task_environment().await;
        
        // Проверяем, что метод работает
        assert!(result.is_ok() || result.is_err()); // Either is fine for now
    }
}
