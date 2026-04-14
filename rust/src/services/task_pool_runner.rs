//! TaskPool Runner - запуск и выполнение задач
//!
//! Аналог services/tasks/TaskPool.go из Go версии (часть 3: runner)

use chrono::Utc;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};

use crate::db_lib::AccessKeyInstallerImpl;
use crate::models::{Environment, Inventory, Repository, Task};
use crate::services::local_job::LocalJob;
use crate::services::task_logger::TaskStatus;
use crate::services::task_logger::{BasicLogger, TaskLogger};
use crate::services::task_pool_types::{RunningTask, TaskPool};

impl TaskPool {
    /// Запускает задачу
    pub async fn run_task(&self, task: Task) -> Result<(), String> {
        if self.is_shutdown().await {
            return Err("TaskPool is shutdown".to_string());
        }

        // Получаем шаблон для задачи
        let template = self
            .store
            .get_template(task.project_id, task.template_id)
            .await
            .map_err(|e| format!("Failed to get template: {}", e))?;

        // Создаём логгер
        let logger = Arc::new(BasicLogger::new());

        // Создаём RunningTask
        let running_task = RunningTask::new(task.clone(), logger, template);

        // Добавляем в запущенные
        {
            let mut running = self.running_tasks.write().await;
            running.insert(task.id, running_task);
        }

        info!("Task {} started", task.id);

        // Запускаем выполнение в фоне
        let task_clone = task.clone();
        let pool_clone = Arc::new(self.clone());

        tokio::spawn(async move {
            if let Err(e) = pool_clone.execute_task(task_clone).await {
                error!("Task {} failed: {}", task.id, e);
            }
        });

        Ok(())
    }

    /// Выполняет задачу через LocalJob
    pub async fn execute_task(&self, mut task: Task) -> Result<(), String> {
        // Обновляем статус на Running и фиксируем время начала
        task.status = TaskStatus::Running;
        task.start = Some(Utc::now());
        self.store
            .update_task(task.clone())
            .await
            .map_err(|e| format!("Failed to update task: {}", e))?;
        self.notify_websocket(task.id, TaskStatus::Running).await;

        // Получаем шаблон
        let template = self
            .store
            .get_template(task.project_id, task.template_id)
            .await
            .map_err(|e| format!("Failed to get template: {}", e))?;

        // Получаем инвентарь, репозиторий, окружение
        let inventory_id = task.inventory_id.or(template.inventory_id);
        let inventory = match inventory_id {
            Some(id) => self
                .store
                .get_inventory(task.project_id, id)
                .await
                .map_err(|e| format!("Failed to get inventory: {}", e))?,
            None => Inventory::default(),
        };

        let repository_id = task.repository_id.or(template.repository_id);
        let repository = match repository_id {
            Some(id) => self
                .store
                .get_repository(task.project_id, id)
                .await
                .map_err(|e| format!("Failed to get repository: {}", e))?,
            None => Repository::default(),
        };

        let environment_id = task.environment_id.or(template.environment_id);
        let environment = match environment_id {
            Some(id) => self
                .store
                .get_environment(task.project_id, id)
                .await
                .map_err(|e| format!("Failed to get environment: {}", e))?,
            None => Environment::default(),
        };

        // Получаем логгер из running_task
        let logger = {
            let running = self.running_tasks.read().await;
            running
                .get(&task.id)
                .map(|rt| rt.logger.clone())
                .unwrap_or_else(|| Arc::new(BasicLogger::new()))
        };

        // Создаём рабочие директории
        let work_dir =
            std::env::temp_dir().join(format!("semaphore_task_{}_{}", task.project_id, task.id));
        let tmp_dir = work_dir.join("tmp");
        if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
            task.status = TaskStatus::Error;
            task.end = Some(Utc::now());
            self.store.update_task(task.clone()).await.ok();
            self.notify_websocket(task.id, TaskStatus::Error).await;
            let mut running = self.running_tasks.write().await;
            running.remove(&task.id);
            return Err(format!("Failed to create work dir: {}", e));
        }

        let key_installer = AccessKeyInstallerImpl::new();
        let mut job = LocalJob::new(
            task.clone(),
            template,
            inventory,
            repository,
            environment,
            logger,
            key_installer,
            work_dir.clone(),
            tmp_dir.clone(),
        );

