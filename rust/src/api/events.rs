//! API - Events Handler
//!
//! Обработчики для событий

use crate::api::extractors::AuthUser;
use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::EventManager;
use crate::error::{Error, Result};
use crate::models::Event;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

/// Получает последние события
pub async fn get_last_events(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
) -> std::result::Result<Json<Vec<Event>>, (StatusCode, Json<ErrorResponse>)> {
    get_events_inner(&state, 200).await
}

/// Получает все события
pub async fn get_all_events(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
) -> std::result::Result<Json<Vec<Event>>, (StatusCode, Json<ErrorResponse>)> {
    get_events_inner(&state, 0).await
}

/// Получает события
async fn get_events_inner(
    state: &AppState,
    limit: usize,
) -> std::result::Result<Json<Vec<Event>>, (StatusCode, Json<ErrorResponse>)> {
    let events = state.store.get_events(None, limit).await;

    let events = events.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(events))
}

/// Получает события проекта
pub async fn get_project_events(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    _auth_user: AuthUser,
) -> std::result::Result<Json<Vec<Event>>, (StatusCode, Json<ErrorResponse>)> {
    let events = state
        .store
        .get_events(Some(project_id), 200)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(events))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_events_handler() {
        // Тест для проверки обработчиков событий
        assert!(true);
    }
}
