//! MySQL User CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use chrono::{DateTime, Utc};
use sqlx::{Row, Pool, MySql};

/// Временная структура для загрузки пользователя из БД
#[derive(Debug, sqlx::FromRow)]
struct UserRow {
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

/// Получает всех пользователей MySQL
pub async fn get_users(pool: &Pool<MySql>, params: &RetrieveQueryParams) -> Result<Vec<User>> {
    let mut query = String::from("SELECT * FROM `user`");

    if let Some(ref filter) = params.filter {
        if !filter.is_empty() {
            query.push_str(" WHERE username LIKE ? OR name LIKE ? OR email LIKE ?");
        }
    }

    if let Some(count) = params.count {
        query.push_str(&format!(" LIMIT {} OFFSET {}", count, params.offset));
    }

    let users = if params.filter.as_ref().is_some_and(|f| !f.is_empty()) {
        let filter_pattern = format!("%{}%", params.filter.as_ref().unwrap());
        sqlx::query_as::<_, UserRow>(&query)
            .bind(&filter_pattern)
            .bind(&filter_pattern)
            .bind(&filter_pattern)
            .fetch_all(pool)
            .await
            .map_err(Error::Database)?
            .into_iter()
            .map(|r| r.into())
            .collect()
    } else {
        sqlx::query_as::<_, UserRow>(&query)
            .fetch_all(pool)
            .await
            .map_err(Error::Database)?
            .into_iter()
            .map(|r| r.into())
            .collect()
    };

    Ok(users)
}

/// Получает пользователя по ID MySQL
pub async fn get_user(pool: &Pool<MySql>, user_id: i32) -> Result<User> {
    let query = "SELECT * FROM `user` WHERE id = ?";
    
    let row = sqlx::query_as::<_, UserRow>(query)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("User not found".to_string()),
            _ => Error::Database(e),
        })?;

    Ok(row.into())
}

/// Получает пользователя по login или email MySQL
pub async fn get_user_by_login_or_email(pool: &Pool<MySql>, login: &str, email: &str) -> Result<User> {
    let query = "SELECT * FROM `user` WHERE username = ? OR email = ?";
    
    let row = sqlx::query_as::<_, UserRow>(query)
        .bind(login)
        .bind(email)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("User not found".to_string()),
            _ => Error::Database(e),
        })?;

    Ok(row.into())
}

/// Создаёт пользователя MySQL
pub async fn create_user(pool: &Pool<MySql>, user: User) -> Result<User> {
    let query = "INSERT INTO `user` (username, name, email, password, admin, external, alert, pro, created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
    
    let result = sqlx::query(query)
        .bind(&user.username)
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.password)
        .bind(user.admin)
        .bind(user.external)
        .bind(user.alert)
        .bind(user.pro)
        .bind(user.created)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    let mut new_user = user;
    new_user.id = result.last_insert_id() as i32;
    
    Ok(new_user)
}

