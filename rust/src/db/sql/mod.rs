//! SQL-хранилище (SQLite)

pub mod runner;
pub mod types;
pub mod init;
pub mod migrations;
pub mod queries;
pub mod utils;
pub mod template_crud;
pub mod template_vault;
pub mod template_roles;
pub mod template_utils;
pub mod user_crud;
pub mod user_auth;
pub mod user_totp;
pub mod task_crud;
pub mod task_output;
pub mod task_stage;
pub mod integration_crud;
pub mod integration_matcher;
pub mod integration_extract;
pub mod project_invite;
pub mod terraform_inventory;
pub mod access_key;
pub mod environment;
pub mod event;
pub mod inventory;
pub mod repository;
pub mod schedule;
pub mod session;
pub mod view;

use crate::db::store::*;
use crate::models::{User, Project, Task, TaskWithTpl, TaskOutput, TaskStage, Template, Inventory, Repository, Environment, AccessKey, Integration, Schedule, Session, APIToken, Event, Runner, View, Role, ProjectInvite, ProjectInviteWithUser, ProjectUser, RetrieveQueryParams, TerraformInventoryAlias, TerraformInventoryState, SecretStorage};
use crate::error::{Error, Result};
use crate::services::task_logger::TaskStatus;
use crate::db::sql::types::{SqlDb, SqlDialect};
use async_trait::async_trait;
use sqlx::{SqlitePool, PgPool, MySqlPool, Row};
use std::collections::HashMap;

/// SQL-хранилище данных (на базе SQLite, MySQL, PostgreSQL)
pub struct SqlStore {
    db: SqlDb,
}

impl SqlStore {
    /// Создаёт новое SQL-хранилище
    pub async fn new(database_url: &str) -> Result<Self> {
        // Используем функцию создания подключения из init.rs
        let db = init::create_database_connection(database_url).await?;

        Ok(Self { db })
    }

    /// Получает диалект БД
    fn get_dialect(&self) -> SqlDialect {
        self.db.get_dialect()
    }

    /// Получает SQLite pool
    fn get_sqlite_pool(&self) -> Result<&SqlitePool> {
        self.db.get_sqlite_pool()
            .ok_or_else(|| Error::Other("SQLite pool not found".to_string()))
    }

    /// Получает PostgreSQL pool
    fn get_postgres_pool(&self) -> Result<&PgPool> {
        self.db.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает MySQL pool
    fn get_mysql_pool(&self) -> Result<&MySqlPool> {
        self.db.get_mysql_pool()
            .ok_or_else(|| Error::Other("MySQL pool not found".to_string()))
    }
}

#[async_trait]
impl ConnectionManager for SqlStore {
    async fn connect(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => self.get_sqlite_pool()?.close().await,
            SqlDialect::PostgreSQL => self.get_postgres_pool()?.close().await,
            SqlDialect::MySQL => self.get_mysql_pool()?.close().await,
        }
        Ok(())
    }

    fn is_permanent(&self) -> bool {
        true
    }
}

#[async_trait]
impl MigrationManager for SqlStore {
    fn get_dialect(&self) -> &str {
        match self.db.get_dialect() {
            SqlDialect::SQLite => "sqlite",
            SqlDialect::MySQL => "mysql",
            SqlDialect::PostgreSQL => "postgresql",
        }
    }

    async fn is_initialized(&self) -> Result<bool> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT name FROM sqlite_master WHERE type='table' AND name='migration'";
                let result = sqlx::query(query)
                    .fetch_optional(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result.is_some())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT table_name FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND table_name = 'migration'";
                let result = sqlx::query(query)
                    .fetch_optional(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result.is_some())
            }
            SqlDialect::MySQL => {
                let query = "SELECT table_name FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND table_name = 'migration'";
                let result = sqlx::query(query)
                    .fetch_optional(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result.is_some())
            }
        }
    }

    async fn apply_migration(&self, version: i64, name: String) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO migration (version, name) VALUES (?, ?)";
                sqlx::query(query)
                    .bind(version)
                    .bind(name)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO migration (version, name) VALUES ($1, $2)";
                sqlx::query(query)
                    .bind(version)
                    .bind(name)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO migration (version, name) VALUES (?, ?)";
                sqlx::query(query)
                    .bind(version)
                    .bind(name)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn is_migration_applied(&self, version: i64) -> Result<bool> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT COUNT(*) FROM migration WHERE version = ?";
                let count: i64 = sqlx::query_scalar(query)
                    .bind(version)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(count > 0)
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT COUNT(*) FROM migration WHERE version = $1";
                let count: i64 = sqlx::query_scalar(query)
                    .bind(version)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(count > 0)
            }
            SqlDialect::MySQL => {
                let query = "SELECT COUNT(*) FROM migration WHERE version = ?";
                let count: i64 = sqlx::query_scalar(query)
                    .bind(version)
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(count > 0)
            }
        }
    }
}

