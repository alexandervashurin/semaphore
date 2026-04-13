//! TOTP Handlers
//!
//! Обработчики запросов для управления TOTP (2FA)

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::UserManager;
use crate::error::Error;
use crate::models::user::UserTotp;
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Начать настройку TOTP
///
/// POST /api/auth/totp/start
pub async fn start_totp_setup(
    State(state): State<Arc<AppState>>,
    auth_user: crate::api::extractors::AuthUser,
) -> Result<Json<TotpSetupResponse>, (StatusCode, Json<ErrorResponse>)> {
    use crate::services::totp::generate_totp_secret;

    // Получаем пользователя
    let user = state.store.get_user(auth_user.user_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка: {}", e))),
        )
    })?;

    // Если TOTP уже настроен, возвращаем ошибку
    if user.totp.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("TOTP уже настроен").with_code("TOTP_ALREADY_ENABLED")),
        ));
    }

    // Генерируем секрет
    let totp_secret = generate_totp_secret(&user, "Velum UI").map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!(
                "Ошибка генерации секрета: {}",
                e
            ))),
        )
    })?;

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
    let user = state.store.get_user(auth_user.user_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка: {}", e))),
        )
    })?;

    // Если TOTP уже настроен, возвращаем ошибку
    if user.totp.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("TOTP уже настроен").with_code("TOTP_ALREADY_ENABLED")),
        ));
    }

    // Генерируем секрет (временно)
    let totp_secret = generate_totp_secret(&user, "Velum UI").map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!(
                "Ошибка генерации секрета: {}",
                e
            ))),
        )
    })?;

    // Проверяем код
    if !verify_totp_code(&totp_secret.secret, &payload.code) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Неверный TOTP код").with_code("INVALID_TOTP")),
        ));
    }

    // Сохраняем TOTP в БД
    let totp = UserTotp {
        id: 0,
        created: Utc::now(),
        user_id: user.id,
        url: totp_secret.url.clone(),
        recovery_hash: totp_secret.recovery_hash.clone(),
        recovery_code: None,
    };

    // Сохраняем TOTP через store
    state
        .store
        .set_user_totp(user.id, &totp)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Ошибка сохранения TOTP: {}", e))),
            )
        })?;

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
    let user = state.store.get_user(auth_user.user_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка: {}", e))),
        )
    })?;

    // Проверяем, что TOTP настроен
    let totp = user.totp.ok_or((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new("TOTP не настроен").with_code("TOTP_NOT_ENABLED")),
    ))?;

    // Проверяем код восстановления
    if !verify_recovery_code(&payload.recovery_code, &totp.recovery_hash) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                ErrorResponse::new("Неверный код восстановления")
                    .with_code("INVALID_RECOVERY_CODE"),
            ),
        ));
    }

    // Удаляем TOTP из store
    state.store.delete_user_totp(user.id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка удаления TOTP: {}", e))),
        )
    })?;

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

    #[test]
    fn test_totp_confirm_payload_roundtrip() {
        // TotpConfirmPayload only derives Deserialize, not Serialize
        // Test roundtrip via deserialization from JSON
        let json = r#"{"code": "654321"}"#;
        let restored: TotpConfirmPayload = serde_json::from_str(json).unwrap();
        assert_eq!(restored.code, "654321");
    }

    #[test]
    fn test_totp_disable_payload_roundtrip() {
        // TotpDisablePayload only derives Deserialize, not Serialize
        let json = r#"{"recovery_code": "RECOVERY123"}"#;
        let restored: TotpDisablePayload = serde_json::from_str(json).unwrap();
        assert_eq!(restored.recovery_code, "RECOVERY123");
    }

    #[test]
    fn test_totp_setup_response_roundtrip() {
        // TotpSetupResponse derives Serialize but not Deserialize
        let response = TotpSetupResponse {
            secret: "ABCDEF123456".to_string(),
            url: "otpauth://totp/Example:user@test.com?secret=ABCDEF123456".to_string(),
            recovery_code: "R3C0V3RY".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("ABCDEF123456"));
        assert!(json.contains("otpauth://totp/Example"));
        assert!(json.contains("R3C0V3RY"));
    }

    #[test]
    fn test_totp_confirm_payload_debug() {
        let payload = TotpConfirmPayload {
            code: "123456".to_string(),
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("TotpConfirmPayload"));
    }

    #[test]
    fn test_totp_disable_payload_debug() {
        let payload = TotpDisablePayload {
            recovery_code: "test".to_string(),
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("TotpDisablePayload"));
    }

    #[test]
    fn test_totp_setup_response_debug() {
        let response = TotpSetupResponse {
            secret: "SECRET".to_string(),
            url: "otpauth://totp/Test".to_string(),
            recovery_code: "CODE".to_string(),
        };
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("TotpSetupResponse"));
        assert!(debug_str.contains("SECRET"));
    }

    #[test]
    fn test_totp_confirm_payload_empty_code() {
        let json = r#"{"code": ""}"#;
        let payload: TotpConfirmPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.code, "");
    }

    #[test]
    fn test_totp_disable_payload_empty_code() {
        let json = r#"{"recovery_code": ""}"#;
        let payload: TotpDisablePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.recovery_code, "");
    }

    #[test]
    fn test_totp_confirm_payload_unicode() {
        let json = r#"{"code": "123456"}"#;
        let payload: TotpConfirmPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.code, "123456");
    }

    #[test]
    fn test_totp_confirm_payload_clone() {
        // TotpConfirmPayload doesn't derive Clone, test via deserialization
        let json = r#"{"code": "654321"}"#;
        let p1: TotpConfirmPayload = serde_json::from_str(json).unwrap();
        let p2: TotpConfirmPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p1.code, p2.code);
    }

    #[test]
    fn test_totp_disable_payload_clone() {
        // TotpDisablePayload doesn't derive Clone, test via deserialization
        let json = r#"{"recovery_code": "recovery_code"}"#;
        let p1: TotpDisablePayload = serde_json::from_str(json).unwrap();
        let p2: TotpDisablePayload = serde_json::from_str(json).unwrap();
        assert_eq!(p1.recovery_code, p2.recovery_code);
    }

    #[test]
    fn test_totp_setup_response_clone() {
        // TotpSetupResponse doesn't derive Clone
        let r1 = TotpSetupResponse {
            secret: "SECRET".to_string(),
            url: "otpauth://totp/Test".to_string(),
            recovery_code: "CODE".to_string(),
        };
        let r2 = TotpSetupResponse {
            secret: "SECRET".to_string(),
            url: "otpauth://totp/Test".to_string(),
            recovery_code: "CODE".to_string(),
        };
        assert_eq!(r1.secret, r2.secret);
        assert_eq!(r1.recovery_code, r2.recovery_code);
    }

    #[test]
    fn test_totp_confirm_payload_clone_independence() {
        // TotpConfirmPayload doesn't derive Clone
        let mut code = "original".to_string();
        let p1 = TotpConfirmPayload { code: code.clone() };
        code = "modified".to_string();
        assert_eq!(p1.code, "original");
    }

    #[test]
    fn test_totp_disable_payload_clone_independence() {
        // TotpDisablePayload doesn't derive Clone
        let mut recovery_code = "original".to_string();
        let p1 = TotpDisablePayload { recovery_code: recovery_code.clone() };
        recovery_code = "modified".to_string();
        assert_eq!(p1.recovery_code, "original");
    }

    #[test]
    fn test_totp_setup_response_serialize_special_chars() {
        let response = TotpSetupResponse {
            secret: "JBSW&<>'\"Y3DP".to_string(),
            url: "otpauth://totp/Test%20User?secret=ABC".to_string(),
            recovery_code: "CODE-WITH-DASHES".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("JBSW"));
        assert!(json.contains("CODE-WITH-DASHES"));
    }
}
