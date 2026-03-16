//! Inventory Handlers
//!
//! Обработчики запросов для управления инвентарями

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::Inventory;
use crate::models::inventory::InventoryType;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;
use crate::db::store::InventoryManager;
use crate::db::store::RepositoryManager;

/// Получить список инвентарей проекта
///
/// GET /api/projects/:project_id/inventories
pub async fn get_inventories(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Inventory>>, (StatusCode, Json<ErrorResponse>)> {
    let inventories: Result<Vec<Inventory>, Error> = state.store
        .get_inventories(project_id)
        .await;

    let inventories = inventories.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(e.to_string()))
    ))?;

    Ok(Json(inventories))
}

/// Создать инвентарь
///
/// POST /api/projects/:project_id/inventories
pub async fn create_inventory(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<InventoryCreatePayload>,
) -> Result<(StatusCode, Json<Inventory>), (StatusCode, Json<ErrorResponse>)> {
    let mut inventory = Inventory::new(
        project_id,
        payload.name,
        payload.inventory_type,
    );
    inventory.inventory_data = payload.inventory;
    inventory.ssh_key_id = payload.ssh_key_id;
    inventory.become_key_id = payload.become_key_id;
    if !payload.ssh_login.is_empty() {
        inventory.ssh_login = payload.ssh_login;
    }
    if payload.ssh_port > 0 {
        inventory.ssh_port = payload.ssh_port;
    }

    let created: Result<Inventory, Error> = state.store
        .create_inventory(inventory)
        .await;

    let created = created.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(e.to_string()))
    ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить инвентарь по ID
///
/// GET /api/projects/:project_id/inventories/:inventory_id
pub async fn get_inventory(
    State(state): State<Arc<AppState>>,
    Path((project_id, inventory_id)): Path<(i32, i32)>,
) -> Result<Json<Inventory>, (StatusCode, Json<ErrorResponse>)> {
    let inventory = state.store.get_inventory(project_id, inventory_id)
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

    Ok(Json(inventory))
}

/// Обновить инвентарь
///
/// PUT /api/projects/:project_id/inventories/:inventory_id
pub async fn update_inventory(
    State(state): State<Arc<AppState>>,
    Path((project_id, inventory_id)): Path<(i32, i32)>,
    Json(payload): Json<InventoryUpdatePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut inventory = state.store.get_inventory(project_id, inventory_id)
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

    if let Some(name) = payload.name { inventory.name = name; }
    if let Some(t) = payload.inventory_type { inventory.inventory_type = t; }
    if let Some(data) = payload.inventory { inventory.inventory_data = data; }
    // backward compat
    if let Some(data) = payload.inventory_data { inventory.inventory_data = data; }
    if let Some(v) = payload.ssh_key_id { inventory.ssh_key_id = Some(v); }
    if let Some(v) = payload.become_key_id { inventory.become_key_id = Some(v); }
    if let Some(v) = payload.ssh_login { inventory.ssh_login = v; }
    if let Some(v) = payload.ssh_port { inventory.ssh_port = v; }

    state.store.update_inventory(inventory)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удалить инвентарь
///
/// DELETE /api/projects/:project_id/inventories/:inventory_id
pub async fn delete_inventory(
    State(state): State<Arc<AppState>>,
    Path((project_id, inventory_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_inventory(project_id, inventory_id)
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

/// Payload для создания инвентаря (совместим с Go semaphore API)
#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryCreatePayload {
    pub name: String,
    /// Тип инвентаря: "static", "file", "static_yaml", "terraform_inventory"
    #[serde(default)]
    pub inventory_type: InventoryType,
    /// Содержимое инвентаря (INI/YAML/путь к файлу)
    #[serde(default)]
    pub inventory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub become_key_id: Option<i32>,
    #[serde(default)]
    pub ssh_login: String,
    #[serde(default = "default_ssh_port")]
    pub ssh_port: i32,
}

fn default_ssh_port() -> i32 { 22 }

/// Payload для обновления инвентаря
#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_type: Option<InventoryType>,
    /// Содержимое инвентаря
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub become_key_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_port: Option<i32>,
    // backward compat alias
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_data: Option<String>,
}

// ============================================================================
// Playbook Helpers
// ============================================================================

/// Получить список playbook-файлов из репозитория
///
/// GET /api/projects/:project_id/inventories/playbooks
pub async fn get_playbooks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
    // Получаем все репозитории проекта
    let repositories = state.store.get_repositories(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    let mut all_playbooks = Vec::new();

    // Для каждого репозитория получаем список playbook-файлов
    for repo in repositories {
        // Получаем путь к репозиторию
        let repo_path = format!("/tmp/semaphore/repos/{}/{}", project_id, repo.id);
        
        // Проверяем существование директории
        if std::path::Path::new(&repo_path).exists() {
            match crate::db::sql::template_utils::list_playbooks(&repo_path).await {
                Ok(playbooks) => {
                    for playbook in playbooks {
                        all_playbooks.push(format!("{}/{}", repo.name, playbook));
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to list playbooks for repo {}: {}", repo.id, e);
                }
            }
        }
    }

    Ok(Json(all_playbooks))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_create_payload_deserialize() {
        let json = r#"{
            "name": "Production Servers",
            "inventory": "static"
        }"#;
        let payload: InventoryCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Production Servers");
        assert_eq!(payload.inventory_type, InventoryType::Static);
    }

    #[test]
    fn test_inventory_update_payload_deserialize_all_fields() {
        let json = r#"{
            "name": "Updated Inventory",
            "inventory": "[webservers]\nweb1 ansible_host=1.2.3.4"
        }"#;
        let payload: InventoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Updated Inventory".to_string()));
        assert!(payload.inventory.is_some());
    }

    #[test]
    fn test_inventory_update_payload_deserialize_partial() {
        let json = r#"{"name": "Updated"}"#;
        let payload: InventoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Updated".to_string()));
        assert_eq!(payload.inventory_data, None);
    }
}
