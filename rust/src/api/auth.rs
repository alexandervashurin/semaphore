//! Модуль аутентификации и авторизации
//!
//! Предоставляет:
//! - JWT токены для аутентификации
//! - Проверку и валидацию токенов
//! - Управление сессиями

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::models::User;

/// Секретный ключ для JWT (в production должен загружаться из конфига)
const JWT_SECRET: &str = "semaphore-jwt-secret-key-change-in-production";

/// Время жизни токена (24 часа)
const TOKEN_EXPIRY_HOURS: i64 = 24;

/// Claims для JWT токена
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// ID пользователя
    pub sub: i32,
    /// Имя пользователя
    pub username: String,
    /// Email
    pub email: String,
    /// Администратор
    pub admin: bool,
    /// Время истечения
    pub exp: usize,
    /// Время выпуска
    pub iat: usize,
}

/// Информация о токене
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// JWT токен
    pub token: String,
    /// Тип токена
    pub token_type: String,
    /// Время жизни в секундах
    pub expires_in: i64,
}

/// Сервис аутентификации
pub struct AuthService {
    secret: String,
}

impl AuthService {
    /// Создаёт новый сервис аутентификации
    pub fn new() -> Self {
        Self {
            secret: JWT_SECRET.to_string(),
        }
    }

    /// Создаёт сервис с кастомным секретом
    pub fn with_secret(secret: String) -> Self {
        Self { secret }
    }

    /// Генерирует JWT токен для пользователя
    pub fn generate_token(&self, user: &User) -> Result<TokenInfo, AuthError> {
        let now = Utc::now();
        let expiry = now + Duration::hours(TOKEN_EXPIRY_HOURS);

        let claims = JwtClaims {
            sub: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            admin: user.admin,
            exp: expiry.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?;

        Ok(TokenInfo {
            token,
            token_type: "Bearer".to_string(),
            expires_in: TOKEN_EXPIRY_HOURS * 3600,
        })
    }

    /// Проверяет и декодирует JWT токен
    pub fn verify_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    /// Обновляет токен (выпускает новый)
    pub fn refresh_token(&self, old_token: &str) -> Result<TokenInfo, AuthError> {
        let claims = self.verify_token(old_token)?;
        
        // Создаём "фейкового" пользователя для генерации нового токена
        let user = User {
            id: claims.sub,
            created: Utc::now(),
            username: claims.username,
            name: String::new(),
            email: claims.email,
            password: String::new(),
            admin: claims.admin,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };

        self.generate_token(&user)
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

/// Ошибки аутентификации
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Неверный токен: {0}")]
    InvalidToken(String),

    #[error("Токен истёк")]
    TokenExpired,

    #[error("Токен ещё не действителен")]
    TokenNotYetValid,

    #[error("Ошибка генерации токена: {0}")]
    TokenGeneration(String),

    #[error("Пользователь не найден")]
    UserNotFound,

    #[error("Неверный логин или пароль")]
    InvalidCredentials,
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;
        match err.kind() {
            ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            ErrorKind::ImmatureSignature => AuthError::TokenNotYetValid,
            ErrorKind::InvalidToken => AuthError::InvalidToken("Неверный формат токена".to_string()),
            ErrorKind::InvalidSignature => AuthError::InvalidToken("Неверная подпись".to_string()),
            _ => AuthError::InvalidToken(err.to_string()),
        }
    }
}

/// Извлекает токен из заголовка Authorization
pub fn extract_token_from_header(auth_header: Option<&str>) -> Option<&str> {
    auth_header
        .filter(|h| h.starts_with("Bearer "))
        .map(|h| &h[7..])
}

/// Проверяет пароль пользователя
pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

/// Хеширует пароль
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user() -> User {
        User {
            id: 1,
            created: Utc::now(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: String::new(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        }
    }

    #[test]
    fn test_generate_and_verify_token() {
        let service = AuthService::new();
        let user = create_test_user();

        let token_info = service.generate_token(&user).unwrap();
        assert_eq!(token_info.token_type, "Bearer");
        assert_eq!(token_info.expires_in, TOKEN_EXPIRY_HOURS * 3600);

        let claims = service.verify_token(&token_info.token).unwrap();
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.email, user.email);
        assert_eq!(claims.admin, user.admin);
    }

    #[test]
    fn test_extract_token_from_header() {
        assert_eq!(
            extract_token_from_header(Some("Bearer token123")),
            Some("token123")
        );
        assert_eq!(extract_token_from_header(Some("Basic token123")), None);
        assert_eq!(extract_token_from_header(None), None);
    }

    #[test]
    fn test_password_hashing() {
        let password = "secure_password123";
        let hash = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hash));
        assert!(!verify_password("wrong_password", &hash));
    }

    #[test]
    fn test_invalid_token() {
        let service = AuthService::new();
        
        let result = service.verify_token("invalid_token");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            AuthError::InvalidToken(_) => (),
            _ => panic!("Ожидалась ошибка InvalidToken"),
        }
    }
}
