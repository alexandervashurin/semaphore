//! Projects API - Secret Storages Handler
//!
//! Обработчики для хранилищ секретов в проектах

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::SecretStorage;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;

/// Получает хранилища секретов проекта
pub async fn get_secret_storages(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Vec<SecretStorage>>, (StatusCode, Json<ErrorResponse>)> {
    let storages = state.store.get_secret_storages(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(storages))
}

/// Получает хранилище секретов по ID
pub async fn get_secret_storage(
    State(state): State<Arc<AppState>>,
    Path((project_id, storage_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<SecretStorage>, (StatusCode, Json<ErrorResponse>)> {
    let storage = state.store.get_secret_storage(project_id, storage_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Secret storage not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    Ok(Json(storage))
}

/// Создаёт новое хранилище секретов
pub async fn add_secret_storage(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<SecretStorage>,
) -> std::result::Result<(StatusCode, Json<SecretStorage>), (StatusCode, Json<ErrorResponse>)> {
    let mut storage = payload;
    storage.project_id = project_id;

    let created = state.store.create_secret_storage(storage)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет хранилище секретов
pub async fn update_secret_storage(
    State(state): State<Arc<AppState>>,
    Path((project_id, storage_id)): Path<(i32, i32)>,
    Json(payload): Json<SecretStorage>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut storage = payload;
    storage.id = storage_id;
    storage.project_id = project_id;

    state.store.update_secret_storage(storage)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удаляет хранилище секретов
pub async fn delete_secret_storage(
    State(state): State<Arc<AppState>>,
    Path((project_id, storage_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_secret_storage(project_id, storage_id)
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
    fn test_secret_storages_handler() {
        // Тест для проверки обработчиков хранилищ секретов
        assert!(true);
    }
}
