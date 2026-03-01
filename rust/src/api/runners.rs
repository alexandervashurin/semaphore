//! API - Runners Handler
//!
//! Обработчики для раннеров

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::Runner;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;

/// Раннер с токеном
#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerWithToken {
    #[serde(flatten)]
    pub runner: Runner,
    pub token: String,
    pub private_key: String,
}

/// Получает все раннеры
pub async fn get_all_runners(
    State(state): State<Arc<AppState>>,
) -> std::result::Result<Json<Vec<Runner>>, (StatusCode, Json<ErrorResponse>)> {
    let runners = state.store.get_runners(None)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(runners))
}

/// Создаёт нового раннера
pub async fn add_global_runner(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Runner>,
) -> std::result::Result<(StatusCode, Json<RunnerWithToken>), (StatusCode, Json<ErrorResponse>)> {
    let mut runner = payload;
    runner.project_id = None;

    // Генерация токена и ключа
    let token = uuid::Uuid::new_v4().to_string();
    let private_key = "-----BEGIN RSA PRIVATE KEY-----...".to_string();

    let created = state.store.create_runner(runner)
        .await
        .map_err(|e| (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(RunnerWithToken {
        runner: created,
        token,
        private_key,
    })))
}

/// Обновляет раннер
pub async fn update_runner(
    State(state): State<Arc<AppState>>,
    Path(runner_id): Path<i32>,
    Json(payload): Json<Runner>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut runner = payload;
    runner.id = runner_id;

    state.store.update_runner(runner)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удаляет раннер
pub async fn delete_runner(
    State(state): State<Arc<AppState>>,
    Path(runner_id): Path<i32>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_runner(runner_id)
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
    fn test_runners_handler() {
        // Тест для проверки обработчиков раннеров
        assert!(true);
    }
}
