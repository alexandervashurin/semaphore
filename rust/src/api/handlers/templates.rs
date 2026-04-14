//! Templates Handlers
//!
//! Обработчики запросов для управления шаблонами

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::{ProjectStore, TaskManager, TemplateManager};
use crate::error::Error;
use crate::models::template::{TemplateApp, TemplateType};
use crate::models::Template;
use crate::services::task_logger::TaskStatus;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Получить список шаблонов проекта
///
/// GET /api/projects/:project_id/templates
pub async fn get_templates(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Template>>, (StatusCode, Json<ErrorResponse>)> {
    let templates = state.store.get_templates(project_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

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
        r#type: payload
            .r#type
            .as_deref()
            .unwrap_or("ansible")
            .parse()
            .unwrap_or(TemplateType::Default),
        app: payload
            .app
            .as_deref()
            .unwrap_or("ansible")
            .parse()
            .unwrap_or(TemplateApp::Ansible),
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
        require_approval: payload.require_approval.unwrap_or(false),
        task_params: payload.task_params,
        survey_vars: payload.survey_vars,
        vaults: payload.vaults,
        parent_template_id: payload.parent_template_id,
        execution_image: payload.execution_image,
        pre_template_id: payload.pre_template_id,
        post_template_id: payload.post_template_id,
        fail_template_id: payload.fail_template_id,
        deploy_environment_id: payload.deploy_environment_id,
    };

    let created = state.store.create_template(template).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить шаблон по ID
///
/// GET /api/projects/:project_id/templates/:template_id
pub async fn get_template(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> Result<Json<Template>, (StatusCode, Json<ErrorResponse>)> {
    let template = state
        .store
        .get_template(project_id, template_id)
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
    let mut template = state
        .store
        .get_template(project_id, template_id)
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

    if let Some(name) = payload.name {
        template.name = name;
    }
    if let Some(playbook) = payload.playbook {
        template.playbook = playbook;
    }
    if let Some(description) = payload.description {
        template.description = description;
    }
    if let Some(v) = payload.inventory_id {
        template.inventory_id = Some(v);
    }
    if let Some(v) = payload.repository_id {
        template.repository_id = Some(v);
    }
    if let Some(v) = payload.environment_id {
        template.environment_id = Some(v);
    }
    if let Some(v) = payload.vault_key_id {
        template.vault_key_id = Some(v);
    }
    if let Some(v) = payload.view_id {
        template.view_id = Some(v);
    }
    if payload.view_id == Some(0) {
        template.view_id = None;
    }
    if let Some(v) = payload.build_template_id {
        template.build_template_id = Some(v);
    }
    if let Some(v) = payload.git_branch {
        template.git_branch = Some(v);
    }
    if let Some(v) = payload.arguments {
        template.arguments = Some(v);
    }
    if let Some(v) = payload.r#type {
        template.r#type = v.parse().unwrap_or(TemplateType::Default);
    }
    if let Some(v) = payload.app {
        template.app = v.parse().unwrap_or(TemplateApp::Ansible);
    }
    if let Some(v) = payload.autorun {
        template.autorun = v;
    }
    if let Some(v) = payload.allow_override_args_in_task {
        template.allow_override_args_in_task = v;
    }
    if let Some(v) = payload.allow_override_branch_in_task {
        template.allow_override_branch_in_task = v;
    }
    if let Some(v) = payload.allow_inventory_in_task {
        template.allow_inventory_in_task = v;
    }
    if let Some(v) = payload.allow_parallel_tasks {
        template.allow_parallel_tasks = v;
    }
    if let Some(v) = payload.suppress_success_alerts {
        template.suppress_success_alerts = v;
    }
    if let Some(v) = payload.require_approval {
        template.require_approval = v;
    }
    if let Some(v) = payload.task_params {
        template.task_params = Some(v);
    }
    if let Some(v) = payload.survey_vars {
        template.survey_vars = Some(v);
    }
    if let Some(v) = payload.vaults {
        template.vaults = Some(v);
    }

    state.store.update_template(template).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::OK)
}

/// Удалить шаблон
///
/// DELETE /api/projects/:project_id/templates/:template_id
pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_template(project_id, template_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Остановить все задачи шаблона
///
/// POST /api/projects/:project_id/templates/:template_id/stop_all_tasks
pub async fn stop_all_template_tasks(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Получаем все активные задачи шаблона
    let tasks = state
        .store
        .get_tasks(project_id, Some(template_id))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    // Останавливаем каждую активную задачу
    for task_with_tpl in tasks {
        if task_with_tpl.task.status.is_active() {
            state
                .store
                .update_task_status(project_id, task_with_tpl.task.id, TaskStatus::Stopped)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(e.to_string())),
                    )
                })?;
        }
    }

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub survey_vars: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaults: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_template_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_template_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_template_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_template_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deploy_environment_id: Option<i32>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub survey_vars: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaults: Option<serde_json::Value>,
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

    #[test]
    fn test_template_create_payload_defaults() {
        let json = r#"{
            "name": "Test",
            "playbook": "test.yml"
        }"#;
        let payload: TemplateCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Test");
        assert_eq!(payload.playbook, "test.yml");
        assert_eq!(payload.description, "");
        assert!(!payload.autorun);
        assert!(!payload.allow_override_args_in_task);
        assert!(!payload.allow_override_branch_in_task);
        assert!(!payload.allow_inventory_in_task);
        assert!(!payload.allow_parallel_tasks);
        assert!(!payload.suppress_success_alerts);
        assert!(payload.require_approval.is_none());
    }

    #[test]
    fn test_template_create_payload_all_flags_true() {
        let json = r#"{
            "name": "Test",
            "playbook": "test.yml",
            "autorun": true,
            "allow_override_args_in_task": true,
            "allow_override_branch_in_task": true,
            "allow_inventory_in_task": true,
            "allow_parallel_tasks": true,
            "suppress_success_alerts": true,
            "require_approval": true
        }"#;
        let payload: TemplateCreatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.autorun);
        assert!(payload.allow_override_args_in_task);
        assert!(payload.allow_override_branch_in_task);
        assert!(payload.allow_inventory_in_task);
        assert!(payload.allow_parallel_tasks);
        assert!(payload.suppress_success_alerts);
        assert_eq!(payload.require_approval, Some(true));
    }

    #[test]
    fn test_template_create_payload_with_json_fields() {
        let json = r#"{
            "name": "Test",
            "playbook": "test.yml",
            "task_params": {"key": "value"},
            "survey_vars": [{"name": "var1"}],
            "vaults": [{"vault_id": 1}]
        }"#;
        let payload: TemplateCreatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.task_params.is_some());
        assert!(payload.survey_vars.is_some());
        assert!(payload.vaults.is_some());
        let task_params = payload.task_params.unwrap();
        assert_eq!(task_params["key"], "value");
    }

    #[test]
    fn test_template_create_payload_roundtrip() {
        let original = TemplateCreatePayload {
            name: "Roundtrip".to_string(),
            playbook: "round.yml".to_string(),
            description: "Desc".to_string(),
            inventory_id: Some(1),
            repository_id: Some(2),
            environment_id: Some(3),
            vault_key_id: Some(4),
            view_id: Some(5),
            build_template_id: Some(6),
            git_branch: Some("main".to_string()),
            arguments: Some("--verbose".to_string()),
            r#type: Some("ansible".to_string()),
            app: Some("ansible".to_string()),
            autorun: true,
            allow_override_args_in_task: false,
            allow_override_branch_in_task: true,
            allow_inventory_in_task: false,
            allow_parallel_tasks: true,
            suppress_success_alerts: false,
            require_approval: Some(true),
            task_params: None,
            survey_vars: None,
            vaults: None,
            parent_template_id: Some(10),
            execution_image: Some("custom:latest".to_string()),
            pre_template_id: Some(7),
            post_template_id: Some(8),
            fail_template_id: Some(9),
            deploy_environment_id: Some(11),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: TemplateCreatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.playbook, original.playbook);
        assert_eq!(restored.description, original.description);
        assert_eq!(restored.git_branch, original.git_branch);
        assert_eq!(restored.execution_image, original.execution_image);
        assert_eq!(restored.parent_template_id, original.parent_template_id);
    }

    #[test]
    fn test_template_create_payload_serialize_skip_optionals() {
        let payload = TemplateCreatePayload {
            name: "Minimal".to_string(),
            playbook: "min.yml".to_string(),
            description: "".to_string(),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            vault_key_id: None,
            view_id: None,
            build_template_id: None,
            git_branch: None,
            arguments: None,
            r#type: None,
            app: None,
            autorun: false,
            allow_override_args_in_task: false,
            allow_override_branch_in_task: false,
            allow_inventory_in_task: false,
            allow_parallel_tasks: false,
            suppress_success_alerts: false,
            require_approval: None,
            task_params: None,
            survey_vars: None,
            vaults: None,
            parent_template_id: None,
            execution_image: None,
            pre_template_id: None,
            post_template_id: None,
            fail_template_id: None,
            deploy_environment_id: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(!json.contains("inventory_id"));
        assert!(!json.contains("git_branch"));
        assert!(!json.contains("execution_image"));
        assert!(json.contains("\"name\":\"Minimal\""));
    }

    #[test]
    fn test_template_update_payload_roundtrip() {
        let original = TemplateUpdatePayload {
            name: Some("Updated".to_string()),
            playbook: Some("up.yml".to_string()),
            description: Some("Updated desc".to_string()),
            inventory_id: Some(1),
            repository_id: Some(2),
            environment_id: Some(3),
            vault_key_id: Some(4),
            view_id: Some(5),
            build_template_id: Some(6),
            git_branch: Some("develop".to_string()),
            arguments: Some("--dry-run".to_string()),
            r#type: Some("terraform".to_string()),
            app: Some("terraform".to_string()),
            autorun: Some(true),
            allow_override_args_in_task: Some(false),
            allow_override_branch_in_task: Some(true),
            allow_inventory_in_task: Some(false),
            allow_parallel_tasks: Some(true),
            suppress_success_alerts: Some(false),
            require_approval: Some(true),
            task_params: Some(serde_json::json!({"key": "val"})),
            survey_vars: None,
            vaults: None,
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: TemplateUpdatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.playbook, original.playbook);
        assert_eq!(restored.autorun, original.autorun);
        assert_eq!(restored.require_approval, original.require_approval);
    }

    #[test]
    fn test_template_create_payload_unicode_name() {
        let json = r#"{
            "name": "Шаблон Развёртывания",
            "playbook": "deploy.yml",
            "description": "Описание шаблона"
        }"#;
        let payload: TemplateCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Шаблон Развёртывания");
        assert_eq!(payload.description, "Описание шаблона");
    }

    #[test]
    fn test_template_create_payload_empty_name_and_playbook() {
        let json = r#"{
            "name": "",
            "playbook": ""
        }"#;
        let payload: TemplateCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "");
        assert_eq!(payload.playbook, "");
    }

    #[test]
    fn test_template_update_payload_all_fields_null() {
        let json = r#"{
            "name": null,
            "playbook": null,
            "description": null,
            "inventory_id": null,
            "repository_id": null,
            "environment_id": null,
            "vault_key_id": null,
            "view_id": null,
            "build_template_id": null,
            "git_branch": null,
            "arguments": null,
            "type": null,
            "app": null,
            "autorun": null,
            "allow_override_args_in_task": null,
            "allow_override_branch_in_task": null,
            "allow_inventory_in_task": null,
            "allow_parallel_tasks": null,
            "suppress_success_alerts": null,
            "require_approval": null,
            "task_params": null,
            "survey_vars": null,
            "vaults": null
        }"#;
        let payload: TemplateUpdatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.name.is_none());
        assert!(payload.playbook.is_none());
        assert!(payload.autorun.is_none());
    }

    #[test]
    fn test_template_create_payload_debug_format() {
        let payload = TemplateCreatePayload {
            name: "Debug".to_string(),
            playbook: "debug.yml".to_string(),
            description: "".to_string(),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            vault_key_id: None,
            view_id: None,
            build_template_id: None,
            git_branch: None,
            arguments: None,
            r#type: None,
            app: None,
            autorun: false,
            allow_override_args_in_task: false,
            allow_override_branch_in_task: false,
            allow_inventory_in_task: false,
            allow_parallel_tasks: false,
            suppress_success_alerts: false,
            require_approval: None,
            task_params: None,
            survey_vars: None,
            vaults: None,
            parent_template_id: None,
            execution_image: None,
            pre_template_id: None,
            post_template_id: None,
            fail_template_id: None,
            deploy_environment_id: None,
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("TemplateCreatePayload"));
        assert!(debug_str.contains("Debug"));
    }

    #[test]
    fn test_template_update_payload_debug_format() {
        let payload = TemplateUpdatePayload {
            name: Some("Upd".to_string()),
            playbook: None,
            description: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            vault_key_id: None,
            view_id: None,
            build_template_id: None,
            git_branch: None,
            arguments: None,
            r#type: None,
            app: None,
            autorun: None,
            allow_override_args_in_task: None,
            allow_override_branch_in_task: None,
            allow_inventory_in_task: None,
            allow_parallel_tasks: None,
            suppress_success_alerts: None,
            require_approval: None,
            task_params: None,
            survey_vars: None,
            vaults: None,
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("TemplateUpdatePayload"));
    }

    #[test]
    fn test_template_create_payload_special_chars_in_playbook() {
        let json = r#"{
            "name": "Test",
            "playbook": "roles/deploy/tasks/main.yml",
            "git_branch": "feature/add-support#42"
        }"#;
        let payload: TemplateCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.playbook, "roles/deploy/tasks/main.yml");
        assert_eq!(
            payload.git_branch,
            Some("feature/add-support#42".to_string())
        );
    }

    #[test]
    fn test_template_update_payload_single_field() {
        let json = r#"{"allow_parallel_tasks": true}"#;
        let payload: TemplateUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.allow_parallel_tasks, Some(true));
        assert!(payload.name.is_none());
    }

    #[test]
    fn test_template_create_payload_with_chained_ids() {
        let json = r#"{
            "name": "Chained",
            "playbook": "chain.yml",
            "pre_template_id": 1,
            "post_template_id": 2,
            "fail_template_id": 3,
            "build_template_id": 4,
            "parent_template_id": 5
        }"#;
        let payload: TemplateCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.pre_template_id, Some(1));
        assert_eq!(payload.post_template_id, Some(2));
        assert_eq!(payload.fail_template_id, Some(3));
        assert_eq!(payload.build_template_id, Some(4));
        assert_eq!(payload.parent_template_id, Some(5));
    }

    #[test]
    fn test_template_update_payload_clone_independence() {
        // TemplateUpdatePayload doesn't derive Clone
        let json = r#"{"name": "Original"}"#;
        let p1: TemplateUpdatePayload = serde_json::from_str(json).unwrap();
        let p2: TemplateUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(p1.name, p2.name);
    }
}
