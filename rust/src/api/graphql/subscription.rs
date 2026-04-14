//! GraphQL Subscription корень — real-time события задач
//!
//! ## Подписки
//! - `taskCreated` — новая задача создана в любом проекте
//! - `taskStatus(projectId?)` — изменение статуса задачи (фильтр по project_id опционален)
//! - `taskOutput(taskId)` — строки лога выполняющейся задачи
//!
//! ## Публикация из других модулей
//! ```rust,ignore
//! use crate::api::graphql::subscription;
//! subscription::publish_task_created(task);
//! subscription::publish_task_status(event);
//! subscription::publish_task_output(line);
//! ```

use async_graphql::{Context, Result, Subscription};
use async_stream::stream;
use futures_util::Stream;
use once_cell::sync::Lazy;
use tokio::sync::broadcast;

use super::types::{Task, TaskOutputLine, TaskStatusEvent};

// ── Broadcast channels ────────────────────────────────────────────────────────

/// Канал создания задач
static TASK_CREATED_TX: Lazy<broadcast::Sender<Task>> = Lazy::new(|| broadcast::channel(256).0);

/// Канал изменений статуса задач
static TASK_STATUS_TX: Lazy<broadcast::Sender<TaskStatusEvent>> =
    Lazy::new(|| broadcast::channel(256).0);

/// Канал строк вывода задач
static TASK_OUTPUT_TX: Lazy<broadcast::Sender<TaskOutputLine>> =
    Lazy::new(|| broadcast::channel(1024).0);

// ── Public publish helpers ────────────────────────────────────────────────────

/// Публикует событие создания задачи
pub fn publish_task_created(task: Task) {
    let _ = TASK_CREATED_TX.send(task);
}

/// Публикует событие изменения статуса задачи
pub fn publish_task_status(event: TaskStatusEvent) {
    let _ = TASK_STATUS_TX.send(event);
}

/// Публикует строку вывода задачи
pub fn publish_task_output(line: TaskOutputLine) {
    let _ = TASK_OUTPUT_TX.send(line);
}

