//! Authentication Handlers
//!
//! Обработчики запросов для аутентификации

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;

/// Health check endpoint
pub async fn health() -> &'static str {
    "OK"
}

/// Вход в систему
///
/// POST /api/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    use crate::api::auth_local::{LocalAuthService, verify_password};
    use crate::services::totp::verify_totp_code;

    // Находим пользователя
    let user = state.store.get_user_by_login_or_email(&payload.username, &payload.username)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Неверный логин или пароль")
                    .with_code("INVALID_CREDENTIALS")),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Ошибка сервера")),
            ),
        })?;

    // Проверяем пароль
    if !verify_password(&payload.password, &user.password) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Неверный логин или пароль")
                .with_code("INVALID_CREDENTIALS")),
        ));
    }

    // Проверяем TOTP, если настроен
    if let Some(ref totp) = user.totp {
        let totp_code = payload.totp_code
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Требуется TOTP код")
                    .with_code("TOTP_REQUIRED")),
            ))?;

        if !verify_totp_code(&totp.url, &totp_code) {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Неверный TOTP код")
                    .with_code("INVALID_TOTP")),
            ));
        }
    }

    // Генерируем токен
    let auth_service = LocalAuthService::new(state.store.clone());
    let token_info = auth_service.generate_token(&user)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка генерации токена: {}", e))
                .with_code("TOKEN_GENERATION_ERROR")),
        ))?;

    Ok(Json(LoginResponse {
        token: token_info.token,
        token_type: token_info.token_type,
        expires_in: token_info.expires_in,
        totp_required: None,
    }))
}

/// Выход из системы
///
/// POST /api/auth/logout
pub async fn logout(
    State(_state): State<Arc<AppState>>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Реализовать выход (добавление токена в чёрный список)
    Ok(StatusCode::OK)
}

// ============================================================================
// Types
// ============================================================================

/// Payload для входа
#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub totp_code: Option<String>,
}

/// Response после входа
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp_required: Option<bool>,
}

/// Response для настройки TOTP
#[derive(Debug, Serialize)]
pub struct TotpSetupResponse {
    pub secret: String,
    pub url: String,
    pub recovery_code: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_payload_deserialize() {
        let json = r#"{
            "username": "admin",
            "password": "password123",
            "totp_code": "123456"
        }"#;
        
        let payload: LoginPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, "admin");
        assert_eq!(payload.password, "password123");
        assert_eq!(payload.totp_code, Some("123456".to_string()));
    }

    #[test]
    fn test_login_payload_deserialize_no_totp() {
        let json = r#"{
            "username": "admin",
            "password": "password123"
        }"#;
        
        let payload: LoginPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, "admin");
        assert_eq!(payload.password, "password123");
        assert_eq!(payload.totp_code, None);
    }

    #[test]
    fn test_login_response_serialize() {
        let response = LoginResponse {
            token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 86400,
            totp_required: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test_token"));
        assert!(json.contains("Bearer"));
        assert!(!json.contains("totp_required")); // skip_serializing_if
    }
}
