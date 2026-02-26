//! TaskRunner Logging - логирование и статусы
//!
//! Аналог services/tasks/task_runner_logging.go из Go версии

use chrono::Utc;
use crate::error::Result;
use crate::services::task_runner::TaskRunner;
use crate::models::TaskStatus;

impl TaskRunner {
    /// save_status сохраняет статус задачи и уведомляет пользователей
    pub async fn save_status(&self) {
        use serde_json::json;
        
        // Формирование сообщения для WebSocket
        let message = json!({
            "type": "update",
            "start": self.task.created,
            "end": self.task.end,
            "status": self.task.status.to_string(),
            "task_id": self.task.id,
            "template_id": self.task.template_id,
            "project_id": self.task.project_id,
            "version": self.task.version,
        });

        // Отправка через WebSocket
        for &user_id in &self.users {
            // TODO: Интеграция с WebSocketManager
            // self.pool.ws_manager.send(user_id, message.clone()).await;
        }
        
        // Уведомление слушателей статусов
        for listener in &self.status_listeners {
            listener(self.task.status);
        }
    }

    /// log записывает лог задачи
    pub fn log(&self, msg: &str) {
        use tracing::info;
        
        info!("[Task {}] {}", self.task.id, msg);
        
        // Запись в БД
        let task_output = crate::models::TaskOutput {
            id: 0,
            task_id: self.task.id,
            output: msg.to_string(),
            time: Utc::now(),
        };
        
        // TODO: Асинхронная запись в БД
        // let _ = self.pool.store.create_task_output(task_output).await;
        
        // Уведомление слушателей логов
        let now = Utc::now();
        for listener in &self.log_listeners {
            listener(now, msg.to_string());
        }
    }

    /// set_status устанавливает статус задачи
    pub async fn set_status(&mut self, status: TaskStatus) {
        self.task.status = status;
        self.save_status().await;
    }

    /// get_status возвращает текущий статус
    pub fn get_status(&self) -> TaskStatus {
        self.task.status
    }

    /// notify_status_change уведомляет об изменении статуса
    pub async fn notify_status_change(&self, status: TaskStatus) {
        self.save_status().await;
        
        for listener in &self.status_listeners {
            listener(status);
        }
    }

    /// notify_log уведомляет о новом логе
    pub fn notify_log(&self, msg: &str) {
        let now = Utc::now();
        for listener in &self.log_listeners {
            listener(now, msg.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::models::{Task, Project};
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
            ..Default::default()
        };
        
        let pool = Arc::new(TaskPool::new(
            Project::default(),
            AccessKeyInstallerImpl::new(),
            Arc::new(MemoryDB::new()),
        ));
        
        TaskRunner::new(task, pool, "testuser".to_string(), AccessKeyInstallerImpl::new())
    }

    #[test]
    fn test_get_status() {
        let runner = create_test_task_runner();
        assert_eq!(runner.get_status(), TaskStatus::Waiting);
    }

    #[tokio::test]
    async fn test_set_status() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Running).await;
        assert_eq!(runner.get_status(), TaskStatus::Running);
    }

    #[test]
    fn test_log() {
        let runner = create_test_task_runner();
        runner.log("Test log message");
        // Просто проверяем, что метод вызывается без паники
    }

    #[tokio::test]
    async fn test_notify_status_change() {
        let runner = create_test_task_runner();
        runner.notify_status_change(TaskStatus::Success).await;
        // Проверяем, что метод вызывается без паники
    }
}
