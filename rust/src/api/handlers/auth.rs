//! Authentication Handlers
//!
//! Обработчики запросов для аутентификации

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::api::extractors::AuthUser;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;
use crate::db::store::{UserManager, LdapGroupMappingManager, ProjectStore};
use crate::models::ProjectUser;

/// Health check endpoint
pub async fn health() -> &'static str {
    "OK"
}

/// Расширенная проверка здоровья сервиса
/// 
/// GET /api/health/live
pub async fn health_live(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    use serde_json::json;
    
    // Проверка подключения к БД
    let db_status = match state.store.ping().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };
    
    let status = if db_status == "connected" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (
        status,
        Json(json!({
            "status": if status == StatusCode::OK { "healthy" } else { "unhealthy" },
            "database": db_status,
            "version": env!("CARGO_PKG_VERSION"),
        }))
    )
}

/// Готовность сервиса к приёму запросов
/// 
/// GET /api/health/ready
pub async fn health_ready(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    use serde_json::json;
    
    // Проверка подключения к БД
    let db_ready = state.store.ping().await.is_ok();
    
    let status = if db_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (
        status,
        Json(json!({
            "ready": db_ready,
            "checks": {
                "database": db_ready,
            }
        }))
    )
}

/// Вход в систему
///
/// POST /api/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    use crate::api::auth_local::{LocalAuthService, verify_password};
    use crate::services::totp::verify_totp_code;

    tracing::info!("Login attempt for user: {}", payload.username);

    // Пробуем локальную аутентификацию
    let user_result = state.store
        .get_user_by_login_or_email(&payload.username, &payload.username)
        .await;

    let user = match user_result {
        Ok(u) => {
            tracing::info!("User found locally: id={}, username={}", u.id, u.username);

            // Проверяем пароль
            if !verify_password(&payload.password, &u.password) {
                tracing::warn!("Invalid local password for user: {}", u.username);
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse::new("Неверный логин или пароль")
                        .with_code("INVALID_CREDENTIALS")),
                ).into_response();
            }
            u
        }
        Err(_) => {
            // Локальный пользователь не найден — пробуем LDAP если включён
            let ldap_cfg = state.config.ldap_config();
            if ldap_cfg.is_enabled() {
                tracing::info!("Trying LDAP auth for user: {}", payload.username);
                match crate::api::auth_ldap::ldap_authenticate(
                    &ldap_cfg,
                    &payload.username,
                    &payload.password,
                ).await {
                    Ok(ldap_user) => {
                        // LDAP auth прошла — найти или создать пользователя локально
                        let ldap_local_user = match state.store
                            .get_user_by_login_or_email(&ldap_user.username, &ldap_user.email)
                            .await
                        {
                            Ok(existing) => existing,
                            Err(_) => {
                                // Создаём нового пользователя из LDAP
                                use crate::models::User;
                                use chrono::Utc;
                                let new_user = User {
                                    id: 0,
                                    created: Utc::now(),
                                    username: ldap_user.username.clone(),
                                    name: ldap_user.name.clone(),
                                    email: ldap_user.email.clone(),
                                    password: String::new(), // LDAP пользователь без локального пароля
                                    admin: false,
                                    external: true,
                                    alert: true,
                                    pro: false,
                                    totp: None,
                                    email_otp: None,
                                };
                                match state.store.create_user(new_user, "").await {
                                    Ok(u) => {
                                        tracing::info!("Created LDAP user: {}", u.username);
                                        u
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to create LDAP user: {}", e);
                                        return (
                                            StatusCode::INTERNAL_SERVER_ERROR,
                                            Json(ErrorResponse::new("Ошибка создания пользователя")
                                                .with_code("USER_CREATION_ERROR")),
                                        ).into_response();
                                    }
                                }
                            }
                        };
                        // Синхронизируем LDAP-группы → команды проектов
                        if !ldap_user.groups.is_empty() {
                            if let Ok(mappings) = state.store.get_mappings_for_groups(&ldap_user.groups).await {
                                for mapping in mappings {
                                    let role = match mapping.role.as_str() {
                                        "owner" => crate::models::ProjectUserRole::Owner,
                                        "manager" => crate::models::ProjectUserRole::Manager,
                                        "guest" => crate::models::ProjectUserRole::Guest,
                                        _ => crate::models::ProjectUserRole::TaskRunner,
                                    };
                                    let pu = ProjectUser::new(mapping.project_id, ldap_local_user.id, role);
                                    let _ = state.store.create_project_user(pu).await;
                                    tracing::debug!(
                                        "LDAP sync: user {} → project {} ({})",
                                        ldap_local_user.username, mapping.project_id, mapping.role
                                    );
                                }
                            }
                        }
                        ldap_local_user
                    }
                    Err(e) => {
                        tracing::warn!("LDAP auth failed for {}: {}", payload.username, e);
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(ErrorResponse::new("Неверный логин или пароль")
                                .with_code("INVALID_CREDENTIALS")),
                        ).into_response();
                    }
                }
            } else {
                tracing::warn!("User not found: {}", payload.username);
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse::new("Неверный логин или пароль")
                        .with_code("INVALID_CREDENTIALS")),
                ).into_response();
            }
        }
    };

    // Проверяем TOTP, если настроен
    if let Some(ref totp) = user.totp {
        let totp_code = match payload.totp_code {
            Some(code) => code,
            None => return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Требуется TOTP код")
                    .with_code("TOTP_REQUIRED")),
            ).into_response(),
        };

        if !verify_totp_code(&totp.url, &totp_code) {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Неверный TOTP код")
                    .with_code("INVALID_TOTP")),
            ).into_response();
        }
    }

    // Генерируем токен
    let auth_service = LocalAuthService::new(state.store.clone());
    let token_info = match auth_service.generate_token(&user) {
        Ok(info) => info,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка генерации токена: {}", e))
                .with_code("TOKEN_GENERATION_ERROR")),
        ).into_response(),
    };

    // Устанавливаем cookie "semaphore" для Vue upstream (как в Go backend)
    let cookie_value = format!(
        "semaphore={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        token_info.token,
        token_info.expires_in
    );

    let headers = AppendHeaders([(header::SET_COOKIE, cookie_value)]);

    (
        headers,
        Json(LoginResponse {
            token: token_info.token,
            token_type: token_info.token_type,
            expires_in: token_info.expires_in,
            refresh_token: token_info.refresh_token,
            refresh_expires_in: token_info.refresh_expires_in,
            totp_required: None,
        })
    ).into_response()
}

