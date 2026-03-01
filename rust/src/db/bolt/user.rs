//! User - операции с пользователями в BoltDB
//!
//! Аналог db/bolt/user.go из Go версии

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::Result;
use crate::models::{User, RetrieveQueryParams};
use bcrypt::{hash, verify, DEFAULT_COST};

impl BoltStore {
    /// Создаёт нового пользователя
    pub async fn create_user(&self, mut user: User, password: &str) -> Result<User> {
        // Хешируем пароль
        let hashed_password = hash(password, DEFAULT_COST)
            .map_err(|e| crate::error::Error::Other(format!("Bcrypt error: {}", e)))?;
        
        user.password = hashed_password;
        user.created = chrono::Utc::now();
        
        let user_clone = user.clone();
        
        self.db.update(|tx| {
            let bucket = tx.create_bucket_if_not_exists(b"users")?;
            
            let str = serde_json::to_vec(&user_clone)?;
            
            let id = bucket.next_sequence()?;
            let id = i64::MAX - id as i64;
            
            let mut user_with_id = user_clone;
            user_with_id.id = id as i32;
            
            let str = serde_json::to_vec(&user_with_id)?;
            bucket.put(id.to_be_bytes(), str)?;
            
            Ok(user_with_id)
        }).await
    }

    /// Создаёт пользователя без пароля (внешний пользователь)
    pub async fn create_user_without_password(&self, mut user: User) -> Result<User> {
        user.created = chrono::Utc::now();
        user.external = true;
        
        let user_clone = user.clone();
        
        self.db.update(|tx| {
            let bucket = tx.create_bucket_if_not_exists(b"users")?;
            
            let str = serde_json::to_vec(&user_clone)?;
            
            let id = bucket.next_sequence()?;
            let id = i64::MAX - id as i64;
            
            let mut user_with_id = user_clone;
            user_with_id.id = id as i32;
            
            let str = serde_json::to_vec(&user_with_id)?;
            bucket.put(id.to_be_bytes(), str)?;
            
            Ok(user_with_id)
        }).await
    }

