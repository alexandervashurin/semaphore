//! User CRUD - операции с пользователями
//!
//! Адаптер для декомпозированных модулей
//! 
//! Новые модули: sqlite::user, postgres::user, mysql::user

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use chrono::{DateTime, Utc};

/// Временная структура для загрузки пользователя из БД
#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: i32,
    pub created: DateTime<Utc>,
    pub username: String,
    pub name: String,
    pub email: String,
    pub password: String,
    pub admin: bool,
    pub external: bool,
    pub alert: bool,
    pub pro: bool,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: row.id,
            created: row.created,
            username: row.username,
            name: row.name,
            email: row.email,
            password: row.password,
            admin: row.admin,
            external: row.external,
            alert: row.alert,
            pro: row.pro,
            totp: None,
            email_otp: None,
        }
    }
}

impl SqlDb {
    /// Получает всех пользователей
    pub async fn get_users(&self, params: &RetrieveQueryParams) -> Result<Vec<User>> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::user::get_users(pool, params).await
    }

    /// Получает пользователя по ID
    pub async fn get_user(&self, user_id: i32) -> Result<User> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::user::get_user(pool, user_id).await
    }

    /// Получает пользователя по login или email
    pub async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::user::get_user_by_login_or_email(pool, login, email).await
    }

    /// Создаёт пользователя
    pub async fn create_user(&self, user: User) -> Result<User> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::user::create_user(pool, user).await
    }

    /// Обновляет пользователя
    pub async fn update_user(&self, user: User) -> Result<()> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::user::update_user(pool, user).await
    }

    /// Удаляет пользователя
    pub async fn delete_user(&self, user_id: i32) -> Result<()> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::user::delete_user(pool, user_id).await
    }

    /// Получает количество пользователей
    pub async fn get_user_count(&self) -> Result<usize> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::user::get_user_count(pool).await
    }
}
