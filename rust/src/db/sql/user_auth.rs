//! User Auth - аутентификация и пароли
//!
//! Аналог db/sql/user.go из Go версии (часть 2: аутентификация)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use bcrypt::{hash, verify, DEFAULT_COST};

impl SqlDb {
    /// Устанавливает пароль пользователя
    pub async fn set_user_password(&self, user_id: i32, password: &str) -> Result<()> {
        // Хешируем пароль
        let hashed_password = hash_password(password)?;
        
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("UPDATE user SET password = ? WHERE id = ?")
                    .bind(&hashed_password)
                    .bind(user_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Проверяет пароль пользователя
    pub async fn verify_user_password(&self, user_id: i32, password: &str) -> Result<bool> {
        let user = self.get_user(user_id).await?;
        
        // Проверяем пароль
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
    verify(password, hash)
        .map_err(|e| Error::Other(format!("Failed to verify password: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use chrono::Utc;

    async fn create_test_db() -> SqlDb {
        let temp_db = env::temp_dir().join("test_user_auth.db");
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

    #[tokio::test]
    async fn test_set_user_password() {
        let db = create_test_db().await;
        
        let user = User {
            id: 0,
            created: Utc::now(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "old_hash".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        
        let created = db.create_user(user).await.unwrap();
        
        // Устанавливаем новый пароль
        db.set_user_password(created.id, "new_password123").await.unwrap();
        
        // Проверяем что пароль обновился
        let is_valid = db.verify_user_password(created.id, "new_password123").await.unwrap();
        assert!(is_valid);
        
        let is_valid = db.verify_user_password(created.id, "old_password").await.unwrap();
        assert!(!is_valid);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_verify_user_password() {
        let db = create_test_db().await;
        
        let user = User {
            id: 0,
            created: Utc::now(),
            username: "testuser2".to_string(),
            name: "Test User".to_string(),
            email: "test2@example.com".to_string(),
            password: hash_password("correct_password").unwrap(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        
        let created = db.create_user(user).await.unwrap();
        
        // Проверяем правильный пароль
        let is_valid = db.verify_user_password(created.id, "correct_password").await.unwrap();
        assert!(is_valid);
        
        // Проверяем неправильный пароль
        let is_valid = db.verify_user_password(created.id, "wrong_password").await.unwrap();
        assert!(!is_valid);
        
        // Cleanup
        let _ = db.close().await;
    }
}
