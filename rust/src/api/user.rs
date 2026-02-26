//! User API - управление текущим пользователем и API токенами
//!
//! Аналог api/user.go из Go версии

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::api::extractors::AuthUser;
use crate::error::{Error, Result};
use crate::models::{User, APIToken};

/// Контроллер пользователя
pub struct UserController {
    // TODO: Интеграция с subscription service
}

impl UserController {
    /// Создаёт новый контроллер
    pub fn new() -> Self {
        Self {}
    }

    /// Получает текущего пользователя
    pub async fn get_user(
        State(state): State<Arc<AppState>>,
        AuthUser(user): AuthUser,
    ) -> Result<Json<UserResponse>> {
        // Получаем полную информацию о пользователе
        let full_user = state.store.get_user(user.user_id).await?;

        let response = UserResponse {
            user: full_user,
            can_create_project: user.admin || state.config.non_admin_can_create_project,
            has_active_subscription: false, // TODO: Интеграция с subscription service
        };

        Ok(Json(response))
    }

    /// Получает API токены пользователя
    pub async fn get_api_tokens(
        State(state): State<Arc<AppState>>,
        AuthUser(user): AuthUser,
    ) -> Result<Json<Vec<APIToken>>> {
        let tokens = state.store.get_api_tokens(user.user_id).await?;

        // Обрезаем ID токенов до 8 символов для безопасности
        let mut result = Vec::new();
        for mut token in tokens {
            if token.id.len() >= 8 {
                token.id = token.id[..8].to_string();
            }
            result.push(token);
        }

        Ok(Json(result))
    }

    /// Создаёт новый API токен
    pub async fn create_api_token(
        State(state): State<Arc<AppState>>,
        AuthUser(user): AuthUser,
    ) -> Result<(StatusCode, Json<APIToken>)> {
        // Генерируем случайный ID токена
        let mut token_bytes = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut token_bytes);
        let token_id = base64::encode(&token_bytes);

        let token = state.store.create_api_token(APIToken {
            id: token_id.to_lowercase(),
            user_id: user.user_id,
            expired: false,
        }).await?;

        Ok((StatusCode::CREATED, Json(token)))
    }

    /// Удаляет API токен
    pub async fn delete_api_token(
        State(state): State<Arc<AppState>>,
        AuthUser(user): AuthUser,
        Path(token_id): Path<String>,
    ) -> Result<StatusCode> {
        state.store.delete_api_token(user.user_id, &token_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    /// Обновляет профиль пользователя
    pub async fn update_profile(
        State(state): State<Arc<AppState>>,
        AuthUser(user): AuthUser,
        Json(profile): Json<UserProfileUpdate>,
    ) -> Result<Json<User>> {
        // Получаем текущего пользователя
        let mut current_user = state.store.get_user(user.user_id).await?;

        // Обновляем поля
        if let Some(name) = profile.name {
            current_user.name = name;
        }

        if let Some(email) = profile.email {
            current_user.email = email;
        }

        // Сохраняем изменения
        let updated_user = state.store.update_user(current_user).await?;
        Ok(Json(updated_user))
    }

    /// Меняет пароль пользователя
    pub async fn change_password(
        State(state): State<Arc<AppState>>,
        AuthUser(user): AuthUser,
        Json(request): Json<PasswordChangeRequest>,
    ) -> Result<StatusCode> {
        // Получаем текущего пользователя
        let mut current_user = state.store.get_user(user.user_id).await?;

        // Проверяем старый пароль
        let valid = crate::api::auth::verify_password(&request.old_password, &current_user.password);
        if !valid {
            return Err(Error::Other("Invalid old password".to_string()));
        }

        // Хешируем новый пароль
        let new_hash = crate::api::auth::hash_password(&request.new_password)?;
        current_user.password = new_hash;

        // Сохраняем изменения
        state.store.update_user(current_user).await?;

        Ok(StatusCode::NO_CONTENT)
    }
}

impl Default for UserController {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Типы данных
// ============================================================================

/// Ответ с информацией о пользователе
#[derive(Debug, Serialize)]
pub struct UserResponse {
    #[serde(flatten)]
    pub user: User,
    /// Может ли создавать проекты
    pub can_create_project: bool,
    /// Есть ли активная подписка
    pub has_active_subscription: bool,
}

/// Обновление профиля пользователя
#[derive(Debug, Deserialize)]
pub struct UserProfileUpdate {
    /// Имя пользователя
    pub name: Option<String>,
    /// Email пользователя
    pub email: Option<String>,
}

/// Запрос на смену пароля
#[derive(Debug, Deserialize)]
pub struct PasswordChangeRequest {
    /// Старый пароль
    pub old_password: String,
    /// Новый пароль
    pub new_password: String,
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_controller_creation() {
        let controller = UserController::new();
        assert!(controller.subscription_service.is_none());
    }

    #[test]
    fn test_user_profile_update_serialization() {
        let update = UserProfileUpdate {
            name: Some("New Name".to_string()),
            email: Some("new@example.com".to_string()),
        };

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("New Name"));
        assert!(json.contains("new@example.com"));
    }

    #[test]
    fn test_password_change_request_deserialization() {
        let json = r#"{"old_password": "old123", "new_password": "new456"}"#;
        let request: PasswordChangeRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.old_password, "old123");
        assert_eq!(request.new_password, "new456");
    }
}
