//! GraphQL Subscription корень — real-time события

use async_graphql::{Context, Object, Subscription, Result};
use futures_util::stream::{self, Stream, StreamExt};
use tokio::sync::broadcast;

use super::types::Task;

/// Корневой тип для Subscription
pub struct SubscriptionRoot;

/// Канал для real-time событий задач
pub static TASK_CHANNEL: once_cell::sync::Lazy<broadcast::Sender<Task>> = 
    once_cell::sync::Lazy::new(|| broadcast::channel(100).0);

#[Subscription]
impl SubscriptionRoot {
    /// Подписка на создание задач
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

/// Опубликовать событие о новой задаче
pub fn publish_task_created(task: Task) {
    let _ = TASK_CHANNEL.send(task);
}
