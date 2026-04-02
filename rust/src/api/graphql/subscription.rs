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
static TASK_CREATED_TX: Lazy<broadcast::Sender<Task>> =
    Lazy::new(|| broadcast::channel(256).0);

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
