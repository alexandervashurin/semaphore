//! TaskRunner WebSocket - WebSocket уведомления
//!
//! Аналог services/tasks/task_runner_websocket.go из Go версии

use serde_json::json;
use chrono::Utc;
use crate::services::task_runner::TaskRunner;
use crate::services::task_logger::TaskStatus;

impl TaskRunner {
    /// send_websocket_update отправляет обновление статуса через WebSocket
    pub async fn send_websocket_update(&self) {
        let message = json!({
            "type": "update",
            "status": self.task.status.to_string(),
            "task_id": self.task.id,
            "template_id": self.task.template_id,
            "project_id": self.task.project_id,
        });

        // Отправка всем пользователям
        for &user_id in &self.users {
            // TODO: Интеграция с WebSocketManager
            // self.pool.ws_manager.send(user_id, message.clone()).await;
        }
    }

    /// notify_status_change уведомляет об изменении статуса
    pub async fn notify_status_change(&self, status: TaskStatus) {
        self.send_websocket_update().await;
        
        // Уведомление слушателей статусов
        for listener in &self.status_listeners {
            listener(status);
        }
    }

    /// notify_log уведомляет о новом логе
    pub fn notify_log(&self, time: chrono::DateTime<Utc>, msg: &str) {
        for listener in &self.log_listeners {
            listener(time, msg.to_string());
        }
    }

    /// broadcast_update отправляет обновление всем подключенным клиентам
    pub async fn broadcast_update(&self, event_type: &str, data: serde_json::Value) {
        let message = json!({
            "type": event_type,
            "task_id": self.task.id,
            "data": data,
        });

        for &user_id in &self.users {
            // TODO: Интеграция с WebSocketManager
            // self.pool.ws_manager.send(user_id, message.clone()).await;
        }
    }

    /// send_task_started уведомляет о старте задачи
    pub async fn send_task_started(&self) {
        let data = json!({
            "status": "starting",
            "start_time": self.task.created,
        });
        self.broadcast_update("task_started", data).await;
    }

    /// send_task_completed уведомляет о завершении задачи
    pub async fn send_task_completed(&self) {
        let data = json!({
            "status": self.task.status.to_string(),
            "end_time": self.task.end,
        });
        self.broadcast_update("task_completed", data).await;
    }

    /// send_task_failed уведомляет об ошибке задачи
    pub async fn send_task_failed(&self, error: &str) {
        let data = json!({
            "status": "error",
            "error": error,
        });
        self.broadcast_update("task_failed", data).await;
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
    async fn test_send_websocket_update() {
        let runner = create_test_task_runner();
        runner.send_websocket_update().await;
        // Просто проверяем, что метод вызывается без паники
    }

    #[tokio::test]
    async fn test_send_task_started() {
        let runner = create_test_task_runner();
        runner.send_task_started().await;
        // Просто проверяем, что метод вызывается без паники
    }

    #[tokio::test]
    async fn test_send_task_completed() {
        let runner = create_test_task_runner();
        runner.send_task_completed().await;
        // Просто проверяем, что метод вызывается без паники
    }
}
