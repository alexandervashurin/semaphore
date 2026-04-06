//! TaskPool Status - обработка статусов и WebSocket уведомления
//!
//! Аналог services/tasks/TaskPool.go из Go версии (часть 4: статусы)

use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::api::websocket::WsMessage;
use crate::models::Task;
use crate::services::task_logger::TaskStatus;
use crate::services::task_pool_types::TaskPool;

/// Сообщение статуса задачи
#[derive(Debug, Clone)]
pub struct TaskStatusMessage {
    /// Тип сообщения
    pub message_type: String,
    /// ID задачи
    pub task_id: i32,
    /// Статус
    pub status: TaskStatus,
    /// Время начала
    pub start: Option<DateTime<Utc>>,
    /// Время окончания
    pub end: Option<DateTime<Utc>>,
    /// ID шаблона
    pub template_id: i32,
    /// ID проекта
    pub project_id: i32,
    /// Версия
    pub version: Option<String>,
}

impl TaskStatusMessage {
    /// Создаёт новое сообщение статуса
    pub fn new(task: &Task) -> Self {
        Self {
            message_type: "update".to_string(),
            task_id: task.id,
            status: task.status,
            start: task.start,
            end: task.end,
            template_id: task.template_id,
            project_id: task.project_id,
            version: task.version.clone(),
        }
    }

    /// Сериализует сообщение в JSON
    pub fn to_json(&self) -> String {
        use serde::Serialize;

        #[derive(Serialize)]
        struct SerializableMessage<'a> {
            #[serde(rename = "type")]
            message_type: &'a str,
            task_id: i32,
            status: &'a str,
            start: Option<DateTime<Utc>>,
            end: Option<DateTime<Utc>>,
            template_id: i32,
            project_id: i32,
            version: Option<&'a str>,
        }

        let status_string = self.status.to_string();
        let msg = SerializableMessage {
            message_type: &self.message_type,
            task_id: self.task_id,
            status: status_string.as_str(),
            start: self.start,
            end: self.end,
            template_id: self.template_id,
            project_id: self.project_id,
            version: self.version.as_deref(),
        };

        serde_json::to_string(&msg).unwrap_or_default()
    }
}

impl TaskPool {
    /// Обновляет статус задачи и отправляет уведомление
    pub async fn update_task_status(&self, task_id: i32, status: TaskStatus) -> Result<(), String> {
        // Обновляем статус в БД
        self.store
            .update_task_status(self.project.id, task_id, status)
            .await
            .map_err(|e| format!("Failed to update task status: {}", e))?;

        info!("Task {} status updated to {:?}", task_id, status);

        // Отправляем WebSocket уведомление
        self.notify_websocket(task_id, status).await;

        Ok(())
    }

    /// Отправляет WebSocket уведомление
    pub(crate) async fn notify_websocket(&self, task_id: i32, status: TaskStatus) {
        // Получаем задачу из БД
        let task = match self.store.get_task(self.project.id, task_id).await {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to get task for notification: {}", e);
                return;
            }
        };