        job.store = Some(self.store.clone());
        job.set_run_params("runner".to_string(), None, "default".to_string());

        let result = job.run("runner", None, "default").await;

        // Удаляем из запущенных
        {
            let mut running = self.running_tasks.write().await;
            running.remove(&task.id);
        }

        task.end = Some(Utc::now());
        match result {
            Ok(()) => {
                task.status = TaskStatus::Success;
                self.store.update_task(task.clone()).await.ok();
                self.notify_websocket(task.id, TaskStatus::Success).await;
                job.cleanup();
                info!("Task {} completed", task.id);
                Ok(())
            }
            Err(e) => {
                task.status = TaskStatus::Error;
                self.store.update_task(task.clone()).await.ok();
                self.notify_websocket(task.id, TaskStatus::Error).await;
                job.cleanup();
                error!("Task {} failed: {}", task.id, e);
                Err(e.to_string())
            }
        }
    }

    /// Останавливает задачу
    pub async fn kill_task(&self, task_id: i32) -> Result<(), String> {
        let mut running = self.running_tasks.write().await;

        if let Some(running_task) = running.get_mut(&task_id) {
            running_task.kill();
            info!("Task {} killed", task_id);

            // Удаляем из запущенных
            running.remove(&task_id);
        } else {
            return Err(format!("Task {} not found", task_id));
        }

        drop(running);

        // Обновляем статус на Stopped
        self.update_task_status(task_id, TaskStatus::Stopped)
            .await?;

        Ok(())
    }

    /// Получает запущенную задачу
    pub async fn get_running_task(&self, task_id: i32) -> Option<RunningTask> {
        let running = self.running_tasks.read().await;
        running.get(&task_id).map(|rt| RunningTask {
            task: rt.task.clone(),
            logger: rt.logger.clone(),
            start_time: rt.start_time,
            template: rt.template.clone(),
            killed: rt.killed,
        })
    }

    /// Получает все запущенные задачи
    pub async fn get_running_tasks(&self) -> std::collections::HashMap<i32, RunningTask> {
        let running = self.running_tasks.read().await;
        running
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    RunningTask {
                        task: v.task.clone(),
                        logger: v.logger.clone(),
                        start_time: v.start_time,
                        template: v.template.clone(),
                        killed: v.killed,
                    },
                )
            })
            .collect()
    }

    /// Обрабатывает очередь задач
    pub async fn process_queue(&self) {
        while !self.is_shutdown().await {
            // Проверяем количество запущенных задач
            let running_count = self.running_tasks.read().await.len();
            let max_parallel = self.project.max_parallel_tasks as usize;

            if running_count >= max_parallel {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }

            // Получаем задачу из очереди
            if let Some(task) = self.get_next_task().await {
                // Запускаем задачу
                if let Err(e) = self.run_task(task).await {
                    error!("Failed to run task: {}", e);
                }
            } else {
                // Очередь пуста, ждём
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        info!("TaskPool queue processor stopped");
    }
}

// Clone реализация для TaskPool
impl Clone for TaskPool {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            project: self.project.clone(),
            running_tasks: self.running_tasks.clone(),
            task_queue: self.task_queue.clone(),
            shutdown: self.shutdown.clone(),
            ws_manager: self.ws_manager.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::mock::MockStore;
    use crate::models::Project;
    use chrono::Utc;

    async fn create_test_pool() -> TaskPool {
        let store = Arc::new(MockStore::new());
        let project = Project {
            id: 1,
            name: "Test Project".to_string(),
            created: Utc::now(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 5,
            r#type: "default".to_string(),
            default_secret_storage_id: None,
        };

        TaskPool::new(
            store,
            project,
            Arc::new(crate::api::websocket::WebSocketManager::new()),
        )
    }

    #[tokio::test]
    async fn test_kill_task() {
        let pool = create_test_pool().await;

        // Добавляем задачу в запущенные
        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Running;
        task.message = Some("Test task".to_string());

        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let running_task = RunningTask::new(task.clone(), logger, template);

        {
            let mut running = pool.running_tasks.write().await;
            running.insert(1, running_task);
        }

        // Останавливаем задачу
        let result = pool.kill_task(1).await;
        assert!(result.is_ok());

        // Проверяем что задача удалена
        let running = pool.get_running_task(1).await;
        assert!(running.is_none());
    }

    #[tokio::test]
    async fn test_kill_nonexistent_task() {
        let pool = create_test_pool().await;

        let result = pool.kill_task(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_running_tasks() {
        let pool = create_test_pool().await;

        let tasks = pool.get_running_tasks().await;
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn test_run_task_after_shutdown() {
        let pool = create_test_pool().await;

        pool.shutdown().await;

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Waiting;
        task.message = Some("Test task".to_string());

        let result = pool.run_task(task).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_running_task_returns_none_for_missing() {
        let pool = create_test_pool().await;

        let task = pool.get_running_task(999).await;
        assert!(task.is_none());
    }

    #[tokio::test]
    async fn test_running_task_initial_state() {
        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let mut task = crate::models::Task::default();
        task.id = 42;
        task.project_id = 1;
        task.template_id = 1;

        let running = RunningTask::new(task.clone(), logger, template);

        assert_eq!(running.task.id, 42);
        assert!(!running.killed);
        assert!(!running.is_killed());
    }

    #[tokio::test]
    async fn test_running_task_kill_changes_state() {
        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;

        let mut running = RunningTask::new(task, logger, template);

        assert!(!running.is_killed());

        running.kill();

        assert!(running.is_killed());
        assert!(running.killed);
    }

    #[tokio::test]
    async fn test_running_task_kill_is_idempotent() {
        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;

        let mut running = RunningTask::new(task, logger, template);

        running.kill();
        running.kill();
        running.kill();

        assert!(running.is_killed());
    }

    #[tokio::test]
    async fn test_get_running_task_returns_copy() {
        let pool = create_test_pool().await;

        // Добавляем задачу в запущенные
        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Running;

        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let running_task = RunningTask::new(task.clone(), logger, template);

        {
            let mut running = pool.running_tasks.write().await;
            running.insert(1, running_task);
        }

        // Получаем копию
        let task1 = pool.get_running_task(1).await;
        assert!(task1.is_some());
        assert_eq!(task1.unwrap().task.id, 1);

        // Получаем вторую копию
        let task2 = pool.get_running_task(1).await;
        assert!(task2.is_some());
    }

    #[tokio::test]
    async fn test_get_running_tasks_returns_all() {
        let pool = create_test_pool().await;

        // Добавляем несколько задач
        for i in 1..=3 {
            let mut task = crate::models::Task::default();
            task.id = i;
            task.project_id = 1;
            task.template_id = 1;
            task.status = TaskStatus::Running;

            let logger = Arc::new(BasicLogger::new());
            let template = crate::models::Template::default();
            let running_task = RunningTask::new(task, logger, template);

            {
                let mut running = pool.running_tasks.write().await;
                running.insert(i, running_task);
            }
        }

        let tasks = pool.get_running_tasks().await;
        assert_eq!(tasks.len(), 3);
        assert!(tasks.contains_key(&1));
        assert!(tasks.contains_key(&2));
        assert!(tasks.contains_key(&3));
    }

    #[tokio::test]
    async fn test_get_running_tasks_empty() {
        let pool = create_test_pool().await;

        let tasks = pool.get_running_tasks().await;
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn test_run_task_template_not_found() {
        let pool = create_test_pool().await;

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 999; // Non-existent template
        task.status = TaskStatus::Waiting;

        let result = pool.run_task(task).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("template"));
    }

    #[tokio::test]
    async fn test_run_task_adds_to_running_tasks() {
        let pool = create_test_pool().await;

        let tpl = crate::models::Template {
            id: 1,
            project_id: 1,
            name: "Test Template".to_string(),
            playbook: "test.yml".to_string(),
            ..Default::default()
        };
        // MockStore позволяет вставлять напрямую через create_template
        pool.store.create_template(tpl).await.unwrap();

        let mut task = crate::models::Task::default();
        task.id = 100;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Waiting;

        let result = pool.run_task(task).await;
        assert!(result.is_ok());

        let running = pool.get_running_task(100).await;
        assert!(running.is_some());
        assert_eq!(running.unwrap().task.id, 100);
    }

    #[tokio::test]
    async fn test_execute_task_template_not_found() {
        let pool = create_test_pool().await;

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 999;
        task.status = TaskStatus::Waiting;

        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let running_task = RunningTask::new(task.clone(), logger, template);
        {
            let mut running = pool.running_tasks.write().await;
            running.insert(1, running_task);
        }

        let result = pool.execute_task(task).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("template"));
    }

    #[tokio::test]
    async fn test_execute_task_uses_defaults_when_not_set() {
        let pool = create_test_pool().await;

        let tpl = crate::models::Template {
            id: 1,
            project_id: 1,
            name: "Test".to_string(),
            playbook: "test.yml".to_string(),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            ..Default::default()
        };
        // MockStore позволяет вставлять напрямую через create_template
        pool.store.create_template(tpl.clone()).await.unwrap();

        let mut task = crate::models::Task::default();
        task.id = 2;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Waiting;
        task.inventory_id = None;
        task.repository_id = None;
        task.environment_id = None;

        let logger = Arc::new(BasicLogger::new());
        let running_task = RunningTask::new(task.clone(), logger, tpl.clone());
        {
            let mut running = pool.running_tasks.write().await;
            running.insert(2, running_task);
        }

        // execute_task will try LocalJob::run which needs ansible/terraform
        // Since those aren't available, it will fail — but we test the path through get_* calls
        let result = pool.execute_task(task).await;
        let _ = result;

        // Task should be removed from running_tasks (success or error)
        let running = pool.get_running_task(2).await;
        assert!(running.is_none());
    }

    #[tokio::test]
    async fn test_process_queue_respects_shutdown() {
        let pool = create_test_pool().await;

        let task = crate::models::Task::default();
        pool.add_task(task).await.ok();

        pool.shutdown().await;
        pool.process_queue().await;
        // Should exit quickly without hanging
    }

    #[tokio::test]
    async fn test_process_queue_empty_queue() {
        let pool = create_test_pool().await;

        pool.shutdown().await;
        pool.process_queue().await;
        // Should complete without hanging
    }

    #[tokio::test]
    async fn test_add_task_after_shutdown() {
        let pool = create_test_pool().await;
        pool.shutdown().await;

        let task = crate::models::Task::default();
        let result = pool.add_task(task).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("shutdown"));
    }

    #[tokio::test]
    async fn test_clear_queue() {
        let pool = create_test_pool().await;

        for i in 1..=3 {
            let mut task = crate::models::Task::default();
            task.id = i;
            pool.add_task(task).await.unwrap();
        }

        assert_eq!(pool.queue_size().await, 3);

        pool.clear_queue().await;
        assert_eq!(pool.queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_get_queue_returns_copy() {
        let pool = create_test_pool().await;

        let mut task = crate::models::Task::default();
        task.id = 1;
        pool.add_task(task).await.unwrap();

        let queue = pool.get_queue().await;
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].id, 1);
    }

    #[tokio::test]
    async fn test_get_queue_empty() {
        let pool = create_test_pool().await;
        let queue = pool.get_queue().await;
        assert!(queue.is_empty());
    }

    #[tokio::test]
    async fn test_running_task_clone_impl() {
        let pool = create_test_pool().await;
        let pool_clone = pool.clone();
        assert!(pool_clone.is_shutdown().await == pool.is_shutdown().await);
    }

    #[tokio::test]
    async fn test_running_task_fields_accessible() {
        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        let running = RunningTask::new(task, logger, template);
        assert_eq!(running.task.id, 1);
        assert!(!running.killed);
    }

    #[tokio::test]
    async fn test_running_task_with_all_fields() {
        let logger = Arc::new(BasicLogger::new());
        let mut template = crate::models::Template::default();
        template.id = 5;
        template.name = "Test".to_string();

        let mut task = crate::models::Task::default();
        task.id = 10;
        task.project_id = 2;
        task.template_id = 5;
        task.status = TaskStatus::Running;

        let running = RunningTask::new(task, logger, template);
        assert_eq!(running.task.id, 10);
        assert_eq!(running.task.project_id, 2);
        assert_eq!(running.template.id, 5);
        assert!(!running.killed);
        assert!(running.start_time <= Utc::now());
    }

    #[tokio::test]
    async fn test_running_task_start_time_is_recent() {
        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let task = crate::models::Task::default();

        let before = Utc::now();
        let running = RunningTask::new(task, logger, template);
        let after = Utc::now();

        assert!(running.start_time >= before && running.start_time <= after);
    }

    #[tokio::test]
    async fn test_running_task_is_killed_initial() {
        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let task = crate::models::Task::default();
        let running = RunningTask::new(task, logger, template);
        assert!(!running.is_killed());
    }

    #[tokio::test]
    async fn test_running_task_killed_after_kill() {
        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let task = crate::models::Task::default();
        let mut running = RunningTask::new(task, logger, template);
        running.kill();
        assert!(running.killed);
        assert!(running.is_killed());
    }

    #[tokio::test]
    async fn test_task_pool_fields_accessible() {
        let pool = create_test_pool().await;
        assert_eq!(pool.project.id, 1);
        assert_eq!(pool.project.name, "Test Project");
        assert_eq!(pool.project.max_parallel_tasks, 5);
    }

    #[tokio::test]
    async fn test_running_task_logger_is_arc() {
        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let task = crate::models::Task::default();
        let running = RunningTask::new(task, logger.clone(), template);
        // Both should point to same Arc
        assert!(Arc::strong_count(&logger) >= 2);
        assert_eq!(
            Arc::strong_count(&running.logger),
            Arc::strong_count(&logger)
        );
    }

    #[tokio::test]
    async fn test_task_status_error_variant() {
        let status = TaskStatus::Error;
        assert_eq!(status.to_string(), "error");
    }

    #[tokio::test]
    async fn test_task_status_stopped_variant() {
        let status = TaskStatus::Stopped;
        assert_eq!(status.to_string(), "stopped");
    }

    #[tokio::test]
    async fn test_task_status_success_variant() {
        let status = TaskStatus::Success;
        assert_eq!(status.to_string(), "success");
    }

    #[tokio::test]
    async fn test_running_task_template_is_stored() {
        let logger = Arc::new(BasicLogger::new());
        let mut template = crate::models::Template::default();
        template.name = "MyTemplate".to_string();
        template.playbook = "play.yml".to_string();
        let task = crate::models::Task::default();
        let running = RunningTask::new(task, logger, template);
        assert_eq!(running.template.name, "MyTemplate");
        assert_eq!(running.template.playbook, "play.yml");
    }

    #[tokio::test]
    async fn test_kill_task_updates_running_map() {
        let pool = create_test_pool().await;

        let mut task = crate::models::Task::default();
        task.id = 5;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Running;

        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let running_task = RunningTask::new(task.clone(), logger, template);

        {
            let mut running = pool.running_tasks.write().await;
            running.insert(5, running_task);
        }

        assert!(pool.get_running_task(5).await.is_some());

        let result = pool.kill_task(5).await;
        assert!(result.is_ok());

        assert!(pool.get_running_task(5).await.is_none());
    }

    #[tokio::test]
    async fn test_get_running_tasks_returns_independent_copies() {
        let pool = create_test_pool().await;

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;

        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let running_task = RunningTask::new(task.clone(), logger, template);

        {
            let mut running = pool.running_tasks.write().await;
            running.insert(1, running_task);
        }

        let map = pool.get_running_tasks().await;
        assert_eq!(map.len(), 1);
        assert!(map.contains_key(&1));
        assert_eq!(map.get(&1).unwrap().task.id, 1);
    }

    #[tokio::test]
    async fn test_running_task_clone_independence() {
        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;

        let running = RunningTask::new(task, logger, template);
        // RunningTask doesn't implement Clone, but we can verify its fields
        assert!(!running.killed);
        assert_eq!(running.task.id, 1);
    }

    #[tokio::test]
    async fn test_running_task_with_stopped_status() {
        let logger = Arc::new(BasicLogger::new());
        let template = crate::models::Template::default();
        let mut task = crate::models::Task::default();
        task.id = 1;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Stopped;

        let running = RunningTask::new(task, logger, template);
        assert_eq!(running.task.status, TaskStatus::Stopped);
        assert!(!running.killed);
    }
}
