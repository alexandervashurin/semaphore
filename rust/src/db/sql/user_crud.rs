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
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::user::get_users(pool, params).await
    }

    /// Получает пользователя по ID
    pub async fn get_user(&self, user_id: i32) -> Result<User> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::user::get_user(pool, user_id).await
    }

    /// Получает пользователя по login или email
    pub async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::user::get_user_by_login_or_email(pool, login, email).await
    }

    /// Создаёт пользователя
    pub async fn create_user(&self, user: User) -> Result<User> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::user::create_user(pool, user).await
    }

    /// Обновляет пользователя
    pub async fn update_user(&self, user: User) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::user::update_user(pool, user).await
    }

    /// Удаляет пользователя
    pub async fn delete_user(&self, user_id: i32) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::user::delete_user(pool, user_id).await
    }

    /// Получает количество пользователей
    pub async fn get_user_count(&self) -> Result<usize> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::user::get_user_count(pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user::ValidationError;
    use chrono::Utc;

    #[test]
    fn test_user_row_to_user_conversion() {
        let row = UserRow {
            id: 1,
            created: Utc::now(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "hashed".to_string(),
            admin: false,
            external: true,
            alert: true,
            pro: false,
        };
        let user: User = row.into();
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
        assert!(!user.admin);
        assert!(user.external);
        assert!(user.alert);
        assert!(!user.pro);
        assert!(user.totp.is_none());
        assert!(user.email_otp.is_none());
    }

    #[test]
    fn test_user_row_all_flags_true() {
        let row = UserRow {
            id: 42,
            created: Utc::now(),
            username: "admin".to_string(),
            name: "Admin".to_string(),
            email: "admin@example.com".to_string(),
            password: "hash".to_string(),
            admin: true,
            external: false,
            alert: true,
            pro: true,
        };
        let user: User = row.into();
        assert!(user.admin);
        assert!(!user.external);
        assert!(user.alert);
        assert!(user.pro);
    }

    #[test]
    fn test_user_row_debug_format() {
        let row = UserRow {
            id: 1,
            created: Utc::now(),
            username: "debug".to_string(),
            name: "Debug".to_string(),
            email: "debug@test.com".to_string(),
            password: "pwd".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
        };
        let debug_str = format!("{:?}", row);
        assert!(debug_str.contains("UserRow"));
        assert!(debug_str.contains("debug"));
    }

    #[test]
    fn test_user_validation_success() {
        let row = UserRow {
            id: 1,
            created: Utc::now(),
            username: "validuser".to_string(),
            name: "Valid".to_string(),
            email: "valid@test.com".to_string(),
            password: "hash".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
        };
        let user: User = row.into();
        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_user_validation_empty_username() {
        let row = UserRow {
            id: 1,
            created: Utc::now(),
            username: "".to_string(),
            name: "Valid".to_string(),
            email: "valid@test.com".to_string(),
            password: "hash".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
        };
        let user: User = row.into();
        assert!(matches!(user.validate(), Err(ValidationError::UsernameEmpty)));
    }

    #[test]
    fn test_user_serialization_skips_password() {
        let row = UserRow {
            id: 1,
            created: Utc::now(),
            username: "seruser".to_string(),
            name: "Serial".to_string(),
            email: "ser@test.com".to_string(),
            password: "secret".to_string(),
            admin: true,
            external: false,
            alert: false,
            pro: true,
        };
        let user: User = row.into();
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"username\":\"seruser\""));
        assert!(json.contains("\"admin\":true"));
        assert!(json.contains("\"pro\":true"));
        assert!(!json.contains("secret"));
    }

    #[test]
    fn test_user_row_id_zero() {
        let row = UserRow {
            id: 0,
            created: Utc::now(),
            username: "zero".to_string(),
            name: "Zero".to_string(),
            email: "zero@test.com".to_string(),
            password: "hash".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
        };
        let user: User = row.into();
        assert_eq!(user.id, 0);
    }

    #[test]
    fn test_user_clone_after_conversion() {
        let row = UserRow {
            id: 100,
            created: Utc::now(),
            username: "cloneuser".to_string(),
            name: "Clone".to_string(),
            email: "clone@test.com".to_string(),
            password: "hash".to_string(),
            admin: false,
            external: true,
            alert: false,
            pro: false,
        };
        let user: User = row.into();
        let cloned = user.clone();
        assert_eq!(cloned.username, "cloneuser");
        assert_eq!(cloned.external, true);
    }

    #[test]
    fn test_user_role_serialization() {
        let role = ProjectUserRole::Manager;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"manager\"");
    }

    #[test]
    fn test_user_role_deserialization() {
        let role: ProjectUserRole = serde_json::from_str("\"owner\"").unwrap();
        assert_eq!(role, ProjectUserRole::Owner);
    }

    #[test]
    fn test_user_row_from_row_conversion_preserves_all_fields() {
        let now = Utc::now();
        let row = UserRow {
            id: 99,
            created: now,
            username: "fulluser".to_string(),
            name: "Full User".to_string(),
            email: "full@test.com".to_string(),
            password: "secure_hash".to_string(),
            admin: true,
            external: true,
            alert: true,
            pro: true,
        };
        let user: User = row.into();
        assert_eq!(user.id, 99);
        assert_eq!(user.created, now);
        assert_eq!(user.username, "fulluser");
        assert_eq!(user.name, "Full User");
        assert_eq!(user.email, "full@test.com");
        assert!(user.admin);
        assert!(user.external);
        assert!(user.alert);
        assert!(user.pro);
    }
}
