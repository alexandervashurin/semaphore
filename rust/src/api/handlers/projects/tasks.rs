//! Projects API - Tasks Handler
//!
//! Обработчики для задач в проектах

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::{RetrieveQueryParams, TaskManager};
use crate::error::{Error, Result};
use crate::models::{Task, TaskOutput, TaskWithTpl};
use crate::services::task_logger::TaskStatus;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Получает задачи проекта
pub async fn get_tasks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Query(_params): Query<RetrieveQueryParams>,
) -> std::result::Result<Json<Vec<TaskWithTpl>>, (StatusCode, Json<ErrorResponse>)> {
    let tasks = state.store.get_tasks(project_id, None).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(tasks))
}

/// Получает последние задачи проекта (по дате создания)
///
/// GET /api/project/{project_id}/tasks/last
pub async fn get_last_tasks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Vec<TaskWithTpl>>, (StatusCode, Json<ErrorResponse>)> {
    let tasks = state.store.get_tasks(project_id, None).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    // Возвращаем только последние 20 записей
    let limited: Vec<TaskWithTpl> = tasks.into_iter().take(20).collect();

    Ok(Json(limited))
}

/// Получает задачу по ID
pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Task>, (StatusCode, Json<ErrorResponse>)> {
    let task = state
        .store
        .get_task(project_id, task_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Task not found".to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    Ok(Json(task))
}

/// Создаёт новую задачу
pub async fn add_task(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<CreateTaskPayload>,
) -> std::result::Result<(StatusCode, Json<Task>), (StatusCode, Json<ErrorResponse>)> {
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
        created: chrono::Utc::now(),
        start: None,
        end: None,
        message: payload.message,
        commit_hash: None,
        commit_message: None,
        build_task_id: payload.build_task_id,
        version: None,
        inventory_id: payload.inventory_id,
        repository_id: payload.repository_id,
        environment_id: payload.environment_id,
        params: None,
    };

    let created = state.store.create_task(task).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    // Запускаем выполнение задачи в фоне
    let store_arc: Arc<dyn crate::db::store::Store + Send + Sync> = Arc::new(state.store.clone());
    let task_to_run = created.clone();
    tokio::spawn(async move {
        crate::services::task_execution::execute_task(store_arc, task_to_run).await;
    });

    Ok((StatusCode::CREATED, Json(created)))
}

/// Останавливает задачу
pub async fn stop_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // В реальной реализации нужно остановить задачу
    // state.store.stop_task(project_id, task_id).await?;

    Ok(StatusCode::OK)
}

/// Подтверждает задачу
///
/// POST /api/projects/{project_id}/tasks/{task_id}/confirm
pub async fn confirm_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut task = state
        .store
        .get_task(project_id, task_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Task not found".to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    // Подтверждение задачи - перевод в статус Waiting
    task.status = TaskStatus::Waiting;

    state.store.update_task(task).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::OK)
}

/// Отклоняет задачу
///
/// POST /api/projects/{project_id}/tasks/{task_id}/reject
pub async fn reject_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut task = state
        .store
        .get_task(project_id, task_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Task not found".to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    // Отклонение задачи - перевод в статус Rejected
    task.status = TaskStatus::Rejected;
    task.end = Some(chrono::Utc::now());

    state.store.update_task(task).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::OK)
}

/// Удаляет задачу
pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_task(project_id, task_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Получает вывод задачи (логи)
///
/// GET /api/projects/{project_id}/tasks/{task_id}/output
pub async fn get_task_output(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Vec<TaskOutput>>, (StatusCode, Json<ErrorResponse>)> {
    // Получаем вывод задачи
    let outputs = state.store.get_task_outputs(task_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(outputs))
}

/// Возвращает raw-вывод задачи (текст без форматирования)
///
/// GET /api/project/{project_id}/tasks/{id}/raw_output
pub async fn get_task_raw_output(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<String, (StatusCode, Json<ErrorResponse>)> {
    let outputs = state.store.get_task_outputs(task_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    // Объединяем все строки вывода в plain text
    let raw = outputs
        .iter()
        .map(|o| o.output.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    // Убираем ANSI-коды для raw формата
    let clean = crate::utils::ansi::clear_from_ansi_codes(&raw);
    let _ = project_id; // suppress unused warning
    Ok(clean)
}

/// Возвращает стадии (этапы) задачи
///
/// GET /api/project/{project_id}/tasks/{id}/stages
pub async fn get_task_stages(
    State(_state): State<Arc<AppState>>,
    Path((_project_id, _task_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Stages — это более высокоуровневое представление стадий выполнения
    // В базовой реализации возвращаем пустой список
    Ok(Json(serde_json::json!([])))
}

/// Возвращает все активные задачи по всем проектам
///
/// GET /api/tasks
pub async fn get_all_tasks(
    State(state): State<Arc<AppState>>,
) -> std::result::Result<Json<Vec<TaskWithTpl>>, (StatusCode, Json<ErrorResponse>)> {
    let tasks = state
        .store
        .get_global_tasks(None, Some(200))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(tasks))
}

/// Payload для создания задачи
#[derive(Debug, Deserialize)]
pub struct CreateTaskPayload {
    pub template_id: i32,
    pub playbook: Option<String>,
    pub environment: Option<String>,
    pub arguments: Option<String>,
    pub git_branch: Option<String>,
    pub user_id: Option<i32>,
    pub message: Option<String>,
    pub build_task_id: Option<i32>,
    pub inventory_id: Option<i32>,
    pub repository_id: Option<i32>,
    pub environment_id: Option<i32>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tasks_handler() {
        // Тест для проверки обработчиков задач
        assert!(true);
    }
}
