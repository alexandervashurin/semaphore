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

/// Получить список инвентарей проекта
///
/// GET /api/projects/:project_id/inventories
pub async fn get_inventories(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Inventory>>, (StatusCode, Json<ErrorResponse>)> {
    let inventories = state.store.get_inventories(project_id)
        .await
        .map_err(|e| (
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
    let inventory = Inventory::new(
        project_id,
        payload.name,
        payload.inventory_type,
    );

    let created = state.store.create_inventory(inventory)
        .await
        .map_err(|e| (
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

    if let Some(name) = payload.name {
        inventory.name = name;
    }
    if let Some(data) = payload.inventory_data {
        inventory.inventory_data = data;
    }

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

/// Payload для создания инвентаря
#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryCreatePayload {
    pub name: String,
    #[serde(rename = "inventory")]
    pub inventory_type: InventoryType,
}

/// Payload для обновления инвентаря
#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "inventory")]
    pub inventory_data: Option<String>,
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
        assert!(payload.inventory_data.is_some());
    }

    #[test]
    fn test_inventory_update_payload_deserialize_partial() {
        let json = r#"{"name": "Updated"}"#;
        let payload: InventoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Updated".to_string()));
        assert_eq!(payload.inventory_data, None);
    }
}
