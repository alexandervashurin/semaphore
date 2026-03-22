//! Audit Log - операции с журналом аудита в SQL (PostgreSQL)

use sqlx::FromRow;
use crate::error::{Error, Result};
use crate::models::audit_log::{AuditLog, AuditAction, AuditObjectType, AuditLevel, AuditLogFilter, AuditLogResult};
use crate::db::sql::types::SqlDb;
use chrono::Utc;
use serde_json::Value as JsonValue;

/// SQL представление AuditLog для чтения из БД
#[derive(Debug, Clone, FromRow)]
pub struct SqlAuditLog {
    pub id: i64,
    pub project_id: Option<i64>,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: String,
    pub object_type: String,
    pub object_id: Option<i64>,
    pub object_name: Option<String>,
    pub description: String,
    pub level: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created: chrono::DateTime<Utc>,
}

impl SqlDb {
    fn audit_pg_pool(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Создаёт новую запись audit log
    #[allow(clippy::too_many_arguments)]
    pub async fn create_audit_log(
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
        details: Option<JsonValue>,
    ) -> Result<AuditLog> {
        let row = sqlx::query_as::<_, SqlAuditLog>(
            r#"
            INSERT INTO audit_log
                (project_id, user_id, username, action, object_type, object_id, object_name,
                 description, level, ip_address, user_agent, details, created)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#
        )
        .bind(project_id)
        .bind(user_id)
        .bind(username)
        .bind(action.to_string())
        .bind(object_type.to_string())
        .bind(object_id)
        .bind(object_name)
        .bind(description)
        .bind(level.to_string())
        .bind(ip_address)
        .bind(user_agent)
        .bind(details)
        .bind(Utc::now())
        .fetch_one(self.audit_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(self.convert_sql_audit_log(row))
    }

    /// Получает запись audit log по ID
    pub async fn get_audit_log(&self, id: i64) -> Result<AuditLog> {
        let row = sqlx::query_as::<_, SqlAuditLog>(
            r#"SELECT * FROM audit_log WHERE id = $1"#
        )
        .bind(id)
        .fetch_one(self.audit_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(self.convert_sql_audit_log(row))
    }

    /// Поиск записей audit log с фильтрацией и пагинацией
    pub async fn search_audit_logs(&self, filter: &AuditLogFilter) -> Result<AuditLogResult> {
        let mut where_clauses = Vec::new();

        if let Some(project_id) = filter.project_id {
            where_clauses.push(format!("project_id = {}", project_id));
        }
        if let Some(user_id) = filter.user_id {
            where_clauses.push(format!("user_id = {}", user_id));
        }
        if let Some(ref username) = filter.username {
            where_clauses.push(format!("username LIKE '{}'", username.replace('\'', "''")));
        }
        if let Some(ref action) = filter.action {
            where_clauses.push(format!("action = '{}'", action.to_string().replace('\'', "''")));
        }
        if let Some(ref object_type) = filter.object_type {
            where_clauses.push(format!("object_type = '{}'", object_type.to_string().replace('\'', "''")));
        }
        if let Some(object_id) = filter.object_id {
            where_clauses.push(format!("object_id = {}", object_id));
        }
        if let Some(ref level) = filter.level {
            where_clauses.push(format!("level = '{}'", level.to_string().replace('\'', "''")));
        }
        if let Some(ref search) = filter.search {
            where_clauses.push(format!("description LIKE '{}'", search.replace('\'', "''")));
        }
        if let Some(date_from) = filter.date_from {
            where_clauses.push(format!("created >= '{}'", date_from.naive_utc().format("%Y-%m-%d %H:%M:%S")));
        }
        if let Some(date_to) = filter.date_to {
            where_clauses.push(format!("created <= '{}'", date_to.naive_utc().format("%Y-%m-%d %H:%M:%S")));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let sort = match filter.sort.as_str() {
            "created" | "user_id" | "project_id" | "action" | "object_type" | "level" => filter.sort.clone(),
            _ => "created".to_string(),
        };

        let order = if filter.order.to_lowercase() == "asc" { "ASC" } else { "DESC" };

        let count_query = format!("SELECT COUNT(*) FROM audit_log {}", where_clause);

        let total = sqlx::query_scalar::<_, i64>(&count_query)
            .fetch_one(self.audit_pg_pool()?)
            .await
            .map_err(Error::Database)?;

        let data_query = format!(
            "SELECT * FROM audit_log {} ORDER BY {} {} LIMIT $1 OFFSET $2",
            where_clause, sort, order
        );

        let rows = sqlx::query_as::<_, SqlAuditLog>(&data_query)
            .bind(filter.limit)
            .bind(filter.offset)
            .fetch_all(self.audit_pg_pool()?)
            .await
            .map_err(Error::Database)?;

        let records = rows.into_iter().map(|row| self.convert_sql_audit_log(row)).collect();

        Ok(AuditLogResult {
            total,
            records,
            limit: filter.limit,
            offset: filter.offset,
        })
    }

    /// Получает записи audit log по project_id с пагинацией
    pub async fn get_audit_logs_by_project(
        &self,
        project_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>> {
        let rows = sqlx::query_as::<_, SqlAuditLog>(
            "SELECT * FROM audit_log WHERE project_id = $1 ORDER BY created DESC LIMIT $2 OFFSET $3"
        )
        .bind(project_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.audit_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| self.convert_sql_audit_log(row)).collect())
    }

    /// Получает записи audit log по user_id с пагинацией
    pub async fn get_audit_logs_by_user(
        &self,
        user_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>> {
        let rows = sqlx::query_as::<_, SqlAuditLog>(
            "SELECT * FROM audit_log WHERE user_id = $1 ORDER BY created DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.audit_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| self.convert_sql_audit_log(row)).collect())
    }

    /// Получает записи audit log по action с пагинацией
    pub async fn get_audit_logs_by_action(
        &self,
        action: &AuditAction,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>> {
        let rows = sqlx::query_as::<_, SqlAuditLog>(
            "SELECT * FROM audit_log WHERE action = $1 ORDER BY created DESC LIMIT $2 OFFSET $3"
        )
        .bind(action.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(self.audit_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| self.convert_sql_audit_log(row)).collect())
    }

    /// Удаляет старые записи audit log (до указанной даты)
    pub async fn delete_audit_logs_before(&self, before: chrono::DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query("DELETE FROM audit_log WHERE created < $1")
            .bind(before)
            .execute(self.audit_pg_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(result.rows_affected())
    }

    /// Очищает весь audit log
    pub async fn clear_audit_log(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM audit_log")
            .execute(self.audit_pg_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(result.rows_affected())
    }

    /// Конвертирует SqlAuditLog в AuditLog
    fn convert_sql_audit_log(&self, sql: SqlAuditLog) -> AuditLog {
        let action = match sql.action.as_str() {
            "login" => AuditAction::Login,
            "logout" => AuditAction::Logout,
            "login_failed" => AuditAction::LoginFailed,
            "password_changed" => AuditAction::PasswordChanged,
            "password_reset_requested" => AuditAction::PasswordResetRequested,
            "two_factor_enabled" => AuditAction::TwoFactorEnabled,
            "two_factor_disabled" => AuditAction::TwoFactorDisabled,
            "user_created" => AuditAction::UserCreated,
            "user_updated" => AuditAction::UserUpdated,
            "user_deleted" => AuditAction::UserDeleted,
            "user_joined_project" => AuditAction::UserJoinedProject,
            "user_left_project" => AuditAction::UserLeftProject,
            "user_role_changed" => AuditAction::UserRoleChanged,
            "project_created" => AuditAction::ProjectCreated,
            "project_updated" => AuditAction::ProjectUpdated,
            "project_deleted" => AuditAction::ProjectDeleted,
            "task_created" => AuditAction::TaskCreated,
            "task_started" => AuditAction::TaskStarted,
            "task_completed" => AuditAction::TaskCompleted,
            "task_failed" => AuditAction::TaskFailed,
            "task_stopped" => AuditAction::TaskStopped,
            "task_deleted" => AuditAction::TaskDeleted,
            "template_created" => AuditAction::TemplateCreated,
            "template_updated" => AuditAction::TemplateUpdated,
            "template_deleted" => AuditAction::TemplateDeleted,
            "template_run" => AuditAction::TemplateRun,
            "inventory_created" => AuditAction::InventoryCreated,
            "inventory_updated" => AuditAction::InventoryUpdated,
            "inventory_deleted" => AuditAction::InventoryDeleted,
            "repository_created" => AuditAction::RepositoryCreated,
            "repository_updated" => AuditAction::RepositoryUpdated,
            "repository_deleted" => AuditAction::RepositoryDeleted,
            "environment_created" => AuditAction::EnvironmentCreated,
            "environment_updated" => AuditAction::EnvironmentUpdated,
            "environment_deleted" => AuditAction::EnvironmentDeleted,
            "access_key_created" => AuditAction::AccessKeyCreated,
            "access_key_updated" => AuditAction::AccessKeyUpdated,
            "access_key_deleted" => AuditAction::AccessKeyDeleted,
            "integration_created" => AuditAction::IntegrationCreated,
            "integration_updated" => AuditAction::IntegrationUpdated,
            "integration_deleted" => AuditAction::IntegrationDeleted,
            "webhook_triggered" => AuditAction::WebhookTriggered,
            "schedule_created" => AuditAction::ScheduleCreated,
            "schedule_updated" => AuditAction::ScheduleUpdated,
            "schedule_deleted" => AuditAction::ScheduleDeleted,
            "schedule_triggered" => AuditAction::ScheduleTriggered,
            "runner_created" => AuditAction::RunnerCreated,
            "runner_updated" => AuditAction::RunnerUpdated,
            "runner_deleted" => AuditAction::RunnerDeleted,
            "runner_connected" => AuditAction::RunnerConnected,
            "runner_disconnected" => AuditAction::RunnerDisconnected,
            "config_changed" => AuditAction::ConfigChanged,
            "backup_created" => AuditAction::BackupCreated,
            "restore_performed" => AuditAction::RestorePerformed,
            "migration_applied" => AuditAction::MigrationApplied,
            _ => AuditAction::Other,
        };

        let object_type = match sql.object_type.as_str() {
            "user" => AuditObjectType::User,
            "project" => AuditObjectType::Project,
            "task" => AuditObjectType::Task,
            "template" => AuditObjectType::Template,
            "inventory" => AuditObjectType::Inventory,
            "repository" => AuditObjectType::Repository,
            "environment" => AuditObjectType::Environment,
            "access_key" => AuditObjectType::AccessKey,
            "integration" => AuditObjectType::Integration,
            "schedule" => AuditObjectType::Schedule,
            "runner" => AuditObjectType::Runner,
            "view" => AuditObjectType::View,
            "secret" => AuditObjectType::Secret,
            "system" => AuditObjectType::System,
            _ => AuditObjectType::Other,
        };

        let level = match sql.level.as_str() {
            "info" => AuditLevel::Info,
            "warning" => AuditLevel::Warning,
            "error" => AuditLevel::Error,
            "critical" => AuditLevel::Critical,
            _ => AuditLevel::Info,
        };

        AuditLog {
            id: sql.id,
            project_id: sql.project_id,
            user_id: sql.user_id,
            username: sql.username,
            action,
            object_type,
            object_id: sql.object_id,
            object_name: sql.object_name,
            description: sql.description,
            level,
            ip_address: sql.ip_address,
            user_agent: sql.user_agent,
            details: sql.details,
            created: sql.created,
        }
    }
}
