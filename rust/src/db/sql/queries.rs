//! SQL DB Queries - вспомогательные функции
//!
//! Этот файл содержит только тесты. Основные CRUD операции перемещены в:
//! - user_crud.rs - операции с пользователями
//! - template_crud.rs - операции с шаблонами
//! - и т.д.

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use std::env;
    use chrono::Utc;

    async fn create_test_db() -> SqlDb {
        let temp_db = env::temp_dir().join("test_queries.db");
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
                created DATETIME NOT NULL
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();
        
        db
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let db = create_test_db().await;
        
        let user = User {
            id: 0,
            created: Utc::now(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "hashed_password".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        
        let created = db.create_user(user.clone()).await.unwrap();
        assert!(created.id > 0);
        
        let retrieved = db.get_user(created.id).await.unwrap();
        assert_eq!(retrieved.username, "testuser");
        assert_eq!(retrieved.email, "test@example.com");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_get_user_by_login_or_email() {
        let db = create_test_db().await;
        
        let user = User {
            id: 0,
            created: Utc::now(),
            username: "testuser2".to_string(),
            name: "Test User".to_string(),
            email: "test2@example.com".to_string(),
            password: "hashed_password".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        
        db.create_user(user.clone()).await.unwrap();
        
        // Поиск по username
        let retrieved = db.get_user_by_login_or_email("testuser2", "").await.unwrap();
        assert_eq!(retrieved.username, "testuser2");
        
        // Поиск по email
        let retrieved = db.get_user_by_login_or_email("", "test2@example.com").await.unwrap();
        assert_eq!(retrieved.email, "test2@example.com");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_user() {
        let db = create_test_db().await;
        
        let user = User {
            id: 0,
            created: Utc::now(),
            username: "testuser3".to_string(),
            name: "Test User".to_string(),
            email: "test3@example.com".to_string(),
            password: "hashed_password".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        
        let created = db.create_user(user).await.unwrap();
        
        let mut updated = created.clone();
        updated.name = "Updated Name".to_string();
        
        db.update_user(updated).await.unwrap();
        
        let retrieved = db.get_user(created.id).await.unwrap();
        assert_eq!(retrieved.name, "Updated Name");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_user() {
        let db = create_test_db().await;
        
        let user = User {
            id: 0,
            created: Utc::now(),
            username: "testuser4".to_string(),
            name: "Test User".to_string(),
            email: "test4@example.com".to_string(),
            password: "hashed_password".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        
        let created = db.create_user(user).await.unwrap();
        
        db.delete_user(created.id).await.unwrap();
        
        let result = db.get_user(created.id).await;
        assert!(result.is_err());
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_get_users() {
        let db = create_test_db().await;
        
        // Создаём несколько пользователей
        for i in 0..5 {
            let user = User {
                id: 0,
                created: Utc::now(),
                username: format!("user{}", i),
                name: format!("User {}", i),
                email: format!("user{}@example.com", i),
                password: "hashed_password".to_string(),
                admin: false,
                external: false,
                alert: false,
                pro: false,
                totp: None,
                email_otp: None,
            };
            db.create_user(user).await.unwrap();
        }
        
        let params = crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(10),
            sort_by: None,
            sort_inverted: false,
            filter: None,
        };
        
        let users = db.get_users(&params).await.unwrap();
        assert!(users.len() >= 5);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_get_user_count() {
        let db = create_test_db().await;
        
        let initial_count = db.get_user_count().await.unwrap();
        
        // Создаём пользователя
        let user = User {
            id: 0,
            created: Utc::now(),
            username: "countuser".to_string(),
            name: "Count User".to_string(),
            email: "count@example.com".to_string(),
            password: "hashed_password".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        db.create_user(user).await.unwrap();
        
        let new_count = db.get_user_count().await.unwrap();
        assert_eq!(new_count, initial_count + 1);
        
        // Cleanup
        let _ = db.close().await;
    }
}
