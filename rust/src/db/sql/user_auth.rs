//! User Auth - аутентификация и пароли
//!
//! Аналог db/sql/user.go из Go версии (часть 2: аутентификация)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use bcrypt::{hash, verify, DEFAULT_COST};

impl SqlDb {
    fn pg_pool_user_auth(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Устанавливает пароль пользователя
    pub async fn set_user_password(&self, user_id: i32, password: &str) -> Result<()> {
        let hashed_password = hash_password(password)?;

        sqlx::query("UPDATE \"user\" SET password = $1 WHERE id = $2")
            .bind(&hashed_password)
            .bind(user_id)
            .execute(self.pg_pool_user_auth()?)
            .await
            .map_err(Error::Database)?;

        Ok(())
    }

    /// Проверяет пароль пользователя
    pub async fn verify_user_password(&self, user_id: i32, password: &str) -> Result<bool> {
        let user = self.get_user(user_id).await?;

        let is_valid = verify_password(password, &user.password)?;

        Ok(is_valid)
    }
}

/// Хеширует пароль
pub fn hash_password(password: &str) -> Result<String> {
    hash(password, DEFAULT_COST)
        .map_err(|e| Error::Other(format!("Failed to hash password: {}", e)))
}

/// Проверяет пароль
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    verify(password, hash).map_err(|e| Error::Other(format!("Failed to verify password: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test_password123";
        let hashed = hash_password(password).unwrap();

        assert_ne!(password, hashed);
        assert!(hashed.starts_with("$2"));
    }

    #[test]
    fn test_verify_password_valid() {
        let password = "test_password123";
        let hashed = hash_password(password).unwrap();

        let is_valid = verify_password(password, &hashed).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_password_invalid() {
        let password = "test_password123";
        let hashed = hash_password(password).unwrap();

        let is_valid = verify_password("wrong_password", &hashed).unwrap();
        assert!(!is_valid);
    }
}
