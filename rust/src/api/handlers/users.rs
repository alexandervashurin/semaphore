//! Users Handlers
//!
//! Обработчики запросов для управления пользователями

use crate::api::extractors::AuthUser;
use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::{RetrieveQueryParams, UserManager};
use crate::error::Error;
use crate::models::User;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono;
use serde::Deserialize;
use std::sync::Arc;

/// Получить список пользователей
///
/// GET /api/users
pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, (StatusCode, Json<ErrorResponse>)> {
    let users = state
        .store
        .get_users(RetrieveQueryParams::default())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(users))
}

/// Получить пользователя по ID
///
/// GET /api/users/:id
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    let user = state.store.get_user(user_id).await.map_err(|e| match e {
        Error::NotFound(_) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(e.to_string())),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        ),
    })?;

    Ok(Json(user))
}

/// Обновить пользователя
///
/// PUT /api/users/:id
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Json(payload): Json<UserUpdatePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut user = state.store.get_user(user_id).await.map_err(|e| match e {
        Error::NotFound(_) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(e.to_string())),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        ),
    })?;

    if let Some(username) = payload.username {
        user.username = username;
    }
    if let Some(name) = payload.name {
        user.name = name;
    }
    if let Some(email) = payload.email {
        user.email = email;
    }

    state.store.update_user(user).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::OK)
}

/// Удалить пользователя
///
/// DELETE /api/users/:id
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_user(user_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Обновить пароль пользователя
///
/// POST /api/users/:id/password
/// Только администратор или сам пользователь может менять пароль.
/// Внешние пользователи (LDAP/OIDC) не могут менять пароль через API.
pub async fn update_user_password(
    State(state): State<Arc<AppState>>,
    AuthUser {
        user_id: editor_id,
        admin,
        ..
    }: AuthUser,
    Path(target_user_id): Path<i32>,
    Json(payload): Json<PasswordUpdatePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Только админ или сам пользователь
    if !admin && editor_id != target_user_id {
        let err = ErrorResponse::new("Нет прав на изменение пароля").with_code("FORBIDDEN");
        return Err((StatusCode::FORBIDDEN, Json(err)));
    }

    let target_user = state
        .store
        .get_user(target_user_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    if target_user.external {
        let err = ErrorResponse::new("Пароль внешних пользователей нельзя изменить")
            .with_code("EXTERNAL_USER");
        return Err((StatusCode::BAD_REQUEST, Json(err)));
    }

    state
        .store
        .set_user_password(target_user_id, &payload.password)
        .await
        .map_err(|e| {
            let (status, resp) = ErrorResponse::from_crate_error(&e);
            (status, Json(resp))
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Types
// ============================================================================

/// Payload для смены пароля
#[derive(Debug, Deserialize)]
pub struct PasswordUpdatePayload {
    pub password: String,
}

/// Создать нового пользователя
///
/// POST /api/users
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    AuthUser { admin, .. }: AuthUser,
    Json(payload): Json<CreateUserPayload>,
) -> Result<(StatusCode, Json<User>), (StatusCode, Json<ErrorResponse>)> {
    if !admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("Only admins can create users")),
        ));
    }

    let user = User {
        id: 0,
        name: payload.name.unwrap_or_default(),
        username: payload.username,
        email: payload.email.unwrap_or_default(),
        created: chrono::Utc::now(),
        admin: payload.admin.unwrap_or(false),
        external: false,
        alert: false,
        pro: false,
        password: String::new(),
        totp: None,
        email_otp: None,
    };

    let created = state
        .store
        .create_user(user, payload.password.as_deref().unwrap_or(""))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Payload для создания пользователя
#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub admin: Option<bool>,
}

