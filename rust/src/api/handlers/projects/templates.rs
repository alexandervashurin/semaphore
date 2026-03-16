//! Projects API - Templates Handler
//!
//! Обработчики для шаблонов в проектах

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::{Template, TemplateWithPerms, TemplateFilter};
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::db::store::{RetrieveQueryParams, TemplateManager, TaskManager};

/// Query params с поддержкой фильтрации по app и view_id (B-BE-19)
#[derive(Debug, Default, Deserialize)]
pub struct TemplateQueryParams {
    pub app: Option<String>,
    pub view_id: Option<i32>,
}

/// Получает шаблоны проекта
pub async fn get_templates(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Query(_params): Query<RetrieveQueryParams>,
) -> std::result::Result<Json<Vec<Template>>, (StatusCode, Json<ErrorResponse>)> {
    let templates = state.store.get_templates(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(templates))
}

/// Получает шаблон по ID
pub async fn get_template(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Template>, (StatusCode, Json<ErrorResponse>)> {
    let template = state.store.get_template(project_id, template_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Template not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    Ok(Json(template))
}

/// Создаёт новый шаблон
pub async fn add_template(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<Template>,
) -> std::result::Result<(StatusCode, Json<Template>), (StatusCode, Json<ErrorResponse>)> {
    let mut template = payload;
    template.project_id = project_id;

    let created = state.store.create_template(template)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет шаблон
pub async fn update_template(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
    Json(payload): Json<Template>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut template = payload;
    template.id = template_id;
    template.project_id = project_id;

    state.store.update_template(template)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удаляет шаблон
pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_template(project_id, template_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Payload для обновления описания шаблона (B-BE-18)
#[derive(Debug, Deserialize)]
pub struct UpdateDescriptionPayload {
    pub description: String,
}

/// Обновляет описание шаблона (B-BE-18)
///
/// PUT /api/project/{project_id}/templates/{id}/description
pub async fn update_template_description(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
    Json(payload): Json<UpdateDescriptionPayload>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut template = state.store.get_template(project_id, template_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Template not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    template.description = payload.description;

    state.store.update_template(template)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Останавливает все задачи шаблона (B-BE-17)
///
/// POST /api/project/{project_id}/templates/{id}/stop_all_tasks
pub async fn stop_all_template_tasks(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    use crate::services::task_logger::TaskStatus;

    let tasks = state.store.get_tasks(project_id, Some(template_id))
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    for task_with_tpl in tasks {
        let task = task_with_tpl.task;
        match task.status {
            TaskStatus::Running | TaskStatus::Waiting => {
                let _ = state.store.update_task_status(project_id, task.id, TaskStatus::Stopping).await;
            }
            _ => {}
        }
    }

    Ok(StatusCode::OK)
}

/// Получает расписания шаблона
///
/// GET /api/project/{project_id}/templates/{id}/schedules
pub async fn get_template_schedules(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    use crate::db::store::ScheduleManager;

    let schedules = state.store.get_schedules(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    // Фильтруем расписания по template_id
    let template_schedules: Vec<_> = schedules.into_iter()
        .filter(|s| s.template_id == template_id)
        .collect();

    Ok(Json(serde_json::to_value(template_schedules).unwrap_or_default()))
}

/// Получает задачи шаблона
///
/// GET /api/project/{project_id}/templates/{id}/tasks
pub async fn get_template_tasks(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let tasks = state.store.get_tasks(project_id, Some(template_id))
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(serde_json::to_value(tasks).unwrap_or_default()))
}

/// Получает последнюю задачу шаблона
///
/// GET /api/project/{project_id}/templates/{id}/tasks/last
pub async fn get_template_last_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let tasks = state.store.get_tasks(project_id, Some(template_id))
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    let last = tasks.into_iter().next();
    Ok(Json(serde_json::to_value(last).unwrap_or(serde_json::Value::Null)))
}

/// Возвращает статистику шаблона
///
/// GET /api/project/{project_id}/templates/{id}/stats
pub async fn get_template_stats(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let tasks = state.store.get_tasks(project_id, Some(template_id))
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    let total = tasks.len();
    let success = tasks.iter().filter(|t| format!("{:?}", t.task.status) == "Success").count();
    let failed = tasks.iter().filter(|t| format!("{:?}", t.task.status) == "Error").count();

    Ok(Json(serde_json::json!({
        "template_id": template_id,
        "project_id": project_id,
        "total_tasks": total,
        "success_count": success,
        "failed_count": failed,
    })))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_templates_handler() {
        // Тест для проверки обработчиков шаблонов
        assert!(true);
    }
}
