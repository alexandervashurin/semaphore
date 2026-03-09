//! Audit Log API handlers
//!
//! Обработчики HTTP запросов для управления audit log

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::api::extractors::AdminUser;
use crate::api::middleware::ErrorResponse;
use crate::db::store::{ProjectStore, AuditLogManager};
use crate::models::audit_log::{AuditLogFilter, AuditAction, AuditObjectType, AuditLevel, AuditLog, AuditLogResult};
use crate::error::Error;
use serde::Deserialize;

/// Query параметры для GET /api/audit-log
#[derive(Debug, Deserialize)]
pub struct AuditLogQueryParams {
    pub project_id: Option<i64>,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: Option<String>,
    pub object_type: Option<String>,
    pub object_id: Option<i64>,
    pub level: Option<String>,
    pub search: Option<String>,
    pub date_from: Option<chrono::DateTime<chrono::Utc>>,
    pub date_to: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

/// GET /api/audit-log - Поиск записей audit log с фильтрацией
#[axum::debug_handler]
pub async fn get_audit_logs(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Query(params): Query<AuditLogQueryParams>,
) -> std::result::Result<Json<AuditLogResult>, (StatusCode, Json<ErrorResponse>)> {
    
    // Построение фильтра
    let action = params.action.and_then(|a| match a.as_str() {
        "login" => Some(AuditAction::Login),
        "logout" => Some(AuditAction::Logout),
        "login_failed" => Some(AuditAction::LoginFailed),
        "password_changed" => Some(AuditAction::PasswordChanged),
        "user_created" => Some(AuditAction::UserCreated),
        "user_updated" => Some(AuditAction::UserUpdated),
        "user_deleted" => Some(AuditAction::UserDeleted),
        "project_created" => Some(AuditAction::ProjectCreated),
        "project_updated" => Some(AuditAction::ProjectUpdated),
        "project_deleted" => Some(AuditAction::ProjectDeleted),
        "task_created" => Some(AuditAction::TaskCreated),
        "task_started" => Some(AuditAction::TaskStarted),
        "task_completed" => Some(AuditAction::TaskCompleted),
        "task_failed" => Some(AuditAction::TaskFailed),
        "task_deleted" => Some(AuditAction::TaskDeleted),
        "template_created" => Some(AuditAction::TemplateCreated),
        "template_updated" => Some(AuditAction::TemplateUpdated),
        "template_deleted" => Some(AuditAction::TemplateDeleted),
        "template_run" => Some(AuditAction::TemplateRun),
        "inventory_created" => Some(AuditAction::InventoryCreated),
        "inventory_updated" => Some(AuditAction::InventoryUpdated),
        "inventory_deleted" => Some(AuditAction::InventoryDeleted),
        "repository_created" => Some(AuditAction::RepositoryCreated),
        "repository_updated" => Some(AuditAction::RepositoryUpdated),
        "repository_deleted" => Some(AuditAction::RepositoryDeleted),
        "environment_created" => Some(AuditAction::EnvironmentCreated),
        "environment_updated" => Some(AuditAction::EnvironmentUpdated),
        "environment_deleted" => Some(AuditAction::EnvironmentDeleted),
        "access_key_created" => Some(AuditAction::AccessKeyCreated),
        "access_key_updated" => Some(AuditAction::AccessKeyUpdated),
        "access_key_deleted" => Some(AuditAction::AccessKeyDeleted),
        "integration_created" => Some(AuditAction::IntegrationCreated),
        "integration_updated" => Some(AuditAction::IntegrationUpdated),
        "integration_deleted" => Some(AuditAction::IntegrationDeleted),
        "webhook_triggered" => Some(AuditAction::WebhookTriggered),
        "schedule_created" => Some(AuditAction::ScheduleCreated),
        "schedule_updated" => Some(AuditAction::ScheduleUpdated),
        "schedule_deleted" => Some(AuditAction::ScheduleDeleted),
        "schedule_triggered" => Some(AuditAction::ScheduleTriggered),
        "runner_created" => Some(AuditAction::RunnerCreated),
        "runner_updated" => Some(AuditAction::RunnerUpdated),
        "runner_deleted" => Some(AuditAction::RunnerDeleted),
        "config_changed" => Some(AuditAction::ConfigChanged),
        "backup_created" => Some(AuditAction::BackupCreated),
        "migration_applied" => Some(AuditAction::MigrationApplied),
        _ => Some(AuditAction::Other),
    });

    let object_type = params.object_type.and_then(|o| match o.as_str() {
        "user" => Some(AuditObjectType::User),
        "project" => Some(AuditObjectType::Project),
        "task" => Some(AuditObjectType::Task),
        "template" => Some(AuditObjectType::Template),
        "inventory" => Some(AuditObjectType::Inventory),
        "repository" => Some(AuditObjectType::Repository),
        "environment" => Some(AuditObjectType::Environment),
        "access_key" => Some(AuditObjectType::AccessKey),
        "integration" => Some(AuditObjectType::Integration),
        "schedule" => Some(AuditObjectType::Schedule),
        "runner" => Some(AuditObjectType::Runner),
        "system" => Some(AuditObjectType::System),
        _ => Some(AuditObjectType::Other),
    });

    let level = params.level.and_then(|l| match l.as_str() {
        "info" => Some(AuditLevel::Info),
        "warning" => Some(AuditLevel::Warning),
        "error" => Some(AuditLevel::Error),
        "critical" => Some(AuditLevel::Critical),
        _ => Some(AuditLevel::Info),
    });

    let filter = AuditLogFilter {
        project_id: params.project_id,
        user_id: params.user_id,
        username: params.username,
        action,
        object_type,
        object_id: params.object_id,
        level,
        search: params.search,
        date_from: params.date_from,
        date_to: params.date_to,
        limit: params.limit.unwrap_or(50),
        offset: params.offset.unwrap_or(0),
        sort: params.sort.unwrap_or_else(|| "created".to_string()),
        order: params.order.unwrap_or_else(|| "desc".to_string()),
    };

    let result = state.store.search_audit_logs(&filter).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(result))
}

