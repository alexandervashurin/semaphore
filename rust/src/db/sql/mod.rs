//! SQL-хранилище (SQLite)

pub mod runner;
pub mod types;
pub mod init;
pub mod migrations;
pub mod queries;
pub mod utils;

#[cfg(test)]
pub mod test_helpers;

// Decomposed modules by dialect
pub mod sqlite;
pub mod postgres;
pub mod mysql;

// Legacy modules (to be removed)
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
use crate::models::{User, UserTotp, Hook, Project, Task, TaskWithTpl, TaskOutput, TaskStage, Template, Inventory, Repository, Environment, AccessKey, Integration, Schedule, Session, APIToken, Event, Runner, View, Role, ProjectInvite, ProjectInviteWithUser, ProjectUser, RetrieveQueryParams, TerraformInventoryAlias, TerraformInventoryState, SecretStorage, SessionVerificationMethod};
use crate::error::{Error, Result};
use crate::services::task_logger::TaskStatus;
use crate::db::sql::types::{SqlDb, SqlDialect};
use async_trait::async_trait;
use sqlx::{SqlitePool, PgPool, MySqlPool, Row};
use std::collections::HashMap;
use chrono::Utc;

/// SQL-хранилище данных (на базе SQLite, MySQL, PostgreSQL)
pub struct SqlStore {
    db: SqlDb,
}

impl SqlStore {
    /// Создаёт новое SQL-хранилище
    pub async fn new(database_url: &str) -> Result<Self> {
        // Используем функцию создания подключения из init.rs
        let db = init::create_database_connection(database_url).await?;

        let store = Self { db };
        store.ensure_schema().await?;
        Ok(store)
    }

    /// Инициализирует схему БД при первом запуске (если таблицы не существуют)
    async fn ensure_schema(&self) -> Result<()> {
        if self.get_dialect() != SqlDialect::SQLite {
            return Ok(());
        }
        let pool = self.get_sqlite_pool()?;

        // Проверяем, есть ли таблица project (основная для CRUD)
        let project_exists: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='project'",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| Error::Database(e))?;

        if project_exists.is_some() {
            // Миграция: добавить колонку created в project__user, если её нет
            Self::migrate_project_user_created(pool).await?;
            return Ok(());
        }

        // Проверяем, есть ли таблица user (для обратной совместимости со старыми БД)
        let user_exists: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='user'",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| Error::Database(e))?;

        tracing::info!("Инициализация схемы БД (создание недостающих таблиц)...");

        // Таблица миграций
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS migration (version INTEGER PRIMARY KEY, name TEXT)",
        )
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