    /// Получает пользователя по ID
    pub async fn get_user(&self, user_id: i32) -> Result<User> {
        self.db.view(|tx| {
            let bucket = tx.bucket(b"users");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Пользователь не найден".to_string()));
            }
            
            let bucket = bucket.unwrap();
            let key = (i64::MAX - user_id as i64).to_be_bytes();
            
            if let Some(v) = bucket.get(key) {
                let user: User = serde_json::from_slice(&v)?;
                Ok(user)
            } else {
                Err(crate::error::Error::NotFound("Пользователь не найден".to_string()))
            }
        }).await
    }

    /// Получает пользователя по login или email
    pub async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        self.db.view(|tx| {
            let bucket = tx.bucket(b"users");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Пользователь не найден".to_string()));
            }
            
            let bucket = bucket.unwrap();
            
            for item in bucket.iter() {
                let (_, v) = item?;
                let user: User = serde_json::from_slice(&v)?;
                
                if user.username == login || user.email == email {
                    return Ok(user);
                }
            }
            
            Err(crate::error::Error::NotFound("Пользователь не найден".to_string()))
        }).await
    }

    /// Получает всех пользователей с фильтрацией
    pub async fn get_users(&self, params: RetrieveQueryParams) -> Result<Vec<User>> {
        let mut users = Vec::new();
        
        self.db.view(|tx| {
            let bucket = tx.bucket(b"users");
            if bucket.is_none() {
                return Ok(());
            }
            
            let bucket = bucket.unwrap();
            let mut cursor = bucket.cursor();
            
            let mut i = 0;
            let mut n = 0;
            
            while let Some((k, v)) = cursor.first() {
                if params.offset > 0 && i < params.offset {
                    i += 1;
                    continue;
                }
                
                let user: User = serde_json::from_slice(v)?;
                
                // Фильтрация по имени/email
                if let Some(ref filter) = params.filter {
                    if !filter.is_empty() {
                        let filter_lower = filter.to_lowercase();
                        if !user.username.to_lowercase().contains(&filter_lower) &&
                           !user.name.to_lowercase().contains(&filter_lower) &&
                           !user.email.to_lowercase().contains(&filter_lower) {
                            continue;
                        }
                    }
                }
                
                users.push(user);
                n += 1;
                
                if n > params.count {
                    break;
                }
            }
            
            Ok(())
        }).await?;
        
        Ok(users)
    }

    /// Обновляет пользователя
    pub async fn update_user(&self, user: User) -> Result<()> {
        self.db.update(|tx| {
            let bucket = tx.bucket(b"users");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Пользователь не найден".to_string()));
            }
            
            let bucket = bucket.unwrap();
            let key = (i64::MAX - user.id as i64).to_be_bytes();
            
            if bucket.get(key).is_none() {
                return Err(crate::error::Error::NotFound("Пользователь не найден".to_string()));
            }
            
            let str = serde_json::to_vec(&user)?;
            bucket.put(key, str)?;
            
            Ok(())
        }).await
    }

    /// Удаляет пользователя
    pub async fn delete_user(&self, user_id: i32) -> Result<()> {
        self.db.update(|tx| {
            let bucket = tx.bucket(b"users");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Пользователь не найден".to_string()));
            }
            
            let bucket = bucket.unwrap();
            let key = (i64::MAX - user_id as i64).to_be_bytes();
            
            if bucket.get(key).is_none() {
                return Err(crate::error::Error::NotFound("Пользователь не найден".to_string()));
            }
            
            bucket.remove(key)?;
            
            Ok(())
        }).await
    }

    /// Устанавливает пароль пользователя
    pub async fn set_user_password(&self, user_id: i32, password: &str) -> Result<()> {
        let hashed_password = hash(password, DEFAULT_COST)
            .map_err(|e| crate::error::Error::Other(format!("Bcrypt error: {}", e)))?;
        
        self.db.update(|tx| {
            let bucket = tx.bucket(b"users");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Пользователь не найден".to_string()));
            }
            
            let bucket = bucket.unwrap();
            let key = (i64::MAX - user_id as i64).to_be_bytes();
            
            if let Some(v) = bucket.get(key) {
                let mut user: User = serde_json::from_slice(&v)?;
                user.password = hashed_password;
                
                let str = serde_json::to_vec(&user)?;
                bucket.put(key, str)?;
                
                Ok(())
            } else {
                Err(crate::error::Error::NotFound("Пользователь не найден".to_string()))
            }
        }).await
    }

    /// Получает всех администраторов
    pub async fn get_all_admins(&self) -> Result<Vec<User>> {
        let mut admins = Vec::new();
        
        self.db.view(|tx| {
            let bucket = tx.bucket(b"users");
            if bucket.is_none() {
                return Ok(());
            }
            
            let bucket = bucket.unwrap();
            
            for item in bucket.iter() {
                let (_, v) = item?;
                let user: User = serde_json::from_slice(&v)?;
                
                if user.admin {
                    admins.push(user);
                }
            }
            
            Ok(())
        }).await?;
        
        Ok(admins)
    }

    /// Получает количество пользователей
    pub async fn get_user_count(&self) -> Result<usize> {
        self.db.view(|tx| {
            let bucket = tx.bucket(b"users");
            if bucket.is_none() {
                return Ok(0);
            }
            
            let bucket = bucket.unwrap();
            Ok(bucket.len())
        }).await
    }

    /// Проверяет пароль пользователя
    pub async fn verify_user_password(&self, user_id: i32, password: &str) -> Result<bool> {
        let user = self.get_user(user_id).await?;
        
        let is_valid = verify(password, &user.password)
            .map_err(|e| crate::error::Error::Other(format!("Bcrypt error: {}", e)))?;
        
        Ok(is_valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_bolt_db() -> BoltStore {
        let path = PathBuf::from("/tmp/test_users.db");
        BoltStore::new(path).unwrap()
    }

    fn create_test_user(username: &str) -> User {
        User {
            id: 0,
            created: Utc::now(),
            username: username.to_string(),
            name: "Test User".to_string(),
            email: format!("{}@example.com", username),
            password: String::new(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        }
    }

    #[tokio::test]
    async fn test_create_user() {
        let db = create_test_bolt_db();
        let user = create_test_user("testuser");
        
        let result = db.create_user(user, "password123").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user() {
        let db = create_test_bolt_db();
        let user = create_test_user("testuser2");
        let created = db.create_user(user, "password123").await.unwrap();
        
        let retrieved = db.get_user(created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().username, "testuser2");
    }

    #[tokio::test]
    async fn test_get_user_by_login_or_email() {
        let db = create_test_bolt_db();
        let user = create_test_user("testuser3");
        let created = db.create_user(user, "password123").await.unwrap();
        
        let retrieved = db.get_user_by_login_or_email("testuser3", created.email.as_str()).await;
        assert!(retrieved.is_ok());
    }

    #[tokio::test]
    async fn test_update_user() {
        let db = create_test_bolt_db();
        let user = create_test_user("testuser4");
        let mut created = db.create_user(user, "password123").await.unwrap();
        
        created.name = "Updated Name".to_string();
        let result = db.update_user(created).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let db = create_test_bolt_db();
        let user = create_test_user("testuser5");
        let created = db.create_user(user, "password123").await.unwrap();
        
        let result = db.delete_user(created.id).await;
        assert!(result.is_ok());
        
        let retrieved = db.get_user(created.id).await;
        assert!(retrieved.is_err());
    }

    #[tokio::test]
    async fn test_get_users() {
        let db = create_test_bolt_db();
        
        // Создаём несколько пользователей
        for i in 0..5 {
            let user = create_test_user(&format!("user{}", i));
            db.create_user(user, "password123").await.unwrap();
        }
        
        let params = RetrieveQueryParams {
            offset: 0,
            count: Some(10),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        };
        
        let users = db.get_users(params).await;
        assert!(users.is_ok());
        assert!(users.unwrap().len() >= 5);
    }

    #[tokio::test]
    async fn test_verify_user_password() {
        let db = create_test_bolt_db();
        let user = create_test_user("testuser6");
        let created = db.create_user(user, "password123").await.unwrap();
        
        let is_valid = db.verify_user_password(created.id, "password123").await;
        assert!(is_valid.is_ok());
        assert!(is_valid.unwrap());
    }
}