/// GET /api/audit-log/:id - Получение записи audit log по ID
#[axum::debug_handler]
pub async fn get_audit_log(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Path(id): Path<i64>,
) -> std::result::Result<Json<AuditLog>, (StatusCode, Json<ErrorResponse>)> {
    let record = state.store.get_audit_log(id).await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;
    
    Ok(Json(record))
}

/// GET /api/project/:id/audit-log - Получение audit log проекта
#[axum::debug_handler]
pub async fn get_project_audit_logs(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Path(project_id): Path<i64>,
    Query(params): Query<AuditLogQueryParams>,
) -> std::result::Result<Json<Vec<AuditLog>>, (StatusCode, Json<ErrorResponse>)> {
    // Проверка доступа к проекту
    state.store.get_project(project_id as i32).await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;
    
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let records = state.store.get_audit_logs_by_project(project_id, limit, offset).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;
    
    Ok(Json(records))
}

/// DELETE /api/audit-log/clear - Очистка audit log (только супер-админ)
#[axum::debug_handler]
pub async fn clear_audit_log(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let deleted = state.store.clear_audit_log().await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;
    
    Ok(Json(serde_json::json!({
        "deleted": deleted,
        "message": "Audit log очищен"
    })))
}

/// DELETE /api/audit-log/expiry - Удаление старых записей
#[axum::debug_handler]
pub async fn delete_old_audit_logs(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Query(params): Query<ExpiryParams>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let before = params.before;
    let deleted = state.store.delete_audit_logs_before(before).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;
    
    Ok(Json(serde_json::json!({
        "deleted": deleted,
        "before": before,
        "message": format!("Удалено {} записей до {}", deleted, before)
    })))
}

#[derive(Debug, Deserialize)]
pub struct ExpiryParams {
    pub before: chrono::DateTime<chrono::Utc>,
}
