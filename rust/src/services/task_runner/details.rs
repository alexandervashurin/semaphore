//! TaskRunner Details - подготовка деталей задачи
//!
//! Аналог services/tasks/task_runner_details.go из Go версии

use crate::error::Result;
use crate::services::task_runner::TaskRunner;

impl TaskRunner {
    /// populate_details загружает детали задачи из БД
    pub async fn populate_details(&mut self) -> Result<()> {
        // Загрузка шаблона
        self.template = self
            .pool
            .store
            .get_template(self.task.project_id, self.task.template_id)
            .await?;

        // Загрузка инвентаря
        if let Some(inventory_id) = self.task.inventory_id {
            self.inventory = self
                .pool
                .store
                .get_inventory(self.template.project_id, inventory_id)
                .await?;
        }

        // Загрузка репозитория
        if let Some(repository_id) = self.task.repository_id {
            self.repository = self
                .pool
                .store
                .get_repository(self.template.project_id, repository_id)
                .await?;
        }

        // Загрузка окружения
        if let Some(environment_id) = self.task.environment_id {
            self.environment = self
                .pool
                .store
                .get_environment(self.template.project_id, environment_id)
                .await?;
        }

        Ok(())
    }

    /// populate_task_environment подготавливает окружение задачи
    pub async fn populate_task_environment(&mut self) -> Result<()> {
        // Получение пользователей для уведомлений
        // self.users = self.pool.store
        //     .get_template_users(self.task.template_id)
        //     .await?;

        // Получение алертов
        // let (alert, alert_chat) = self.pool.store
        //     .get_task_alert_chat(self.task.template_id)
        //     .await?;

        // self.alert = alert;
        // self.alert_chat = alert_chat;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::MockStore;
    use crate::db::store::*;
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::models::{Environment, Inventory, Project, Repository, Task, Template};
    use crate::services::task_logger::TaskStatus;
    use crate::services::task_pool::TaskPool;
    use chrono::Utc;
    use std::sync::Arc;

    fn create_test_task_runner() -> TaskRunner {
        let task = Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: TaskStatus::Waiting,
            message: None,
            commit_hash: None,
            commit_message: None,
            version: None,
            project_id: 1,
            inventory_id: Some(1),
            repository_id: Some(1),
            environment_id: Some(1),
            ..Default::default()
        };

        let pool = Arc::new(TaskPool::new(Arc::new(MockStore::new()), 5));

        TaskRunner::new(
            task,
            pool,
            "testuser".to_string(),
            AccessKeyInstallerImpl::new(),
        )
    }

    fn create_test_task_runner_with_store(store: Arc<MockStore>) -> TaskRunner {
        let task = Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: TaskStatus::Waiting,
            message: None,
            commit_hash: None,
            commit_message: None,
            version: None,
            project_id: 1,
            inventory_id: Some(1),
            repository_id: Some(1),
            environment_id: Some(1),
            ..Default::default()
        };

        let pool = Arc::new(TaskPool::new(store, 5));

