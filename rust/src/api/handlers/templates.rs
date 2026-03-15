//! Templates Handlers
//!
//! Обработчики запросов для управления шаблонами

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::api::state::AppState;
use crate::models::Template;
use crate::models::template::{TemplateType, TemplateApp};
use crate::error::Error;
use crate::api::middleware::ErrorResponse;
use crate::db::store::{TemplateManager, ProjectStore};

/// Получить список шаблонов проекта
///
/// GET /api/projects/:project_id/templates
pub async fn get_templates(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Template>>, (StatusCode, Json<ErrorResponse>)> {
    let templates = state.store.get_templates(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(templates))
}

/// Создать шаблон
///
/// POST /api/projects/:project_id/templates
pub async fn create_template(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<TemplateCreatePayload>,
) -> Result<(StatusCode, Json<Template>), (StatusCode, Json<ErrorResponse>)> {
    let template = Template {
        id: 0,
        project_id,
        name: payload.name,
        playbook: payload.playbook,
        description: payload.description,
        inventory_id: payload.inventory_id,
        repository_id: payload.repository_id,
        environment_id: payload.environment_id,
        r#type: payload.r#type.as_deref().unwrap_or("ansible").parse().unwrap_or(TemplateType::Default),
        app: payload.app.as_deref().unwrap_or("ansible").parse().unwrap_or(TemplateApp::Ansible),
        git_branch: payload.git_branch.or_else(|| Some("main".to_string())),
        created: Utc::now(),
        arguments: payload.arguments,
        vault_key_id: payload.vault_key_id,
        view_id: payload.view_id,
        build_template_id: payload.build_template_id,
        autorun: payload.autorun,
        allow_override_args_in_task: payload.allow_override_args_in_task,
        allow_override_branch_in_task: payload.allow_override_branch_in_task,
        allow_inventory_in_task: payload.allow_inventory_in_task,
        allow_parallel_tasks: payload.allow_parallel_tasks,
        suppress_success_alerts: payload.suppress_success_alerts,
    };

    let created = state.store.create_template(template)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить шаблон по ID
///
/// GET /api/projects/:project_id/templates/:template_id
pub async fn get_template(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> Result<Json<Template>, (StatusCode, Json<ErrorResponse>)> {
    let template = state.store.get_template(project_id, template_id)
        .await
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

    Ok(Json(template))
}

/// Обновить шаблон
///
/// PUT /api/projects/:project_id/templates/:template_id
pub async fn update_template(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
    Json(payload): Json<TemplateUpdatePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut template = state.store.get_template(project_id, template_id)
        .await
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

    if let Some(name) = payload.name { template.name = name; }
    if let Some(playbook) = payload.playbook { template.playbook = playbook; }
    if let Some(description) = payload.description { template.description = description; }
    if let Some(v) = payload.inventory_id { template.inventory_id = Some(v); }
    if let Some(v) = payload.repository_id { template.repository_id = Some(v); }
    if let Some(v) = payload.environment_id { template.environment_id = Some(v); }
    if let Some(v) = payload.vault_key_id { template.vault_key_id = Some(v); }
    if let Some(v) = payload.view_id { template.view_id = Some(v); }
    if payload.view_id == Some(0) { template.view_id = None; }
    if let Some(v) = payload.build_template_id { template.build_template_id = Some(v); }
    if let Some(v) = payload.git_branch { template.git_branch = Some(v); }
    if let Some(v) = payload.arguments { template.arguments = Some(v); }
    if let Some(v) = payload.r#type { template.r#type = v.parse().unwrap_or(TemplateType::Default); }
    if let Some(v) = payload.app { template.app = v.parse().unwrap_or(TemplateApp::Ansible); }
    if let Some(v) = payload.autorun { template.autorun = v; }
    if let Some(v) = payload.allow_override_args_in_task { template.allow_override_args_in_task = v; }
    if let Some(v) = payload.allow_override_branch_in_task { template.allow_override_branch_in_task = v; }
    if let Some(v) = payload.allow_inventory_in_task { template.allow_inventory_in_task = v; }
    if let Some(v) = payload.allow_parallel_tasks { template.allow_parallel_tasks = v; }
    if let Some(v) = payload.suppress_success_alerts { template.suppress_success_alerts = v; }

    state.store.update_template(template)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удалить шаблон
///
/// DELETE /api/projects/:project_id/templates/:template_id
pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_template(project_id, template_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Types
// ============================================================================

/// Payload для создания/обновления шаблона (единый — используется для PUT тоже)
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateCreatePayload {
    pub name: String,
    pub playbook: String,
    #[serde(default)]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_key_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_template_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    #[serde(default)]
    pub autorun: bool,
    #[serde(default)]
    pub allow_override_args_in_task: bool,
    #[serde(default)]
    pub allow_override_branch_in_task: bool,
    #[serde(default)]
    pub allow_inventory_in_task: bool,
    #[serde(default)]
    pub allow_parallel_tasks: bool,
    #[serde(default)]
    pub suppress_success_alerts: bool,
}

/// Payload для обновления шаблона
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playbook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_key_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_template_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autorun: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_override_args_in_task: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_override_branch_in_task: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_inventory_in_task: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_parallel_tasks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_success_alerts: Option<bool>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_create_payload_deserialize() {
        let json = r#"{
            "name": "Test Template",
            "playbook": "site.yml",
            "inventory_id": 1,
            "repository_id": 2,
            "environment_id": 3
        }"#;
        let payload: TemplateCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Test Template");
        assert_eq!(payload.playbook, "site.yml");
        assert_eq!(payload.inventory_id, Some(1));
        assert_eq!(payload.repository_id, Some(2));
        assert_eq!(payload.environment_id, Some(3));
    }

    #[test]
    fn test_template_update_payload_deserialize_all_fields() {
        let json = r#"{
            "name": "Updated Template",
            "playbook": "updated.yml",
            "description": "New description"
        }"#;
        let payload: TemplateUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Updated Template".to_string()));
        assert_eq!(payload.playbook, Some("updated.yml".to_string()));
        assert_eq!(payload.description, Some("New description".to_string()));
    }

    #[test]
    fn test_template_update_payload_deserialize_partial() {
        let json = r#"{"name": "Updated"}"#;
        let payload: TemplateUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Updated".to_string()));
        assert_eq!(payload.playbook, None);
        assert_eq!(payload.description, None);
    }

    #[test]
    fn test_template_update_payload_deserialize_empty() {
        let json = r#"{}"#;
        let payload: TemplateUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, None);
        assert_eq!(payload.playbook, None);
        assert_eq!(payload.description, None);
    }
}
