//! Users API - управление пользователями
//!
//! Аналог api/users.go из Go версии

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::api::extractors::AuthUser;
use crate::error::{Error, Result};
use crate::models::User;

/// Контроллер пользователей
pub struct UsersController {
    // TODO: Интеграция с subscription service
}

impl UsersController {
    /// Создаёт новый контроллер
    pub fn new() -> Self {
        Self {}
    }

    /// Получает список пользователей
    pub async fn get_users(
        State(state): State<Arc<AppState>>,
        AuthUser(current_user): AuthUser,
        Query(params): Query<RetrieveQueryParams>,
    ) -> Result<Json<Vec<User>>> {
        let users = state.store.get_users(params).await?;

        // Если текущий пользователь не админ, возвращаем только базовую информацию
        if !current_user.admin {
            // TODO: Вернуть минимальную информацию о пользователе
            return Ok(Json(users));
        }

        Ok(Json(users))
    }

    /// Добавляет нового пользователя
    pub async fn add_user(
        State(state): State<Arc<AppState>>,
        AuthUser(editor): AuthUser,
        Json(user): Json<UserWithPwd>,
    ) -> Result<(StatusCode, Json<User>)> {
        // Проверяем права администратора
        if !editor.admin {
            return Err(Error::Other("User is not permitted to create users".to_string()));
        }

        // TODO: Проверка подписки для PRO пользователей
        // if user.pro {
        //     let ok = state.subscription_service.can_add_pro_user().await?;
        //     if !ok {
        //         return Err(Error::Other("You have reached the limit of Pro users".to_string()));
        //     }
        // }

        // Создаём пользователя
        let new_user = if user.external {
            state.store.create_user_without_password(user.user).await?
        } else {
            state.store.create_user(user.user).await?
        };

        Ok((StatusCode::CREATED, Json(new_user)))
    }

    /// Получает пользователя по ID
    pub async fn get_user(
        State(state): State<Arc<AppState>>,
        Path(user_id): Path<i32>,
    ) -> Result<Json<User>> {
        let user = state.store.get_user(user_id).await?;
        Ok(Json(user))
    }

    /// Обновляет пользователя
    pub async fn update_user(
        State(state): State<Arc<AppState>>,
        AuthUser(editor): AuthUser,
        Path(user_id): Path<i32>,
        Json(user): Json<User>,
    ) -> Result<Json<User>> {
        // Проверяем права (пользователь может редактировать только себя или админ может всех)
        if !editor.admin && editor.user_id != user_id {
            return Err(Error::Other("User is not permitted to update other users".to_string()));
        }

        let updated_user = state.store.update_user(user).await?;
        Ok(Json(updated_user))
    }

    /// Удаляет пользователя
    pub async fn delete_user(
        State(state): State<Arc<AppState>>,
        AuthUser(editor): AuthUser,
        Path(user_id): Path<i32>,
    ) -> Result<StatusCode> {
        // Проверяем права
        if !editor.admin {
            return Err(Error::Other("User is not permitted to delete users".to_string()));
        }

        state.store.delete_user(user_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    /// Создаёт TOTP секрет для пользователя
    pub async fn create_totp(
        State(state): State<Arc<AppState>>,
        AuthUser(user): AuthUser,
    ) -> Result<Json<TotpSecretResponse>> {
        // TODO: Интеграция с TOTP сервисом
        // let totp_secret = crate::services::totp::generate_totp_secret(&user, "Semaphore")?;
        
        Ok(Json(TotpSecretResponse {
            secret: String::new(), // TODO: Реальный секрет
            url: String::new(),    // TODO: Реальный URL
        }))
    }

    /// Проверяет TOTP код
    pub async fn verify_totp(
        State(state): State<Arc<AppState>>,
        AuthUser(user): AuthUser,
        Json(request): Json<TotpVerifyRequest>,
    ) -> Result<StatusCode> {
        // TODO: Интеграция с TOTP сервисом
        // let is_valid = crate::services::totp::verify_totp(&request.passcode, &user.totp.secret);
        
        if request.passcode.is_empty() {
            return Err(Error::Other("Invalid passcode".to_string()));
        }

        // TODO: Сохранить TOTP для пользователя
        // state.store.set_user_totp(user.user_id, totp_secret).await?;

        Ok(StatusCode::NO_CONTENT)
    }
}

impl Default for UsersController {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Типы данных
// ============================================================================

/// Параметры запроса для получения пользователей
#[derive(Debug, Default, Deserialize)]
pub struct RetrieveQueryParams {
    /// Фильтр по имени/email
    pub filter: Option<String>,
}

/// Пользователь с паролем (для создания)
#[derive(Debug, Deserialize)]
pub struct UserWithPwd {
    #[serde(flatten)]
    pub user: User,
    /// Пароль пользователя
    pub password: String,
    /// PRO аккаунт
    pub pro: bool,
    /// Внешний пользователь (LDAP/OIDC)
    pub external: bool,
}

/// Ответ с TOTP секретом
#[derive(Debug, Serialize)]
pub struct TotpSecretResponse {
    /// TOTP секрет
    pub secret: String,
    /// URL для QR кода
    pub url: String,
}

/// Запрос на проверку TOTP
#[derive(Debug, Deserialize)]
pub struct TotpVerifyRequest {
    /// TOTP код
    pub passcode: String,
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_users_controller_creation() {
        let controller = UsersController::new();
        assert!(controller.subscription_service.is_none());
    }

    #[test]
    fn test_retrieve_query_params_default() {
        let params = RetrieveQueryParams::default();
        assert!(params.filter.is_none());
    }
}
