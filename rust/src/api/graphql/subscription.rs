//! GraphQL Subscription корень — real-time события (v5.1)
//!
//! ## Подписки
//! - `taskCreated` — новая задача создана
//! - `taskStatus(projectId?)` — изменение статуса задачи
//! - `taskOutput(taskId)` — строки лога выполняющейся задачи
//!
//! ## Публикация (из других модулей)
//! ```rust,ignore
//! use crate::api::graphql::subscription;
//! subscription::publish_task_created(task);
//! subscription::publish_task_status(event);
//! subscription::publish_task_output(line);
//! ```

use async_graphql::{Context, Object, Result, Subscription};
use futures_util::stream::{self, Stream, StreamExt};
use once_cell::sync::Lazy;
use tokio::sync::broadcast;

use super::types::{Task, TaskOutputLine, TaskStatusEvent};

// ── Broadcast channels ───────────────────────────────────────────────────────

static TASK_CREATED_TX: Lazy<broadcast::Sender<Task>> =
    Lazy::new(|| broadcast::channel(256).0);

static TASK_STATUS_TX: Lazy<broadcast::Sender<TaskStatusEvent>> =
    Lazy::new(|| broadcast::channel(256).0);

static TASK_OUTPUT_TX: Lazy<broadcast::Sender<TaskOutputLine>> =
    Lazy::new(|| broadcast::channel(1024).0);

// ── Public publish helpers ───────────────────────────────────────────────────

pub fn publish_task_created(task: Task) {
    let _ = TASK_CREATED_TX.send(task);
}

pub fn publish_task_status(event: TaskStatusEvent) {
    let _ = TASK_STATUS_TX.send(event);
}

pub fn publish_task_output(line: TaskOutputLine) {
    let _ = TASK_OUTPUT_TX.send(line);
}

// ── SubscriptionRoot ─────────────────────────────────────────────────────────

pub struct SubscriptionRoot;

/// Канал для real-time событий задач
pub static TASK_CHANNEL: once_cell::sync::Lazy<broadcast::Sender<Task>> =
    once_cell::sync::Lazy::new(|| broadcast::channel(100).0);

#[Subscription]
impl SubscriptionRoot {
    /// Подписка на создание задач во всех проектах.
    async fn task_created(&self, _ctx: &Context<'_>) -> Result<impl Stream<Item = Task>> {
        let mut rx = TASK_CHANNEL.subscribe();

        Ok(stream::unfold(rx, move |mut rx| async move {
            loop {
                match rx.recv().await {
                    Ok(task) => {
                        return Some((task, rx));
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        return None;
                    }
                }
            }
        }))
    }

    /// Подписка на изменение статуса задачи
    async fn task_status_changed(&self, _ctx: &Context<'_>) -> Result<impl Stream<Item = Task>> {
        let mut rx = TASK_CHANNEL.subscribe();

        Ok(stream::unfold(rx, move |mut rx| async move {
            loop {
                match rx.recv().await {
                    Ok(task) => return Some((task, rx)),
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        }))
    }
}

// ── Helper ───────────────────────────────────────────────────────────────────

fn recv_broadcast<T: Clone + Send + 'static>(
    mut rx: broadcast::Receiver<T>,
) -> impl Stream<Item = T> {
    async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(item) => yield item,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("GraphQL subscription lagged {n}");
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }
}
