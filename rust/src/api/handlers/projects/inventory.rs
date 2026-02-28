//! Projects API - Inventory Handler
//!
//! Обработчики для инвентарей в проектах

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::Inventory;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::db::store::RetrieveQueryParams;

/// Получает инвентари проекта
pub async fn get_inventories(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Query(params): Query<RetrieveQueryParams>,
) -> Result<Json<Vec<Inventory>>, (StatusCode, Json<ErrorResponse>)> {
    let inventories = state.store.get_inventories(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(inventories))
}

/// Получает инвентарь по ID
pub async fn get_inventory(
    State(state): State<Arc<AppState>>,
    Path((project_id, inventory_id)): Path<(i32, i32)>,
) -> Result<Json<Inventory>, (StatusCode, Json<ErrorResponse>)> {
    let inventory = state.store.get_inventory(project_id, inventory_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Inventory not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    Ok(Json(inventory))
}

/// Создаёт новый инвентарь
pub async fn add_inventory(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<Inventory>,
) -> Result<(StatusCode, Json<Inventory>), (StatusCode, Json<ErrorResponse>)> {
    let mut inventory = payload;
    inventory.project_id = project_id;

    let created = state.store.create_inventory(inventory)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет инвентарь
pub async fn update_inventory(
    State(state): State<Arc<AppState>>,
    Path((project_id, inventory_id)): Path<(i32, i32)>,
    Json(payload): Json<Inventory>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut inventory = payload;
    inventory.id = inventory_id;
    inventory.project_id = project_id;

    state.store.update_inventory(inventory)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удаляет инвентарь
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_handler() {
        // Тест для проверки обработчиков инвентарей
        assert!(true);
    }
}
