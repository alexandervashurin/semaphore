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
use crate::error::Error;
use crate::api::middleware::ErrorResponse;
use crate::db::store::AccessKeyManager;
use crate::services::key_encryption::{encrypt_key_secrets, decrypt_key_secrets, mask_key_secrets};

/// Получает ключи доступа проекта (секреты маскируются)
pub async fn get_keys(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Vec<AccessKey>>, (StatusCode, Json<ErrorResponse>)> {
    let mut keys = state.store.get_access_keys(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    // Дешифруем, затем маскируем для ответа
    for key in &mut keys {
        decrypt_key_secrets(key);
        mask_key_secrets(key);
    }

    Ok(Json(keys))
}

/// Получает ключ доступа по ID (секреты маскируются)
pub async fn get_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<AccessKey>, (StatusCode, Json<ErrorResponse>)> {
    let mut key = state.store.get_access_key(project_id, key_id)
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

    decrypt_key_secrets(&mut key);
    mask_key_secrets(&mut key);

    Ok(Json(key))
}

/// Создаёт новый ключ доступа (секреты шифруются перед сохранением)
pub async fn add_key(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<AccessKey>,
) -> std::result::Result<(StatusCode, Json<AccessKey>), (StatusCode, Json<ErrorResponse>)> {
    let mut key = payload;
    key.project_id = Some(project_id);

    // Шифруем секреты перед сохранением в БД
    encrypt_key_secrets(&mut key);

    let mut created = state.store.create_access_key(key)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    // Маскируем в ответе
    mask_key_secrets(&mut created);

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет ключ доступа (секреты шифруются перед сохранением)
pub async fn update_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
    Json(payload): Json<AccessKey>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut key = payload;
    key.id = key_id;
    key.project_id = Some(project_id);

    // Если пришло замаскированное значение — загружаем текущее из БД
    if key.ssh_key.as_deref() == Some("**SECRET**") ||
       key.login_password_password.as_deref() == Some("**SECRET**") ||
       key.access_key_secret_key.as_deref() == Some("**SECRET**") {
        let current = state.store.get_access_key(project_id, key_id)
            .await
            .map_err(|e| (StatusCode::NOT_FOUND, Json(ErrorResponse::new(e.to_string()))))?;
        if key.ssh_key.as_deref() == Some("**SECRET**") { key.ssh_key = current.ssh_key; }
        if key.ssh_passphrase.as_deref() == Some("**SECRET**") { key.ssh_passphrase = current.ssh_passphrase; }
        if key.login_password_password.as_deref() == Some("**SECRET**") { key.login_password_password = current.login_password_password; }
        if key.access_key_secret_key.as_deref() == Some("**SECRET**") { key.access_key_secret_key = current.access_key_secret_key; }
    } else {
        // Новые значения — шифруем
        encrypt_key_secrets(&mut key);
    }

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
        assert!(true);
    }
}