        if user_exists.is_none() {
            // Таблица пользователей (только при первом запуске)
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
                )",
            )
            .execute(pool)
            .await
            .map_err(|e| Error::Database(e))?;

            // Таблица опций
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS option (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            )
            .execute(pool)
            .await
            .map_err(|e| Error::Database(e))?;

            sqlx::query("INSERT OR IGNORE INTO migration (version, name) VALUES (1, 'initial_schema')")
                .execute(pool)
                .await
                .map_err(|e| Error::Database(e))?;
        }

        // Таблица проектов (для CRUD) — создаём если отсутствует
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created DATETIME NOT NULL,
                alert INTEGER NOT NULL DEFAULT 0,
                alert_chat TEXT,
                max_parallel_tasks INTEGER NOT NULL DEFAULT 0,
                type TEXT NOT NULL DEFAULT '',
                default_secret_storage_id INTEGER
            )",
        )
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

        // project__user для связи пользователей с проектами
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project__user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                user_id INTEGER NOT NULL REFERENCES user(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(project_id, user_id)
            )",
        )
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

        // task_output для логов задач (GET /api/.../tasks/{id}/output)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task_output (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                output TEXT NOT NULL,
                time DATETIME NOT NULL,
                stage_id INTEGER
            )",
        )
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

        // project_invite для приглашений в проект
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project_invite (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                user_id INTEGER NOT NULL REFERENCES user(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                created DATETIME NOT NULL,
                updated DATETIME NOT NULL,
                token TEXT NOT NULL DEFAULT '',
                inviter_user_id INTEGER NOT NULL
            )",
        )
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

        tracing::info!("Схема БД инициализирована");
        Ok(())
    }

    /// Миграция: добавить колонку created в project__user, если её нет
    async fn migrate_project_user_created(pool: &SqlitePool) -> Result<()> {
        let table_exists: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='project__user'",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| Error::Database(e))?;

        if table_exists.is_none() {
            return Ok(());
        }

        let has_created: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('project__user') WHERE name='created'",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| Error::Database(e))?;

        if has_created.unwrap_or(0) == 0 {
            sqlx::query(
                "ALTER TABLE project__user ADD COLUMN created DATETIME NOT NULL DEFAULT '2020-01-01 00:00:00'",
            )
            .execute(pool)
            .await
            .map_err(|e| Error::Database(e))?;
            tracing::info!("Миграция: добавлена колонка created в project__user");
        }
        Ok(())
    }

    #[cfg(test)]
    /// Инициализирует таблицу user для тестов (без миграций)
    pub async fn init_user_table_for_test(&self) -> Result<()> {
        let schema = "CREATE TABLE IF NOT EXISTS user (
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
        )";
        sqlx::query(schema)
            .execute(self.get_sqlite_pool()?)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(())
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
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM \"user\" ORDER BY id";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

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
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `user` ORDER BY id";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

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
        }
    }

    async fn create_user(&self, user: User, password: &str) -> Result<User> {
        use crate::api::auth_local::hash_password;
        
        // Хешируем пароль перед сохранением
        let password_hash = hash_password(password)?;
        
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO user (username, name, email, password, admin, external, alert, pro, created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
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
                    .bind(&password_hash)
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
                    .bind(&password_hash)
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
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM \"user\" WHERE admin = $1";
                let rows = sqlx::query(query)
                    .bind(true)
                    .fetch_all(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

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
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `user` WHERE admin = 1";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

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
                    created: row.try_get("created").unwrap_or_else(|_| Utc::now()),
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
                    created: row.try_get("created").unwrap_or_else(|_| Utc::now()),
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
                    created: row.try_get("created").unwrap_or_else(|_| Utc::now()),
                    username: row.get("username"),
                    name: row.get("name"),
                }).collect())
            }
        }
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
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("UPDATE user SET totp = ? WHERE id = ?")
                    .bind(&totp_json)
                    .bind(user_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("UPDATE \"user\" SET totp = $1 WHERE id = $2")
                    .bind(&totp_json)
                    .bind(user_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                sqlx::query("UPDATE `user` SET totp = ? WHERE id = ?")
                    .bind(&totp_json)
                    .bind(user_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_user_totp(&self, user_id: i32) -> Result<()> {
        // Удаляем TOTP (устанавливаем в NULL)
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("UPDATE user SET totp = NULL WHERE id = ?")
                    .bind(user_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("UPDATE \"user\" SET totp = NULL WHERE id = $1")
                    .bind(user_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                sqlx::query("UPDATE `user` SET totp = NULL WHERE id = ?")
                    .bind(user_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl HookManager for SqlStore {
    async fn get_hooks_by_template(&self, template_id: i32) -> Result<Vec<Hook>> {
        // Заглушка - возвращаем пустой список
        Ok(Vec::new())
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

    async fn create_project_user(&self, project_user: ProjectUser) -> Result<()> {
        let role_str = project_user.role.to_string();
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("INSERT INTO project__user (project_id, user_id, role, created) VALUES (?, ?, ?, ?)")
                    .bind(project_user.project_id)
                    .bind(project_user.user_id)
                    .bind(&role_str)
                    .bind(project_user.created)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("INSERT INTO project__user (project_id, user_id, role, created) VALUES ($1, $2, $3, $4)")
                    .bind(project_user.project_id)
                    .bind(project_user.user_id)
                    .bind(&role_str)
                    .bind(project_user.created)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                sqlx::query("INSERT INTO project__user (project_id, user_id, `role`, created) VALUES (?, ?, ?, ?)")
                    .bind(project_user.project_id)
                    .bind(project_user.user_id)
                    .bind(&role_str)
                    .bind(project_user.created)
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
                let query = "SELECT * FROM template WHERE project_id = ? ORDER BY name";
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
                    inventory_id: row.try_get("inventory_id").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok().flatten(),
                    environment_id: row.try_get("environment_id").ok().flatten(),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM template WHERE project_id = $1 ORDER BY name";
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
                    inventory_id: row.try_get("inventory_id").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok().flatten(),
                    environment_id: row.try_get("environment_id").ok().flatten(),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `template` WHERE project_id = ? ORDER BY name";
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
                    inventory_id: row.try_get("inventory_id").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok().flatten(),
                    environment_id: row.try_get("environment_id").ok().flatten(),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                }).collect())
            }
        }
    }

    async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM template WHERE id = ? AND project_id = ?";
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
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM template WHERE id = $1 AND project_id = $2";
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
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `template` WHERE id = ? AND project_id = ?";
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
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                })
            }
        }
    }

    async fn create_template(&self, mut template: Template) -> Result<Template> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO template (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, created, arguments, vault_key_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
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
                    .bind(template.created)
                    .bind(&template.arguments)
                    .bind(template.vault_key_id)
                    .fetch_one(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                template.id = id;
                Ok(template)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO template (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, created, arguments, vault_key_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id";
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
                    .bind(template.created)
                    .bind(&template.arguments)
                    .bind(template.vault_key_id)
                    .fetch_one(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                template.id = id;
                Ok(template)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `template` (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, created, arguments, vault_key_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
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
                    .bind(template.created)
                    .bind(&template.arguments)
                    .bind(template.vault_key_id)
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
                let query = "UPDATE template SET name = ?, playbook = ?, description = ?, inventory_id = ?, repository_id = ?, environment_id = ?, type = ?, app = ?, git_branch = ?, arguments = ?, vault_key_id = ? WHERE id = ? AND project_id = ?";
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
                    .bind(&template.vault_key_id)
                    .bind(template.id)
                    .bind(template.project_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE template SET name = $1, playbook = $2, description = $3, inventory_id = $4, repository_id = $5, environment_id = $6, type = $7, app = $8, git_branch = $9, arguments = $10, vault_key_id = $11 WHERE id = $12 AND project_id = $13";
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
                    .bind(&template.vault_key_id)
                    .bind(template.id)
                    .bind(template.project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `template` SET name = ?, playbook = ?, description = ?, inventory_id = ?, repository_id = ?, environment_id = ?, type = ?, app = ?, git_branch = ?, arguments = ?, vault_key_id = ? WHERE id = ? AND project_id = ?";
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
                    .bind(&template.vault_key_id)
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
                    created: row.get("created"),
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
    async fn get_tasks(&self, project_id: i32, template_id: Option<i32>) -> Result<Vec<TaskWithTpl>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = if template_id.is_some() {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN template tpl ON t.template_id = tpl.id LEFT JOIN \"user\" u ON t.user_id = u.id WHERE t.project_id = ? AND t.template_id = ? ORDER BY t.created DESC"
                } else {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN template tpl ON t.template_id = tpl.id LEFT JOIN \"user\" u ON t.user_id = u.id WHERE t.project_id = ? ORDER BY t.created DESC"
                };
                let mut q = sqlx::query(query).bind(project_id);
                if let Some(tid) = template_id {
                    q = q.bind(tid);
                }
                let rows = q.fetch_all(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskWithTpl {
                    task: Task {
                        id: row.get("id"),
                        template_id: row.get("template_id"),
                        project_id: row.get("project_id"),
                        status: row.get("status"),
                        playbook: row.try_get("playbook").ok().flatten(),
                        environment: row.try_get("environment").ok().flatten(),
                        secret: None,
                        arguments: row.try_get("arguments").ok().flatten(),
                        git_branch: row.try_get("git_branch").ok().flatten(),
                        user_id: row.try_get("user_id").ok(),
                        integration_id: row.try_get("integration_id").ok(),
                        schedule_id: row.try_get("schedule_id").ok(),
                        created: row.get("created"),
                        start: row.try_get("start_time").ok(),
                        end: row.try_get("end_time").ok(),
                        message: row.try_get("message").ok().flatten(),
                        commit_hash: row.try_get("commit_hash").ok().flatten(),
                        commit_message: row.try_get("commit_message").ok().flatten(),
                        build_task_id: row.try_get("build_task_id").ok(),
                        version: row.try_get("version").ok().flatten(),
                        inventory_id: row.try_get("inventory_id").ok(),
                        repository_id: row.try_get("repository_id").ok(),
                        environment_id: row.try_get("environment_id").ok(),
                        params: None,
                    },
                    tpl_playbook: row.get("tpl_playbook"),
                    tpl_type: row.try_get("tpl_type").ok(),
                    tpl_app: row.try_get("tpl_app").ok(),
                    user_name: row.try_get("user_name").ok(),
                    build_task: None,
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = if template_id.is_some() {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN template tpl ON t.template_id = tpl.id LEFT JOIN \"user\" u ON t.user_id = u.id WHERE t.project_id = $1 AND t.template_id = $2 ORDER BY t.created DESC"
                } else {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN template tpl ON t.template_id = tpl.id LEFT JOIN \"user\" u ON t.user_id = u.id WHERE t.project_id = $1 ORDER BY t.created DESC"
                };
                let mut q = sqlx::query(query).bind(project_id);
                if let Some(tid) = template_id {
                    q = q.bind(tid);
                }
                let rows = q.fetch_all(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskWithTpl {
                    task: Task {
                        id: row.get("id"),
                        template_id: row.get("template_id"),
                        project_id: row.get("project_id"),
                        status: row.get("status"),
                        playbook: row.try_get("playbook").ok().flatten(),
                        environment: row.try_get("environment").ok().flatten(),
                        secret: None,
                        arguments: row.try_get("arguments").ok().flatten(),
                        git_branch: row.try_get("git_branch").ok().flatten(),
                        user_id: row.try_get("user_id").ok(),
                        integration_id: row.try_get("integration_id").ok(),
                        schedule_id: row.try_get("schedule_id").ok(),
                        created: row.get("created"),
                        start: row.try_get("start_time").ok(),
                        end: row.try_get("end_time").ok(),
                        message: row.try_get("message").ok().flatten(),
                        commit_hash: row.try_get("commit_hash").ok().flatten(),
                        commit_message: row.try_get("commit_message").ok().flatten(),
                        build_task_id: row.try_get("build_task_id").ok(),
                        version: row.try_get("version").ok().flatten(),
                        inventory_id: row.try_get("inventory_id").ok(),
                        repository_id: row.try_get("repository_id").ok(),
                        environment_id: row.try_get("environment_id").ok(),
                        params: None,
                    },
                    tpl_playbook: row.get("tpl_playbook"),
                    tpl_type: row.try_get("tpl_type").ok(),
                    tpl_app: row.try_get("tpl_app").ok(),
                    user_name: row.try_get("user_name").ok(),
                    build_task: None,
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = if template_id.is_some() {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN `template` tpl ON t.template_id = tpl.id LEFT JOIN `user` u ON t.user_id = u.id WHERE t.project_id = ? AND t.template_id = ? ORDER BY t.created DESC"
                } else {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN `template` tpl ON t.template_id = tpl.id LEFT JOIN `user` u ON t.user_id = u.id WHERE t.project_id = ? ORDER BY t.created DESC"
                };
                let mut q = sqlx::query(query).bind(project_id);
                if let Some(tid) = template_id {
                    q = q.bind(tid);
                }
                let rows = q.fetch_all(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskWithTpl {
                    task: Task {
                        id: row.get("id"),
                        template_id: row.get("template_id"),
                        project_id: row.get("project_id"),
                        status: row.get("status"),
                        playbook: row.try_get("playbook").ok().flatten(),
                        environment: row.try_get("environment").ok().flatten(),
                        secret: None,
                        arguments: row.try_get("arguments").ok().flatten(),
                        git_branch: row.try_get("git_branch").ok().flatten(),
                        user_id: row.try_get("user_id").ok(),
                        integration_id: row.try_get("integration_id").ok(),
                        schedule_id: row.try_get("schedule_id").ok(),
                        created: row.get("created"),
                        start: row.try_get("start_time").ok(),
                        end: row.try_get("end_time").ok(),
                        message: row.try_get("message").ok().flatten(),
                        commit_hash: row.try_get("commit_hash").ok().flatten(),
                        commit_message: row.try_get("commit_message").ok().flatten(),
                        build_task_id: row.try_get("build_task_id").ok(),
                        version: row.try_get("version").ok().flatten(),
                        inventory_id: row.try_get("inventory_id").ok(),
                        repository_id: row.try_get("repository_id").ok(),
                        environment_id: row.try_get("environment_id").ok(),
                        params: None,
                    },
                    tpl_playbook: row.try_get("tpl_playbook").ok().flatten(),
                    tpl_type: row.try_get("tpl_type").ok(),
                    tpl_app: row.try_get("tpl_app").ok(),
                    user_name: row.try_get("user_name").ok(),
                    build_task: None,
                }).collect())
            }
        }
    }

    async fn get_task(&self, _project_id: i32, task_id: i32) -> Result<Task> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM task WHERE id = ?";
                let row = sqlx::query(query).bind(task_id).fetch_one(self.get_sqlite_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Задача не найдена".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Task {
                    id: row.get("id"),
                    template_id: row.get("template_id"),
                    project_id: row.get("project_id"),
                    status: row.get("status"),
                    playbook: row.try_get("playbook").ok().flatten(),
                    environment: row.try_get("environment").ok().flatten(),
                    secret: None,
                    arguments: row.try_get("arguments").ok().flatten(),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    user_id: row.try_get("user_id").ok(),
                    integration_id: row.try_get("integration_id").ok(),
                    schedule_id: row.try_get("schedule_id").ok(),
                    created: row.get("created"),
                    start: row.try_get("start_time").ok(),
                    end: row.try_get("end_time").ok(),
                    message: row.try_get("message").ok().flatten(),
                    commit_hash: row.try_get("commit_hash").ok().flatten(),
                    commit_message: row.try_get("commit_message").ok().flatten(),
                    build_task_id: row.try_get("build_task_id").ok(),
                    version: row.try_get("version").ok().flatten(),
                    inventory_id: row.try_get("inventory_id").ok(),
                    repository_id: row.try_get("repository_id").ok(),
                    environment_id: row.try_get("environment_id").ok(),
                    params: None,
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM task WHERE id = $1";
                let row = sqlx::query(query).bind(task_id).fetch_one(self.get_postgres_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Задача не найдена".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Task {
                    id: row.get("id"),
                    template_id: row.get("template_id"),
                    project_id: row.get("project_id"),
                    status: row.get("status"),
                    playbook: row.try_get("playbook").ok().flatten(),
                    environment: row.try_get("environment").ok().flatten(),
                    secret: None,
                    arguments: row.try_get("arguments").ok().flatten(),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    user_id: row.try_get("user_id").ok(),
                    integration_id: row.try_get("integration_id").ok(),
                    schedule_id: row.try_get("schedule_id").ok(),
                    created: row.get("created"),
                    start: row.try_get("start_time").ok(),
                    end: row.try_get("end_time").ok(),
                    message: row.try_get("message").ok().flatten(),
                    commit_hash: row.try_get("commit_hash").ok().flatten(),
                    commit_message: row.try_get("commit_message").ok().flatten(),
                    build_task_id: row.try_get("build_task_id").ok(),
                    version: row.try_get("version").ok().flatten(),
                    inventory_id: row.try_get("inventory_id").ok(),
                    repository_id: row.try_get("repository_id").ok(),
                    environment_id: row.try_get("environment_id").ok(),
                    params: None,
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `task` WHERE id = ?";
                let row = sqlx::query(query).bind(task_id).fetch_one(self.get_mysql_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Задача не найдена".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Task {
                    id: row.get("id"),
                    template_id: row.get("template_id"),
                    project_id: row.get("project_id"),
                    status: row.get("status"),
                    playbook: row.try_get("playbook").ok().flatten(),
                    environment: row.try_get("environment").ok().flatten(),
                    secret: None,
                    arguments: row.try_get("arguments").ok().flatten(),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    user_id: row.try_get("user_id").ok(),
                    integration_id: row.try_get("integration_id").ok(),
                    schedule_id: row.try_get("schedule_id").ok(),
                    created: row.get("created"),
                    start: row.try_get("start_time").ok(),
                    end: row.try_get("end_time").ok(),
                    message: row.try_get("message").ok().flatten(),
                    commit_hash: row.try_get("commit_hash").ok().flatten(),
                    commit_message: row.try_get("commit_message").ok().flatten(),
                    build_task_id: row.try_get("build_task_id").ok(),
                    version: row.try_get("version").ok().flatten(),
                    inventory_id: row.try_get("inventory_id").ok(),
                    repository_id: row.try_get("repository_id").ok(),
                    environment_id: row.try_get("environment_id").ok(),
                    params: None,
                })
            }
        }
    }

    async fn create_task(&self, mut task: Task) -> Result<Task> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO task (template_id, project_id, status, playbook, environment, arguments, git_branch, user_id, integration_id, schedule_id, created, start_time, end_time, message, commit_hash, commit_message, build_task_id, version, inventory_id, repository_id, environment_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(task.template_id)
                    .bind(task.project_id)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(task.created)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .fetch_one(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
                task.id = id;
                Ok(task)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO task (template_id, project_id, status, playbook, environment, arguments, git_branch, user_id, integration_id, schedule_id, created, start_time, end_time, message, commit_hash, commit_message, build_task_id, version, inventory_id, repository_id, environment_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(task.template_id)
                    .bind(task.project_id)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(task.created)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .fetch_one(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
                task.id = id;
                Ok(task)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `task` (template_id, project_id, status, playbook, environment, arguments, git_branch, user_id, integration_id, schedule_id, created, start_time, end_time, message, commit_hash, commit_message, build_task_id, version, inventory_id, repository_id, environment_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
                let result = sqlx::query(query)
                    .bind(task.template_id)
                    .bind(task.project_id)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(task.created)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
                task.id = result.last_insert_id() as i32;
                Ok(task)
            }
        }
    }

    async fn update_task(&self, task: Task) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE task SET status = ?, playbook = ?, environment = ?, arguments = ?, git_branch = ?, user_id = ?, integration_id = ?, schedule_id = ?, start_time = ?, end_time = ?, message = ?, commit_hash = ?, commit_message = ?, build_task_id = ?, version = ?, inventory_id = ?, repository_id = ?, environment_id = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .bind(task.id)
                    .execute(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE task SET status = $1, playbook = $2, environment = $3, arguments = $4, git_branch = $5, user_id = $6, integration_id = $7, schedule_id = $8, start_time = $9, end_time = $10, message = $11, commit_hash = $12, commit_message = $13, build_task_id = $14, version = $15, inventory_id = $16, repository_id = $17, environment_id = $18 WHERE id = $19";
                sqlx::query(query)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .bind(task.id)
                    .execute(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `task` SET status = ?, playbook = ?, environment = ?, arguments = ?, git_branch = ?, user_id = ?, integration_id = ?, schedule_id = ?, start_time = ?, end_time = ?, message = ?, commit_hash = ?, commit_message = ?, build_task_id = ?, version = ?, inventory_id = ?, repository_id = ?, environment_id = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .bind(task.id)
                    .execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_task(&self, _project_id: i32, task_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM task WHERE id = ?";
                sqlx::query(query).bind(task_id).execute(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM task WHERE id = $1";
                sqlx::query(query).bind(task_id).execute(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `task` WHERE id = ?";
                sqlx::query(query).bind(task_id).execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn get_task_outputs(&self, task_id: i32) -> Result<Vec<TaskOutput>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM task_output WHERE task_id = ? ORDER BY time";
                let rows = sqlx::query(query).bind(task_id).fetch_all(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskOutput {
                    id: row.get("id"),
                    task_id: row.get("task_id"),
                    project_id: row.get("project_id"),
                    stage_id: row.try_get("stage_id").ok(),
                    time: row.get("time"),
                    output: row.get("output"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM task_output WHERE task_id = $1 ORDER BY time";
                let rows = sqlx::query(query).bind(task_id).fetch_all(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskOutput {
                    id: row.get("id"),
                    task_id: row.get("task_id"),
                    project_id: row.get("project_id"),
                    stage_id: row.try_get("stage_id").ok(),
                    time: row.get("time"),
                    output: row.get("output"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `task_output` WHERE task_id = ? ORDER BY time";
                let rows = sqlx::query(query).bind(task_id).fetch_all(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskOutput {
                    id: row.get("id"),
                    task_id: row.get("task_id"),
                    project_id: row.get("project_id"),
                    stage_id: row.try_get("stage_id").ok(),
                    time: row.get("time"),
                    output: row.get("output"),
                }).collect())
            }
        }
    }

    async fn create_task_output(&self, mut output: TaskOutput) -> Result<TaskOutput> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO task_output (task_id, project_id, time, output) VALUES (?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(output.task_id)
                    .bind(output.project_id)
                    .bind(output.time)
                    .bind(&output.output)
                    .fetch_one(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
                output.id = id;
                Ok(output)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO task_output (task_id, project_id, time, output) VALUES ($1, $2, $3, $4) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(output.task_id)
                    .bind(output.project_id)
                    .bind(output.time)
                    .bind(&output.output)
                    .fetch_one(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
                output.id = id;
                Ok(output)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `task_output` (task_id, project_id, time, output) VALUES (?, ?, ?, ?)";
                let result = sqlx::query(query)
                    .bind(output.task_id)
                    .bind(output.project_id)
                    .bind(output.time)
                    .bind(&output.output)
                    .execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
                output.id = result.last_insert_id() as i32;
                Ok(output)
            }
        }
    }

    async fn update_task_status(&self, _project_id: i32, task_id: i32, status: TaskStatus) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE task SET status = ? WHERE id = ?";
                sqlx::query(query).bind(&status.to_string()).bind(task_id).execute(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE task SET status = $1 WHERE id = $2";
                sqlx::query(query).bind(&status.to_string()).bind(task_id).execute(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `task` SET status = ? WHERE id = ?";
                sqlx::query(query).bind(&status.to_string()).bind(task_id).execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ScheduleManager for SqlStore {
    async fn get_schedules(&self, project_id: i32) -> Result<Vec<Schedule>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM schedule WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query).bind(project_id).fetch_all(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM schedule WHERE project_id = $1 ORDER BY name";
                let rows = sqlx::query(query).bind(project_id).fetch_all(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `schedule` WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query).bind(project_id).fetch_all(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                }).collect())
            }
        }
    }

    async fn get_schedule(&self, _project_id: i32, schedule_id: i32) -> Result<Schedule> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM schedule WHERE id = ?";
                let row = sqlx::query(query).bind(schedule_id).fetch_one(self.get_sqlite_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Расписание не найдено".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM schedule WHERE id = $1";
                let row = sqlx::query(query).bind(schedule_id).fetch_one(self.get_postgres_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Расписание не найдено".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `schedule` WHERE id = ?";
                let row = sqlx::query(query).bind(schedule_id).fetch_one(self.get_mysql_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Расписание не найдено".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                })
            }
        }
    }

    async fn create_schedule(&self, mut schedule: Schedule) -> Result<Schedule> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO schedule (project_id, template_id, cron, name, active, created) VALUES (?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(schedule.project_id)
                    .bind(schedule.template_id)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(&schedule.created)
                    .fetch_one(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
                schedule.id = id;
                Ok(schedule)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO schedule (project_id, template_id, cron, name, active, created) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(schedule.project_id)
                    .bind(schedule.template_id)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(&schedule.created)
                    .fetch_one(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
                schedule.id = id;
                Ok(schedule)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `schedule` (project_id, template_id, cron, name, active, created) VALUES (?, ?, ?, ?, ?, ?)";
                let result = sqlx::query(query)
                    .bind(schedule.project_id)
                    .bind(schedule.template_id)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(&schedule.created)
                    .execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
                schedule.id = result.last_insert_id() as i32;
                Ok(schedule)
            }
        }
    }

    async fn update_schedule(&self, schedule: Schedule) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE schedule SET cron = ?, name = ?, active = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(schedule.id)
                    .execute(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE schedule SET cron = $1, name = $2, active = $3 WHERE id = $4";
                sqlx::query(query)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(schedule.id)
                    .execute(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `schedule` SET cron = ?, name = ?, active = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(schedule.id)
                    .execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_schedule(&self, _project_id: i32, schedule_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM schedule WHERE id = ?";
                sqlx::query(query).bind(schedule_id).execute(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM schedule WHERE id = $1";
                sqlx::query(query).bind(schedule_id).execute(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `schedule` WHERE id = ?";
                sqlx::query(query).bind(schedule_id).execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn set_schedule_active(&self, _project_id: i32, schedule_id: i32, active: bool) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE schedule SET active = ? WHERE id = ?";
                sqlx::query(query).bind(active).bind(schedule_id).execute(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE schedule SET active = $1 WHERE id = $2";
                sqlx::query(query).bind(active).bind(schedule_id).execute(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `schedule` SET active = ? WHERE id = ?";
                sqlx::query(query).bind(active).bind(schedule_id).execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn set_schedule_commit_hash(&self, _project_id: i32, schedule_id: i32, hash: &str) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("UPDATE schedule SET last_commit_hash = ? WHERE id = ?")
                    .bind(hash)
                    .bind(schedule_id)
                    .execute(self.get_sqlite_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("UPDATE schedule SET last_commit_hash = $1 WHERE id = $2")
                    .bind(hash)
                    .bind(schedule_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                sqlx::query("UPDATE `schedule` SET last_commit_hash = ? WHERE id = ?")
                    .bind(hash)
                    .bind(schedule_id)
                    .execute(self.get_mysql_pool()?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl SessionManager for SqlStore {
    async fn get_session(&self, _user_id: i32, session_id: i32) -> Result<Session> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM session WHERE id = ?";
                let row = sqlx::query(query).bind(session_id).fetch_one(self.get_sqlite_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Сессия не найдена".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Session {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    created: row.get("created"),
                    last_active: row.get("last_active"),
                    ip: row.try_get("ip").ok().unwrap_or_default(),
                    user_agent: row.try_get("user_agent").ok().unwrap_or_default(),
                    expired: row.get("expired"),
                    verification_method: row.try_get("verification_method").ok().unwrap_or(SessionVerificationMethod::None),
                    verified: row.try_get("verified").ok().unwrap_or(false),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM session WHERE id = $1";
                let row = sqlx::query(query).bind(session_id).fetch_one(self.get_postgres_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Сессия не найдена".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Session {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    created: row.get("created"),
                    last_active: row.get("last_active"),
                    ip: row.try_get("ip").ok().unwrap_or_default(),
                    user_agent: row.try_get("user_agent").ok().unwrap_or_default(),
                    expired: row.get("expired"),
                    verification_method: row.try_get("verification_method").ok().unwrap_or(SessionVerificationMethod::None),
                    verified: row.try_get("verified").ok().unwrap_or(false),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `session` WHERE id = ?";
                let row = sqlx::query(query).bind(session_id).fetch_one(self.get_mysql_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Сессия не найдена".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Session {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    created: row.get("created"),
                    last_active: row.get("last_active"),
                    ip: row.try_get("ip").ok().unwrap_or_default(),
                    user_agent: row.try_get("user_agent").ok().unwrap_or_default(),
                    expired: row.get("expired"),
                    verification_method: row.try_get("verification_method").ok().unwrap_or(SessionVerificationMethod::None),
                    verified: row.try_get("verified").ok().unwrap_or(false),
                })
            }
        }
    }

    async fn create_session(&self, mut session: Session) -> Result<Session> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO session (user_id, created, last_active, ip, user_agent, expired, verification_method, verified) VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(session.user_id)
                    .bind(session.created)
                    .bind(session.last_active)
                    .bind(&session.ip)
                    .bind(&session.user_agent)
                    .bind(session.expired)
                    .bind(&session.verification_method)
                    .bind(session.verified)
                    .fetch_one(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
                session.id = id;
                Ok(session)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO session (user_id, created, last_active, ip, user_agent, expired, verification_method, verified) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(session.user_id)
                    .bind(session.created)
                    .bind(session.last_active)
                    .bind(&session.ip)
                    .bind(&session.user_agent)
                    .bind(session.expired)
                    .bind(&session.verification_method)
                    .bind(session.verified)
                    .fetch_one(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
                session.id = id;
                Ok(session)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `session` (user_id, created, last_active, ip, user_agent, expired, verification_method, verified) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
                let result = sqlx::query(query)
                    .bind(session.user_id)
                    .bind(session.created)
                    .bind(session.last_active)
                    .bind(&session.ip)
                    .bind(&session.user_agent)
                    .bind(session.expired)
                    .bind(&session.verification_method)
                    .bind(session.verified)
                    .execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
                session.id = result.last_insert_id() as i32;
                Ok(session)
            }
        }
    }

    async fn expire_session(&self, _user_id: i32, session_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE session SET expired = 1 WHERE id = ?";
                sqlx::query(query).bind(session_id).execute(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE session SET expired = TRUE WHERE id = $1";
                sqlx::query(query).bind(session_id).execute(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `session` SET expired = 1 WHERE id = ?";
                sqlx::query(query).bind(session_id).execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn verify_session(&self, _user_id: i32, _session_id: i32) -> Result<()> {
        // TODO: реализовать проверку сессии
        Ok(())
    }

    async fn touch_session(&self, _user_id: i32, session_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE session SET last_active = ? WHERE id = ?";
                sqlx::query(query).bind(Utc::now()).bind(session_id).execute(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE session SET last_active = $1 WHERE id = $2";
                sqlx::query(query).bind(Utc::now()).bind(session_id).execute(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `session` SET last_active = ? WHERE id = ?";
                sqlx::query(query).bind(Utc::now()).bind(session_id).execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl TokenManager for SqlStore {
    async fn get_api_tokens(&self, user_id: i32) -> Result<Vec<APIToken>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM api_token WHERE user_id = ? ORDER BY created DESC";
                let rows = sqlx::query(query).bind(user_id).fetch_all(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| APIToken {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    name: row.get("name"),
                    token: row.get("token"),
                    created: row.get("created"),
                    expired: row.get("expired"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM api_token WHERE user_id = $1 ORDER BY created DESC";
                let rows = sqlx::query(query).bind(user_id).fetch_all(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| APIToken {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    name: row.get("name"),
                    token: row.get("token"),
                    created: row.get("created"),
                    expired: row.get("expired"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `api_token` WHERE user_id = ? ORDER BY created DESC";
                let rows = sqlx::query(query).bind(user_id).fetch_all(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| APIToken {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    name: row.get("name"),
                    token: row.get("token"),
                    created: row.get("created"),
                    expired: row.get("expired"),
                }).collect())
            }
        }
    }

    async fn create_api_token(&self, mut token: APIToken) -> Result<APIToken> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO api_token (user_id, name, token, created, expired) VALUES (?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(token.user_id)
                    .bind(&token.name)
                    .bind(&token.token)
                    .bind(token.created)
                    .bind(token.expired)
                    .fetch_one(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
                token.id = id;
                Ok(token)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO api_token (user_id, name, token, created, expired) VALUES ($1, $2, $3, $4, $5) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(token.user_id)
                    .bind(&token.name)
                    .bind(&token.token)
                    .bind(token.created)
                    .bind(token.expired)
                    .fetch_one(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
                token.id = id;
                Ok(token)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `api_token` (user_id, name, token, created, expired) VALUES (?, ?, ?, ?, ?)";
                let result = sqlx::query(query)
                    .bind(token.user_id)
                    .bind(&token.name)
                    .bind(&token.token)
                    .bind(token.created)
                    .bind(token.expired)
                    .execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
                token.id = result.last_insert_id() as i32;
                Ok(token)
            }
        }
    }

    async fn get_api_token(&self, token_id: i32) -> Result<APIToken> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM api_token WHERE id = ?";
                let row = sqlx::query(query).bind(token_id).fetch_one(self.get_sqlite_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Токен не найден".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(APIToken {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    name: row.get("name"),
                    token: row.get("token"),
                    created: row.get("created"),
                    expired: row.get("expired"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM api_token WHERE id = $1";
                let row = sqlx::query(query).bind(token_id).fetch_one(self.get_postgres_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Токен не найден".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(APIToken {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    name: row.get("name"),
                    token: row.get("token"),
                    created: row.get("created"),
                    expired: row.get("expired"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `api_token` WHERE id = ?";
                let row = sqlx::query(query).bind(token_id).fetch_one(self.get_mysql_pool()?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Токен не найден".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(APIToken {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    name: row.get("name"),
                    token: row.get("token"),
                    created: row.get("created"),
                    expired: row.get("expired"),
                })
            }
        }
    }

    async fn expire_api_token(&self, _user_id: i32, token_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE api_token SET expired = 1 WHERE id = ?";
                sqlx::query(query).bind(token_id).execute(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE api_token SET expired = TRUE WHERE id = $1";
                sqlx::query(query).bind(token_id).execute(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `api_token` SET expired = 1 WHERE id = ?";
                sqlx::query(query).bind(token_id).execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_api_token(&self, _user_id: i32, token_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM api_token WHERE id = ?";
                sqlx::query(query).bind(token_id).execute(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM api_token WHERE id = $1";
                sqlx::query(query).bind(token_id).execute(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `api_token` WHERE id = ?";
                sqlx::query(query).bind(token_id).execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl EventManager for SqlStore {
    async fn get_events(&self, project_id: Option<i32>, limit: usize) -> Result<Vec<Event>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = if project_id.is_some() {
                    "SELECT * FROM event WHERE project_id = ? ORDER BY created DESC LIMIT ?"
                } else {
                    "SELECT * FROM event ORDER BY created DESC LIMIT ?"
                };
                let mut q = sqlx::query(query);
                if let Some(pid) = project_id {
                    q = q.bind(pid);
                }
                let rows = q.bind(limit as i64).fetch_all(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| Event {
                    id: row.get("id"),
                    project_id: row.try_get("project_id").ok(),
                    user_id: row.try_get("user_id").ok(),
                    object_id: row.try_get("object_id").ok(),
                    object_type: row.get("object_type"),
                    description: row.get("description"),
                    created: row.get("created"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = if project_id.is_some() {
                    "SELECT * FROM event WHERE project_id = $1 ORDER BY created DESC LIMIT $2"
                } else {
                    "SELECT * FROM event ORDER BY created DESC LIMIT $1"
                };
                let mut q = sqlx::query(query);
                if let Some(pid) = project_id {
                    q = q.bind(pid);
                }
                let rows = q.bind(limit as i64).fetch_all(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| Event {
                    id: row.get("id"),
                    project_id: row.try_get("project_id").ok(),
                    user_id: row.try_get("user_id").ok(),
                    object_id: row.try_get("object_id").ok(),
                    object_type: row.get("object_type"),
                    description: row.get("description"),
                    created: row.get("created"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = if project_id.is_some() {
                    "SELECT * FROM `event` WHERE project_id = ? ORDER BY created DESC LIMIT ?"
                } else {
                    "SELECT * FROM `event` ORDER BY created DESC LIMIT ?"
                };
                let mut q = sqlx::query(query);
                if let Some(pid) = project_id {
                    q = q.bind(pid);
                }
                let rows = q.bind(limit as i64).fetch_all(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| Event {
                    id: row.get("id"),
                    project_id: row.try_get("project_id").ok(),
                    user_id: row.try_get("user_id").ok(),
                    object_id: row.try_get("object_id").ok(),
                    object_type: row.get("object_type"),
                    description: row.get("description"),
                    created: row.get("created"),
                }).collect())
            }
        }
    }

    async fn create_event(&self, mut event: Event) -> Result<Event> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO event (project_id, user_id, object_id, object_type, description, created) VALUES (?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(&event.project_id)
                    .bind(&event.user_id)
                    .bind(&event.object_id)
                    .bind(&event.object_type)
                    .bind(&event.description)
                    .bind(event.created)
                    .fetch_one(self.get_sqlite_pool()?).await.map_err(|e| Error::Database(e))?;
                event.id = id;
                Ok(event)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO event (project_id, user_id, object_id, object_type, description, created) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(&event.project_id)
                    .bind(&event.user_id)
                    .bind(&event.object_id)
                    .bind(&event.object_type)
                    .bind(&event.description)
                    .bind(event.created)
                    .fetch_one(self.get_postgres_pool()?).await.map_err(|e| Error::Database(e))?;
                event.id = id;
                Ok(event)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `event` (project_id, user_id, object_id, object_type, description, created) VALUES (?, ?, ?, ?, ?, ?)";
                let result = sqlx::query(query)
                    .bind(&event.project_id)
                    .bind(&event.user_id)
                    .bind(&event.object_id)
                    .bind(&event.object_type)
                    .bind(&event.description)
                    .bind(event.created)
                    .execute(self.get_mysql_pool()?).await.map_err(|e| Error::Database(e))?;
                event.id = result.last_insert_id() as i32;
                Ok(event)
            }
        }
    }
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
    
    async fn set_view_positions(&self, project_id: i32, positions: Vec<(i32, i32)>) -> Result<()> {
        // positions: Vec<(view_id, position)>
        for (view_id, position) in positions {
            match self.get_dialect() {
                SqlDialect::SQLite => {
                    let query = "UPDATE view SET position = ? WHERE id = ? AND project_id = ?";
                    sqlx::query(query)
                        .bind(position)
                        .bind(view_id)
                        .bind(project_id)
                        .execute(self.get_sqlite_pool()?)
                        .await
                        .map_err(|e| Error::Database(e))?;
                }
                SqlDialect::PostgreSQL => {
                    let query = "UPDATE view SET position = $1 WHERE id = $2 AND project_id = $3";
                    sqlx::query(query)
                        .bind(position)
                        .bind(view_id)
                        .bind(project_id)
                        .execute(self.get_postgres_pool()?)
                        .await
                        .map_err(|e| Error::Database(e))?;
                }
                SqlDialect::MySQL => {
                    let query = "UPDATE `view` SET position = ? WHERE id = ? AND project_id = ?";
                    sqlx::query(query)
                        .bind(position)
                        .bind(view_id)
                        .bind(project_id)
                        .execute(self.get_mysql_pool()?)
                        .await
                        .map_err(|e| Error::Database(e))?;
                }
            }
        }
        Ok(())
    }
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
