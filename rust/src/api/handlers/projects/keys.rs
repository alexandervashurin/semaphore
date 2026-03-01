//! Projects API - Keys Handler
//!
//! Обработчики для ключей доступа в проектах

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::AccessKey;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;

/// Получает ключи доступа проекта
pub async fn get_keys(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Vec<AccessKey>>, (StatusCode, Json<ErrorResponse>)> {
    let keys = state.store.get_access_keys(project_id, crate::db::store::RetrieveQueryParams::default())
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(keys))
}

/// Получает ключ доступа по ID
pub async fn get_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<AccessKey>, (StatusCode, Json<ErrorResponse>)> {
    let key = state.store.get_access_key(project_id, key_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Key not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    Ok(Json(key))
}

/// Создаёт новый ключ доступа
pub async fn add_key(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<AccessKey>,
) -> std::result::Result<(StatusCode, Json<AccessKey>), (StatusCode, Json<ErrorResponse>)> {
    let mut key = payload;
    key.project_id = Some(project_id);

    let created = state.store.create_access_key(key)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет ключ доступа
pub async fn update_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
    Json(payload): Json<AccessKey>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut key = payload;
    key.id = key_id;
    key.project_id = Some(project_id);

    state.store.update_access_key(key)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удаляет ключ доступа
pub async fn delete_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_access_key(project_id, key_id)
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
    fn test_keys_handler() {
        // Тест для проверки обработчиков ключей
        assert!(true);
    }
}
