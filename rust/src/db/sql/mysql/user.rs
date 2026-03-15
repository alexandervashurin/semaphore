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
