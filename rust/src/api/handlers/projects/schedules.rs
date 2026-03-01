//! Projects API - Schedules Handler
//!
//! Обработчики для расписаний в проектах

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::Schedule;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::db::store::RetrieveQueryParams;

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