        TaskRunner::new(
            task,
            pool,
            "testuser".to_string(),
            AccessKeyInstallerImpl::new(),
        )
    }

    #[tokio::test]
    async fn test_populate_details_fails_when_template_missing() {
        // TaskRunner без seeded template — должен вернуть NotFound
        let mut runner = create_test_task_runner();
        let result = runner.populate_details().await;
        assert!(result.is_err(), "Should fail when template not found");
    }

    #[tokio::test]
    async fn test_populate_details_loads_template_when_present() {
        let store = Arc::new(MockStore::new());
        let mut tpl = Template::default();
        tpl.id = 1;
        tpl.project_id = 1;
        tpl.name = "test_template".to_string();
        store.as_ref().create_template(tpl).await.unwrap();

        let mut runner = create_test_task_runner_with_store(store);
        let result = runner.populate_details().await;
        // Template загружен, но inventory/repository/environment не найдены
        assert!(result.is_err()); // inventory not found — это ожидаемо
    }

    #[tokio::test]
    async fn test_populate_details_skips_inventory_when_none() {
        let store = Arc::new(MockStore::new());
        let mut tpl = Template::default();
        tpl.id = 1;
        tpl.project_id = 1;
        store.as_ref().create_template(tpl).await.unwrap();

        let task = Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: TaskStatus::Waiting,
            project_id: 1,
            inventory_id: None,
            repository_id: Some(1),
            environment_id: Some(1),
            ..Default::default()
        };

        let pool = Arc::new(TaskPool::new(store, 5));
        let mut runner = TaskRunner::new(
            task,
            pool,
            "testuser".to_string(),
            AccessKeyInstallerImpl::new(),
        );

        let result = runner.populate_details().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_populate_details_skips_repository_when_none() {
        let store = Arc::new(MockStore::new());
        let mut tpl = Template::default();
        tpl.id = 1;
        tpl.project_id = 1;
        store.as_ref().create_template(tpl).await.unwrap();

        let task = Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: TaskStatus::Waiting,
            project_id: 1,
            inventory_id: None,
            repository_id: None,
            environment_id: Some(1),
            ..Default::default()
        };

        let pool = Arc::new(TaskPool::new(store, 5));
        let mut runner = TaskRunner::new(
            task,
            pool,
            "testuser".to_string(),
            AccessKeyInstallerImpl::new(),
        );

        let result = runner.populate_details().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_populate_details_skips_environment_when_none() {
        let store = Arc::new(MockStore::new());
        let mut tpl = Template::default();
        tpl.id = 1;
        tpl.project_id = 1;
        store.as_ref().create_template(tpl).await.unwrap();

        let task = Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: TaskStatus::Waiting,
            project_id: 1,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            ..Default::default()
        };

        let pool = Arc::new(TaskPool::new(store, 5));
        let mut runner = TaskRunner::new(
            task,
            pool,
            "testuser".to_string(),
            AccessKeyInstallerImpl::new(),
        );

        let result = runner.populate_details().await;
        assert!(result.is_ok(), "Should succeed when all optional fields are None");
    }

    #[tokio::test]
    async fn test_populate_details_loads_all_entities() {
        let store = Arc::new(MockStore::new());

        let mut tpl = Template::default();
        tpl.id = 1;
        tpl.project_id = 1;
        tpl.name = "test".to_string();
        store.as_ref().create_template(tpl).await.unwrap();

        let mut inv = Inventory::default();
        inv.id = 1;
        inv.project_id = 1;
        inv.name = "test_inventory".to_string();
        store.as_ref().create_inventory(inv).await.unwrap();

        let mut repo = Repository::default();
        repo.id = 1;
        repo.project_id = 1;
        repo.name = "test_repo".to_string();
        store.as_ref().create_repository(repo).await.unwrap();

        let mut env = Environment::default();
        env.id = 1;
        env.project_id = 1;
        env.name = "test_env".to_string();
        store.as_ref().create_environment(env).await.unwrap();

        let task = Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: TaskStatus::Waiting,
            project_id: 1,
            inventory_id: Some(1),
            repository_id: Some(1),
            environment_id: Some(1),
            ..Default::default()
        };

        let pool = Arc::new(TaskPool::new(store, 5));
        let mut runner = TaskRunner::new(
            task,
            pool,
            "testuser".to_string(),
            AccessKeyInstallerImpl::new(),
        );

        let result = runner.populate_details().await;
        assert!(result.is_ok(), "Should succeed with all entities seeded");
        assert_eq!(runner.template.id, 1);
        assert_eq!(runner.inventory.id, 1);
        assert_eq!(runner.repository.id, 1);
        assert_eq!(runner.environment.id, 1);
    }

    #[tokio::test]
    async fn test_populate_task_environment_always_ok() {
        let mut runner = create_test_task_runner();
        let result = runner.populate_task_environment().await;
        // Метод-заглушка всегда возвращает Ok(())
        assert!(result.is_ok());
    }
}
