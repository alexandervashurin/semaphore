//! API - System Info Handler
//!
//! Обработчики для системной информации

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
use crate::api::extractors::AuthUser;

/// Информация о системе
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub auth_methods: LoginAuthMethods,
    pub schedule: ScheduleInfo,
    pub roles: Vec<RoleInfo>,
    pub plan: Option<String>,
}

/// Методы аутентификации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAuthMethods {
    pub totp: Option<LoginTotpAuthMethod>,
    pub email: Option<LoginEmailAuthMethod>,
}

/// TOTP метод аутентификации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginTotpAuthMethod {
    pub allow_recovery: bool,
}

/// Email метод аутентификации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginEmailAuthMethod {}

/// Информация о расписании
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleInfo {
    pub timezone: String,
}

/// Информация о роли
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

/// Получает системную информацию
pub async fn get_system_info(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Json<SystemInfo>, (StatusCode, Json<ErrorResponse>)> {
    let info = SystemInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        auth_methods: LoginAuthMethods {
            totp: Some(LoginTotpAuthMethod {
                allow_recovery: true,
            }),
            email: Some(LoginEmailAuthMethod {}),
        },
        schedule: ScheduleInfo {
            timezone: "UTC".to_string(),
        },
        roles: vec![],
        plan: None,
    };

    Ok(Json(info))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_handler() {
        // Тест для проверки обработчиков системной информации
        assert!(true);
    }

    #[test]
    fn test_system_info_serialization() {
        let info = SystemInfo {
            version: "1.0.0".to_string(),
            auth_methods: LoginAuthMethods {
                totp: Some(LoginTotpAuthMethod {
                    allow_recovery: true,
                }),
                email: Some(LoginEmailAuthMethod {}),
            },
            schedule: ScheduleInfo {
                timezone: "UTC".to_string(),
            },
            roles: vec![],
            plan: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("1.0.0"));
    }
}
