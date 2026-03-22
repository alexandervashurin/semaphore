//! SQL-хранилище (PostgreSQL)

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

// PostgreSQL-specific module
pub mod postgres;

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
use crate::db::sql::types::SqlDb;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use chrono::Utc;

/// SQL-хранилище данных (PostgreSQL)
pub struct SqlStore {
    db: SqlDb,
}

impl SqlStore {
    /// Создаёт новое SQL-хранилище
    pub async fn new(database_url: &str) -> Result<Self> {
        let db = init::create_database_connection(database_url).await?;

        let store = Self { db };
        store.ensure_schema().await?;
        Ok(store)
    }

    /// Инициализирует схему БД при первом запуске
    async fn ensure_schema(&self) -> Result<()> {
        self.ensure_schema_postgres().await
    }

    /// Инициализирует схему БД для PostgreSQL
    async fn ensure_schema_postgres(&self) -> Result<()> {
        let pool = self.get_postgres_pool()?;

        tracing::info!("Применение схемы БД PostgreSQL (CREATE TABLE IF NOT EXISTS)...");

        // Таблица миграций
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS migration (version INTEGER PRIMARY KEY, name VARCHAR(255))",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // Таблица пользователей
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"user\" (
                id SERIAL PRIMARY KEY,
                username VARCHAR(255) NOT NULL UNIQUE,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                password VARCHAR(255) NOT NULL,
                admin BOOLEAN NOT NULL DEFAULT false,
                external BOOLEAN NOT NULL DEFAULT false,
                alert BOOLEAN NOT NULL DEFAULT false,
                pro BOOLEAN NOT NULL DEFAULT false,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                totp TEXT,
                email_otp TEXT
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // Таблица опций
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS option (key VARCHAR(255) PRIMARY KEY, value TEXT NOT NULL)",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // Таблица проектов
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                alert BOOLEAN NOT NULL DEFAULT false,
                alert_chat VARCHAR(255),
                max_parallel_tasks INTEGER NOT NULL DEFAULT 0,
                type VARCHAR(50) NOT NULL DEFAULT '',
                default_secret_storage_id INTEGER
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // Таблица secret_storage
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS secret_storage (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                type VARCHAR(50) NOT NULL DEFAULT 'local',
                params TEXT NOT NULL DEFAULT '{}',
                read_only BOOLEAN NOT NULL DEFAULT false,
                source_storage_type VARCHAR(50),
                secret TEXT
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // Таблица project_user
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project_user (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                user_id INTEGER NOT NULL REFERENCES \"user\"(id) ON DELETE CASCADE,
                role VARCHAR(50) NOT NULL,
                created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // task_output для логов задач
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task_output (
                id SERIAL PRIMARY KEY,
                task_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                output TEXT NOT NULL,
                time TIMESTAMPTZ NOT NULL,
                stage_id INTEGER
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // project_invite для приглашений в проект
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project_invite (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                user_id INTEGER NOT NULL REFERENCES \"user\"(id) ON DELETE CASCADE,
                role VARCHAR(50) NOT NULL,
                created TIMESTAMPTZ NOT NULL,
                updated TIMESTAMPTZ NOT NULL,
                token TEXT NOT NULL DEFAULT '',
                inviter_user_id INTEGER NOT NULL
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // access_key — ключи доступа
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS access_key (
                id SERIAL PRIMARY KEY,
                project_id INTEGER REFERENCES project(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                type VARCHAR(50) NOT NULL DEFAULT 'none',
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
                created TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // inventory — инвентари Ansible
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS inventory (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                inventory_type VARCHAR(50) NOT NULL DEFAULT 'static',
                inventory_data TEXT NOT NULL DEFAULT '',
                key_id INTEGER,
                secret_storage_id INTEGER,
                ssh_login VARCHAR(255),
                ssh_port INTEGER,
                extra_vars TEXT,
                ssh_key_id INTEGER,
                become_key_id INTEGER,
                vaults TEXT,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // repository — Git-репозитории
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS repository (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                git_url TEXT NOT NULL DEFAULT '',
                git_type VARCHAR(50) NOT NULL DEFAULT 'git',
                git_branch VARCHAR(255),
                key_id INTEGER,
                git_path TEXT,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // environment — переменные окружения
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS environment (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                json TEXT NOT NULL DEFAULT '{}',
                secret_storage_id INTEGER,
                secrets TEXT,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // template — шаблоны задач
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS template (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                playbook TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                inventory_id INTEGER,
                repository_id INTEGER,
                environment_id INTEGER,
                type VARCHAR(50) NOT NULL DEFAULT 'ansible',
                app TEXT NOT NULL DEFAULT '',
                git_branch VARCHAR(255),
                created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                arguments TEXT,
                vault_key_id INTEGER,
                allow_override_args_vars BOOLEAN NOT NULL DEFAULT false,
                allow_override_branch_in_task BOOLEAN NOT NULL DEFAULT false,
                allow_inventory_in_task BOOLEAN NOT NULL DEFAULT false,
                allow_parallel_tasks BOOLEAN NOT NULL DEFAULT false,
                suppress_success_alerts BOOLEAN NOT NULL DEFAULT false,
                start_version TEXT,
                build_template_id INTEGER,
                view_id INTEGER,
                autorun BOOLEAN NOT NULL DEFAULT false,
                survey_vars TEXT,
                task_params TEXT,
                vaults TEXT,
                deleted BOOLEAN NOT NULL DEFAULT false
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // task — история запусков задач
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL,
                template_id INTEGER,
                status VARCHAR(50) NOT NULL DEFAULT 'waiting',
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
                start_time TIMESTAMPTZ,
                end_time TIMESTAMPTZ,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // schedule — расписания (cron)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schedule (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                template_id INTEGER NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                cron TEXT NOT NULL DEFAULT '',
                cron_format TEXT,
                active BOOLEAN NOT NULL DEFAULT true,
                last_commit_hash TEXT,
                repository_id INTEGER,
                created TIMESTAMPTZ,
                run_at TEXT,
                delete_after_run BOOLEAN NOT NULL DEFAULT false
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // view — представления (группировки шаблонов)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS view (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                title TEXT NOT NULL DEFAULT '',
                position INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // project_role — кастомные роли проекта
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project_role (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                slug VARCHAR(255) NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                permissions INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // playbook — хранимые YAML плейбуки
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS playbook (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL DEFAULT '',
                description TEXT,
                playbook_type TEXT NOT NULL DEFAULT 'ansible',
                repository_id INTEGER,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // integration — входящие webhook-триггеры
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS integration (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name TEXT NOT NULL DEFAULT '',
                template_id INTEGER,
                auth_method TEXT NOT NULL DEFAULT 'none',
                auth_header TEXT,
                auth_secret_id INTEGER
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // integration_alias — псевдонимы интеграций
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS integration_alias (
                id SERIAL PRIMARY KEY,
                integration_id INTEGER NOT NULL REFERENCES integration(id) ON DELETE CASCADE,
                project_id INTEGER NOT NULL,
                alias TEXT NOT NULL UNIQUE
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // event — журнал событий
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS event (
                id SERIAL PRIMARY KEY,
                project_id INTEGER REFERENCES project(id) ON DELETE CASCADE,
                user_id INTEGER,
                object_id INTEGER,
                object_type TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                created TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // runner — исполнители задач
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS runner (
                id SERIAL PRIMARY KEY,
                project_id INTEGER REFERENCES project(id) ON DELETE CASCADE,
                token TEXT NOT NULL DEFAULT '',
                name TEXT NOT NULL DEFAULT '',
                active BOOLEAN NOT NULL DEFAULT true,
                last_active TIMESTAMPTZ,
                webhook TEXT,
                max_parallel_tasks INTEGER,
                tag TEXT,
                created TIMESTAMPTZ
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // api_token — токены API для пользователей
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS api_token (
                id SERIAL PRIMARY KEY,
                user_id INTEGER NOT NULL REFERENCES \"user\"(id) ON DELETE CASCADE,
                name TEXT NOT NULL DEFAULT '',
                token TEXT NOT NULL DEFAULT '',
                created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expired BOOLEAN NOT NULL DEFAULT false
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // playbook_run — история запусков плейбуков
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS playbook_run (
                id SERIAL PRIMARY KEY,
                playbook_id INTEGER NOT NULL REFERENCES playbook(id) ON DELETE CASCADE,
                task_id INTEGER,
                status TEXT NOT NULL DEFAULT 'running',
                started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                finished_at TIMESTAMPTZ,
                triggered_by TEXT
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // integration_extract_value — значения из интеграций
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS integration_extract_value (
                id SERIAL PRIMARY KEY,
                integration_id INTEGER NOT NULL REFERENCES integration(id) ON DELETE CASCADE,
                name TEXT NOT NULL DEFAULT '',
                value_source TEXT NOT NULL DEFAULT 'body',
                key TEXT NOT NULL DEFAULT '',
                variable TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // integration_match_matcher — матчеры вебхуков
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS integration_match_matcher (
                id SERIAL PRIMARY KEY,
                integration_id INTEGER NOT NULL REFERENCES integration(id) ON DELETE CASCADE,
                match_type TEXT NOT NULL DEFAULT 'body',
                method TEXT NOT NULL DEFAULT 'equals',
                name TEXT NOT NULL DEFAULT '',
                value TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // workflow — DAG пайплайны из шаблонов
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS workflow (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name TEXT NOT NULL DEFAULT '',
                description TEXT,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // workflow_node — узлы в DAG-графе workflow
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS workflow_node (
                id SERIAL PRIMARY KEY,
                workflow_id INTEGER NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,
                template_id INTEGER NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                pos_x REAL NOT NULL DEFAULT 0,
                pos_y REAL NOT NULL DEFAULT 0
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // workflow_edge — рёбра в DAG-графе workflow
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS workflow_edge (
                id SERIAL PRIMARY KEY,
                workflow_id INTEGER NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,
                from_node_id INTEGER NOT NULL REFERENCES workflow_node(id) ON DELETE CASCADE,
                to_node_id INTEGER NOT NULL REFERENCES workflow_node(id) ON DELETE CASCADE,
                condition TEXT NOT NULL DEFAULT 'success'
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // workflow_run — история запусков workflow
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS workflow_run (
                id SERIAL PRIMARY KEY,
                workflow_id INTEGER NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,
                project_id INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                message TEXT,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                started TIMESTAMPTZ,
                finished TIMESTAMPTZ
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // notification_policy — политики уведомлений
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS notification_policy (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                channel_type TEXT NOT NULL DEFAULT 'slack',
                webhook_url TEXT NOT NULL,
                trigger TEXT NOT NULL DEFAULT 'on_failure',
                template_id INTEGER,
                enabled BOOLEAN NOT NULL DEFAULT TRUE,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // credential_type — пользовательские типы учётных данных
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS credential_type (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                input_schema TEXT NOT NULL DEFAULT '[]',
                injectors TEXT NOT NULL DEFAULT '[]',
                created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // credential_instance — экземпляры учётных данных проекта
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS credential_instance (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                credential_type_id INTEGER NOT NULL REFERENCES credential_type(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                values TEXT NOT NULL DEFAULT '{}',
                description TEXT,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // drift_config — конфигурация мониторинга дрейфа
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS drift_config (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL,
                template_id INTEGER NOT NULL,
                enabled BOOLEAN NOT NULL DEFAULT true,
                schedule TEXT,
                created TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // drift_result — результаты проверок дрейфа
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS drift_result (
                id SERIAL PRIMARY KEY,
                drift_config_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                template_id INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'clean',
                summary TEXT,
                task_id INTEGER,
                checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // task_snapshot — снапшоты успешных запусков (Rollback)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task_snapshot (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL,
                template_id INTEGER NOT NULL,
                task_id INTEGER NOT NULL,
                git_branch TEXT,
                git_commit TEXT,
                arguments TEXT,
                inventory_id INTEGER,
                environment_id INTEGER,
                message TEXT,
                label TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        // ldap_group_mapping — маппинг LDAP-групп на проекты
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ldap_group_mapping (
                id SERIAL PRIMARY KEY,
                ldap_group_dn TEXT NOT NULL,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                role TEXT NOT NULL DEFAULT 'task:runner',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(ldap_group_dn, project_id)
            )",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        tracing::info!("Схема БД инициализирована");
        Ok(())
    }

    /// Получает PostgreSQL pool
    fn get_postgres_pool(&self) -> Result<&PgPool> {
        self.db.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }
}

#[async_trait]
impl SecretStorageManager for SqlStore {
    async fn get_secret_storages(&self, project_id: i32) -> Result<Vec<SecretStorage>> {
        let storages = sqlx::query_as::<_, SecretStorage>(
            "SELECT * FROM secret_storage WHERE project_id = $1 ORDER BY name"
        )
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(storages)
    }

    async fn get_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<SecretStorage> {
        let storage = sqlx::query_as::<_, SecretStorage>(
            "SELECT * FROM secret_storage WHERE id = $1 AND project_id = $2"
        )
        .bind(storage_id)
        .bind(project_id)
        .fetch_optional(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        storage.ok_or(Error::NotFound("SecretStorage not found".to_string()))
    }

    async fn create_secret_storage(&self, mut storage: SecretStorage) -> Result<SecretStorage> {
        let query = "INSERT INTO secret_storage (project_id, name, type, params, read_only) VALUES ($1, $2, $3, $4, $5) RETURNING id";
        let id: i32 = sqlx::query_scalar(query)
            .bind(storage.project_id)
            .bind(&storage.name)
            .bind(storage.r#type.to_string())
            .bind(&storage.params)
            .bind(storage.read_only)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        storage.id = id;
        Ok(storage)
    }

    async fn update_secret_storage(&self, storage: SecretStorage) -> Result<()> {
        sqlx::query(
            "UPDATE secret_storage SET name = $1, type = $2, params = $3, read_only = $4 WHERE id = $5 AND project_id = $6"
        )
        .bind(&storage.name)
        .bind(storage.r#type.to_string())
        .bind(&storage.params)
        .bind(storage.read_only)
        .bind(storage.id)
        .bind(storage.project_id)
        .execute(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    async fn delete_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM secret_storage WHERE id = $1 AND project_id = $2")
            .bind(storage_id)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

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