        // Отправляем WebSocket уведомление через broadcast
        let ws_msg = WsMessage::Status {
            task_id,
            status: status.to_string(),
            time: Utc::now(),
        };
        if let Err(e) = self.ws_manager.broadcast(ws_msg) {
            // Ошибка отправки — нет подписчиков, не критично
            info!("WebSocket broadcast for task {}: {}", task_id, e);
        } else {
            info!("WebSocket notification sent for task {}", task_id);
        }
    }

    /// Логирует задачу
    pub async fn log_task(&self, task_id: i32, output: &str) -> Result<(), String> {
        use crate::models::TaskOutput;

        let task_output = TaskOutput {
            id: 0,
            task_id,
            project_id: self.project.id,
            output: output.to_string(),
            time: Utc::now(),
            stage_id: None,
        };

        self.store
            .create_task_output(task_output)
            .await
            .map_err(|e| format!("Failed to create task output: {}", e))?;

        info!("Task {} output logged", task_id);

        Ok(())
    }

    /// Получает логи задачи
    pub async fn get_task_logs(
        &self,
        task_id: i32,
    ) -> Result<Vec<crate::models::TaskOutput>, String> {
        use crate::db::store::RetrieveQueryParams;

        let params = RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            sort_by: None,
            sort_inverted: false,
            filter: None,
        };

        self.store
            .get_task_outputs(task_id)
            .await
            .map_err(|e| format!("Failed to get task outputs: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Task;

    #[test]
    fn test_task_status_message_creation() {
        let task = Task {
            id: 1,
            project_id: 1,
            template_id: 1,
            status: TaskStatus::Running,
            message: Some("Test task".to_string()),
            commit_hash: None,
            commit_message: None,
            version: Some("1.0.0".to_string()),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            arguments: None,
            params: None,
            playbook: None,
            start: Some(Utc::now()),
            end: None,
            created: Utc::now(),
            user_id: None,
            integration_id: None,
            schedule_id: None,
            git_branch: None,
            secret: None,
            environment: None,
            build_task_id: None,
        };

        let message = TaskStatusMessage::new(&task);
        assert_eq!(message.task_id, 1);
        assert_eq!(message.status, TaskStatus::Running);
        assert_eq!(message.message_type, "update");
    }

    #[test]
    fn test_task_status_message_to_json() {
        let task = Task {
            id: 1,
            project_id: 1,
            template_id: 1,
            status: TaskStatus::Success,
            message: Some("Test task".to_string()),
            commit_hash: None,
            commit_message: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            arguments: None,
            params: None,
            playbook: None,
            start: Some(Utc::now()),
            end: Some(Utc::now()),
            created: Utc::now(),
            user_id: None,
            integration_id: None,
            schedule_id: None,
            git_branch: None,
            secret: None,
            environment: None,
            build_task_id: None,
        };

        let message = TaskStatusMessage::new(&task);
        let json = message.to_json();

        assert!(json.contains("\"type\":\"update\""));
        assert!(json.contains("\"task_id\":1"));
        assert!(json.contains("\"status\":\"success\""));
    }

    #[test]
    fn test_task_status_message_with_null_fields() {
        let task = Task {
            id: 42,
            project_id: 2,
            template_id: 3,
            status: TaskStatus::Waiting,
            message: None,
            commit_hash: None,
            commit_message: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            arguments: None,
            params: None,
            playbook: None,
            start: None,
            end: None,
            created: Utc::now(),
            user_id: None,
            integration_id: None,
            schedule_id: None,
            git_branch: None,
            secret: None,
            environment: None,
            build_task_id: None,
        };

        let message = TaskStatusMessage::new(&task);
        let json = message.to_json();

        assert!(json.contains("\"task_id\":42"));
        assert!(json.contains("\"status\":\"waiting\""));
        assert!(json.contains("\"template_id\":3"));
        assert!(json.contains("\"project_id\":2"));
        // start и end должны быть null
        assert!(json.contains("\"start\":null"));
        assert!(json.contains("\"end\":null"));
        assert!(json.contains("\"version\":null"));
    }

    #[test]
    fn test_task_status_message_json_contains_all_fields() {
        let task = Task {
            id: 100,
            project_id: 10,
            template_id: 20,
            status: TaskStatus::Running,
            message: None,
            commit_hash: None,
            commit_message: None,
            version: Some("v2.0".to_string()),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            arguments: None,
            params: None,
            playbook: None,
            start: Some(Utc::now()),
            end: None,
            created: Utc::now(),
            user_id: None,
            integration_id: None,
            schedule_id: None,
            git_branch: None,
            secret: None,
            environment: None,
            build_task_id: None,
        };

        let message = TaskStatusMessage::new(&task);
        let json = message.to_json();

        // Проверяем все обязательные поля
        assert!(json.contains("\"task_id\":100"));
        assert!(json.contains("\"template_id\":20"));
        assert!(json.contains("\"project_id\":10"));
        assert!(json.contains("\"status\":\"running\""));
        assert!(json.contains("\"version\":\"v2.0\""));
        // start должен быть, end — null
        assert!(!json.contains("\"start\":null"));
        assert!(json.contains("\"end\":null"));
    }

    #[test]
    fn test_task_status_message_status_serialization() {
        let base_task = Task {
            id: 1,
            project_id: 1,
            template_id: 1,
            status: TaskStatus::Waiting,
            message: None,
            commit_hash: None,
            commit_message: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            arguments: None,
            params: None,
            playbook: None,
            start: None,
            end: None,
            created: Utc::now(),
            user_id: None,
            integration_id: None,
            schedule_id: None,
            git_branch: None,
            secret: None,
            environment: None,
            build_task_id: None,
        };

        // Проверяем разные статусы
        let statuses = [
            (TaskStatus::Waiting, "waiting"),
            (TaskStatus::Running, "running"),
            (TaskStatus::Success, "success"),
            (TaskStatus::Error, "error"),
            (TaskStatus::Stopped, "stopped"),
            (TaskStatus::Starting, "starting"),
        ];

        for (status, expected_str) in statuses {
            let mut task = base_task.clone();
            task.status = status;
            let message = TaskStatusMessage::new(&task);
            let json = message.to_json();
            assert!(
                json.contains(&format!("\"status\":\"{}\"", expected_str)),
                "Expected status '{}' in JSON for {:?}",
                expected_str,
                status
            );
        }
    }
}
