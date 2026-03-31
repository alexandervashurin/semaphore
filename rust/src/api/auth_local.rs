//! Local Authentication Module
//!
//! Локальная аутентификация пользователей по паролю

use crate::api::store_wrapper::StoreWrapper;
use crate::db::store::{Store, UserManager};
use crate::error::{Error, Result};
use crate::models::User;
use std::sync::Arc;

/// Сервис локальной аутентификации
pub struct LocalAuthService {
    store: StoreWrapper,
}

/// Информация о токене
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub refresh_expires_in: i64,
}

/// Claims для JWT токена
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub username: String,
    pub email: String,
    pub admin: bool,
    pub exp: usize,
    pub iat: usize,
}

/// Claims для refresh токена (долгоживущий, без admin/email для минимального размера)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RefreshClaims {
    pub sub: i32,
    /// Тип токена — всегда "refresh", чтобы нельзя было использовать как access
    pub typ: String,
    pub exp: usize,
    pub iat: usize,
}

impl LocalAuthService {
    /// Создаёт новый сервис локальной аутентификации
    pub fn new(store: StoreWrapper) -> Self {
        Self { store }
    }

    /// Аутентифицирует пользователя по логину и паролю
    pub async fn login(&self, username: &str, password: &str) -> Result<User> {
        // Находим пользователя по логину или email
        let user = self
            .store
            .get_user_by_login_or_email(username, username)
            .await?;

        // Проверяем пароль
        if !verify_password(password, &user.password) {
            return Err(Error::Unauthorized(
                "Invalid username or password".to_string(),
            ));
        }

        // Проверяем, не внешний ли это пользователь
        if user.external {
            return Err(Error::Unauthorized(
                "External user cannot login with password".to_string(),
            ));
        }

        Ok(user)
    }

    /// Генерирует JWT access + refresh токены для пользователя
    pub fn generate_token(&self, user: &User) -> Result<TokenInfo> {
        use chrono::Utc;
        use jsonwebtoken::{encode, EncodingKey, Header};

        let secret = std::env::var("SEMAPHORE_JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string());
        let key = EncodingKey::from_secret(secret.as_bytes());

        let now = Utc::now().timestamp() as usize;

        // Access token — 24 часа
        const ACCESS_TTL: i64 = 86400;
        let access_claims = Claims {
            sub: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            admin: user.admin,
            exp: now + ACCESS_TTL as usize,
            iat: now,
        };
        let token = encode(&Header::default(), &access_claims, &key)
            .map_err(|e| Error::Other(format!("Token generation error: {}", e)))?;

        // Refresh token — 30 дней
        const REFRESH_TTL: i64 = 86400 * 30;
        let refresh_claims = RefreshClaims {
            sub: user.id,
            typ: "refresh".to_string(),
            exp: now + REFRESH_TTL as usize,
            iat: now,
        };
        let refresh_token = encode(&Header::default(), &refresh_claims, &key)
            .map_err(|e| Error::Other(format!("Refresh token generation error: {}", e)))?;

        Ok(TokenInfo {
            token,
            token_type: "Bearer".to_string(),
            expires_in: ACCESS_TTL,
            refresh_token,
            refresh_expires_in: REFRESH_TTL,
        })
    }

    /// Проверяет refresh token и возвращает user_id
    pub fn verify_refresh_token(&self, token: &str) -> Result<i32> {
        use jsonwebtoken::{decode, DecodingKey, Validation};

        let secret = std::env::var("SEMAPHORE_JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string());

        let token_data = decode::<RefreshClaims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| Error::Unauthorized(format!("Invalid refresh token: {}", e)))?;

        if token_data.claims.typ != "refresh" {
            return Err(Error::Unauthorized("Not a refresh token".to_string()));
        }

        Ok(token_data.claims.sub)
    }

    /// Проверяет JWT токен и возвращает claims
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        use jsonwebtoken::{decode, DecodingKey, Validation};

        // Получаем секретный ключ из окружения или используем дефолтный
        let secret = std::env::var("SEMAPHORE_JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string());

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| Error::Unauthorized(format!("Token verification error: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Регистрирует нового пользователя
    pub async fn register(
        &self,
        username: &str,
        email: &str,
        name: &str,
        password: &str,
    ) -> Result<User> {
        use crate::models::User;
        use chrono::Utc;

        // Проверяем, существует ли уже пользователь с таким username или email
        match self.store.get_user_by_login_or_email(username, email).await {
            Ok(_) => {
                return Err(Error::Other(
                    "User with this username or email already exists".to_string(),
                ));
            }
            Err(Error::NotFound(_)) => {
                // Пользователь не найден, продолжаем
            }
            Err(e) => return Err(e),
        }

        // Хешируем пароль
        let password_hash = hash_password(password)?;

        // Создаём нового пользователя
        let user = User {
            id: 0,
            created: Utc::now(),
            username: username.to_string(),
            name: name.to_string(),
            email: email.to_string(),
            password: password_hash,
            admin: false,
            external: false,
            alert: true,
            pro: false,
            totp: None,
            email_otp: None,
        };

        // Сохраняем в БД - create_user теперь требует password отдельным аргументом
        let new_user = self.store.create_user(user, password).await?;

        Ok(new_user)
    }
}

/// Проверяет пароль против хэша
pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

/// Хеширует пароль
pub fn hash_password(password: &str) -> Result<String> {
    let cost = 12; // bcrypt cost factor
    bcrypt::hash(password, cost).map_err(|e| Error::Other(format!("Password hashing error: {}", e)))
}

/// Меняет пароль пользователя
pub async fn change_password(
    store: &dyn Store,
    user_id: i32,
    old_password: &str,
    new_password: &str,
) -> Result<()> {
    // Получаем пользователя
    let mut user = store.get_user(user_id).await?;

    // Проверяем старый пароль
    if !verify_password(old_password, &user.password) {
        return Err(Error::Unauthorized("Invalid old password".to_string()));
    }

    // Хешируем новый пароль
    let new_hash = hash_password(new_password)?;
    user.password = new_hash;

    // Сохраняем изменения
    store.update_user(user).await?;

    Ok(())
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        // Проверяем, что хэш не равен паролю
        assert_ne!(hash, password);

        // Проверяем, что хэш имеет правильную длину
        assert_eq!(hash.len(), 60); // bcrypt hash length
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash));
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(!verify_password("wrong_password", &hash));
    }

    #[test]
    fn test_verify_password_empty() {
        assert!(!verify_password("", "any_hash"));
        assert!(!verify_password("password", ""));
    }
}
