//! Local Authentication Module
//!
//! Локальная аутентификация пользователей по паролю

use crate::error::{Error, Result};
use crate::models::User;
use crate::db::store::Store;
use std::sync::Arc;

/// Сервис локальной аутентификации
pub struct LocalAuthService {
    store: Arc<dyn Store + Send + Sync>,
}

impl LocalAuthService {
    /// Создаёт новый сервис локальной аутентификации
    pub fn new(store: Arc<dyn Store + Send + Sync>) -> Self {
        Self { store }
    }

    /// Аутентифицирует пользователя по логину и паролю
    pub async fn login(&self, username: &str, password: &str) -> Result<User> {
        // Находим пользователя по логину или email
        let user = self.store.get_user_by_login_or_email(username, username).await?;

        // Проверяем пароль
        if !verify_password(password, &user.password) {
            return Err(Error::Unauthorized("Invalid username or password".to_string()));
        }

        // Проверяем, не внешний ли это пользователь
        if user.external {
            return Err(Error::Unauthorized("External user cannot login with password".to_string()));
        }

        Ok(user)
    }

    /// Регистрирует нового пользователя
    pub async fn register(&self, username: &str, email: &str, name: &str, password: &str) -> Result<User> {
        use crate::models::User;
        use chrono::Utc;

        // Проверяем, существует ли уже пользователь с таким username или email
        match self.store.get_user_by_login_or_email(username, email).await {
            Ok(_) => {
                return Err(Error::Other("User with this username or email already exists".to_string()));
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

        // Сохраняем в БД
        let new_user = self.store.create_user(user).await?;

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
    bcrypt::hash(password, cost)
        .map_err(|e| Error::Other(format!("Password hashing error: {}", e)))
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