/// Выход из системы
///
/// POST /api/auth/logout
pub async fn logout(
    State(_state): State<Arc<AppState>>,
) -> Result<
    (AppendHeaders<[(axum::http::HeaderName, &'static str); 1]>, StatusCode),
    (StatusCode, Json<ErrorResponse>),
> {
    // Очищаем cookie для Vue (как в Go backend)
    let headers = AppendHeaders([(
        header::SET_COOKIE,
        "semaphore=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
    )]);
    Ok((headers, StatusCode::OK))
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
        refresh_token: token_info.refresh_token,
        refresh_expires_in: token_info.refresh_expires_in,
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

        // Отключаем TOTP после использования recovery code
        state.store.delete_user_totp(user.id)
            .await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Ошибка отключения TOTP: {}", e))),
            ))?;
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
        refresh_token: token_info.refresh_token,
        refresh_expires_in: token_info.refresh_expires_in,
        totp_required: None,
    }))
}

/// Обновление access token по refresh token
///
/// POST /api/auth/refresh
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshPayload>,
) -> impl IntoResponse {
    use crate::api::auth_local::LocalAuthService;

    let auth_service = LocalAuthService::new(state.store.clone());

    // Верифицируем refresh token
    let user_id = match auth_service.verify_refresh_token(&payload.refresh_token) {
        Ok(id) => id,
        Err(_) => return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Недействительный refresh token")
                .with_code("INVALID_REFRESH_TOKEN")),
        ).into_response(),
    };

    // Загружаем пользователя (проверяем что он ещё существует и активен)
    let user = match state.store.get_user(user_id).await {
        Ok(u) => u,
        Err(_) => return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Пользователь не найден")
                .with_code("USER_NOT_FOUND")),
        ).into_response(),
    };

    // Генерируем новую пару токенов
    let token_info = match auth_service.generate_token(&user) {
        Ok(info) => info,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка генерации токена: {}", e))
                .with_code("TOKEN_GENERATION_ERROR")),
        ).into_response(),
    };

    Json(LoginResponse {
        token: token_info.token,
        token_type: token_info.token_type,
        expires_in: token_info.expires_in,
        refresh_token: token_info.refresh_token,
        refresh_expires_in: token_info.refresh_expires_in,
        totp_required: None,
    }).into_response()
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

/// Payload для входа (Vue отправляет auth, Go — auth)
#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    #[serde(alias = "auth")]
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
    pub refresh_token: String,
    pub refresh_expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp_required: Option<bool>,
}

/// Payload для обновления токена
#[derive(Debug, Deserialize)]
pub struct RefreshPayload {
    pub refresh_token: String,
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
    fn test_login_payload_deserialize_auth_alias() {
        // Vue отправляет "auth" вместо "username"
        let json = r#"{
            "auth": "admin",
            "password": "admin123"
        }"#;
        
        let payload: LoginPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, "admin");
        assert_eq!(payload.password, "admin123");
    }

    #[test]
    fn test_login_response_serialize() {
        let response = LoginResponse {
            token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 86400,
            refresh_token: "refresh_token".to_string(),
            refresh_expires_in: 86400 * 30,
            totp_required: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test_token"));
        assert!(json.contains("Bearer"));
        assert!(json.contains("refresh_token"));
        assert!(!json.contains("totp_required")); // skip_serializing_if
    }
}
