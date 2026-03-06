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
use crate::api::extractors::AuthUser;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;
use crate::db::store::UserManager;

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

/// Верификация сессии (TOTP)
///
/// POST /api/auth/verify
pub async fn verify_session(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifySessionPayload>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    use crate::api::auth_local::{LocalAuthService, verify_password};
    use crate::services::totp::verify_totp_code;

    // Находим пользователя по токену сессии
    // В реальной реализации нужно получить сессию по токену
    let user = state.store.get_user_by_login_or_email(&payload.username, &payload.username)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Пользователь не найден")
                    .with_code("USER_NOT_FOUND")),
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
            Json(ErrorResponse::new("Неверный пароль")
                .with_code("INVALID_PASSWORD")),
        ));
    }

    // Проверяем TOTP код
    if let Some(ref totp) = user.totp {
        if !verify_totp_code(&totp.url, &payload.verify_code) {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Неверный TOTP код")
                    .with_code("INVALID_TOTP")),
            ));
        }
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("TOTP не настроен")
                .with_code("TOTP_NOT_ENABLED")),
        ));
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

/// Восстановление доступа через recovery code
///
/// POST /api/auth/recovery
pub async fn recovery_session(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RecoverySessionPayload>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    use crate::api::auth_local::LocalAuthService;
    use crate::services::totp::verify_recovery_code;

    // Находим пользователя
    let user = state.store.get_user_by_login_or_email(&payload.username, &payload.username)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Пользователь не найден")
                    .with_code("USER_NOT_FOUND")),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Ошибка сервера")),
            ),
        })?;

    // Проверяем recovery code
    if let Some(ref totp) = user.totp {
        if !verify_recovery_code(&payload.recovery_code, &totp.recovery_hash) {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Неверный recovery код")
                    .with_code("INVALID_RECOVERY_CODE")),
            ));
        }

        // TODO: Отключаем TOTP после использования recovery code
        // state.store.delete_totp(user.id).await?;
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("TOTP не настроен")
                .with_code("TOTP_NOT_ENABLED")),
        ));
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

/// Текущий пользователь
///
/// GET /api/user/
/// Возвращает данные аутентифицированного пользователя с can_create_project и has_active_subscription
pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, admin, .. }: AuthUser,
) -> Result<Json<crate::api::user::UserResponse>, (StatusCode, Json<ErrorResponse>)> {
    let full_user = state.store.get_user(user_id).await.map_err(|e| {
        let (status, resp) = ErrorResponse::from_crate_error(&e);
        (status, Json(resp))
    })?;

    let response = crate::api::user::UserResponse {
        user: full_user,
        can_create_project: admin || state.config.non_admin_can_create_project(),
        has_active_subscription: false, // TODO: Интеграция с subscription service
    };

    Ok(Json(response))
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

/// Payload для верификации сессии
#[derive(Debug, Deserialize)]
pub struct VerifySessionPayload {
    pub username: String,
    pub password: String,
    pub verify_code: String,
}

/// Payload для восстановления через recovery code
#[derive(Debug, Deserialize)]
pub struct RecoverySessionPayload {
    pub username: String,
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
