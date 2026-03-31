//! PostgreSQL User CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres, Row};

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

/// Получает всех пользователей PostgreSQL
pub async fn get_users(pool: &Pool<Postgres>, params: &RetrieveQueryParams) -> Result<Vec<User>> {
    let mut query = String::from("SELECT * FROM \"user\"");

    // Добавляем фильтр если указан
    if let Some(ref filter) = params.filter {
        if !filter.is_empty() {
            query.push_str(" WHERE username LIKE $1 OR name LIKE $2 OR email LIKE $3");
        }
    }

    // Добавляем лимит и оффсет
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

/// Получает пользователя по ID PostgreSQL
pub async fn get_user(pool: &Pool<Postgres>, user_id: i32) -> Result<User> {
    let query = "SELECT * FROM \"user\" WHERE id = $1";

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

/// Получает пользователя по login или email PostgreSQL
pub async fn get_user_by_login_or_email(
    pool: &Pool<Postgres>,
    login: &str,
    email: &str,
) -> Result<User> {
    let query = "SELECT * FROM \"user\" WHERE username = $1 OR email = $2";

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

/// Создаёт пользователя PostgreSQL
pub async fn create_user(pool: &Pool<Postgres>, user: User) -> Result<User> {
    let query = "INSERT INTO \"user\" (username, name, email, password, admin, external, alert, pro, created) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id";

    let id: i32 = sqlx::query_scalar(query)
        .bind(&user.username)
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.password)
        .bind(user.admin)
        .bind(user.external)
        .bind(user.alert)
        .bind(user.pro)
        .bind(user.created)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

    let mut new_user = user;
    new_user.id = id;

    Ok(new_user)
}

/// Обновляет пользователя PostgreSQL
pub async fn update_user(pool: &Pool<Postgres>, user: User) -> Result<()> {
    let query = "UPDATE \"user\" SET username = $1, name = $2, email = $3, password = $4, admin = $5, external = $6, alert = $7, pro = $8 WHERE id = $9";

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

/// Удаляет пользователя PostgreSQL
pub async fn delete_user(pool: &Pool<Postgres>, user_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM \"user\" WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

/// Получает количество пользователей PostgreSQL
pub async fn get_user_count(pool: &Pool<Postgres>) -> Result<usize> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM \"user\"")
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;
    Ok(count as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Требуется PostgreSQL
    async fn test_postgres_user_crud() {
        // Тест требует подключения к PostgreSQL
        // Запускается только с POSTGRES_TEST_URL
    }
}
