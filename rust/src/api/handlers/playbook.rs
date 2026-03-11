//! Handlers для Playbook API

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::api::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaybookSimple {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    pub playbook_type: String,
}

/// GET /api/project/{project_id}/playbooks
pub async fn get_project_playbooks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> impl IntoResponse {
    // Временно возвращаем пустой список
    // Полная реализация требует добавления методов в Store trait
    Json(json!({
        "error": "Playbook API в разработке. Используйте inventory API для управления playbook файлами.",
        "alternative": "/api/project/{id}/inventory"
    }))
}

/// POST /api/project/{project_id}/playbooks
pub async fn create_playbook(
    State(_state): State<Arc<AppState>>,
    Path(_project_id): Path<i32>,
) -> impl IntoResponse {
    Json(json!({"error": "Playbook API в разработке"}))
}

/// PUT /api/project/{project_id}/playbooks/{id}
pub async fn update_playbook(
    State(_state): State<Arc<AppState>>,
    Path((_project_id, _id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    Json(json!({"error": "Playbook API в разработке"}))
}

/// DELETE /api/project/{project_id}/playbooks/{id}
pub async fn delete_playbook(
    State(_state): State<Arc<AppState>>,
    Path((_project_id, _id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    Json(json!({"error": "Playbook API в разработке"}))
}