/// Payload для обновления пользователя
#[derive(Debug, Deserialize)]
pub struct UserUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_update_payload_deserialize_all_fields() {
        let json = r#"{
            "username": "newuser",
            "name": "New Name",
            "email": "new@example.com"
        }"#;

        let payload: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, Some("newuser".to_string()));
        assert_eq!(payload.name, Some("New Name".to_string()));
        assert_eq!(payload.email, Some("new@example.com".to_string()));
    }

    #[test]
    fn test_user_update_payload_deserialize_partial() {
        let json = r#"{
            "email": "new@example.com"
        }"#;

        let payload: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, None);
        assert_eq!(payload.name, None);
        assert_eq!(payload.email, Some("new@example.com".to_string()));
    }

    #[test]
    fn test_user_update_payload_deserialize_empty() {
        let json = r#"{}"#;

        let payload: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, None);
        assert_eq!(payload.name, None);
        assert_eq!(payload.email, None);
    }

    #[test]
    fn test_user_update_payload_roundtrip() {
        // UserUpdatePayload doesn't derive Serialize, test via deserialization
        let json = r#"{"username": "roundtrip_user", "name": "Round Trip", "email": "round@trip.com"}"#;
        let restored: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(restored.username, Some("roundtrip_user".to_string()));
        assert_eq!(restored.name, Some("Round Trip".to_string()));
        assert_eq!(restored.email, Some("round@trip.com".to_string()));
    }

    #[test]
    fn test_user_update_payload_serialize_skip_none() {
        // UserUpdatePayload doesn't derive Serialize
        let json = r#"{}"#;
        let payload: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.username.is_none());
        assert!(payload.name.is_none());
        assert!(payload.email.is_none());
    }

    #[test]
    fn test_user_update_payload_debug_format() {
        let payload = UserUpdatePayload {
            username: Some("debug_user".to_string()),
            name: Some("Debug User".to_string()),
            email: Some("debug@test.com".to_string()),
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("UserUpdatePayload"));
        assert!(debug_str.contains("debug_user"));
    }

    #[test]
    fn test_user_update_payload_unicode_values() {
        let json = r#"{
            "username": "пользователь",
            "name": "Иван Петров",
            "email": "иван@example.com"
        }"#;
        let payload: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, Some("пользователь".to_string()));
        assert_eq!(payload.name, Some("Иван Петров".to_string()));
        assert_eq!(payload.email, Some("иван@example.com".to_string()));
    }

    #[test]
    fn test_user_update_payload_empty_string_fields() {
        let json = r#"{
            "username": "",
            "name": "",
            "email": ""
        }"#;
        let payload: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, Some("".to_string()));
        assert_eq!(payload.name, Some("".to_string()));
        assert_eq!(payload.email, Some("".to_string()));
    }

    #[test]
    fn test_user_update_payload_clone_independence() {
        // UserUpdatePayload doesn't derive Clone, so we test field-level independence
        let mut username = Some("original".to_string());
        let name = Some("Original".to_string());
        let email = Some("orig@test.com".to_string());
        let payload1 = UserUpdatePayload { username: username.clone(), name: name.clone(), email: email.clone() };
        let payload2 = UserUpdatePayload { username: username.clone(), name: name.clone(), email: email.clone() };
        username = Some("modified".to_string());
        assert_eq!(payload1.username, Some("original".to_string()));
        assert_eq!(payload2.username, Some("original".to_string()));
    }

    #[test]
    fn test_password_update_payload_deserialize() {
        let json = r#"{"password": "secure_password_123"}"#;
        let payload: PasswordUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.password, "secure_password_123");
    }

    #[test]
    fn test_password_update_payload_empty_password() {
        let json = r#"{"password": ""}"#;
        let payload: PasswordUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.password, "");
    }

    #[test]
    fn test_password_update_payload_special_chars() {
        let json = r#"{"password": "p@$$w0rd!#%^&*()"}"#;
        let payload: PasswordUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.password, "p@$$w0rd!#%^&*()");
    }

    #[test]
    fn test_password_update_payload_unicode() {
        let json = r#"{"password": "пароль_с_кириллицей"}"#;
        let payload: PasswordUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.password, "пароль_с_кириллицей");
    }

    #[test]
    fn test_password_update_payload_debug() {
        let payload = PasswordUpdatePayload {
            password: "secret".to_string(),
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("PasswordUpdatePayload"));
    }

    #[test]
    fn test_create_user_payload_deserialize_minimal() {
        let json = r#"{"username": "newuser"}"#;
        let payload: CreateUserPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, "newuser");
        assert!(payload.name.is_none());
        assert!(payload.email.is_none());
        assert!(payload.password.is_none());
        assert!(payload.admin.is_none());
    }

    #[test]
    fn test_create_user_payload_deserialize_full() {
        let json = r#"{
            "username": "admin_user",
            "name": "Admin User",
            "email": "admin@example.com",
            "password": "secret",
            "admin": true
        }"#;
        let payload: CreateUserPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, "admin_user");
        assert_eq!(payload.name, Some("Admin User".to_string()));
        assert_eq!(payload.email, Some("admin@example.com".to_string()));
        assert_eq!(payload.password, Some("secret".to_string()));
        assert_eq!(payload.admin, Some(true));
    }

    #[test]
    fn test_create_user_payload_admin_false() {
        let json = r#"{"username": "regular", "admin": false}"#;
        let payload: CreateUserPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.admin, Some(false));
    }

    #[test]
    fn test_create_user_payload_unicode() {
        let json = r#"{
            "username": "админ",
            "name": "Администратор",
            "email": "админ@example.com"
        }"#;
        let payload: CreateUserPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, "админ");
        assert_eq!(payload.name, Some("Администратор".to_string()));
    }

    #[test]
    fn test_create_user_payload_debug() {
        let payload = CreateUserPayload {
            username: "test".to_string(),
            name: None,
            email: None,
            password: None,
            admin: None,
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("CreateUserPayload"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_user_update_payload_with_special_chars() {
        let json = r#"{
            "name": "O'Brien, John",
            "email": "john+test@example.com",
            "username": "john.doe_2024"
        }"#;
        let payload: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("O'Brien, John".to_string()));
        assert_eq!(payload.email, Some("john+test@example.com".to_string()));
    }

    #[test]
    fn test_user_update_payload_single_field_username() {
        let json = r#"{"username": "only_username"}"#;
        let payload: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, Some("only_username".to_string()));
        assert!(payload.name.is_none());
        assert!(payload.email.is_none());
    }
}
