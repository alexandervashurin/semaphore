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
use crate::db::store::{UserManager, RetrieveQueryParams};

/// Контроллер пользователей
pub struct UsersController {
    /// Сервис подписок (опционально)
    pub subscription_service: Option<()>,
}

impl UsersController {
    /// Создаёт новый контроллер
    pub fn new() -> Self {
        Self {
            subscription_service: None,
        }
    }

    /// Получает список пользователей
    pub async fn get_users(
        State(state): State<Arc<AppState>>,
        AuthUser { user_id, admin, .. }: AuthUser,
        Query(params): Query<RetrieveQueryParams>,
    ) -> Result<Json<Vec<User>>> {
        let users = state.store.get_users(params).await?;

        // Если текущий пользователь не админ, возвращаем только базовую информацию
        if !admin {
            // TODO: Вернуть минимальную информацию о пользователе
            return Ok(Json(users));
        }

        Ok(Json(users))
    }

    /// Добавляет нового пользователя
    pub async fn add_user(
        State(state): State<Arc<AppState>>,
        AuthUser { admin, .. }: AuthUser,
        Json(user): Json<UserWithPwd>,
    ) -> Result<(StatusCode, Json<User>)> {
        // Проверяем права администратора
        if !admin {
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
        // let new_user = if user.external {
        //     state.store.create_user_without_password(user.user).await?
        // } else {
        //     state.store.create_user(user.user, "").await?
        // };
        let new_user = state.store.create_user(user.user, "").await?;

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
        AuthUser { user_id, admin, .. }: AuthUser,
        Path(update_user_id): Path<i32>,
        Json(user): Json<User>,
    ) -> Result<Json<User>> {
        // Проверяем права (пользователь может редактировать только себя или админ может всех)
        if !admin && user_id != update_user_id {
            return Err(Error::Other("User is not permitted to update other users".to_string()));
        }

        let mut user_to_update = state.store.get_user(update_user_id).await?;

        // Обновляем поля
        user_to_update.name = user.name;
        user_to_update.email = user.email;

        state.store.update_user(user_to_update.clone()).await?;
        Ok(Json(user_to_update))
    }

    /// Удаляет пользователя
    pub async fn delete_user(
        State(state): State<Arc<AppState>>,
        AuthUser { admin, .. }: AuthUser,
        Path(user_id): Path<i32>,
    ) -> Result<StatusCode> {
        // Проверяем права
        if !admin {
            return Err(Error::Other("User is not permitted to delete users".to_string()));
        }

        state.store.delete_user(user_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    /// Создаёт TOTP секрет для пользователя
    pub async fn create_totp(
        State(state): State<Arc<AppState>>,
        AuthUser { user_id, .. }: AuthUser,
    ) -> Result<Json<TotpSecretResponse>> {
        use crate::services::totp::generate_totp_secret;

        // Получаем пользователя
        let user = state.store.get_user(user_id).await?;

        // Если TOTP уже настроен, возвращаем ошибку
        if user.totp.is_some() {
            return Err(Error::Other("TOTP already enabled".to_string()));
        }

        // Генерируем секрет
        let totp_secret = generate_totp_secret(&user, "Velum UI")?;

        Ok(Json(TotpSecretResponse {
            secret: totp_secret.secret,
            url: totp_secret.url,
        }))
    }

    /// Проверяет TOTP код
    pub async fn verify_totp(
        State(state): State<Arc<AppState>>,
        AuthUser { user_id, .. }: AuthUser,
        Json(request): Json<TotpVerifyRequest>,
    ) -> Result<StatusCode> {
        use crate::services::totp::verify_totp_code;

        // Получаем пользователя
        let user = state.store.get_user(user_id).await?;

        // Проверяем что TOTP настроен
        let totp = user.totp
            .ok_or_else(|| Error::Other("TOTP not enabled".to_string()))?;

        // Проверяем код
        let is_valid = verify_totp_code(&totp.url, &request.passcode);
        if !is_valid {
            return Err(Error::Other("Invalid TOTP code".to_string()));
        }

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
