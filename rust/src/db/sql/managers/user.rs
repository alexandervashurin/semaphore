//! UserManager - управление пользователями
//!
//! Реализация трейта UserManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::models::{User, UserTotp, ProjectUser};
use crate::error::{Error, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

#[async_trait]
impl UserManager for SqlStore {
    async fn get_users(&self, _params: RetrieveQueryParams) -> Result<Vec<User>> {
        let query = "SELECT * FROM \"user\" ORDER BY id";
            let rows = sqlx::query(query)
                .fetch_all(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;

            Ok(rows.into_iter().map(|row| User {
                id: row.get("id"),
                created: row.try_get("created").unwrap_or_else(|_| Utc::now()),
                username: row.get("username"),
                name: row.get("name"),
                email: row.get("email"),
                password: row.get("password"),
                admin: row.get("admin"),
                external: row.get("external"),
                alert: row.get("alert"),
                pro: row.get("pro"),
                totp: None,
                email_otp: None,
            }).collect())
    }

    async fn get_user(&self, user_id: i32) -> Result<User> {
        let query = "SELECT * FROM \"user\" WHERE id = $1";
            let row = sqlx::query(query)
                .bind(user_id)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Пользователь не найден".to_string()),
                    _ => Error::Database(e),
                })?;

            Ok(User {
                id: row.get("id"),
                created: row.try_get("created").unwrap_or_else(|_| Utc::now()),
                username: row.get("username"),
                name: row.get("name"),
                email: row.get("email"),
                password: row.get("password"),
                admin: row.get("admin"),
                external: row.get("external"),
                alert: row.get("alert"),
                pro: row.get("pro"),
                totp: None,
                email_otp: None,
            })
    }

    async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        let query = "SELECT id, created, username, name, email, password, admin, external, alert, pro FROM \"user\" WHERE username = $1 OR email = $2";
            let row = sqlx::query(query)
                .bind(login)
                .bind(email)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Пользователь не найден".to_string()),
                    _ => Error::Database(e),
                })?;

            let id: i32 = row.try_get("id")
                .map_err(Error::Database)?;
            let created: chrono::DateTime<chrono::Utc> = row.try_get("created")
                .map_err(Error::Database)?;
            let username: String = row.try_get("username")
                .map_err(Error::Database)?;
            let name: String = row.try_get("name")
                .map_err(Error::Database)?;
            let email: String = row.try_get("email")
                .map_err(Error::Database)?;
            let password: String = row.try_get("password")
                .map_err(Error::Database)?;
            let admin: bool = row.try_get("admin")
                .map_err(Error::Database)?;
            let external: bool = row.try_get("external")
                .map_err(Error::Database)?;
            let alert: bool = row.try_get("alert")
                .map_err(Error::Database)?;
            let pro: bool = row.try_get("pro")
                .map_err(Error::Database)?;

            Ok(User {
                id,
                created,
                username,
                name,
                email,
                password,
                admin,
                external,
                alert,
                pro,
                totp: None,
                email_otp: None,
            })
    }

    async fn create_user(&self, user: User, password: &str) -> Result<User> {
        use crate::api::auth_local::hash_password;
        
        // Хешируем пароль перед сохранением
        let password_hash = hash_password(password)?;
        
        let query = "INSERT INTO \"user\" (username, name, email, password, admin, external, alert, pro, created) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)";
            sqlx::query(query)
                .bind(&user.username)
                .bind(&user.name)
                .bind(&user.email)
                .bind(&password_hash)
                .bind(user.admin)
                .bind(user.external)
                .bind(user.alert)
                .bind(user.pro)
                .bind(user.created)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;

        self.get_user_by_login_or_email(&user.username, &user.email).await
    }

    async fn update_user(&self, user: User) -> Result<()> {
        let query = "UPDATE \"user\" SET username = $1, name = $2, email = $3, admin = $4, external = $5, alert = $6, pro = $7 WHERE id = $8";
            sqlx::query(query)
                .bind(&user.username)
                .bind(&user.name)
                .bind(&user.email)
                .bind(user.admin)
                .bind(user.external)
                .bind(user.alert)
                .bind(user.pro)
                .bind(user.id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_user(&self, user_id: i32) -> Result<()> {
        let query = "DELETE FROM \"user\" WHERE id = $1";
            sqlx::query(query)
                .bind(user_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    async fn set_user_password(&self, user_id: i32, password: &str) -> Result<()> {
        let query = "UPDATE \"user\" SET password = $1 WHERE id = $2";
            sqlx::query(query)
                .bind(password)
                .bind(user_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_all_admins(&self) -> Result<Vec<User>> {
        let query = "SELECT * FROM \"user\" WHERE admin = $1";
            let rows = sqlx::query(query)
                .bind(true)
                .fetch_all(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;

            Ok(rows.into_iter().map(|row| User {
                id: row.get("id"),
                created: row.try_get("created").unwrap_or_else(|_| Utc::now()),
                username: row.get("username"),
                name: row.get("name"),
                email: row.get("email"),
                password: row.get("password"),
                admin: row.get("admin"),
                external: row.get("external"),
                alert: row.get("alert"),
                pro: row.get("pro"),
                totp: None,
                email_otp: None,
            }).collect())
    }

    async fn get_user_count(&self) -> Result<usize> {
        let query = "SELECT COUNT(*) FROM \"user\"";
            let count: i64 = sqlx::query_scalar(query)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
            Ok(count as usize)
    }

    async fn get_project_users(&self, project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<ProjectUser>> {
        let query = "SELECT pu.*, u.username, u.name, u.email
                 FROM project_user pu
                 JOIN \"user\" u ON pu.user_id = u.id
                 WHERE pu.project_id = $1
                 ORDER BY pu.id";
            let rows = sqlx::query(query)
                .bind(project_id)
                .fetch_all(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;

            Ok(rows.into_iter().map(|row| ProjectUser {
                id: row.get("id"),
                project_id: row.get("project_id"),
                user_id: row.get("user_id"),
                role: row.get("role"),
                created: row.try_get("created").unwrap_or_else(|_| Utc::now()),
                username: row.get("username"),
                name: row.get("name"),
            }).collect())
    }

    async fn get_user_totp(&self, user_id: i32) -> Result<Option<UserTotp>> {
        // Получаем пользователя и возвращаем его TOTP
        let user = self.get_user(user_id).await?;
        Ok(user.totp)
    }

    async fn set_user_totp(&self, user_id: i32, totp: &UserTotp) -> Result<()> {
        // Сериализуем TOTP в JSON
        let totp_json = serde_json::to_string(totp)
            .map_err(|e| Error::Other(format!("Failed to serialize TOTP: {}", e)))?;
        
        // Обновляем user.totp
        sqlx::query("UPDATE \"user\" SET totp = $1 WHERE id = $2")
                .bind(&totp_json)
                .bind(user_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_user_totp(&self, user_id: i32) -> Result<()> {
        // Удаляем TOTP (устанавливаем в NULL)
        sqlx::query("UPDATE \"user\" SET totp = NULL WHERE id = $1")
                .bind(user_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }
}

