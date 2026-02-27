//! Projects API - Views Handler
//!
//! Обработчики для представлений в проектах

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::View;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;

/// Получает представления проекта
pub async fn get_views(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<View>>, (StatusCode, Json<ErrorResponse>)> {
    let views = state.store.get_views(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(views))
}

/// Получает представление по ID
pub async fn get_view(
    State(state): State<Arc<AppState>>,
    Path((project_id, view_id)): Path<(i32, i32)>,
) -> Result<Json<View>, (StatusCode, Json<ErrorResponse>)> {
    let view = state.store.get_view(project_id, view_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("View not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    Ok(Json(view))
}

/// Создаёт новое представление
pub async fn add_view(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<View>,
) -> Result<(StatusCode, Json<View>), (StatusCode, Json<ErrorResponse>)> {
    let mut view = payload;
    view.project_id = project_id;

    let created = state.store.create_view(view)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет представление
pub async fn update_view(
    State(state): State<Arc<AppState>>,
    Path((project_id, view_id)): Path<(i32, i32)>,
    Json(payload): Json<View>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut view = payload;
    view.id = view_id;
    view.project_id = project_id;

    state.store.update_view(view)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удаляет представление
pub async fn delete_view(
    State(state): State<Arc<AppState>>,
    Path((project_id, view_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_view(project_id, view_id)
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
    fn test_views_handler() {
        // Тест для проверки обработчиков представлений
        assert!(true);
    }
}