#[async_trait]
impl OptionsManager for SqlStore {
    async fn get_options(&self) -> Result<HashMap<String, String>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT key, value FROM option";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| {
                    let key: String = row.get("key");
                    let value: String = row.get("value");
                    (key, value)
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT key, value FROM option";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| {
                    let key: String = row.get("key");
                    let value: String = row.get("value");
                    (key, value)
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT key, value FROM option";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| {
                    let key: String = row.get("key");
                    let value: String = row.get("value");
                    (key, value)
                }).collect())
            }
        }
    }

    async fn get_option(&self, key: &str) -> Result<Option<String>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT value FROM option WHERE key = ?";
                let result = sqlx::query_scalar::<_, String>(query)
                    .bind(key)
                    .fetch_optional(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result)
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT value FROM option WHERE key = $1";
                let result = sqlx::query_scalar::<_, String>(query)
                    .bind(key)
                    .fetch_optional(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result)
            }
            SqlDialect::MySQL => {
                let query = "SELECT value FROM option WHERE key = ?";
                let result = sqlx::query_scalar::<_, String>(query)
                    .bind(key)
                    .fetch_optional(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result)
            }
        }
    }

    async fn set_option(&self, key: &str, value: &str) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT OR REPLACE INTO option (key, value) VALUES (?, ?)";
                sqlx::query(query)
                    .bind(key)
                    .bind(value)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO option (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value";
                sqlx::query(query)
                    .bind(key)
                    .bind(value)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO option (key, value) VALUES (?, ?) ON DUPLICATE KEY UPDATE value = VALUES(value)";
                sqlx::query(query)
                    .bind(key)
                    .bind(value)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_option(&self, key: &str) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM option WHERE key = ?";
                sqlx::query(query)
                    .bind(key)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM option WHERE key = $1";
                sqlx::query(query)
                    .bind(key)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM option WHERE key = ?";
                sqlx::query(query)
                    .bind(key)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl UserManager for SqlStore {
    async fn get_users(&self, _params: RetrieveQueryParams) -> Result<Vec<User>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM user ORDER BY id";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| User {
                    id: row.get("id"),
                    created: row.get("created"),
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
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM \"user\" ORDER BY id";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| User {
                    id: row.get("id"),
                    created: row.get("created"),
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
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `user` ORDER BY id";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| User {
                    id: row.get("id"),
                    created: row.get("created"),
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
        }
    }

    async fn get_user(&self, user_id: i32) -> Result<User> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM user WHERE id = ?";
                let row = sqlx::query(query)
                    .bind(user_id)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Пользователь не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(User {
                    id: row.get("id"),
                    created: row.get("created"),
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
            SqlDialect::PostgreSQL => {
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
                    created: row.get("created"),
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
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `user` WHERE id = ?";
                let row = sqlx::query(query)
                    .bind(user_id)
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Пользователь не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(User {
                    id: row.get("id"),
                    created: row.get("created"),
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
        }
    }

    async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM user WHERE username = ? OR email = ?";
                let row = sqlx::query(query)
                    .bind(login)
                    .bind(email)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Пользователь не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(User {
                    id: row.get("id"),
                    created: row.get("created"),
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
            SqlDialect::PostgreSQL => {
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
                    .map_err(|e| Error::Database(e))?;
                let created: chrono::DateTime<chrono::Utc> = row.try_get("created")
                    .map_err(|e| Error::Database(e))?;
                let username: String = row.try_get("username")
                    .map_err(|e| Error::Database(e))?;
                let name: String = row.try_get("name")
                    .map_err(|e| Error::Database(e))?;
                let email: String = row.try_get("email")
                    .map_err(|e| Error::Database(e))?;
                let password: String = row.try_get("password")
                    .map_err(|e| Error::Database(e))?;
                let admin: bool = row.try_get("admin")
                    .map_err(|e| Error::Database(e))?;
                let external: bool = row.try_get("external")
                    .map_err(|e| Error::Database(e))?;
                let alert: bool = row.try_get("alert")
                    .map_err(|e| Error::Database(e))?;
                let pro: bool = row.try_get("pro")
                    .map_err(|e| Error::Database(e))?;

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
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `user` WHERE username = ? OR email = ?";
                let row = sqlx::query(query)
                    .bind(login)
                    .bind(email)
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Пользователь не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(User {
                    id: row.get("id"),
                    created: row.get("created"),
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
        }
    }

    async fn create_user(&self, user: User, password: &str) -> Result<User> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO user (username, name, email, password, admin, external, alert, pro, created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(&user.username)
                    .bind(&user.name)
                    .bind(&user.email)
                    .bind(password)
                    .bind(user.admin)
                    .bind(user.external)
                    .bind(user.alert)
                    .bind(user.pro)
                    .bind(user.created)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO \"user\" (username, name, email, password, admin, external, alert, pro, created) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)";
                sqlx::query(query)
                    .bind(&user.username)
                    .bind(&user.name)
                    .bind(&user.email)
                    .bind(password)
                    .bind(user.admin)
                    .bind(user.external)
                    .bind(user.alert)
                    .bind(user.pro)
                    .bind(user.created)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `user` (username, name, email, password, admin, external, alert, pro, created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(&user.username)
                    .bind(&user.name)
                    .bind(&user.email)
                    .bind(password)
                    .bind(user.admin)
                    .bind(user.external)
                    .bind(user.alert)
                    .bind(user.pro)
                    .bind(user.created)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }

        self.get_user_by_login_or_email(&user.username, &user.email).await
    }

    async fn update_user(&self, user: User) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE user SET username = ?, name = ?, email = ?, admin = ?, external = ?, alert = ?, pro = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&user.username)
                    .bind(&user.name)
                    .bind(&user.email)
                    .bind(user.admin)
                    .bind(user.external)
                    .bind(user.alert)
                    .bind(user.pro)
                    .bind(user.id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
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
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `user` SET username = ?, name = ?, email = ?, admin = ?, external = ?, alert = ?, pro = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&user.username)
                    .bind(&user.name)
                    .bind(&user.email)
                    .bind(user.admin)
                    .bind(user.external)
                    .bind(user.alert)
                    .bind(user.pro)
                    .bind(user.id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_user(&self, user_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM user WHERE id = ?";
                sqlx::query(query)
                    .bind(user_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM \"user\" WHERE id = $1";
                sqlx::query(query)
                    .bind(user_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `user` WHERE id = ?";
                sqlx::query(query)
                    .bind(user_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn set_user_password(&self, user_id: i32, password: &str) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE user SET password = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(password)
                    .bind(user_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE \"user\" SET password = $1 WHERE id = $2";
                sqlx::query(query)
                    .bind(password)
                    .bind(user_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `user` SET password = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(password)
                    .bind(user_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn get_all_admins(&self) -> Result<Vec<User>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM user WHERE admin = 1";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| User {
                    id: row.get("id"),
                    created: row.get("created"),
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
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM \"user\" WHERE admin = $1";
                let rows = sqlx::query(query)
                    .bind(true)
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| User {
                    id: row.get("id"),
                    created: row.get("created"),
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
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `user` WHERE admin = 1";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| User {
                    id: row.get("id"),
                    created: row.get("created"),
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
        }
    }

    async fn get_user_count(&self) -> Result<usize> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT COUNT(*) FROM user";
                let count: i64 = sqlx::query_scalar(query)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(count as usize)
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT COUNT(*) FROM \"user\"";
                let count: i64 = sqlx::query_scalar(query)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(count as usize)
            }
            SqlDialect::MySQL => {
                let query = "SELECT COUNT(*) FROM `user`";
                let count: i64 = sqlx::query_scalar(query)
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(count as usize)
            }
        }
    }

    async fn get_project_users(&self, project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<ProjectUser>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT pu.*, u.username, u.name, u.email
                     FROM project__user pu
                     JOIN user u ON pu.user_id = u.id
                     WHERE pu.project_id = ?
                     ORDER BY pu.id";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| ProjectUser {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    user_id: row.get("user_id"),
                    role: row.get("role"),
                    created: row.get("created"),
                    username: row.get("username"),
                    name: row.get("name"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT pu.*, u.username, u.name, u.email
                     FROM project__user pu
                     JOIN \"user\" u ON pu.user_id = u.id
                     WHERE pu.project_id = $1
                     ORDER BY pu.id";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| ProjectUser {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    user_id: row.get("user_id"),
                    role: row.get("role"),
                    created: row.get("created"),
                    username: row.get("username"),
                    name: row.get("name"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT pu.*, u.username, u.name, u.email
                     FROM project__user pu
                     JOIN `user` u ON pu.user_id = u.id
                     WHERE pu.project_id = ?
                     ORDER BY pu.id";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| ProjectUser {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    user_id: row.get("user_id"),
                    role: row.get("role"),
                    created: row.get("created"),
                    username: row.get("username"),
                    name: row.get("name"),
                }).collect())
            }
        }
    }
}

#[async_trait]
impl ProjectStore for SqlStore {
    async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let (query, bind_user_id) = if let Some(uid) = user_id {
                    ("SELECT p.* FROM project p JOIN project__user pu ON p.id = pu.project_id WHERE pu.user_id = ?", Some(uid))
                } else {
                    ("SELECT * FROM project", None)
                };

                let mut q = sqlx::query(query);
                if let Some(uid) = bind_user_id {
                    q = q.bind(uid);
                }

                let rows = q
                    .fetch_all(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let (query, bind_user_id) = if let Some(uid) = user_id {
                    ("SELECT p.* FROM project p JOIN project__user pu ON p.id = pu.project_id WHERE pu.user_id = $1", Some(uid))
                } else {
                    ("SELECT * FROM project", None)
                };

                let mut q = sqlx::query(query);
                if let Some(uid) = bind_user_id {
                    q = q.bind(uid);
                }

                let rows = q
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let (query, bind_user_id) = if let Some(uid) = user_id {
                    ("SELECT p.* FROM project p JOIN project__user pu ON p.id = pu.project_id WHERE pu.user_id = ?", Some(uid))
                } else {
                    ("SELECT * FROM project", None)
                };

                let mut q = sqlx::query(query);
                if let Some(uid) = bind_user_id {
                    q = q.bind(uid);
                }

                let rows = q
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                }).collect())
            }
        }
    }

    async fn get_project(&self, project_id: i32) -> Result<Project> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM project WHERE id = ?";
                let row = sqlx::query(query)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Проект не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM project WHERE id = $1";
                let row = sqlx::query(query)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Проект не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM project WHERE id = ?";
                let row = sqlx::query(query)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Проект не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                })
            }
        }
    }

    async fn create_project(&self, mut project: Project) -> Result<Project> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO project (name, created, alert, alert_chat, max_parallel_tasks, type, default_secret_storage_id) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(&project.name)
                    .bind(project.created)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                project.id = id;
                Ok(project)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO project (name, created, alert, alert_chat, max_parallel_tasks, type, default_secret_storage_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(&project.name)
                    .bind(project.created)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                project.id = id;
                Ok(project)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO project (name, created, alert, alert_chat, max_parallel_tasks, type, default_secret_storage_id) VALUES (?, ?, ?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(&project.name)
                    .bind(project.created)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                // Для MySQL нужно получить последний вставленный ID отдельно
                let last_id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                project.id = last_id;
                Ok(project)
            }
        }
    }

    async fn update_project(&self, project: Project) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE project SET name = ?, alert = ?, alert_chat = ?, max_parallel_tasks = ?, type = ?, default_secret_storage_id = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&project.name)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .bind(project.id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE project SET name = $1, alert = $2, alert_chat = $3, max_parallel_tasks = $4, type = $5, default_secret_storage_id = $6 WHERE id = $7";
                sqlx::query(query)
                    .bind(&project.name)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .bind(project.id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE project SET name = ?, alert = ?, alert_chat = ?, max_parallel_tasks = ?, type = ?, default_secret_storage_id = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&project.name)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .bind(project.id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_project(&self, project_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM project WHERE id = ?";
                sqlx::query(query)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM project WHERE id = $1";
                sqlx::query(query)
                    .bind(project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM project WHERE id = ?";
                sqlx::query(query)
                    .bind(project_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// TemplateManager - CRUD операции для шаблонов
// ============================================================================
#[async_trait]
impl TemplateManager for SqlStore {
    async fn get_templates(&self, project_id: i32) -> Result<Vec<Template>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM template WHERE project_id = ? AND deleted = 0 ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.get("inventory_id"),
                    repository_id: row.get("repository_id"),
                    environment_id: row.get("environment_id"),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.get("git_branch"),
                    deleted: row.get("deleted"),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    template_type: row.get("template_type"),
                    start_version: row.get("start_version"),
                    build_version: row.get("build_version"),
                    survey_vars: row.get("survey_vars"),
                    vaults: row.get("vaults"),
                    tasks: row.get("tasks"),
                    vault_key_id: row.get("vault_key_id"),
                    become_key_id: row.get("become_key_id"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM template WHERE project_id = $1 AND deleted = false ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.get("inventory_id"),
                    repository_id: row.get("repository_id"),
                    environment_id: row.get("environment_id"),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.get("git_branch"),
                    deleted: row.get("deleted"),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    template_type: row.get("template_type"),
                    start_version: row.get("start_version"),
                    build_version: row.get("build_version"),
                    survey_vars: row.get("survey_vars"),
                    vaults: row.get("vaults"),
                    tasks: row.get("tasks"),
                    vault_key_id: row.get("vault_key_id"),
                    become_key_id: row.get("become_key_id"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `template` WHERE project_id = ? AND deleted = 0 ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.get("inventory_id"),
                    repository_id: row.get("repository_id"),
                    environment_id: row.get("environment_id"),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.get("git_branch"),
                    deleted: row.get("deleted"),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    template_type: row.get("template_type"),
                    start_version: row.get("start_version"),
                    build_version: row.get("build_version"),
                    survey_vars: row.get("survey_vars"),
                    vaults: row.get("vaults"),
                    tasks: row.get("tasks"),
                    vault_key_id: row.get("vault_key_id"),
                    become_key_id: row.get("become_key_id"),
                }).collect())
            }
        }
    }

    async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM template WHERE id = ? AND project_id = ? AND deleted = 0";
                let row = sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Шаблон не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.get("inventory_id"),
                    repository_id: row.get("repository_id"),
                    environment_id: row.get("environment_id"),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.get("git_branch"),
                    deleted: row.get("deleted"),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    template_type: row.get("template_type"),
                    start_version: row.get("start_version"),
                    build_version: row.get("build_version"),
                    survey_vars: row.get("survey_vars"),
                    vaults: row.get("vaults"),
                    tasks: row.get("tasks"),
                    vault_key_id: row.get("vault_key_id"),
                    become_key_id: row.get("become_key_id"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM template WHERE id = $1 AND project_id = $2 AND deleted = false";
                let row = sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Шаблон не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.get("inventory_id"),
                    repository_id: row.get("repository_id"),
                    environment_id: row.get("environment_id"),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.get("git_branch"),
                    deleted: row.get("deleted"),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    template_type: row.get("template_type"),
                    start_version: row.get("start_version"),
                    build_version: row.get("build_version"),
                    survey_vars: row.get("survey_vars"),
                    vaults: row.get("vaults"),
                    tasks: row.get("tasks"),
                    vault_key_id: row.get("vault_key_id"),
                    become_key_id: row.get("become_key_id"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `template` WHERE id = ? AND project_id = ? AND deleted = 0";
                let row = sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Шаблон не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.get("inventory_id"),
                    repository_id: row.get("repository_id"),
                    environment_id: row.get("environment_id"),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.get("git_branch"),
                    deleted: row.get("deleted"),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    template_type: row.get("template_type"),
                    start_version: row.get("start_version"),
                    build_version: row.get("build_version"),
                    survey_vars: row.get("survey_vars"),
                    vaults: row.get("vaults"),
                    tasks: row.get("tasks"),
                    vault_key_id: row.get("vault_key_id"),
                    become_key_id: row.get("become_key_id"),
                })
            }
        }
    }

    async fn create_template(&self, mut template: Template) -> Result<Template> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO template (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, deleted, created, arguments, template_type, start_version, build_version, survey_vars, vaults, tasks, vault_key_id, become_key_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(template.project_id)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(template.deleted)
                    .bind(template.created)
                    .bind(&template.arguments)
                    .bind(&template.template_type)
                    .bind(&template.start_version)
                    .bind(&template.build_version)
                    .bind(&template.survey_vars)
                    .bind(&template.vaults)
                    .bind(&template.tasks)
                    .bind(&template.vault_key_id)
                    .bind(&template.become_key_id)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                template.id = id;
                Ok(template)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO template (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, deleted, created, arguments, template_type, start_version, build_version, survey_vars, vaults, tasks, vault_key_id, become_key_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(template.project_id)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(template.deleted)
                    .bind(template.created)
                    .bind(&template.arguments)
                    .bind(&template.template_type)
                    .bind(&template.start_version)
                    .bind(&template.build_version)
                    .bind(&template.survey_vars)
                    .bind(&template.vaults)
                    .bind(&template.tasks)
                    .bind(&template.vault_key_id)
                    .bind(&template.become_key_id)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                template.id = id;
                Ok(template)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `template` (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, deleted, created, arguments, template_type, start_version, build_version, survey_vars, vaults, tasks, vault_key_id, become_key_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(template.project_id)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(template.deleted)
                    .bind(template.created)
                    .bind(&template.arguments)
                    .bind(&template.template_type)
                    .bind(&template.start_version)
                    .bind(&template.build_version)
                    .bind(&template.survey_vars)
                    .bind(&template.vaults)
                    .bind(&template.tasks)
                    .bind(&template.vault_key_id)
                    .bind(&template.become_key_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                let id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                template.id = id;
                Ok(template)
            }
        }
    }

    async fn update_template(&self, template: Template) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE template SET name = ?, playbook = ?, description = ?, inventory_id = ?, repository_id = ?, environment_id = ?, type = ?, app = ?, git_branch = ?, arguments = ?, template_type = ?, start_version = ?, build_version = ?, survey_vars = ?, vaults = ?, tasks = ?, vault_key_id = ?, become_key_id = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(&template.arguments)
                    .bind(&template.template_type)
                    .bind(&template.start_version)
                    .bind(&template.build_version)
                    .bind(&template.survey_vars)
                    .bind(&template.vaults)
                    .bind(&template.tasks)
                    .bind(&template.vault_key_id)
                    .bind(&template.become_key_id)
                    .bind(template.id)
                    .bind(template.project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE template SET name = $1, playbook = $2, description = $3, inventory_id = $4, repository_id = $5, environment_id = $6, type = $7, app = $8, git_branch = $9, arguments = $10, template_type = $11, start_version = $12, build_version = $13, survey_vars = $14, vaults = $15, tasks = $16, vault_key_id = $17, become_key_id = $18 WHERE id = $19 AND project_id = $20";
                sqlx::query(query)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(&template.arguments)
                    .bind(&template.template_type)
                    .bind(&template.start_version)
                    .bind(&template.build_version)
                    .bind(&template.survey_vars)
                    .bind(&template.vaults)
                    .bind(&template.tasks)
                    .bind(&template.vault_key_id)
                    .bind(&template.become_key_id)
                    .bind(template.id)
                    .bind(template.project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `template` SET name = ?, playbook = ?, description = ?, inventory_id = ?, repository_id = ?, environment_id = ?, type = ?, app = ?, git_branch = ?, arguments = ?, template_type = ?, start_version = ?, build_version = ?, survey_vars = ?, vaults = ?, tasks = ?, vault_key_id = ?, become_key_id = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(&template.arguments)
                    .bind(&template.template_type)
                    .bind(&template.start_version)
                    .bind(&template.build_version)
                    .bind(&template.survey_vars)
                    .bind(&template.vaults)
                    .bind(&template.tasks)
                    .bind(&template.vault_key_id)
                    .bind(&template.become_key_id)
                    .bind(template.id)
                    .bind(template.project_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                // Мягкое удаление - устанавливаем deleted = 1
                let query = "UPDATE template SET deleted = 1 WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE template SET deleted = true WHERE id = $1 AND project_id = $2";
                sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `template` SET deleted = 1 WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// InventoryManager - CRUD операции для инвентарей
// ============================================================================
#[async_trait]
impl InventoryManager for SqlStore {
    async fn get_inventories(&self, project_id: i32) -> Result<Vec<Inventory>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM inventory WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Inventory {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    inventory_type: row.get("inventory_type"),
                    inventory_data: row.get("inventory_data"),
                    key_id: row.get("key_id"),
                    secret_storage_id: row.get("secret_storage_id"),
                    ssh_login: row.get("ssh_login"),
                    ssh_port: row.get("ssh_port"),
                    extra_vars: row.get("extra_vars"),
                    ssh_key_id: row.get("ssh_key_id"),
                    become_key_id: row.get("become_key_id"),
                    vaults: row.get("vaults"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM inventory WHERE project_id = $1 ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Inventory {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    inventory_type: row.get("inventory_type"),
                    inventory_data: row.get("inventory_data"),
                    key_id: row.get("key_id"),
                    secret_storage_id: row.get("secret_storage_id"),
                    ssh_login: row.get("ssh_login"),
                    ssh_port: row.get("ssh_port"),
                    extra_vars: row.get("extra_vars"),
                    ssh_key_id: row.get("ssh_key_id"),
                    become_key_id: row.get("become_key_id"),
                    vaults: row.get("vaults"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `inventory` WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Inventory {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    inventory_type: row.get("inventory_type"),
                    inventory_data: row.get("inventory_data"),
                    key_id: row.get("key_id"),
                    secret_storage_id: row.get("secret_storage_id"),
                    ssh_login: row.get("ssh_login"),
                    ssh_port: row.get("ssh_port"),
                    extra_vars: row.get("extra_vars"),
                    ssh_key_id: row.get("ssh_key_id"),
                    become_key_id: row.get("become_key_id"),
                    vaults: row.get("vaults"),
                }).collect())
            }
        }
    }

    async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM inventory WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(inventory_id)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Инвентарь не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Inventory {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    inventory_type: row.get("inventory_type"),
                    inventory_data: row.get("inventory_data"),
                    key_id: row.get("key_id"),
                    secret_storage_id: row.get("secret_storage_id"),
                    ssh_login: row.get("ssh_login"),
                    ssh_port: row.get("ssh_port"),
                    extra_vars: row.get("extra_vars"),
                    ssh_key_id: row.get("ssh_key_id"),
                    become_key_id: row.get("become_key_id"),
                    vaults: row.get("vaults"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM inventory WHERE id = $1 AND project_id = $2";
                let row = sqlx::query(query)
                    .bind(inventory_id)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Инвентарь не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Inventory {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    inventory_type: row.get("inventory_type"),
                    inventory_data: row.get("inventory_data"),
                    key_id: row.get("key_id"),
                    secret_storage_id: row.get("secret_storage_id"),
                    ssh_login: row.get("ssh_login"),
                    ssh_port: row.get("ssh_port"),
                    extra_vars: row.get("extra_vars"),
                    ssh_key_id: row.get("ssh_key_id"),
                    become_key_id: row.get("become_key_id"),
                    vaults: row.get("vaults"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `inventory` WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(inventory_id)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Инвентарь не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Inventory {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    inventory_type: row.get("inventory_type"),
                    inventory_data: row.get("inventory_data"),
                    key_id: row.get("key_id"),
                    secret_storage_id: row.get("secret_storage_id"),
                    ssh_login: row.get("ssh_login"),
                    ssh_port: row.get("ssh_port"),
                    extra_vars: row.get("extra_vars"),
                    ssh_key_id: row.get("ssh_key_id"),
                    become_key_id: row.get("become_key_id"),
                    vaults: row.get("vaults"),
                })
            }
        }
    }

    async fn create_inventory(&self, mut inventory: Inventory) -> Result<Inventory> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO inventory (project_id, name, inventory_type, inventory_data, key_id, secret_storage_id, ssh_login, ssh_port, extra_vars, ssh_key_id, become_key_id, vaults) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(inventory.project_id)
                    .bind(&inventory.name)
                    .bind(&inventory.inventory_type)
                    .bind(&inventory.inventory_data)
                    .bind(inventory.key_id)
                    .bind(&inventory.secret_storage_id)
                    .bind(&inventory.ssh_login)
                    .bind(inventory.ssh_port)
                    .bind(&inventory.extra_vars)
                    .bind(&inventory.ssh_key_id)
                    .bind(&inventory.become_key_id)
                    .bind(&inventory.vaults)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                inventory.id = id;
                Ok(inventory)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO inventory (project_id, name, inventory_type, inventory_data, key_id, secret_storage_id, ssh_login, ssh_port, extra_vars, ssh_key_id, become_key_id, vaults) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(inventory.project_id)
                    .bind(&inventory.name)
                    .bind(&inventory.inventory_type)
                    .bind(&inventory.inventory_data)
                    .bind(inventory.key_id)
                    .bind(&inventory.secret_storage_id)
                    .bind(&inventory.ssh_login)
                    .bind(inventory.ssh_port)
                    .bind(&inventory.extra_vars)
                    .bind(&inventory.ssh_key_id)
                    .bind(&inventory.become_key_id)
                    .bind(&inventory.vaults)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                inventory.id = id;
                Ok(inventory)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `inventory` (project_id, name, inventory_type, inventory_data, key_id, secret_storage_id, ssh_login, ssh_port, extra_vars, ssh_key_id, become_key_id, vaults) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(inventory.project_id)
                    .bind(&inventory.name)
                    .bind(&inventory.inventory_type)
                    .bind(&inventory.inventory_data)
                    .bind(inventory.key_id)
                    .bind(&inventory.secret_storage_id)
                    .bind(&inventory.ssh_login)
                    .bind(inventory.ssh_port)
                    .bind(&inventory.extra_vars)
                    .bind(&inventory.ssh_key_id)
                    .bind(&inventory.become_key_id)
                    .bind(&inventory.vaults)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                let id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                inventory.id = id;
                Ok(inventory)
            }
        }
    }

    async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE inventory SET name = ?, inventory_type = ?, inventory_data = ?, key_id = ?, secret_storage_id = ?, ssh_login = ?, ssh_port = ?, extra_vars = ?, ssh_key_id = ?, become_key_id = ?, vaults = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&inventory.name)
                    .bind(&inventory.inventory_type)
                    .bind(&inventory.inventory_data)
                    .bind(inventory.key_id)
                    .bind(&inventory.secret_storage_id)
                    .bind(&inventory.ssh_login)
                    .bind(inventory.ssh_port)
                    .bind(&inventory.extra_vars)
                    .bind(&inventory.ssh_key_id)
                    .bind(&inventory.become_key_id)
                    .bind(&inventory.vaults)
                    .bind(inventory.id)
                    .bind(inventory.project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE inventory SET name = $1, inventory_type = $2, inventory_data = $3, key_id = $4, secret_storage_id = $5, ssh_login = $6, ssh_port = $7, extra_vars = $8, ssh_key_id = $9, become_key_id = $10, vaults = $11 WHERE id = $12 AND project_id = $13";
                sqlx::query(query)
                    .bind(&inventory.name)
                    .bind(&inventory.inventory_type)
                    .bind(&inventory.inventory_data)
                    .bind(inventory.key_id)
                    .bind(&inventory.secret_storage_id)
                    .bind(&inventory.ssh_login)
                    .bind(inventory.ssh_port)
                    .bind(&inventory.extra_vars)
                    .bind(&inventory.ssh_key_id)
                    .bind(&inventory.become_key_id)
                    .bind(&inventory.vaults)
                    .bind(inventory.id)
                    .bind(inventory.project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `inventory` SET name = ?, inventory_type = ?, inventory_data = ?, key_id = ?, secret_storage_id = ?, ssh_login = ?, ssh_port = ?, extra_vars = ?, ssh_key_id = ?, become_key_id = ?, vaults = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&inventory.name)
                    .bind(&inventory.inventory_type)
                    .bind(&inventory.inventory_data)
                    .bind(inventory.key_id)
                    .bind(&inventory.secret_storage_id)
                    .bind(&inventory.ssh_login)
                    .bind(inventory.ssh_port)
                    .bind(&inventory.extra_vars)
                    .bind(&inventory.ssh_key_id)
                    .bind(&inventory.become_key_id)
                    .bind(&inventory.vaults)
                    .bind(inventory.id)
                    .bind(inventory.project_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM inventory WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(inventory_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM inventory WHERE id = $1 AND project_id = $2";
                sqlx::query(query)
                    .bind(inventory_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `inventory` WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(inventory_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl RepositoryManager for SqlStore {
    async fn get_repositories(&self, project_id: i32) -> Result<Vec<Repository>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM repository WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Repository {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    git_url: row.get("git_url"),
                    git_type: row.get("git_type"),
                    git_branch: row.get("git_branch"),
                    key_id: row.get("key_id"),
                    git_path: row.get("git_path"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM repository WHERE project_id = $1 ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Repository {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    git_url: row.get("git_url"),
                    git_type: row.get("git_type"),
                    git_branch: row.get("git_branch"),
                    key_id: row.get("key_id"),
                    git_path: row.get("git_path"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `repository` WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Repository {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    git_url: row.get("git_url"),
                    git_type: row.get("git_type"),
                    git_branch: row.get("git_branch"),
                    key_id: row.get("key_id"),
                    git_path: row.get("git_path"),
                }).collect())
            }
        }
    }

    async fn get_repository(&self, project_id: i32, repository_id: i32) -> Result<Repository> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM repository WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Репозиторий не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Repository {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    git_url: row.get("git_url"),
                    git_type: row.get("git_type"),
                    git_branch: row.get("git_branch"),
                    key_id: row.get("key_id"),
                    git_path: row.get("git_path"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM repository WHERE id = $1 AND project_id = $2";
                let row = sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Репозиторий не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Repository {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    git_url: row.get("git_url"),
                    git_type: row.get("git_type"),
                    git_branch: row.get("git_branch"),
                    key_id: row.get("key_id"),
                    git_path: row.get("git_path"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `repository` WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Репозиторий не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Repository {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    git_url: row.get("git_url"),
                    git_type: row.get("git_type"),
                    git_branch: row.get("git_branch"),
                    key_id: row.get("key_id"),
                    git_path: row.get("git_path"),
                })
            }
        }
    }

    async fn create_repository(&self, mut repository: Repository) -> Result<Repository> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO repository (project_id, name, git_url, git_type, git_branch, key_id, git_path) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(repository.project_id)
                    .bind(&repository.name)
                    .bind(&repository.git_url)
                    .bind(&repository.git_type)
                    .bind(&repository.git_branch)
                    .bind(repository.key_id)
                    .bind(&repository.git_path)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                repository.id = id;
                Ok(repository)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO repository (project_id, name, git_url, git_type, git_branch, key_id, git_path) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(repository.project_id)
                    .bind(&repository.name)
                    .bind(&repository.git_url)
                    .bind(&repository.git_type)
                    .bind(&repository.git_branch)
                    .bind(repository.key_id)
                    .bind(&repository.git_path)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                repository.id = id;
                Ok(repository)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `repository` (project_id, name, git_url, git_type, git_branch, key_id, git_path) VALUES (?, ?, ?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(repository.project_id)
                    .bind(&repository.name)
                    .bind(&repository.git_url)
                    .bind(&repository.git_type)
                    .bind(&repository.git_branch)
                    .bind(repository.key_id)
                    .bind(&repository.git_path)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                let id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                repository.id = id;
                Ok(repository)
            }
        }
    }

    async fn update_repository(&self, repository: Repository) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE repository SET name = ?, git_url = ?, git_type = ?, git_branch = ?, key_id = ?, git_path = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&repository.name)
                    .bind(&repository.git_url)
                    .bind(&repository.git_type)
                    .bind(&repository.git_branch)
                    .bind(repository.key_id)
                    .bind(&repository.git_path)
                    .bind(repository.id)
                    .bind(repository.project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE repository SET name = $1, git_url = $2, git_type = $3, git_branch = $4, key_id = $5, git_path = $6 WHERE id = $6 AND project_id = $8";
                sqlx::query(query)
                    .bind(&repository.name)
                    .bind(&repository.git_url)
                    .bind(&repository.git_type)
                    .bind(&repository.git_branch)
                    .bind(repository.key_id)
                    .bind(&repository.git_path)
                    .bind(repository.id)
                    .bind(repository.project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `repository` SET name = ?, git_url = ?, git_type = ?, git_branch = ?, key_id = ?, git_path = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&repository.name)
                    .bind(&repository.git_url)
                    .bind(&repository.git_type)
                    .bind(&repository.git_branch)
                    .bind(repository.key_id)
                    .bind(&repository.git_path)
                    .bind(repository.id)
                    .bind(repository.project_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_repository(&self, project_id: i32, repository_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM repository WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM repository WHERE id = $1 AND project_id = $2";
                sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `repository` WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl EnvironmentManager for SqlStore {
    async fn get_environments(&self, project_id: i32) -> Result<Vec<Environment>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM environment WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM environment WHERE project_id = $1 ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `environment` WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                }).collect())
            }
        }
    }

    async fn get_environment(&self, project_id: i32, environment_id: i32) -> Result<Environment> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM environment WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Окружение не найдено".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM environment WHERE id = $1 AND project_id = $2";
                let row = sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Окружение не найдено".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `environment` WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Окружение не найдено".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                })
            }
        }
    }

    async fn create_environment(&self, mut environment: Environment) -> Result<Environment> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO environment (project_id, name, json, secret_storage_id, secrets) VALUES (?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(environment.project_id)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                environment.id = id;
                Ok(environment)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO environment (project_id, name, json, secret_storage_id, secrets) VALUES ($1, $2, $3, $4, $5) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(environment.project_id)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                environment.id = id;
                Ok(environment)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `environment` (project_id, name, json, secret_storage_id, secrets) VALUES (?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(environment.project_id)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                let id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                environment.id = id;
                Ok(environment)
            }
        }
    }

    async fn update_environment(&self, environment: Environment) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE environment SET name = ?, json = ?, secret_storage_id = ?, secrets = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .bind(environment.id)
                    .bind(environment.project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE environment SET name = $1, json = $2, secret_storage_id = $3, secrets = $4 WHERE id = $5 AND project_id = $6";
                sqlx::query(query)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .bind(environment.id)
                    .bind(environment.project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `environment` SET name = ?, json = ?, secret_storage_id = ?, secrets = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .bind(environment.id)
                    .bind(environment.project_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_environment(&self, project_id: i32, environment_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM environment WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM environment WHERE id = $1 AND project_id = $2";
                sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `environment` WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AccessKeyManager for SqlStore {
    async fn get_access_keys(&self, project_id: i32) -> Result<Vec<AccessKey>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM access_key WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM access_key WHERE project_id = $1 ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `access_key` WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                }).collect())
            }
        }
    }

    async fn get_access_key(&self, project_id: i32, key_id: i32) -> Result<AccessKey> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM access_key WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Ключ доступа не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM access_key WHERE id = $1 AND project_id = $2";
                let row = sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Ключ доступа не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `access_key` WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Ключ доступа не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                })
            }
        }
    }

    async fn create_access_key(&self, mut key: AccessKey) -> Result<AccessKey> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO access_key (project_id, name, type, user_id, login_password_login, login_password_password, ssh_key, ssh_passphrase, access_key_access_key, access_key_secret_key, secret_storage_id, owner, environment_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(key.project_id)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                key.id = id;
                Ok(key)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO access_key (project_id, name, type, user_id, login_password_login, login_password_password, ssh_key, ssh_passphrase, access_key_access_key, access_key_secret_key, secret_storage_id, owner, environment_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(key.project_id)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                key.id = id;
                Ok(key)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `access_key` (project_id, name, type, user_id, login_password_login, login_password_password, ssh_key, ssh_passphrase, access_key_access_key, access_key_secret_key, secret_storage_id, owner, environment_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(key.project_id)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                let id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                key.id = id;
                Ok(key)
            }
        }
    }

    async fn update_access_key(&self, key: AccessKey) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE access_key SET name = ?, type = ?, user_id = ?, login_password_login = ?, login_password_password = ?, ssh_key = ?, ssh_passphrase = ?, access_key_access_key = ?, access_key_secret_key = ?, secret_storage_id = ?, owner = ?, environment_id = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .bind(key.id)
                    .bind(key.project_id.unwrap_or(0))
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE access_key SET name = $1, type = $2, user_id = $3, login_password_login = $4, login_password_password = $5, ssh_key = $6, ssh_passphrase = $7, access_key_access_key = $8, access_key_secret_key = $9, secret_storage_id = $10, owner = $11, environment_id = $12 WHERE id = $13 AND project_id = $14";
                sqlx::query(query)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .bind(key.id)
                    .bind(key.project_id.unwrap_or(0))
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `access_key` SET name = ?, type = ?, user_id = ?, login_password_login = ?, login_password_password = ?, ssh_key = ?, ssh_passphrase = ?, access_key_access_key = ?, access_key_secret_key = ?, secret_storage_id = ?, owner = ?, environment_id = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .bind(key.id)
                    .bind(key.project_id.unwrap_or(0))
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_access_key(&self, project_id: i32, key_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM access_key WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM access_key WHERE id = $1 AND project_id = $2";
                sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `access_key` WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl TaskManager for SqlStore {
    async fn get_tasks(&self, _project_id: i32, _template_id: Option<i32>) -> Result<Vec<TaskWithTpl>> { Ok(vec![]) }
    async fn get_task(&self, _project_id: i32, _task_id: i32) -> Result<Task> { Err(Error::NotFound("Задача не найдена".to_string())) }
    async fn create_task(&self, _task: Task) -> Result<Task> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_task(&self, _task: Task) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_task(&self, _project_id: i32, _task_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn get_task_outputs(&self, _task_id: i32) -> Result<Vec<TaskOutput>> { Ok(vec![]) }
    async fn create_task_output(&self, _output: TaskOutput) -> Result<TaskOutput> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_task_status(&self, _project_id: i32, _task_id: i32, _status: TaskStatus) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl ScheduleManager for SqlStore {
    async fn get_schedules(&self, _project_id: i32) -> Result<Vec<Schedule>> { Ok(vec![]) }
    async fn get_schedule(&self, _project_id: i32, _schedule_id: i32) -> Result<Schedule> { Err(Error::NotFound("Расписание не найдено".to_string())) }
    async fn create_schedule(&self, _schedule: Schedule) -> Result<Schedule> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_schedule(&self, _schedule: Schedule) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_schedule(&self, _project_id: i32, _schedule_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn set_schedule_active(&self, _project_id: i32, _schedule_id: i32, _active: bool) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn set_schedule_commit_hash(&self, _project_id: i32, _schedule_id: i32, _hash: &str) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl SessionManager for SqlStore {
    async fn get_session(&self, _user_id: i32, _session_id: i32) -> Result<Session> { Err(Error::NotFound("Сессия не найдена".to_string())) }
    async fn create_session(&self, _session: Session) -> Result<Session> { Err(Error::Other("Не реализовано".to_string())) }
    async fn expire_session(&self, _user_id: i32, _session_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn verify_session(&self, _user_id: i32, _session_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn touch_session(&self, _user_id: i32, _session_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl TokenManager for SqlStore {
    async fn get_api_tokens(&self, _user_id: i32) -> Result<Vec<APIToken>> { Ok(vec![]) }
    async fn create_api_token(&self, _token: APIToken) -> Result<APIToken> { Err(Error::Other("Не реализовано".to_string())) }
    async fn get_api_token(&self, _token_id: &str) -> Result<APIToken> { Err(Error::NotFound("Токен не найден".to_string())) }
    async fn expire_api_token(&self, _user_id: i32, _token_id: &str) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_api_token(&self, _user_id: i32, _token_id: &str) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl EventManager for SqlStore {
    async fn get_events(&self, _project_id: Option<i32>, _limit: usize) -> Result<Vec<Event>> { Ok(vec![]) }
    async fn create_event(&self, _event: Event) -> Result<Event> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl RunnerManager for SqlStore {
    async fn get_runners(&self, _project_id: Option<i32>) -> Result<Vec<Runner>> { Ok(vec![]) }
    async fn get_runner(&self, _runner_id: i32) -> Result<Runner> { Err(Error::NotFound("Раннер не найден".to_string())) }
    async fn create_runner(&self, _runner: Runner) -> Result<Runner> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_runner(&self, _runner: Runner) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_runner(&self, _runner_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl ViewManager for SqlStore {
    async fn get_views(&self, _project_id: i32) -> Result<Vec<View>> { Ok(vec![]) }
    async fn get_view(&self, _project_id: i32, _view_id: i32) -> Result<View> { Err(Error::NotFound("Представление не найдено".to_string())) }
    async fn create_view(&self, _view: View) -> Result<View> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_view(&self, _view: View) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_view(&self, _project_id: i32, _view_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl IntegrationManager for SqlStore {
    async fn get_integrations(&self, _project_id: i32) -> Result<Vec<Integration>> { Ok(vec![]) }
    async fn get_integration(&self, _project_id: i32, _integration_id: i32) -> Result<Integration> { Err(Error::NotFound("Интеграция не найдена".to_string())) }
    async fn create_integration(&self, _integration: Integration) -> Result<Integration> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_integration(&self, _integration: Integration) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_integration(&self, _project_id: i32, _integration_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl ProjectInviteManager for SqlStore {
    async fn get_project_invites(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<ProjectInviteWithUser>> {
        self.get_project_invites(project_id, params).await
    }

    async fn create_project_invite(&self, invite: ProjectInvite) -> Result<ProjectInvite> {
        self.create_project_invite(invite).await
    }

    async fn get_project_invite(&self, project_id: i32, invite_id: i32) -> Result<ProjectInvite> {
        self.get_project_invite(project_id, invite_id).await
    }

    async fn get_project_invite_by_token(&self, token: &str) -> Result<ProjectInvite> {
        self.get_project_invite_by_token(token).await
    }

    async fn update_project_invite(&self, invite: ProjectInvite) -> Result<()> {
        self.update_project_invite(invite).await
    }

    async fn delete_project_invite(&self, project_id: i32, invite_id: i32) -> Result<()> {
        self.delete_project_invite(project_id, invite_id).await
    }
}

#[async_trait]
impl TerraformInventoryManager for SqlStore {
    async fn create_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<TerraformInventoryAlias> {
        self.create_terraform_inventory_alias(alias).await
    }

    async fn update_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<()> {
        self.update_terraform_inventory_alias(alias).await
    }

    async fn get_terraform_inventory_alias_by_alias(&self, alias: &str) -> Result<TerraformInventoryAlias> {
        self.get_terraform_inventory_alias_by_alias(alias).await
    }

    async fn get_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<TerraformInventoryAlias> {
        self.get_terraform_inventory_alias(project_id, inventory_id, alias_id).await
    }

    async fn get_terraform_inventory_aliases(&self, project_id: i32, inventory_id: i32) -> Result<Vec<TerraformInventoryAlias>> {
        self.get_terraform_inventory_aliases(project_id, inventory_id).await
    }

    async fn delete_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<()> {
        self.delete_terraform_inventory_alias(project_id, inventory_id, alias_id).await
    }

    async fn get_terraform_inventory_states(&self, project_id: i32, inventory_id: i32, params: RetrieveQueryParams) -> Result<Vec<TerraformInventoryState>> {
        self.get_terraform_inventory_states(project_id, inventory_id, params).await
    }

    async fn create_terraform_inventory_state(&self, state: TerraformInventoryState) -> Result<TerraformInventoryState> {
        self.create_terraform_inventory_state(state).await
    }

    async fn delete_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<()> {
        self.delete_terraform_inventory_state(project_id, inventory_id, state_id).await
    }

    async fn get_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<TerraformInventoryState> {
        self.get_terraform_inventory_state(project_id, inventory_id, state_id).await
    }

    async fn get_terraform_state_count(&self) -> Result<i32> {
        self.get_terraform_state_count().await
    }
}

#[async_trait]
impl SecretStorageManager for SqlStore {
    async fn get_secret_storages(&self, project_id: i32) -> Result<Vec<SecretStorage>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let storages = sqlx::query_as::<_, SecretStorage>(
                    "SELECT * FROM secret_storage WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool()?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(storages)
            }
            SqlDialect::PostgreSQL => {
                let storages = sqlx::query_as::<_, SecretStorage>(
                    "SELECT * FROM secret_storage WHERE project_id = $1 ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_postgres_pool()?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(storages)
            }
            SqlDialect::MySQL => {
                let storages = sqlx::query_as::<_, SecretStorage>(
                    "SELECT * FROM secret_storage WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_mysql_pool()?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(storages)
            }
        }
    }

    async fn get_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<SecretStorage> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let storage = sqlx::query_as::<_, SecretStorage>(
                    "SELECT * FROM secret_storage WHERE id = ? AND project_id = ?"
                )
                .bind(storage_id)
                .bind(project_id)
                .fetch_optional(self.get_sqlite_pool()?)
                .await
                .map_err(|e| Error::Database(e))?;

                storage.ok_or(Error::NotFound("SecretStorage not found".to_string()))
            }
            SqlDialect::PostgreSQL => {
                let storage = sqlx::query_as::<_, SecretStorage>(
                    "SELECT * FROM secret_storage WHERE id = $1 AND project_id = $2"
                )
                .bind(storage_id)
                .bind(project_id)
                .fetch_optional(self.get_postgres_pool()?)
                .await
                .map_err(|e| Error::Database(e))?;

                storage.ok_or(Error::NotFound("SecretStorage not found".to_string()))
            }
            SqlDialect::MySQL => {
                let storage = sqlx::query_as::<_, SecretStorage>(
                    "SELECT * FROM secret_storage WHERE id = ? AND project_id = ?"
                )
                .bind(storage_id)
                .bind(project_id)
                .fetch_optional(self.get_mysql_pool()?)
                .await
                .map_err(|e| Error::Database(e))?;

                storage.ok_or(Error::NotFound("SecretStorage not found".to_string()))
            }
        }
    }

    async fn create_secret_storage(&self, mut storage: SecretStorage) -> Result<SecretStorage> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO secret_storage (project_id, name, type, params, read_only) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(storage.project_id)
                .bind(&storage.name)
                .bind(&storage.r#type.to_string())
                .bind(&storage.params)
                .bind(storage.read_only)
                .execute(self.get_sqlite_pool()?)
                .await
                .map_err(|e| Error::Database(e))?;

                storage.id = result.last_insert_rowid() as i32;
                Ok(storage)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO secret_storage (project_id, name, type, params, read_only) VALUES ($1, $2, $3, $4, $5) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(storage.project_id)
                    .bind(&storage.name)
                    .bind(&storage.r#type.to_string())
                    .bind(&storage.params)
                    .bind(storage.read_only)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                storage.id = id;
                Ok(storage)
            }
            SqlDialect::MySQL => {
                let result = sqlx::query(
                    "INSERT INTO secret_storage (project_id, name, type, params, read_only) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(storage.project_id)
                .bind(&storage.name)
                .bind(&storage.r#type.to_string())
                .bind(&storage.params)
                .bind(storage.read_only)
                .execute(self.get_mysql_pool()?)
                .await
                .map_err(|e| Error::Database(e))?;

                storage.id = result.last_insert_id() as i32;
                Ok(storage)
            }
        }
    }

    async fn update_secret_storage(&self, storage: SecretStorage) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE secret_storage SET name = ?, type = ?, params = ?, read_only = ? WHERE id = ? AND project_id = ?"
                )
                .bind(&storage.name)
                .bind(&storage.r#type.to_string())
                .bind(&storage.params)
                .bind(storage.read_only)
                .bind(storage.id)
                .bind(storage.project_id)
                .execute(self.get_sqlite_pool()?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query(
                    "UPDATE secret_storage SET name = $1, type = $2, params = $3, read_only = $4 WHERE id = $5 AND project_id = $6"
                )
                .bind(&storage.name)
                .bind(&storage.r#type.to_string())
                .bind(&storage.params)
                .bind(storage.read_only)
                .bind(storage.id)
                .bind(storage.project_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                sqlx::query(
                    "UPDATE secret_storage SET name = ?, type = ?, params = ?, read_only = ? WHERE id = ? AND project_id = ?"
                )
                .bind(&storage.name)
                .bind(&storage.r#type.to_string())
                .bind(&storage.params)
                .bind(storage.read_only)
                .bind(storage.id)
                .bind(storage.project_id)
                .execute(self.get_mysql_pool()?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
        }

        Ok(())
    }

    async fn delete_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("DELETE FROM secret_storage WHERE id = ? AND project_id = ?")
                    .bind(storage_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM secret_storage WHERE id = $1 AND project_id = $2")
                    .bind(storage_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                sqlx::query("DELETE FROM secret_storage WHERE id = ? AND project_id = ?")
                    .bind(storage_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Store for SqlStore {}
