//! Tasks Handlers
//!
//! Обработчики запросов для управления задачами

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::api::state::AppState;
use crate::models::{Task, TaskWithTpl};
use crate::services::task_logger::TaskStatus;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;

/// Получить список задач проекта
///
/// GET /api/projects/:project_id/tasks
pub async fn get_tasks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<TaskWithTpl>>, (StatusCode, Json<ErrorResponse>)> {
    let tasks = state.store.get_tasks(project_id, None)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(tasks))
}

/// Создать задачу
///
/// POST /api/projects/:project_id/tasks
pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<TaskCreatePayload>,
) -> Result<(StatusCode, Json<Task>), (StatusCode, Json<ErrorResponse>)> {
    let task = Task {
        id: 0,
        template_id: payload.template_id,
        project_id,
        status: TaskStatus::Waiting,
        playbook: payload.playbook,
        environment: payload.environment,
        secret: None,
        arguments: payload.arguments,
        git_branch: payload.git_branch,
        user_id: payload.user_id,
        integration_id: None,
        schedule_id: None,
        created: Utc::now(),
        start: None,
        end: None,
        message: None,
        commit_hash: None,
        commit_message: None,
        build_task_id: payload.build_task_id,
        version: None,
        inventory_id: payload.inventory_id,
        params: None,
    };

    let created = state.store.create_task(task)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить задачу по ID
///
/// GET /api/projects/:project_id/tasks/:task_id
pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> Result<Json<Task>, (StatusCode, Json<ErrorResponse>)> {
    let task = state.store.get_task(project_id, task_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    Ok(Json(task))
}

/// Удалить задачу
///
/// DELETE /api/projects/:project_id/tasks/:task_id
pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_task(project_id, task_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Types
// ============================================================================

/// Payload для создания задачи
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskCreatePayload {
    pub template_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playbook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_task_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_id: Option<i32>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_create_payload_deserialize_required_only() {
        let json = r#"{"template_id": 1}"#;
        let payload: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.template_id, 1);
        assert_eq!(payload.playbook, None);
        assert_eq!(payload.environment, None);
    }

    #[test]
    fn test_task_create_payload_deserialize_all_fields() {
        let json = r#"{
            "template_id": 1,
            "playbook": "site.yml",
            "environment": "prod",
            "arguments": "--verbose",
            "git_branch": "main",
            "user_id": 5,
            "build_task_id": 10,
            "inventory_id": 3
        }"#;
        let payload: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.template_id, 1);
        assert_eq!(payload.playbook, Some("site.yml".to_string()));
        assert_eq!(payload.environment, Some("prod".to_string()));
        assert_eq!(payload.arguments, Some("--verbose".to_string()));
        assert_eq!(payload.git_branch, Some("main".to_string()));
        assert_eq!(payload.user_id, Some(5));
        assert_eq!(payload.build_task_id, Some(10));
        assert_eq!(payload.inventory_id, Some(3));
    }
}
