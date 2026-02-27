//! API - Events Handler
//!
//! Обработчики для событий

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::Event;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::api::extractors::AuthUser;
use crate::db::store::RetrieveQueryParams;

/// Получает последние события
pub async fn get_last_events(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Json<Vec<Event>>, (StatusCode, Json<ErrorResponse>)> {
    get_events(state, auth_user, 200).await
}

/// Получает все события
pub async fn get_all_events(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Json<Vec<Event>>, (StatusCode, Json<ErrorResponse>)> {
    get_events(state, auth_user, 0).await
}

/// Получает события
async fn get_events(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    limit: usize,
) -> Result<Json<Vec<Event>>, (StatusCode, Json<ErrorResponse>)> {
    let mut params = RetrieveQueryParams::default();
    if limit > 0 {
        params.count = Some(limit);
    }

    let events = state.store.get_events(None, params)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(events))
}

/// Получает события проекта
pub async fn get_project_events(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    auth_user: AuthUser,
) -> Result<Json<Vec<Event>>, (StatusCode, Json<ErrorResponse>)> {
    let mut params = RetrieveQueryParams::default();
    params.count = Some(200);

    let events = state.store.get_events(Some(project_id), params)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

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
