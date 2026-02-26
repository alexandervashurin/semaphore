//! TaskRunner Lifecycle - жизненный цикл задачи
//!
//! Аналог services/tasks/task_runner_lifecycle.go из Go версии

use std::sync::Arc;
use tracing::{info, error};
use crate::error::Result;
use crate::services::task_runner::TaskRunner;
use crate::services::local_job::LocalJob;
use crate::services::task_logger::TaskLogger;
use crate::db_lib::AccessKeyInstallerImpl;

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
        
        self.job = Some(Box::new(LocalJob::new(
            self.task.clone(),
            self.template.clone(),
            self.inventory.clone(),
            self.repository.clone(),
            self.environment.clone(),
            logger,
            self.key_installer.clone(),
            std::path::PathBuf::from("/tmp/work"),
            std::path::PathBuf::from("/tmp/tmp"),
        )));
        
        // Запуск задачи
        if let Some(ref mut job) = self.job {
            if let Err(e) = job.run(&self.username, self.incoming_version.as_deref(), self.alias.as_deref()).await {
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
        
        let obj_type = EventType::Task;
        let desc = format!(
            "Task {} ({}) finished - {}",
            self.task.id,
            self.template.name,
            self.task.status.to_string().to_uppercase()
        );
        
        match self.pool.store.create_event(Event {
            id: 0,
            object_type: obj_type,
            object_id: self.task.id,
            project_id: self.task.project_id,
            description: desc,
            user_id: None,
            created: chrono::Utc::now(),
        }).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to create task event: {}", e);
                Err(e)
            }
        }
    }

    /// log записывает лог задачи
    pub fn log(&self, msg: &str) {
        info!("[Task {}] {}", self.task.id, msg);
        
        // TODO: Запись в БД
        // self.pool.store.create_task_output(...).await?;
        
        // Уведомление слушателей логов
        let now = chrono::Utc::now();
        for listener in &self.log_listeners {
            listener(now, msg.to_string());
        }
    }

    /// set_status устанавливает статус задачи
    pub async fn set_status(&mut self, status: crate::models::TaskStatus) {
        self.task.status = status;
        self.save_status().await;
    }

    /// save_status сохраняет статус задачи и уведомляет пользователей
    async fn save_status(&self) {
        // Уведомление пользователей через WebSocket
        for &user_id in &self.users {
            // TODO: Отправка WebSocket уведомления
            // sockets.Message(user_id, ...);
        }
        
        // Уведомление слушателей статусов
        for listener in &self.status_listeners {
            listener(self.task.status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::models::{Task, TaskStatus};

    fn create_test_task_runner() -> TaskRunner {
        use crate::services::task_pool::TaskPool;
        use crate::db::MemoryDB;
        
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
            crate::models::Project::default(),
            AccessKeyInstallerImpl::new(),
            Arc::new(MemoryDB::new()),
        ));
        
        TaskRunner::new(task, pool, "testuser".to_string(), AccessKeyInstallerImpl::new())
    }

    #[tokio::test]
    async fn test_task_runner_log() {
        let mut runner = create_test_task_runner();
        runner.log("Test message");
        // Просто проверяем, что метод вызывается без паники
    }

    #[tokio::test]
    async fn test_task_runner_set_status() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Running).await;
        assert_eq!(runner.task.status, TaskStatus::Running);
    }
}
