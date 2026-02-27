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
        description: String::new(),
        inventory_id: payload.inventory_id,
        repository_id: payload.repository_id,
        environment_id: payload.environment_id,
        r#type: TemplateType::Default,
        app: TemplateApp::Ansible,
        git_branch: "main".to_string(),
        deleted: false,
        created: Utc::now(),
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

    if let Some(name) = payload.name {
        template.name = name;
    }
    if let Some(playbook) = payload.playbook {
        template.playbook = playbook;
    }
    if let Some(description) = payload.description {
        template.description = description;
    }

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

/// Payload для создания шаблона
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateCreatePayload {
    pub name: String,
    pub playbook: String,
    pub inventory_id: i32,
    pub repository_id: i32,
    pub environment_id: i32,
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
        assert_eq!(payload.inventory_id, 1);
        assert_eq!(payload.repository_id, 2);
        assert_eq!(payload.environment_id, 3);
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
