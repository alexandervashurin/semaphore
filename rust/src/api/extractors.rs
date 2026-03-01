//! Пользовательские извлекатели Axum
//!
//! Предоставляет извлекатели для:
//! - Аутентифицированных пользователей
//! - Токенов доступа
//! - Заголовков запросов

use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    Json,
};
use std::sync::Arc;

use crate::api::auth_local::LocalAuthService;
use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;

/// Извлекает токен из заголовка Authorization
fn extract_token_from_header(auth_header: Option<&str>) -> Option<&str> {
    auth_header
        .and_then(|h| h.strip_prefix("Bearer "))
        .or(auth_header) // Если нет префикса Bearer, используем как есть
}

/// Извлекатель для аутентифицированного пользователя
///
/// Используется в обработчиках для получения информации о пользователе:
/// ```rust,ignore
/// # // Этот пример требует контекста Axum и не может быть запущен как doctest
/// pub async fn handler(
///     auth_user: AuthUser,
/// ) -> axum::http::StatusCode {
///     println!("Пользователь: {}", auth_user.username);
///     axum::http::StatusCode::OK
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub admin: bool,
}

impl FromRequestParts<State<Arc<AppState>>> for AuthUser {
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &State<Arc<AppState>>,
    ) -> Result<Self, Self::Rejection> {
        // Получаем токен из заголовка
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let token = extract_token_from_header(auth_header)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Требуется аутентификация")
                    .with_code("AUTH_REQUIRED")),
            ))?;

        // Получаем LocalAuthService из состояния
        let auth_service = LocalAuthService::new(std::sync::Arc::new(state.store.clone()));

        // Проверяем токен
        let claims = auth_service.verify_token(token)
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse::new("Неверный токен".to_string())
                        .with_code("AUTH_FAILED")),
                )
            })?;

        Ok(AuthUser {
            user_id: claims.sub,
            username: claims.username,
            email: claims.email,
            admin: claims.admin,
        })
    }
}

/// Извлекатель для опционано аутентифицированного пользователя
///
/// Возвращает None, если пользователь не аутентифицирован
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<State<Arc<AppState>>> for OptionalAuthUser {
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &State<Arc<AppState>>,
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}

/// Извлекатель для JWT токена
///
/// Извлекает сырой JWT токен из заголовка
#[derive(Debug, Clone)]
pub struct AuthToken(pub String);

impl FromRequestParts<State<Arc<AppState>>> for AuthToken {
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &State<Arc<AppState>>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let token = extract_token_from_header(auth_header)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Требуется аутентификация")
                    .with_code("AUTH_REQUIRED")),
            ))?;

        Ok(AuthToken(token.to_string()))
    }
}

/// Извлекатель для проверки административных прав
///
/// Возвращает ошибку, если пользователь не является администратором
#[derive(Debug, Clone)]
pub struct AdminUser(AuthUser);

impl AdminUser {
    pub fn into_inner(self) -> AuthUser {
        self.0
    }
}

impl FromRequestParts<State<Arc<AppState>>> for AdminUser {
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &State<Arc<AppState>>,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;

        if !user.admin {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse::new("Требуется права администратора")
                    .with_code("ADMIN_REQUIRED")),
            ));
        }

        Ok(AdminUser(user))
    }
}

// Ре-экспорт для удобства

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_header() {
        let header = Some("Bearer token123");
        let token = crate::api::auth::extract_token_from_header(header);
        assert_eq!(token, Some("token123"));
    }

    #[test]
    fn test_extract_token_from_invalid_header() {
        let header = Some("Basic token123");
        let token = crate::api::auth::extract_token_from_header(header);
        assert_eq!(token, None);
    }

    #[test]
    fn test_auth_user_structure() {
        let user = AuthUser {
            user_id: 1,
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            admin: true,
        };

        assert_eq!(user.user_id, 1);
        assert!(user.admin);
    }
}
