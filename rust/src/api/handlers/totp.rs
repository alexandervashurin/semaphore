//! TOTP Handlers
//!
//! Обработчики запросов для управления TOTP (2FA)

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::api::state::AppState;
use crate::models::user::UserTotp;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;

/// Начать настройку TOTP
///
/// POST /api/auth/totp/start
pub async fn start_totp_setup(
    State(state): State<Arc<AppState>>,
    auth_user: crate::api::extractors::AuthUser,
) -> Result<Json<TotpSetupResponse>, (StatusCode, Json<ErrorResponse>)> {
    use crate::services::totp::generate_totp_secret;

    // Получаем пользователя
    let user = state.store.get_user(auth_user.user_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка: {}", e))),
        ))?;

    // Если TOTP уже настроен, возвращаем ошибку
    if user.totp.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("TOTP уже настроен")
                .with_code("TOTP_ALREADY_ENABLED")),
        ));
    }

    // Генерируем секрет
    let totp_secret = generate_totp_secret(&user, "Semaphore UI")
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка генерации секрета: {}", e))),
        ))?;

    Ok(Json(TotpSetupResponse {
        secret: totp_secret.secret,
        url: totp_secret.url,
        recovery_code: totp_secret.recovery_code,
    }))
}

/// Подтвердить настройку TOTP
///
/// POST /api/auth/totp/confirm
pub async fn confirm_totp_setup(
    State(state): State<Arc<AppState>>,
    auth_user: crate::api::extractors::AuthUser,
    Json(payload): Json<TotpConfirmPayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    use crate::services::totp::{generate_totp_secret, verify_totp_code};

    // Получаем пользователя
    let user = state.store.get_user(auth_user.user_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка: {}", e))),
        ))?;

    // Если TOTP уже настроен, возвращаем ошибку
    if user.totp.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("TOTP уже настроен")
                .with_code("TOTP_ALREADY_ENABLED")),
        ));
    }

    // Генерируем секрет (временно)
    let totp_secret = generate_totp_secret(&user, "Semaphore UI")
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка генерации секрета: {}", e))),
        ))?;

    // Проверяем код
    if !verify_totp_code(&totp_secret.secret, &payload.code) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Неверный TOTP код")
                .with_code("INVALID_TOTP")),
        ));
    }

    // Сохраняем TOTP в БД
    let _totp = UserTotp {
        id: 0,
        created: Utc::now(),
        user_id: user.id,
        url: totp_secret.url,
        recovery_hash: totp_secret.recovery_hash,
        recovery_code: None,
    };

    // TODO: Реализовать сохранение TOTP в store
    // state.store.create_totp(totp).await?;

    Ok(StatusCode::OK)
}

/// Отключить TOTP
///
/// POST /api/auth/totp/disable
pub async fn disable_totp(
    State(state): State<Arc<AppState>>,
    auth_user: crate::api::extractors::AuthUser,
    Json(payload): Json<TotpDisablePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    use crate::services::totp::verify_recovery_code;

    // Получаем пользователя
    let user = state.store.get_user(auth_user.user_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка: {}", e))),
        ))?;

    // Проверяем, что TOTP настроен
    let totp = user.totp
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("TOTP не настроен")
                .with_code("TOTP_NOT_ENABLED")),
        ))?;

    // Проверяем код восстановления
    if !verify_recovery_code(&payload.recovery_code, &totp.recovery_hash) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Неверный код восстановления")
                .with_code("INVALID_RECOVERY_CODE")),
        ));
    }

    // TODO: Реализовать удаление TOTP из store
    // state.store.delete_totp(user.id).await?;

    Ok(StatusCode::OK)
}

// ============================================================================
// Types
// ============================================================================

/// Response для настройки TOTP
#[derive(Debug, Serialize)]
pub struct TotpSetupResponse {
    pub secret: String,
    pub url: String,
    pub recovery_code: String,
}

/// Payload для подтверждения TOTP
#[derive(Debug, Deserialize)]
pub struct TotpConfirmPayload {
    pub code: String,
}

/// Payload для отключения TOTP
#[derive(Debug, Deserialize)]
pub struct TotpDisablePayload {
    pub recovery_code: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_confirm_payload_deserialize() {
        let json = r#"{"code": "123456"}"#;
        let payload: TotpConfirmPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.code, "123456");
    }

    #[test]
    fn test_totp_disable_payload_deserialize() {
        let json = r#"{"recovery_code": "abcdef123456"}"#;
        let payload: TotpDisablePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.recovery_code, "abcdef123456");
    }

    #[test]
    fn test_totp_setup_response_serialize() {
        let response = TotpSetupResponse {
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            url: "otpauth://totp/Test?secret=ABC".to_string(),
            recovery_code: "ABC123".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("JBSWY3DPEHPK3PXP"));
        assert!(json.contains("otpauth://totp/Test"));
        assert!(json.contains("ABC123"));
    }
}
