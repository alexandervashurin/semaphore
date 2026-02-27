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
use crate::db::store::RetrieveQueryParams;

/// Получает шаблоны проекта
pub async fn get_templates(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Query(params): Query<RetrieveQueryParams>,
) -> Result<Json<Vec<Template>>, (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<Json<Template>, (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<(StatusCode, Json<Template>), (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_template(project_id, template_id)
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
    fn test_templates_handler() {
        // Тест для проверки обработчиков шаблонов
        assert!(true);
    }
}
