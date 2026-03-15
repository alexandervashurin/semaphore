//! SQL-хранилище (SQLite)

pub mod runner;
pub mod types;
pub mod init;
pub mod migrations;
pub mod queries;
pub mod utils;
pub mod audit_log;
pub mod webhook;
pub mod managers;

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
use crate::models::playbook::{Playbook, PlaybookCreate, PlaybookUpdate};
use crate::models::audit_log::{AuditAction, AuditObjectType, AuditLevel, AuditLog, AuditLogFilter, AuditLogResult};
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
        let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;

        // Проверяем, есть ли таблица project (основная для CRUD)
        let project_exists: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='project'",
        )
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;

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
        .map_err(Error::Database)?;

        tracing::info!("Инициализация схемы БД (создание недостающих таблиц)...");

        // Таблица миграций
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS migration (version INTEGER PRIMARY KEY, name TEXT)",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

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
            .map_err(Error::Database)?;

            // Таблица опций
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS option (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            )
            .execute(pool)
            .await
            .map_err(Error::Database)?;

            sqlx::query("INSERT OR IGNORE INTO migration (version, name) VALUES (1, 'initial_schema')")
                .execute(pool)
                .await
                .map_err(Error::Database)?;
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
        .map_err(Error::Database)?;

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
        .map_err(Error::Database)?;

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
        .map_err(Error::Database)?;

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
        .map_err(Error::Database)?;

        // access_key — ключи доступа (SSH, login_password, none, token)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS access_key (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER REFERENCES project(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                type TEXT NOT NULL DEFAULT 'none',
                user_id INTEGER,
                login_password_login TEXT,
                login_password_password TEXT,
                ssh_key TEXT,
                ssh_passphrase TEXT,
                access_key_access_key TEXT,
                access_key_secret_key TEXT,
                secret_storage_id INTEGER,
                owner TEXT,
                environment_id INTEGER,
                created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // inventory — инвентари Ansible
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS inventory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                inventory_type TEXT NOT NULL DEFAULT 'static',
                inventory_data TEXT NOT NULL DEFAULT '',
                key_id INTEGER,
                secret_storage_id INTEGER,
                ssh_login TEXT,
                ssh_port INTEGER,
                extra_vars TEXT,
                ssh_key_id INTEGER,
                become_key_id INTEGER,
                vaults TEXT,
                created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // repository — Git-репозитории
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS repository (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                git_url TEXT NOT NULL DEFAULT '',
                git_type TEXT NOT NULL DEFAULT 'git',
                git_branch TEXT,
                key_id INTEGER,
                git_path TEXT,
                created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // environment — переменные окружения (JSON)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS environment (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                json TEXT NOT NULL DEFAULT '{}',
                secret_storage_id INTEGER,
                secrets TEXT,
                created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // template — шаблоны задач
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS template (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                playbook TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                inventory_id INTEGER,
                repository_id INTEGER,
                environment_id INTEGER,
                type TEXT NOT NULL DEFAULT 'ansible',
                app TEXT NOT NULL DEFAULT '',
                git_branch TEXT,
                created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                arguments TEXT,
                vault_key_id INTEGER,
                allow_override_args_vars INTEGER NOT NULL DEFAULT 0,
                start_version TEXT,
                build_template_id INTEGER,
                view_id INTEGER,
                autorun INTEGER NOT NULL DEFAULT 0,
                survey_vars TEXT,
                deleted INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // task — история запусков задач
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                template_id INTEGER,
                status TEXT NOT NULL DEFAULT 'waiting',
                message TEXT,
                commit_hash TEXT,
                commit_message TEXT,
                version TEXT,
                inventory_id INTEGER,
                repository_id INTEGER,
                environment_id INTEGER,
                environment TEXT,
                secret TEXT,
                user_id INTEGER,
                integration_id INTEGER,
                schedule_id INTEGER,
                build_task_id INTEGER,
                git_branch TEXT,
                arguments TEXT,
                params TEXT,
                playbook TEXT,
                start_time DATETIME,
                end_time DATETIME,
                created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // schedule — расписания (cron)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schedule (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                template_id INTEGER NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                cron TEXT NOT NULL,
                cron_format TEXT,
                active BOOLEAN NOT NULL DEFAULT 1,
                last_commit_hash TEXT,
                repository_id INTEGER,
                created DATETIME
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // view — представления (группировки шаблонов)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS view (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                title TEXT NOT NULL DEFAULT '',
                position INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

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
        .map_err(Error::Database)?;

        if table_exists.is_none() {
            return Ok(());
        }

        let has_created: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('project__user') WHERE name='created'",
        )
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;

        if has_created.unwrap_or(0) == 0 {
            sqlx::query(
                "ALTER TABLE project__user ADD COLUMN created DATETIME NOT NULL DEFAULT '2020-01-01 00:00:00'",
            )
            .execute(pool)
            .await
            .map_err(Error::Database)?;
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
            .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(())
    }

    /// Получает диалект БД
    fn get_dialect(&self) -> SqlDialect {
        self.db.get_dialect()
    }

    /// Получает SQLite pool
    fn get_sqlite_pool(&self) -> Option<&SqlitePool> {
        self.db.get_sqlite_pool()
    }

    /// Получает PostgreSQL pool
    fn get_postgres_pool(&self) -> Option<&PgPool> {
        self.db.get_postgres_pool()
    }

    /// Получает MySQL pool
    fn get_mysql_pool(&self) -> Option<&MySqlPool> {
        self.db.get_mysql_pool()
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
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(storages)
            }
            SqlDialect::PostgreSQL => {
                let storages = sqlx::query_as::<_, SecretStorage>(
                    "SELECT * FROM secret_storage WHERE project_id = $1 ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(storages)
            }
            SqlDialect::MySQL => {
                let storages = sqlx::query_as::<_, SecretStorage>(
                    "SELECT * FROM secret_storage WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

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
                .fetch_optional(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                storage.ok_or(Error::NotFound("SecretStorage not found".to_string()))
            }
            SqlDialect::PostgreSQL => {
                let storage = sqlx::query_as::<_, SecretStorage>(
                    "SELECT * FROM secret_storage WHERE id = $1 AND project_id = $2"
                )
                .bind(storage_id)
                .bind(project_id)
                .fetch_optional(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                storage.ok_or(Error::NotFound("SecretStorage not found".to_string()))
            }
            SqlDialect::MySQL => {
                let storage = sqlx::query_as::<_, SecretStorage>(
                    "SELECT * FROM secret_storage WHERE id = ? AND project_id = ?"
                )
                .bind(storage_id)
                .bind(project_id)
                .fetch_optional(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

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
                .bind(storage.r#type.to_string())
                .bind(&storage.params)
                .bind(storage.read_only)
                .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                storage.id = result.last_insert_rowid() as i32;
                Ok(storage)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO secret_storage (project_id, name, type, params, read_only) VALUES ($1, $2, $3, $4, $5) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(storage.project_id)
                    .bind(&storage.name)
                    .bind(storage.r#type.to_string())
                    .bind(&storage.params)
                    .bind(storage.read_only)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

                storage.id = id;
                Ok(storage)
            }
            SqlDialect::MySQL => {
                let result = sqlx::query(
                    "INSERT INTO secret_storage (project_id, name, type, params, read_only) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(storage.project_id)
                .bind(&storage.name)
                .bind(storage.r#type.to_string())
                .bind(&storage.params)
                .bind(storage.read_only)
                .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

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
                .bind(storage.r#type.to_string())
                .bind(&storage.params)
                .bind(storage.read_only)
                .bind(storage.id)
                .bind(storage.project_id)
                .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query(
                    "UPDATE secret_storage SET name = $1, type = $2, params = $3, read_only = $4 WHERE id = $5 AND project_id = $6"
                )
                .bind(&storage.name)
                .bind(storage.r#type.to_string())
                .bind(&storage.params)
                .bind(storage.read_only)
                .bind(storage.id)
                .bind(storage.project_id)
                .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query(
                    "UPDATE secret_storage SET name = ?, type = ?, params = ?, read_only = ? WHERE id = ? AND project_id = ?"
                )
                .bind(&storage.name)
                .bind(storage.r#type.to_string())
                .bind(&storage.params)
                .bind(storage.read_only)
                .bind(storage.id)
                .bind(storage.project_id)
                .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
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
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM secret_storage WHERE id = $1 AND project_id = $2")
                    .bind(storage_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query("DELETE FROM secret_storage WHERE id = ? AND project_id = ?")
                    .bind(storage_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl AuditLogManager for SqlStore {
    async fn create_audit_log(
        &self,
        project_id: Option<i64>,
        user_id: Option<i64>,
        username: Option<String>,
        action: &AuditAction,
        object_type: &AuditObjectType,
        object_id: Option<i64>,
        object_name: Option<String>,
        description: String,
        level: &AuditLevel,
        ip_address: Option<String>,
        user_agent: Option<String>,
        details: Option<serde_json::Value>,
    ) -> Result<AuditLog> {
        self.db
            .create_audit_log(
                project_id,
                user_id,
                username,
                action,
                object_type,
                object_id,
                object_name,
                description,
                level,
                ip_address,
                user_agent,
                details,
            )
            .await
    }

    async fn get_audit_log(&self, id: i64) -> Result<AuditLog> {
        self.db.get_audit_log(id).await
    }

    async fn search_audit_logs(&self, filter: &AuditLogFilter) -> Result<AuditLogResult> {
        self.db.search_audit_logs(filter).await
    }

    async fn get_audit_logs_by_project(&self, project_id: i64, limit: i64, offset: i64) -> Result<Vec<AuditLog>> {
        self.db.get_audit_logs_by_project(project_id, limit, offset).await
    }

    async fn get_audit_logs_by_user(&self, user_id: i64, limit: i64, offset: i64) -> Result<Vec<AuditLog>> {
        self.db.get_audit_logs_by_user(user_id, limit, offset).await
    }

    async fn get_audit_logs_by_action(&self, action: &AuditAction, limit: i64, offset: i64) -> Result<Vec<AuditLog>> {
        self.db.get_audit_logs_by_action(action, limit, offset).await
    }

    async fn delete_audit_logs_before(&self, before: chrono::DateTime<Utc>) -> Result<u64> {
        self.db.delete_audit_logs_before(before).await
    }

    async fn clear_audit_log(&self) -> Result<u64> {
        self.db.clear_audit_log().await
    }
}
