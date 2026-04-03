//! Projects API - Schedules Handler
//!
//! Обработчики для расписаний в проектах

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::ScheduleManager;
use crate::error::{Error, Result};
use crate::models::Schedule;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Проверяет `cron` до записи в БД. Для `cron_format = run_at` выражение не парсится как cron.
fn schedule_cron_must_parse(schedule: &Schedule) -> Option<String> {
    if schedule.cron_format.as_deref() == Some("run_at") {
        return None;
    }
    if schedule.cron.trim().is_empty() {
        return Some("Cron expression cannot be empty".to_string());
    }
    match schedule.cron.parse::<cron::Schedule>() {
        Ok(_) => None,
        Err(e) => Some(format!("Invalid cron expression: {}", e)),
    }
}

/// Получает расписания проекта
pub async fn get_project_schedules(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Vec<Schedule>>, (StatusCode, Json<ErrorResponse>)> {
    let schedules = state.store.get_schedules(project_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(schedules))
}

/// Получает расписание по ID
pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
    Path((project_id, schedule_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Schedule>, (StatusCode, Json<ErrorResponse>)> {
    let schedule = state
        .store
        .get_schedule(project_id, schedule_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Schedule not found".to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
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

    if let Some(err) = schedule_cron_must_parse(&schedule) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new(err))));
    }

    let created = state.store.create_schedule(schedule).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

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

    if let Some(err) = schedule_cron_must_parse(&schedule) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new(err))));
    }

    state.store.update_schedule(schedule).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::OK)
}

/// Удаляет расписание
pub async fn delete_schedule(
    State(state): State<Arc<AppState>>,
    Path((project_id, schedule_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_schedule(project_id, schedule_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

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
    let active = payload
        .get("active")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut schedule = state
        .store
        .get_schedule(project_id, schedule_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Schedule not found".to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    schedule.active = active;
    state.store.update_schedule(schedule).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

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

    fn sample_schedule(cron: &str, cron_format: Option<&str>) -> Schedule {
        Schedule {
            id: 0,
            template_id: 1,
            project_id: 1,
            cron: cron.to_string(),
            cron_format: cron_format.map(String::from),
            name: "t".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        }
    }

    #[test]
    fn cron_validation_rejects_invalid_expression() {
        let s = sample_schedule("not valid cron syntax", None);
        assert!(schedule_cron_must_parse(&s).is_some());
    }

    #[test]
    fn cron_validation_accepts_standard_expression() {
        let s = sample_schedule("0 0 * * * *", None);
        assert!(schedule_cron_must_parse(&s).is_none());
    }

    #[test]
    fn cron_validation_skips_when_run_at_format() {
        let s = sample_schedule("", Some("run_at"));
        assert!(schedule_cron_must_parse(&s).is_none());
    }

    #[test]
    fn cron_validation_rejects_empty_cron_when_not_run_at() {
        let s = sample_schedule("   ", None);
        assert!(schedule_cron_must_parse(&s).is_some());
    }
}
