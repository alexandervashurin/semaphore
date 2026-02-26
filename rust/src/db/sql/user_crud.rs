//! User CRUD - операции с пользователями
//!
//! Аналог db/sql/user.go из Go версии (часть 1: CRUD)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает всех пользователей
    pub async fn get_users(&self, params: &RetrieveQueryParams) -> Result<Vec<User>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let mut query = String::from("SELECT * FROM user");
                
                // Добавляем фильтр если указан
                if !params.filter.is_empty() {
                    query.push_str(" WHERE username LIKE ? OR name LIKE ? OR email LIKE ?");
                }
                
                // Добавляем лимит и оффсет
                query.push_str(&format!(" LIMIT {} OFFSET {}", params.count, params.offset));
                
                let users = if !params.filter.is_empty() {
                    let filter_pattern = format!("%{}%", params.filter);
                    sqlx::query_as::<_, User>(&query)
                        .bind(&filter_pattern)
                        .bind(&filter_pattern)
                        .bind(&filter_pattern)
                        .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                        .await
                        .map_err(|e| Error::Database(e))?
                } else {
                    sqlx::query_as::<_, User>(&query)
                        .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                        .await
                        .map_err(|e| Error::Database(e))?
                };
                
                Ok(users)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Получает пользователя по ID
    pub async fn get_user(&self, user_id: i32) -> Result<User> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let user = sqlx::query_as::<_, User>("SELECT * FROM user WHERE id = ?")
                    .bind(user_id)
                    .fetch_one(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                Ok(user)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Получает пользователя по login или email
    pub async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let user = sqlx::query_as::<_, User>(
                    "SELECT * FROM user WHERE username = ? OR email = ?"
                )
                .bind(login)
                .bind(email)
                .fetch_one(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(user)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт нового пользователя
    pub async fn create_user(&self, mut user: User) -> Result<User> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO user (username, name, email, password, admin, external, alert, pro, created, totp, email_otp) 
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(&user.username)
                .bind(&user.name)
                .bind(&user.email)
                .bind(&user.password)
                .bind(user.admin)
                .bind(user.external)
                .bind(user.alert)
                .bind(user.pro)
                .bind(user.created)
                .bind(&user.totp)
                .bind(&user.email_otp)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                user.id = result.last_insert_rowid() as i32;
                Ok(user)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт пользователя без пароля (внешний пользователь)
    pub async fn create_user_without_password(&self, mut user: User) -> Result<User> {
        user.password = String::new();
        self.create_user(user).await
    }
    
    /// Обновляет пользователя
    pub async fn update_user(&self, user: User) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE user SET username = ?, name = ?, email = ?, password = ?, 
                     admin = ?, external = ?, alert = ?, pro = ?, totp = ?, email_otp = ? 
                     WHERE id = ?"
                )
                .bind(&user.username)
                .bind(&user.name)
                .bind(&user.email)
                .bind(&user.password)
                .bind(user.admin)
                .bind(user.external)
                .bind(user.alert)
                .bind(user.pro)
                .bind(&user.totp)
                .bind(&user.email_otp)
                .bind(user.id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Удаляет пользователя
    pub async fn delete_user(&self, user_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM user WHERE id = ?")
                    .bind(user_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Получает всех администраторов
    pub async fn get_all_admins(&self) -> Result<Vec<User>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let users = sqlx::query_as::<_, User>("SELECT * FROM user WHERE admin = 1")
                    .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                Ok(users)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Получает количество пользователей
    pub async fn get_user_count(&self) -> Result<usize> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query("SELECT COUNT(*) FROM user")
                    .fetch_one(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                let count: i64 = result.get(0);
                Ok(count as usize)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use chrono::Utc;

    async fn create_test_db() -> SqlDb {
        let temp_db = env::temp_dir().join("test_user.db");
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
        
        let params = RetrieveQueryParams {
            offset: 0,
            count: 10,
            filter: String::new(),
        };
        
        let users = db.get_users(&params).await.unwrap();
        assert!(users.len() >= 5);
        
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
    async fn test_get_all_admins() {
        let db = create_test_db().await;
        
        // Создаём администратора
        let admin = User {
            id: 0,
            created: Utc::now(),
            username: "admin".to_string(),
            name: "Admin User".to_string(),
            email: "admin@example.com".to_string(),
            password: "hashed_password".to_string(),
            admin: true,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        db.create_user(admin).await.unwrap();
        
        let admins = db.get_all_admins().await.unwrap();
        assert!(admins.len() >= 1);
        assert!(admins.iter().any(|u| u.admin));
        
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
