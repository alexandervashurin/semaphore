//! TaskRunner Logging - логирование и статусы
//!
//! Аналог services/tasks/task_runner_logging.go из Go версии

use crate::error::Result;
use crate::services::task_logger::TaskStatus;
use crate::services::task_runner::TaskRunner;
use chrono::Utc;
use std::sync::Arc;

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

        // Отправка статуса через WebSocket (broadcast всем подписчикам)
        self.pool
            .ws_manager
            .send_status(self.task.id, self.task.status.to_string(), Utc::now());

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
            project_id: self.task.project_id,
            output: msg.to_string(),
            time: Utc::now(),
            stage_id: None,
        };

        // Отправка лога через WebSocket
        let now = Utc::now();
        self.pool
            .ws_manager
            .send_log(self.task.id, msg.to_string(), now);

        // Сохранение в БД — fire-and-forget через spawn
        let store = Arc::clone(&self.pool.store);
        let output = task_output;
        tokio::spawn(async move {
            use crate::db::store::TaskManager;
            let _ = store.create_task_output(output).await;
        });

        // Уведомление слушателей логов
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
    async fn test_get_status() {
        let runner = create_test_task_runner();
        assert_eq!(runner.get_status(), TaskStatus::Waiting);
    }

    #[tokio::test]
    async fn test_set_status() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Running).await;
        assert_eq!(runner.get_status(), TaskStatus::Running);
    }

    #[tokio::test]
    async fn test_log() {
        let runner = create_test_task_runner();
        runner.log("Test log message");
    }

    #[tokio::test]
    async fn test_notify_status_change() {
        let runner = create_test_task_runner();
        runner.notify_status_change(TaskStatus::Success).await;
        // Проверяем, что метод вызывается без паники
    }

    #[tokio::test]
    async fn test_save_status_sends_ws_update() {
        let runner = create_test_task_runner();
        // save_status должен завершаться без паники
        runner.save_status().await;
    }

    #[tokio::test]
    async fn test_set_status_triggers_save() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Success).await;
        // set_status вызывает save_status внутри
        assert_eq!(runner.get_status(), TaskStatus::Success);
    }

    #[tokio::test]
    async fn test_log_listener_receives_message() {
        let runner = create_test_task_runner();
        let (tx, rx) = std::sync::mpsc::channel();
        let runner_with_listener = TaskRunner {
            log_listeners: vec![Box::new(move |time, msg| {
                let _ = tx.send((time, msg));
            })],
            ..runner
        };
        runner_with_listener.log("hello listener");
        let (time, msg) = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(msg, "hello listener");
        assert!(time <= Utc::now());
    }

    #[tokio::test]
    async fn test_status_listener_receives_update() {
        let runner = create_test_task_runner();
        let (tx, rx) = std::sync::mpsc::channel();
        let mut runner_with_listener = TaskRunner {
            status_listeners: vec![Box::new(move |status| {
                let _ = tx.send(status);
            })],
            ..runner
        };
        runner_with_listener.set_status(TaskStatus::Error).await;
        let status = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(status, TaskStatus::Error);
    }

    #[tokio::test]
    async fn test_get_status_initial_is_waiting() {
        let runner = create_test_task_runner();
        assert_eq!(runner.get_status(), TaskStatus::Waiting);
    }

    #[tokio::test]
    async fn test_set_status_to_running() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Running).await;
        assert_eq!(runner.get_status(), TaskStatus::Running);
    }

    #[tokio::test]
    async fn test_set_status_to_success() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Success).await;
        assert_eq!(runner.get_status(), TaskStatus::Success);
    }

    #[tokio::test]
    async fn test_set_status_to_stopped() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Stopped).await;
        assert_eq!(runner.get_status(), TaskStatus::Stopped);
    }

    #[tokio::test]
    async fn test_set_status_to_error() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Error).await;
        assert_eq!(runner.get_status(), TaskStatus::Error);
    }

    #[tokio::test]
    async fn test_log_does_not_panic() {
        let runner = create_test_task_runner();
        runner.log("Simple log message");
    }

    #[tokio::test]
    async fn test_log_empty_string() {
        let runner = create_test_task_runner();
        runner.log("");
    }

    #[tokio::test]
    async fn test_log_with_newlines() {
        let runner = create_test_task_runner();
        runner.log("Line 1\nLine 2\nLine 3");
    }

    #[tokio::test]
    async fn test_log_with_unicode() {
        let runner = create_test_task_runner();
        runner.log("Привет мир! Task executed");
    }

    #[tokio::test]
    async fn test_save_status_completes_without_panic() {
        let runner = create_test_task_runner();
        runner.save_status().await;
    }

    #[tokio::test]
    async fn test_set_status_multiple_times() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Running).await;
        assert_eq!(runner.get_status(), TaskStatus::Running);

        runner.set_status(TaskStatus::Success).await;
        assert_eq!(runner.get_status(), TaskStatus::Success);

        runner.set_status(TaskStatus::Error).await;
        assert_eq!(runner.get_status(), TaskStatus::Error);
    }

    #[tokio::test]
    async fn test_log_listener_receives_multiple_messages() {
        let runner = create_test_task_runner();
        let (tx, rx) = std::sync::mpsc::channel();
        let runner_with_listener = TaskRunner {
            log_listeners: vec![Box::new(move |_time, msg| {
                let _ = tx.send(msg);
            })],
            ..runner
        };
        runner_with_listener.log("msg1");
        runner_with_listener.log("msg2");
        runner_with_listener.log("msg3");

        assert_eq!(rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap(), "msg1");
        assert_eq!(rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap(), "msg2");
        assert_eq!(rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap(), "msg3");
    }

    #[tokio::test]
    async fn test_status_listener_receives_all_status_types() {
        let runner = create_test_task_runner();
        let (tx, rx) = std::sync::mpsc::channel();
        let mut runner_with_listener = TaskRunner {
            status_listeners: vec![Box::new(move |status| {
                let _ = tx.send(status);
            })],
            ..runner
        };

        runner_with_listener.set_status(TaskStatus::Running).await;
        assert_eq!(rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap(), TaskStatus::Running);

        runner_with_listener.set_status(TaskStatus::Success).await;
        assert_eq!(rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap(), TaskStatus::Success);
    }

    #[tokio::test]
    async fn test_log_with_special_json_chars() {
        let runner = create_test_task_runner();
        runner.log("{\"key\": \"value\", \"count\": 42}");
    }

    #[tokio::test]
    async fn test_save_status_with_different_task_statuses() {
        let mut runner = create_test_task_runner();
        runner.set_status(TaskStatus::Running).await;
        runner.save_status().await; // should not panic

        runner.set_status(TaskStatus::Success).await;
        runner.save_status().await;
    }

    #[tokio::test]
    async fn test_log_listener_with_empty_message() {
        let runner = create_test_task_runner();
        let (tx, rx) = std::sync::mpsc::channel();
        let runner_with_listener = TaskRunner {
            log_listeners: vec![Box::new(move |time, msg| {
                let _ = tx.send((time, msg));
            })],
            ..runner
        };
        runner_with_listener.log("");
        let (time, msg) = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(msg, "");
        assert!(time <= Utc::now());
    }
}
