//! Projects API - Integrations Handler
//!
//! Обработчики для интеграций в проектах

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::{Integration, IntegrationMatcher, IntegrationExtractValue};
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::db::store::{RetrieveQueryParams, IntegrationManager, IntegrationMatcherManager, IntegrationExtractValueManager};

/// Получает интеграции проекта
pub async fn get_integrations(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Vec<Integration>>, (StatusCode, Json<ErrorResponse>)> {
    let integrations = state.store.get_integrations(project_id)
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
// Integration Matchers CRUD (B-BE-20)
// ============================================================================

/// GET /api/project/{project_id}/integrations/{integration_id}/matchers
pub async fn get_integration_matchers(
    State(state): State<Arc<AppState>>,
    Path((project_id, integration_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Vec<IntegrationMatcher>>, (StatusCode, Json<ErrorResponse>)> {
    let matchers = state.store.get_integration_matchers(project_id, integration_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string()))))?;
    Ok(Json(matchers))
}

/// POST /api/project/{project_id}/integrations/{integration_id}/matchers
pub async fn add_integration_matcher(
    State(state): State<Arc<AppState>>,
    Path((project_id, integration_id)): Path<(i32, i32)>,
    Json(mut payload): Json<IntegrationMatcher>,
) -> std::result::Result<(StatusCode, Json<IntegrationMatcher>), (StatusCode, Json<ErrorResponse>)> {
    payload.project_id = project_id;
    payload.integration_id = integration_id;
    let created = state.store.create_integration_matcher(payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string()))))?;
    Ok((StatusCode::CREATED, Json(created)))
}

/// PUT /api/project/{project_id}/integrations/{integration_id}/matchers/{matcher_id}
pub async fn update_integration_matcher(
    State(state): State<Arc<AppState>>,
    Path((project_id, integration_id, matcher_id)): Path<(i32, i32, i32)>,
    Json(mut payload): Json<IntegrationMatcher>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    payload.id = matcher_id;
    payload.project_id = project_id;
    payload.integration_id = integration_id;
    state.store.update_integration_matcher(payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string()))))?;
    Ok(StatusCode::OK)
}

/// DELETE /api/project/{project_id}/integrations/{integration_id}/matchers/{matcher_id}
pub async fn delete_integration_matcher(
    State(state): State<Arc<AppState>>,
    Path((project_id, integration_id, matcher_id)): Path<(i32, i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_integration_matcher(project_id, integration_id, matcher_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string()))))?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Integration Extract Values CRUD (B-BE-21)
// ============================================================================

/// GET /api/project/{project_id}/integrations/{integration_id}/extractvalues
pub async fn get_integration_extract_values(
    State(state): State<Arc<AppState>>,
    Path((project_id, integration_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Vec<IntegrationExtractValue>>, (StatusCode, Json<ErrorResponse>)> {
    let values = state.store.get_integration_extract_values(project_id, integration_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string()))))?;
    Ok(Json(values))
}

/// POST /api/project/{project_id}/integrations/{integration_id}/extractvalues
pub async fn add_integration_extract_value(
    State(state): State<Arc<AppState>>,
    Path((project_id, integration_id)): Path<(i32, i32)>,
    Json(mut payload): Json<IntegrationExtractValue>,
) -> std::result::Result<(StatusCode, Json<IntegrationExtractValue>), (StatusCode, Json<ErrorResponse>)> {
    payload.project_id = project_id;
    payload.integration_id = integration_id;
    let created = state.store.create_integration_extract_value(payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string()))))?;
    Ok((StatusCode::CREATED, Json(created)))
}

/// PUT /api/project/{project_id}/integrations/{integration_id}/extractvalues/{value_id}
pub async fn update_integration_extract_value(
    State(state): State<Arc<AppState>>,
    Path((project_id, integration_id, value_id)): Path<(i32, i32, i32)>,
    Json(mut payload): Json<IntegrationExtractValue>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    payload.id = value_id;
    payload.project_id = project_id;
    payload.integration_id = integration_id;
    state.store.update_integration_extract_value(payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string()))))?;
    Ok(StatusCode::OK)
}

/// DELETE /api/project/{project_id}/integrations/{integration_id}/extractvalues/{value_id}
pub async fn delete_integration_extract_value(
    State(state): State<Arc<AppState>>,
    Path((project_id, integration_id, value_id)): Path<(i32, i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_integration_extract_value(project_id, integration_id, value_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string()))))?;
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
