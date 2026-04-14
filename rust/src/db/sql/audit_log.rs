//! Audit Log - операции с журналом аудита в SQL (PostgreSQL)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::audit_log::{
    AuditAction, AuditDetails, AuditLevel, AuditLog, AuditLogFilter, AuditLogResult,
    AuditObjectType,
};
use chrono::Utc;
use serde_json::Value as JsonValue;
use sqlx::FromRow;

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
            "#,
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
        let row = sqlx::query_as::<_, SqlAuditLog>(r#"SELECT * FROM audit_log WHERE id = $1"#)
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
            where_clauses.push(format!(
                "action = '{}'",
                action.to_string().replace('\'', "''")
            ));
        }
        if let Some(ref object_type) = filter.object_type {
            where_clauses.push(format!(
                "object_type = '{}'",
                object_type.to_string().replace('\'', "''")
            ));
        }
        if let Some(object_id) = filter.object_id {
            where_clauses.push(format!("object_id = {}", object_id));
        }
        if let Some(ref level) = filter.level {
            where_clauses.push(format!(
                "level = '{}'",
                level.to_string().replace('\'', "''")
            ));
        }
        if let Some(ref search) = filter.search {
            where_clauses.push(format!("description LIKE '{}'", search.replace('\'', "''")));
        }
        if let Some(date_from) = filter.date_from {
            where_clauses.push(format!(
                "created >= '{}'",
                date_from.naive_utc().format("%Y-%m-%d %H:%M:%S")
            ));
        }
        if let Some(date_to) = filter.date_to {
            where_clauses.push(format!(
                "created <= '{}'",
                date_to.naive_utc().format("%Y-%m-%d %H:%M:%S")
            ));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let sort = match filter.sort.as_str() {
            "created" | "user_id" | "project_id" | "action" | "object_type" | "level" => {
                filter.sort.clone()
            }
            _ => "created".to_string(),
        };

        let order = if filter.order.to_lowercase() == "asc" {
            "ASC"
        } else {
            "DESC"
        };

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

        let records = rows
            .into_iter()
            .map(|row| self.convert_sql_audit_log(row))
            .collect();

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

        Ok(rows
            .into_iter()
            .map(|row| self.convert_sql_audit_log(row))
            .collect())
    }

    /// Получает записи audit log по user_id с пагинацией
    pub async fn get_audit_logs_by_user(
        &self,
        user_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>> {
        let rows = sqlx::query_as::<_, SqlAuditLog>(
            "SELECT * FROM audit_log WHERE user_id = $1 ORDER BY created DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.audit_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| self.convert_sql_audit_log(row))
            .collect())
    }

    /// Получает записи audit log по action с пагинацией
    pub async fn get_audit_logs_by_action(
        &self,
        action: &AuditAction,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>> {
        let rows = sqlx::query_as::<_, SqlAuditLog>(
            "SELECT * FROM audit_log WHERE action = $1 ORDER BY created DESC LIMIT $2 OFFSET $3",
        )
        .bind(action.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(self.audit_pg_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| self.convert_sql_audit_log(row))
            .collect())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_audit_log_struct_fields() {
        let sql_audit = SqlAuditLog {
            id: 1,
            project_id: Some(10),
            user_id: Some(5),
            username: Some("admin".to_string()),
            action: "login".to_string(),
            object_type: "user".to_string(),
            object_id: Some(100),
            object_name: Some("User A".to_string()),
            description: "User logged in".to_string(),
            level: "info".to_string(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            details: Some(serde_json::json!({"source": "web"})),
            created: Utc::now(),
        };
        assert_eq!(sql_audit.id, 1);
        assert_eq!(sql_audit.action, "login");
        assert_eq!(sql_audit.level, "info");
    }

    #[test]
    fn test_sql_audit_log_clone() {
        let sql_audit = SqlAuditLog {
            id: 42,
            project_id: None,
            user_id: None,
            username: None,
            action: "task_started".to_string(),
            object_type: "task".to_string(),
            object_id: Some(1),
            object_name: None,
            description: "Task started".to_string(),
            level: "info".to_string(),
            ip_address: None,
            user_agent: None,
            details: None,
            created: Utc::now(),
        };
        let cloned = sql_audit.clone();
        assert_eq!(cloned.id, sql_audit.id);
        assert_eq!(cloned.action, sql_audit.action);
    }

    #[test]
    fn test_audit_action_to_string_all_auth() {
        let actions = [
            (AuditAction::Login, "login"),
            (AuditAction::Logout, "logout"),
            (AuditAction::LoginFailed, "login_failed"),
            (AuditAction::PasswordChanged, "password_changed"),
            (AuditAction::TwoFactorEnabled, "two_factor_enabled"),
            (AuditAction::TwoFactorDisabled, "two_factor_disabled"),
        ];
        for (action, expected) in &actions {
            assert_eq!(action.to_string(), *expected);
        }
    }

    #[test]
    fn test_audit_action_to_string_all_user() {
        let actions = [
            AuditAction::UserCreated,
            AuditAction::UserUpdated,
            AuditAction::UserDeleted,
            AuditAction::UserJoinedProject,
            AuditAction::UserLeftProject,
            AuditAction::UserRoleChanged,
        ];
        for action in &actions {
            let s = action.to_string();
            assert!(s.ends_with("d") || s.ends_with("t"));
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn test_audit_action_to_string_all_task() {
        let actions = [
            (AuditAction::TaskCreated, "task_created"),
            (AuditAction::TaskStarted, "task_started"),
            (AuditAction::TaskCompleted, "task_completed"),
            (AuditAction::TaskFailed, "task_failed"),
            (AuditAction::TaskStopped, "task_stopped"),
            (AuditAction::TaskDeleted, "task_deleted"),
        ];
        for (action, expected) in &actions {
            assert_eq!(action.to_string(), *expected);
        }
    }

    #[test]
    fn test_audit_object_type_to_string_all() {
        let types = [
            (AuditObjectType::User, "user"),
            (AuditObjectType::Project, "project"),
            (AuditObjectType::Task, "task"),
            (AuditObjectType::Template, "template"),
            (AuditObjectType::Inventory, "inventory"),
            (AuditObjectType::Repository, "repository"),
            (AuditObjectType::Environment, "environment"),
            (AuditObjectType::AccessKey, "access_key"),
            (AuditObjectType::Integration, "integration"),
            (AuditObjectType::Schedule, "schedule"),
            (AuditObjectType::Runner, "runner"),
            (AuditObjectType::View, "view"),
            (AuditObjectType::Secret, "secret"),
            (AuditObjectType::System, "system"),
            (AuditObjectType::Kubernetes, "kubernetes"),
            (AuditObjectType::Other, "other"),
        ];
        for (obj_type, expected) in &types {
            assert_eq!(obj_type.to_string(), *expected);
        }
    }

    #[test]
    fn test_audit_level_to_string_all() {
        assert_eq!(AuditLevel::Info.to_string(), "info");
        assert_eq!(AuditLevel::Warning.to_string(), "warning");
        assert_eq!(AuditLevel::Error.to_string(), "error");
        assert_eq!(AuditLevel::Critical.to_string(), "critical");
    }

    #[test]
    fn test_audit_level_partial_ord() {
        assert!(AuditLevel::Info < AuditLevel::Warning);
        assert!(AuditLevel::Warning < AuditLevel::Error);
        assert!(AuditLevel::Error < AuditLevel::Critical);
        assert!(AuditLevel::Info < AuditLevel::Critical);
    }

    #[test]
    fn test_audit_action_deserialize_kubernetes() {
        let actions = [
            "kubernetes_resource_created",
            "kubernetes_resource_scaled",
            "kubernetes_helm_release_installed",
            "kubernetes_helm_release_upgraded",
            "kubernetes_helm_release_rolled_back",
            "kubernetes_helm_release_uninstalled",
        ];
        for action_str in &actions {
            let json = format!("\"{}\"", action_str);
            let result: std::result::Result<AuditAction, _> = serde_json::from_str(&json);
            assert!(result.is_ok(), "Failed to deserialize {}", action_str);
        }
    }

    #[test]
    fn test_audit_action_serialize_roundtrip() {
        let actions = [
            AuditAction::Login,
            AuditAction::TaskFailed,
            AuditAction::ProjectCreated,
            AuditAction::KubernetesResourceScaled,
            AuditAction::Other,
        ];
        for action in &actions {
            let json = serde_json::to_string(action).unwrap();
            let deserialized: AuditAction = serde_json::from_str(&json).unwrap();
            assert_eq!(*action, deserialized);
        }
    }

    #[test]
    fn test_audit_log_filter_defaults() {
        let filter = AuditLogFilter::default();
        assert!(filter.project_id.is_none());
        assert!(filter.user_id.is_none());
        assert!(filter.username.is_none());
        assert!(filter.action.is_none());
        assert!(filter.object_type.is_none());
        // Default trait gives empty strings; serde defaults give proper values
        assert_eq!(filter.offset, 0);
        assert!(filter.sort.is_empty());
        assert!(filter.order.is_empty());
    }

    #[test]
    fn test_audit_log_filter_serde_defaults() {
        let json = "{}";
        let filter: AuditLogFilter = serde_json::from_str(json).unwrap();
        assert_eq!(filter.limit, 50);
        assert_eq!(filter.sort, "created");
        assert_eq!(filter.order, "desc");
    }

    #[test]
    fn test_audit_log_filter_with_all_fields() {
        let filter = AuditLogFilter {
            project_id: Some(1),
            user_id: Some(2),
            username: Some("admin".to_string()),
            action: Some(AuditAction::Login),
            object_type: Some(AuditObjectType::User),
            object_id: Some(3),
            level: Some(AuditLevel::Warning),
            search: Some("test".to_string()),
            date_from: Some(Utc::now()),
            date_to: Some(Utc::now()),
            limit: 100,
            offset: 50,
            sort: "action".to_string(),
            order: "asc".to_string(),
        };
        assert_eq!(filter.project_id, Some(1));
        assert_eq!(filter.limit, 100);
        assert_eq!(filter.offset, 50);
        assert_eq!(filter.sort, "action");
        assert_eq!(filter.order, "asc");
    }

    #[test]
    fn test_audit_log_result_serialization() {
        let log = AuditLog {
            id: 1,
            project_id: Some(10),
            user_id: Some(5),
            username: Some("admin".to_string()),
            action: AuditAction::Login,
            object_type: AuditObjectType::User,
            object_id: None,
            object_name: None,
            description: "Login".to_string(),
            level: AuditLevel::Info,
            ip_address: None,
            user_agent: None,
            details: None,
            created: Utc::now(),
        };
        let result = AuditLogResult {
            total: 1,
            records: vec![log],
            limit: 50,
            offset: 0,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"total\":1"));
        assert!(json.contains("\"records\":["));
        assert!(json.contains("\"limit\":50"));
        assert!(json.contains("\"offset\":0"));
    }

    #[test]
    fn test_audit_details_serialize() {
        let details = AuditDetails {
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("TestAgent".to_string()),
            changes: Some(serde_json::json!({"field": "value"})),
            reason: Some("test".to_string()),
            metadata: None,
        };
        let json = serde_json::to_string(&details).unwrap();
        assert!(json.contains("192.168.1.1"));
        assert!(json.contains("TestAgent"));
    }

    #[test]
    fn test_audit_details_default() {
        let details = AuditDetails::default();
        assert!(details.ip_address.is_none());
        assert!(details.user_agent.is_none());
        assert!(details.changes.is_none());
        assert!(details.reason.is_none());
        assert!(details.metadata.is_none());
    }
}
