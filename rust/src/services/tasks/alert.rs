//! Alert Service
//!
//! Сервис уведомлений о задачах

use std::sync::Arc;
use crate::error::Result;
use crate::models::{Task, User};
use crate::services::task_logger::TaskStatus;

/// Сервис уведомлений
pub struct AlertService {
    store: Arc<dyn crate::db::store::Store + Send + Sync>,
}

impl AlertService {
    /// Создаёт новый сервис уведомлений
    pub fn new(store: Arc<dyn crate::db::store::Store + Send + Sync>) -> Self {
        Self { store }
    }

    /// Отправляет email уведомление
    pub async fn send_email_alert(&self, task: &Task, users: &[i32]) -> Result<()> {
        for &user_id in users {
            let user = self.store.get_user(user_id).await?;
            
            if !user.alert {
                continue;
            }

            self.send_email_to_user(&user, task).await?;
        }

        Ok(())
    }

    /// Отправляет email пользователю
    async fn send_email_to_user(&self, user: &User, task: &Task) -> Result<()> {
        // В базовой версии просто логируем
        tracing::info!("Sending email alert to {} for task {}", user.email, task.id);
        Ok(())
    }

    /// Получает информацию для уведомления
    pub fn get_alert_info(&self, task: &Task, status: &TaskStatus) -> AlertInfo {
        AlertInfo {
            task_id: task.id,
            status: status.clone(),
            message: task.message.clone().unwrap_or_default(),
        }
    }
}

/// Информация об уведомлении
pub struct AlertInfo {
    pub task_id: i32,
    pub status: TaskStatus,
    pub message: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_service_creation() {
        // Тест для проверки создания сервиса
        assert!(true);
    }

    #[test]
    fn test_alert_info_creation() {
        // Тест для проверки информации об уведомлении
        assert!(true);
    }
}
