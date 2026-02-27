//! User TOTP - TOTP верификация
//!
//! Аналог db/sql/user.go из Go версии (часть 3: TOTP)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает TOTP пользователя
    pub async fn get_user_totp(&self, user_id: i32) -> Result<Option<TotpVerification>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let user = sqlx::query("SELECT totp FROM user WHERE id = ?")
                    .bind(user_id)
                    .fetch_one(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                let totp_json: Option<String> = user.get(0);
                
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
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Устанавливает TOTP для пользователя
    pub async fn set_user_totp(&self, user_id: i32, totp: &TotpVerification) -> Result<()> {
        let totp_json = serde_json::to_string(totp)
            .map_err(|e| Error::Other(format!("Failed to serialize TOTP: {}", e)))?;
        
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("UPDATE user SET totp = ? WHERE id = ?")
                    .bind(&totp_json)
                    .bind(user_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Удаляет TOTP у пользователя
    pub async fn delete_user_totp(&self, user_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("UPDATE user SET totp = NULL WHERE id = ?")
                    .bind(user_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
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
    use std::env;
    use chrono::Utc;

    async fn create_test_db() -> SqlDb {
        let temp_db = env::temp_dir().join("test_user_totp.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();
        
        // Создаём таблицу user
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                password TEXT NOT NULL,
                admin INTEGER NOT NULL,
                external INTEGER NOT NULL,
                alert INTEGER NOT NULL,
                pro INTEGER NOT NULL,
                created DATETIME NOT NULL,
                totp TEXT,
                email_otp TEXT
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();
        
        db
    }

    #[tokio::test]
    async fn test_set_and_get_user_totp() {
        let db = create_test_db().await;
        
        let user = User {
            id: 0,
            created: Utc::now(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "hashed".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        
        let created = db.create_user(user).await.unwrap();
        
        // Создаём TOTP
        let totp = TotpVerification {
            secret: "test_secret".to_string(),
            recovery_hash: "test_hash".to_string(),
        };
        
        db.set_user_totp(created.id, &totp).await.unwrap();
        
        // Получаем TOTP
        let retrieved = db.get_user_totp(created.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().secret, "test_secret");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_user_totp() {
        let db = create_test_db().await;
        
        let user = User {
            id: 0,
            created: Utc::now(),
            username: "testuser2".to_string(),
            name: "Test User".to_string(),
            email: "test2@example.com".to_string(),
            password: "hashed".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        
        let created = db.create_user(user).await.unwrap();
        
        // Создаём TOTP
        let totp = TotpVerification {
            secret: "test_secret".to_string(),
            recovery_hash: "test_hash".to_string(),
        };
        
        db.set_user_totp(created.id, &totp).await.unwrap();
        
        // Удаляем TOTP
        db.delete_user_totp(created.id).await.unwrap();
        
        // Проверяем что TOTP удалён
        let retrieved = db.get_user_totp(created.id).await.unwrap();
        assert!(retrieved.is_none());
        
        // Cleanup
        let _ = db.close().await;
    }

    #[test]
    fn test_totp_verification() {
        use crate::services::totp;
        
        // Генерируем секрет
        let secret = totp::generate_totp_secret();
        assert!(!secret.is_empty());
        
        // Генерируем код
        let code = totp::generate_totp_code(&secret);
        assert!(!code.is_empty());
        
        // Проверяем код
        let is_valid = totp::verify_totp(&secret, &code);
        assert!(is_valid);
    }

    #[test]
    fn test_recovery_code_verification() {
        use crate::services::totp;
        
        // Генерируем recovery code
        let (code, hash) = totp::generate_recovery_code();
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