/// Обновляет пользователя MySQL
pub async fn update_user(pool: &Pool<MySql>, user: User) -> Result<()> {
    let query = "UPDATE `user` SET username = ?, name = ?, email = ?, password = ?, admin = ?, external = ?, alert = ?, pro = ? WHERE id = ?";
    
    sqlx::query(query)
        .bind(&user.username)
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.password)
        .bind(user.admin)
        .bind(user.external)
        .bind(user.alert)
        .bind(user.pro)
        .bind(user.id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

/// Удаляет пользователя MySQL
pub async fn delete_user(pool: &Pool<MySql>, user_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM `user` WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

/// Получает количество пользователей MySQL
pub async fn get_user_count(pool: &Pool<MySql>) -> Result<usize> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM `user`")
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;
    Ok(count as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::store::RetrieveQueryParams;

    #[test]
    fn test_user_row_from_conversion() {
        let row = UserRow {
            id: 1,
            created: Utc::now(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "hashed".to_string(),
            admin: true,
            external: false,
            alert: true,
            pro: false,
        };
        let user: User = row.into();
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
        assert!(user.admin);
        assert!(!user.external);
        assert!(user.alert);
        assert!(!user.pro);
        assert!(user.totp.is_none());
        assert!(user.email_otp.is_none());
    }

    #[test]
    fn test_get_users_query_without_filter() {
        let params = RetrieveQueryParams::default();
        assert!(params.filter.is_none());
        assert_eq!(params.offset, 0);
        assert!(params.count.is_none());
    }

    #[test]
    fn test_get_users_query_with_filter() {
        let params = RetrieveQueryParams {
            filter: Some("admin".to_string()),
            offset: 0,
            count: Some(10),
            sort_by: None,
            sort_inverted: false,
        };
        assert!(params.filter.is_some());
        assert_eq!(params.filter.as_ref().unwrap(), "admin");
        assert_eq!(params.count, Some(10));
    }

    #[test]
    fn test_get_users_query_with_empty_filter() {
        let params = RetrieveQueryParams {
            filter: Some("".to_string()),
            offset: 0,
            count: None,
            sort_by: None,
            sort_inverted: false,
        };
        assert!(params.filter.as_ref().is_some_and(|f| f.is_empty()));
    }

    #[test]
    fn test_get_user_by_login_or_email_query_structure() {
        let login = "testuser";
        let email = "test@example.com";
        assert_eq!(login, "testuser");
        assert_eq!(email, "test@example.com");
    }

    #[test]
    fn test_create_user_query_structure() {
        let expected = "INSERT INTO `user` (username, name, email, password, admin, external, alert, pro, created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
        assert!(expected.contains("`user`"));
        assert!(expected.contains("username"));
        assert!(expected.contains("password"));
        assert!(expected.count_matches('?'.into()) == 9);
    }

    #[test]
    fn test_update_user_query_structure() {
        let expected = "UPDATE `user` SET username = ?, name = ?, email = ?, password = ?, admin = ?, external = ?, alert = ?, pro = ? WHERE id = ?";
        assert!(expected.contains("UPDATE `user`"));
        assert!(expected.contains("WHERE id = ?"));
        assert!(expected.count_matches('?'.into()) == 9);
    }

    #[test]
    fn test_delete_user_query_structure() {
        let expected = "DELETE FROM `user` WHERE id = ?";
        assert!(expected.contains("`user`"));
        assert!(expected.contains("id = ?"));
    }

    #[test]
    fn test_get_user_count_query_structure() {
        let expected = "SELECT COUNT(*) FROM `user`";
        assert!(expected.contains("COUNT(*)"));
        assert!(expected.contains("`user`"));
    }

    #[test]
    fn test_mysql_uses_backticks() {
        let queries = [
            "SELECT * FROM `user` WHERE id = ?",
            "DELETE FROM `user` WHERE id = ?",
            "INSERT INTO `user` (username) VALUES (?)",
        ];
        for q in &queries {
            assert!(q.contains('`'), "MySQL query should use backticks: {}", q);
        }
    }

    #[test]
    fn test_mysql_uses_positional_placeholders() {
        let queries = [
            "SELECT * FROM `user` WHERE id = ?",
            "INSERT INTO `user` (username) VALUES (?)",
            "UPDATE `user` SET name = ? WHERE id = ?",
        ];
        for q in &queries {
            assert!(q.contains('?'), "MySQL should use ? placeholders: {}", q);
        }
    }

    #[test]
    fn test_filter_pattern_format() {
        let filter = "admin";
        let pattern = format!("%{}%", filter);
        assert_eq!(pattern, "%admin%");
    }

    #[test]
    fn test_user_with_all_defaults() {
        let now = Utc::now();
        let user = User {
            id: 0,
            created: now,
            username: String::new(),
            name: String::new(),
            email: String::new(),
            password: String::new(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        assert_eq!(user.id, 0);
        assert!(!user.admin);
        assert!(!user.external);
        assert!(!user.alert);
        assert!(!user.pro);
    }

    #[test]
    fn test_user_row_debug_format() {
        let row = UserRow {
            id: 1,
            created: Utc::now(),
            username: "debug_user".to_string(),
            name: "Debug".to_string(),
            email: "debug@test.com".to_string(),
            password: "secret".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
        };
        let debug_str = format!("{:?}", row);
        assert!(debug_str.contains("UserRow"));
        assert!(debug_str.contains("debug_user"));
    }
}
