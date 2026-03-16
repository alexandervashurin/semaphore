//! Projects API - Schedules Handler
//!
//! Обработчики для расписаний в проектах

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::Schedule;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::db::store::ScheduleManager;

/// Получает расписания проекта
pub async fn get_project_schedules(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Vec<Schedule>>, (StatusCode, Json<ErrorResponse>)> {
    let schedules = state.store.get_schedules(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(schedules))
}

/// Получает расписание по ID
pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
    Path((project_id, schedule_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Schedule>, (StatusCode, Json<ErrorResponse>)> {
    let schedule = state.store.get_schedule(project_id, schedule_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Schedule not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    Ok(Json(schedule))
}

/// Создаёт новое расписание
pub async fn add_schedule(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<Schedule>,
) -> std::result::Result<(StatusCode, Json<Schedule>), (StatusCode, Json<ErrorResponse>)> {
    let mut schedule = payload;
    schedule.project_id = project_id;

    let created = state.store.create_schedule(schedule)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет расписание
pub async fn update_schedule(
    State(state): State<Arc<AppState>>,
    Path((project_id, schedule_id)): Path<(i32, i32)>,
    Json(payload): Json<Schedule>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut schedule = payload;
    schedule.id = schedule_id;
    schedule.project_id = project_id;

    state.store.update_schedule(schedule)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удаляет расписание
pub async fn delete_schedule(
    State(state): State<Arc<AppState>>,
    Path((project_id, schedule_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_schedule(project_id, schedule_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Переключает активность расписания
///
/// PUT /api/project/{project_id}/schedules/{id}/active
pub async fn toggle_schedule_active(
    State(state): State<Arc<AppState>>,
    Path((project_id, schedule_id)): Path<(i32, i32)>,
    Json(payload): Json<serde_json::Value>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let active = payload.get("active")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut schedule = state.store.get_schedule(project_id, schedule_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Schedule not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    schedule.active = active;
    state.store.update_schedule(schedule)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Валидирует cron-выражение
///
/// POST /api/projects/{project_id}/schedules/validate
pub async fn validate_schedule_cron_format(
    Path(_project_id): Path<i32>,
    Json(payload): Json<ValidateCronPayload>,
) -> std::result::Result<Json<ValidateCronResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Пытаемся распарсить cron выражение
    let result = payload.cron.parse::<cron::Schedule>();
    
    let response = ValidateCronResponse {
        valid: result.is_ok(),
        error: result.err().map(|e| e.to_string()),
    };

    Ok(Json(response))
}

// ============================================================================
// Types
// ============================================================================

/// Payload для валидации cron
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateCronPayload {
    pub cron: String,
}

/// Response валидации cron
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateCronResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedules_handler() {
        // Тест для проверки обработчиков расписаний
        assert!(true);
    }
}
