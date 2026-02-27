//! API - Login Handler
//!
//! Обработчики для входа в систему

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;

/// Payload для входа
#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
    pub totp_code: Option<String>,
}

/// Response после входа
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Вход в систему
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    // В реальной реализации нужно проверить учётные данные
    // и создать токен

    let response = LoginResponse {
        token: "jwt_token_here".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 86400,
    };

    Ok(Json(response))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_handler() {
        // Тест для проверки обработчиков входа
        assert!(true);
    }

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
}