// ── SubscriptionRoot ──────────────────────────────────────────────────────────

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Подписка на создание задач в любом проекте.
    ///
    /// ```graphql
    /// subscription {
    ///   taskCreated { id projectId status }
    /// }
    /// ```
    async fn task_created(&self, _ctx: &Context<'_>) -> Result<impl Stream<Item = Task>> {
        let rx = TASK_CREATED_TX.subscribe();
        Ok(recv_broadcast(rx))
    }

    /// Подписка на изменения статуса задач.
    ///
    /// Если `project_id` передан — фильтрует только события данного проекта.
    ///
    /// ```graphql
    /// subscription {
    ///   taskStatus(projectId: 1) { taskId projectId status updatedAt }
    /// }
    /// ```
    async fn task_status(
        &self,
        _ctx: &Context<'_>,
        project_id: Option<i32>,
    ) -> Result<impl Stream<Item = TaskStatusEvent>> {
        let rx = TASK_STATUS_TX.subscribe();
        Ok(stream! {
            let mut rx = rx;
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if project_id.is_none() || project_id == Some(event.project_id) {
                            yield event;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("taskStatus subscription lagged by {} messages", n);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        })
    }

    /// Подписка на вывод (логи) выполняющейся задачи.
    ///
    /// ```graphql
    /// subscription {
    ///   taskOutput(taskId: 42) { taskId line timestamp level }
    /// }
    /// ```
    async fn task_output(
        &self,
        _ctx: &Context<'_>,
        task_id: i32,
    ) -> Result<impl Stream<Item = TaskOutputLine>> {
        let rx = TASK_OUTPUT_TX.subscribe();
        Ok(stream! {
            let mut rx = rx;
            loop {
                match rx.recv().await {
                    Ok(line) if line.task_id == task_id => yield line,
                    Ok(_) => continue,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("taskOutput subscription lagged by {} messages", n);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        })
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

/// Превращает broadcast receiver в Stream, пропуская lagged ошибки
fn recv_broadcast<T: Clone + Send + 'static>(
    mut rx: broadcast::Receiver<T>,
) -> impl Stream<Item = T> {
    stream! {
        loop {
            match rx.recv().await {
                Ok(item) => yield item,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("GraphQL subscription lagged by {} messages", n);
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::graphql::types::Task;
    use tokio::sync::broadcast;

    #[test]
    fn test_subscription_root_exists() {
        let root = SubscriptionRoot;
        drop(root);
    }

    #[test]
    fn test_task_output_line_fields() {
        let line = TaskOutputLine {
            task_id: 42,
            line: "TASK [deploy] ************************************".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            level: "info".to_string(),
        };
        assert_eq!(line.task_id, 42);
        assert_eq!(line.level, "info");
        assert!(line.line.contains("TASK [deploy]"));
    }

    #[test]
    fn test_task_output_line_all_levels() {
        let levels = vec!["info", "warning", "error", "debug"];
        for level in &levels {
            let line = TaskOutputLine {
                task_id: 1,
                line: "test".to_string(),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                level: level.to_string(),
            };
            assert_eq!(line.level, *level);
        }
    }

    #[test]
    fn test_task_status_event_fields() {
        let event = TaskStatusEvent {
            task_id: 100,
            project_id: 10,
            status: "running".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(event.task_id, 100);
        assert_eq!(event.project_id, 10);
        assert_eq!(event.status, "running");
    }

    #[test]
    fn test_task_status_event_field_values() {
        let event = TaskStatusEvent {
            task_id: 5,
            project_id: 20,
            status: "success".to_string(),
            updated_at: "2024-06-01T12:00:00Z".to_string(),
        };
        assert_eq!(event.task_id, 5);
        assert_eq!(event.project_id, 20);
        assert_eq!(event.status, "success");
    }

    #[test]
    fn test_task_output_line_field_values() {
        let line = TaskOutputLine {
            task_id: 1,
            line: "PLAY RECAP".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            level: "info".to_string(),
        };
        assert_eq!(line.task_id, 1);
        assert_eq!(line.line, "PLAY RECAP");
        assert_eq!(line.level, "info");
    }

    #[test]
    fn test_task_clone_trait() {
        let task = Task {
            id: 1,
            template_id: 2,
            project_id: 10,
            status: "waiting".to_string(),
            created: "2024-01-01T00:00:00Z".to_string(),
        };
        let cloned = task.clone();
        assert_eq!(cloned.id, task.id);
        assert_eq!(cloned.status, task.status);
    }

    #[test]
    fn test_task_output_line_clone() {
        let line = TaskOutputLine {
            task_id: 1,
            line: "output".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            level: "debug".to_string(),
        };
        let cloned = line.clone();
        assert_eq!(cloned.task_id, line.task_id);
        assert_eq!(cloned.level, line.level);
    }

    #[test]
    fn test_task_status_event_clone() {
        let event = TaskStatusEvent {
            task_id: 10,
            project_id: 5,
            status: "failed".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let cloned = event.clone();
        assert_eq!(cloned.task_id, event.task_id);
        assert_eq!(cloned.status, event.status);
    }

    #[test]
    fn test_broadcast_channel_creation() {
        // Test that broadcast channels can be created with expected capacity
        let (tx, _rx): (broadcast::Sender<Task>, broadcast::Receiver<Task>) =
            broadcast::channel(256);
        // Verify sender exists
        let _ = tx.send(Task {
            id: 1,
            template_id: 1,
            project_id: 1,
            status: "test".to_string(),
            created: "2024-01-01T00:00:00Z".to_string(),
        });
    }

    #[test]
    fn test_publish_task_created_does_not_panic() {
        let task = Task {
            id: 999,
            template_id: 1,
            project_id: 1,
            status: "waiting".to_string(),
            created: "2024-01-01T00:00:00Z".to_string(),
        };
        // Should not panic
        publish_task_created(task);
    }

    #[test]
    fn test_publish_task_status_does_not_panic() {
        let event = TaskStatusEvent {
            task_id: 999,
            project_id: 1,
            status: "running".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        publish_task_status(event);
    }

    #[test]
    fn test_publish_task_output_does_not_panic() {
        let line = TaskOutputLine {
            task_id: 999,
            line: "test output".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            level: "info".to_string(),
        };
        publish_task_output(line);
    }

    #[tokio::test]
    async fn test_broadcast_send_and_receive() {
        let (tx, mut rx) = broadcast::channel::<TaskStatusEvent>(16);
        let event = TaskStatusEvent {
            task_id: 1,
            project_id: 10,
            status: "running".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        tx.send(event.clone()).unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.task_id, 1);
        assert_eq!(received.status, "running");
    }
}
