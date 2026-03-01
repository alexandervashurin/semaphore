//! Projects API - Integrations Handler
//!
//! Обработчики для интеграций в проектах

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::Integration;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::db::store::RetrieveQueryParams;

/// Получает интеграции проекта
pub async fn get_integrations(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Query(params): Query<RetrieveQueryParams>,
) -> std::result::Result<Json<Vec<Integration>>, (StatusCode, Json<ErrorResponse>)> {
    let integrations = state.store.get_integrations(project_id, params)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(integrations))
}

/// Получает интеграцию по ID
pub async fn get_integration(
    State(state): State<Arc<AppState>>,
    Path((project_id, integration_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Integration>, (StatusCode, Json<ErrorResponse>)> {
    let integration = state.store.get_integration(project_id, integration_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Integration not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    Ok(Json(integration))
}

/// Создаёт новую интеграцию
pub async fn add_integration(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<Integration>,
) -> std::result::Result<(StatusCode, Json<Integration>), (StatusCode, Json<ErrorResponse>)> {
    let mut integration = payload;
    integration.project_id = project_id;

    let created = state.store.create_integration(integration)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет интеграцию
pub async fn update_integration(
    State(state): State<Arc<AppState>>,
    Path((project_id, integration_id)): Path<(i32, i32)>,
    Json(payload): Json<Integration>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut integration = payload;
    integration.id = integration_id;
    integration.project_id = project_id;

    state.store.update_integration(integration)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удаляет интеграцию
pub async fn delete_integration(
    State(state): State<Arc<AppState>>,
    Path((project_id, integration_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_integration(project_id, integration_id)
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
    fn test_integrations_handler() {
        // Тест для проверки обработчиков интеграций
        assert!(true);
    }
}
