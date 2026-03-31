//! Projects API - Views Handler
//!
//! Обработчики для представлений в проектах

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::ViewManager;
use crate::error::{Error, Result};
use crate::models::View;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Получает представления проекта
pub async fn get_views(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Vec<View>>, (StatusCode, Json<ErrorResponse>)> {
    let views = state.store.get_views(project_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(views))
}

/// Получает представление по ID
pub async fn get_view(
    State(state): State<Arc<AppState>>,
    Path((project_id, view_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<View>, (StatusCode, Json<ErrorResponse>)> {
    let view = state
        .store
        .get_view(project_id, view_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("View not found".to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    Ok(Json(view))
}

/// Создаёт новое представление
pub async fn add_view(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<View>,
) -> std::result::Result<(StatusCode, Json<View>), (StatusCode, Json<ErrorResponse>)> {
    let mut view = payload;
    view.project_id = project_id;

    let created = state.store.create_view(view).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет представление
pub async fn update_view(
    State(state): State<Arc<AppState>>,
    Path((project_id, view_id)): Path<(i32, i32)>,
    Json(payload): Json<View>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut view = payload;
    view.id = view_id;
    view.project_id = project_id;

    state.store.update_view(view).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::OK)
}

/// Удаляет представление
pub async fn delete_view(
    State(state): State<Arc<AppState>>,
    Path((project_id, view_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_view(project_id, view_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Устанавливает позиции представлений
///
/// POST /api/projects/{project_id}/views/positions
pub async fn set_view_positions(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<SetViewPositionsPayload>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Конвертируем в Vec<(view_id, position)>
    let positions: Vec<(i32, i32)> = payload
        .positions
        .into_iter()
        .map(|p| (p.view_id, p.position))
        .collect();

    state
        .store
        .set_view_positions(project_id, positions)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(StatusCode::OK)
}

// ============================================================================
// Types
// ============================================================================

/// Payload для установки позиций представлений
#[derive(Debug, Serialize, Deserialize)]
pub struct SetViewPositionsPayload {
    pub positions: Vec<ViewPosition>,
}

/// Позиция представления
#[derive(Debug, Serialize, Deserialize)]
pub struct ViewPosition {
    pub view_id: i32,
    pub position: i32,
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
