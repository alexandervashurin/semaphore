//! TaskRunner WebSocket - WebSocket уведомления
//!
//! Аналог services/tasks/task_runner_websocket.go из Go версии

use crate::api::websocket::{WebSocketManager, WsMessage};
use crate::services::task_logger::TaskStatus;
use crate::services::task_runner::TaskRunner;
use chrono::Utc;
use serde_json::json;

impl TaskRunner {
    /// send_websocket_update отправляет обновление статуса через WebSocket
    pub async fn send_websocket_update(&self) {
        self.pool
            .ws_manager
            .send_status(self.task.id, self.task.status.to_string(), Utc::now());
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
        // Отправка лога через WebSocketManager
        self.pool
            .ws_manager
            .send_log(self.task.id, msg.to_string(), time);

        for listener in &self.log_listeners {
            listener(time, msg.to_string());
        }
    }

    /// broadcast_update отправляет обновление всем подключенным клиентам
    pub async fn broadcast_update(&self, event_type: &str, data: serde_json::Value) {
        let status = format!("{}: {}", event_type, data);
        self.pool
            .ws_manager
            .send_status(self.task.id, status, Utc::now());
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
    use crate::db::MockStore;
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::models::{Project, Task};
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

    #[tokio::test]
    async fn test_send_task_failed() {
        let runner = create_test_task_runner();
        runner.send_task_failed("Test error message").await;
        // Просто проверяем, что метод вызывается без паники
    }

    #[tokio::test]
    async fn test_notify_status_change_with_listeners() {
        let runner = create_test_task_runner();
        let (tx, rx) = std::sync::mpsc::channel();
        let runner_with_listener = TaskRunner {
            status_listeners: vec![Box::new(move |status| {
                let _ = tx.send(status);
            })],
            ..runner
        };
        runner_with_listener.notify_status_change(TaskStatus::Running).await;
        let status = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(status, TaskStatus::Running);
    }

    #[tokio::test]
    async fn test_notify_log_with_listeners() {
        let runner = create_test_task_runner();
        let (tx, rx) = std::sync::mpsc::channel();
        let runner_with_listener = TaskRunner {
            log_listeners: vec![Box::new(move |time, msg| {
                let _ = tx.send((time, msg));
            })],
            ..runner
        };
        let now = Utc::now();
        runner_with_listener.notify_log(now, "Test log message");
        let (time, msg) = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(msg, "Test log message");
        assert!(time <= Utc::now());
    }

    #[tokio::test]
    async fn test_broadcast_update() {
        let runner = create_test_task_runner();
        let data = json!({"key": "value"});
        runner.broadcast_update("custom_event", data).await;
        // Просто проверяем, что метод вызывается без паники
    }

    #[tokio::test]
    async fn test_notify_status_change_no_listeners() {
        let runner = create_test_task_runner();
        // Без слушателей — не должно паниковать
        runner.notify_status_change(TaskStatus::Success).await;
    }

    #[tokio::test]
    async fn test_notify_log_no_listeners() {
        let runner = create_test_task_runner();
        // Без слушателей — не должно паниковать
        runner.notify_log(Utc::now(), "Test message");
    }

    #[tokio::test]
    async fn test_send_websocket_update_completes() {
        let runner = create_test_task_runner();
        runner.send_websocket_update().await;
    }

    #[tokio::test]
    async fn test_send_task_started_completes() {
        let runner = create_test_task_runner();
        runner.send_task_started().await;
    }

    #[tokio::test]
    async fn test_send_task_completed_completes() {
        let runner = create_test_task_runner();
        runner.send_task_completed().await;
    }

    #[tokio::test]
    async fn test_send_task_failed_with_empty_error() {
        let runner = create_test_task_runner();
        runner.send_task_failed("").await;
    }

    #[tokio::test]
    async fn test_send_task_failed_with_long_error() {
        let runner = create_test_task_runner();
        let long_err = "x".repeat(1000);
        runner.send_task_failed(&long_err).await;
    }

    #[tokio::test]
    async fn test_notify_status_change_with_all_statuses() {
        let runner = create_test_task_runner();

        for status in [TaskStatus::Waiting, TaskStatus::Running, TaskStatus::Success, TaskStatus::Error, TaskStatus::Stopped] {
            runner.notify_status_change(status).await;
        }
    }

    #[tokio::test]
    async fn test_notify_log_with_unicode() {
        let runner = create_test_task_runner();
        runner.notify_log(Utc::now(), "Привет мир");
    }

    #[tokio::test]
    async fn test_broadcast_update_custom_event() {
        let runner = create_test_task_runner();
        let data = serde_json::json!({"custom": "data"});
        runner.broadcast_update("custom_event", data).await;
    }

    #[tokio::test]
    async fn test_broadcast_update_empty_data() {
        let runner = create_test_task_runner();
        let data = serde_json::json!({});
        runner.broadcast_update("empty", data).await;
    }

    #[tokio::test]
    async fn test_notify_status_change_with_listener_receives_correct() {
        let runner = create_test_task_runner();
        let (tx, rx) = std::sync::mpsc::channel();
        let runner_with_listener = TaskRunner {
            status_listeners: vec![Box::new(move |status| {
                let _ = tx.send(status);
            })],
            ..runner
        };
        runner_with_listener.notify_status_change(TaskStatus::Running).await;
        let received = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(received, TaskStatus::Running);
    }

    #[tokio::test]
    async fn test_notify_log_with_listener_receives_correct() {
        let runner = create_test_task_runner();
        let (tx, rx) = std::sync::mpsc::channel();
        let runner_with_listener = TaskRunner {
            log_listeners: vec![Box::new(move |time, msg| {
                let _ = tx.send((time, msg));
            })],
            ..runner
        };
        let now = Utc::now();
        runner_with_listener.notify_log(now, "log data");
        let (time, msg) = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(msg, "log data");
        assert!(time <= Utc::now());
    }

    #[tokio::test]
    async fn test_notify_log_with_empty_message() {
        let runner = create_test_task_runner();
        runner.notify_log(Utc::now(), "");
    }

    #[tokio::test]
    async fn test_send_task_completed_with_end_time_set() {
        let runner = create_test_task_runner();
        runner.send_task_completed().await;
    }

    #[tokio::test]
    async fn test_send_websocket_update_multiple_times() {
        let runner = create_test_task_runner();
        for _ in 0..5 {
            runner.send_websocket_update().await;
        }
    }

    #[tokio::test]
    async fn test_notify_status_change_no_listeners_all_variants() {
        let runner = create_test_task_runner();
        // Without listeners, all status changes should complete without panic
        runner.notify_status_change(TaskStatus::Waiting).await;
        runner.notify_status_change(TaskStatus::Running).await;
        runner.notify_status_change(TaskStatus::Success).await;
        runner.notify_status_change(TaskStatus::Error).await;
        runner.notify_status_change(TaskStatus::Stopped).await;
    }

    #[tokio::test]
    async fn test_notify_log_no_listeners_multiple_calls() {
        let runner = create_test_task_runner();
        for i in 0..5 {
            runner.notify_log(Utc::now(), &format!("msg {}", i));
        }
    }

    #[tokio::test]
    async fn test_broadcast_update_with_complex_data() {
        let runner = create_test_task_runner();
        let data = serde_json::json!({
            "status": "running",
            "progress": 50,
            "steps": ["step1", "step2"],
        });
        runner.broadcast_update("progress", data).await;
    }
}
