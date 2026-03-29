//! User TOTP - TOTP верификация
//!
//! Аналог db/sql/user.go из Go версии (часть 3: TOTP)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_totp(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает TOTP пользователя
    pub async fn get_user_totp(&self, user_id: i32) -> Result<Option<TotpVerification>> {
        let row = sqlx::query("SELECT totp FROM \"user\" WHERE id = $1")
            .bind(user_id)
            .fetch_one(self.pg_pool_totp()?)
            .await
            .map_err(Error::Database)?;

        let totp_json: Option<String> = row.try_get("totp").ok().flatten();

        if let Some(totp_str) = totp_json {
            if totp_str.is_empty() {
                return Ok(None);
            }

            let totp: TotpVerification = serde_json::from_str(&totp_str)
                .map_err(|e| Error::Other(format!("Failed to parse TOTP: {}", e)))?;

            Ok(Some(totp))
        } else {
            Ok(None)
        }
    }

    /// Устанавливает TOTP для пользователя
    pub async fn set_user_totp(&self, user_id: i32, totp: &TotpVerification) -> Result<()> {
        let totp_json = serde_json::to_string(totp)
            .map_err(|e| Error::Other(format!("Failed to serialize TOTP: {}", e)))?;

        sqlx::query("UPDATE \"user\" SET totp = $1 WHERE id = $2")
            .bind(&totp_json)
            .bind(user_id)
            .execute(self.pg_pool_totp()?)
            .await
            .map_err(Error::Database)?;

        Ok(())
    }

    /// Удаляет TOTP у пользователя
    pub async fn delete_user_totp(&self, user_id: i32) -> Result<()> {
        sqlx::query("UPDATE \"user\" SET totp = NULL WHERE id = $1")
            .bind(user_id)
            .execute(self.pg_pool_totp()?)
            .await
            .map_err(Error::Database)?;

        Ok(())
    }

    /// Проверяет TOTP код
    pub async fn verify_totp_code(&self, user_id: i32, code: &str) -> Result<bool> {
        use crate::services::totp;

        if let Some(totp) = self.get_user_totp(user_id).await? {
            let is_valid = totp::verify_totp_code(&totp.secret, code);
            Ok(is_valid)
        } else {
            Ok(false)
        }
    }

    /// Проверяет recovery code
    pub async fn verify_recovery_code(&self, user_id: i32, code: &str) -> Result<bool> {
        if let Some(totp) = self.get_user_totp(user_id).await? {
            let is_valid = bcrypt::verify(code, &totp.recovery_hash)
                .unwrap_or(false);
            Ok(is_valid)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_verification() {
        use crate::services::totp;
        use crate::models::User;
        use chrono::Utc;

        // Создаём тестового пользователя
        let user = User {
            id: 1,
            created: Utc::now(),
            username: "testuser".to_string(),
            name: "Test".to_string(),
            email: "test@example.com".to_string(),
            password: String::new(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };

        // Генерируем секрет
        let totp_secret = totp::generate_totp_secret(&user, "Velum").unwrap();
        assert!(!totp_secret.secret.is_empty());

        // Генерируем код
        let code = totp::generate_totp_code(&totp_secret.secret).unwrap();
        assert!(!code.is_empty());

        // Проверяем код
        let is_valid = totp::verify_totp(&totp_secret.secret, &code);
        assert!(is_valid);
    }

    #[test]
    fn test_recovery_code_verification() {
        use crate::services::totp;

        // Генерируем recovery code
        let (code, hash) = totp::generate_recovery_code().unwrap();
        assert!(!code.is_empty());
        assert!(!hash.is_empty());

        // Проверяем код
        let is_valid = totp::verify_recovery_code(&code, &hash);
        assert!(is_valid);

        // Проверяем неправильный код
        let is_valid = totp::verify_recovery_code("wrong_code", &hash);
        assert!(!is_valid);
    }
}
