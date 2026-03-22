//! Handlers для Custom Credential Types API

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::api::middleware::ErrorResponse;
use crate::db::store::CredentialTypeManager;
use crate::models::credential_type::{
    CredentialTypeCreate, CredentialTypeUpdate, CredentialInstanceCreate,
};

/// GET /api/credential-types
pub async fn list_credential_types(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::models::credential_type::CredentialType>>, (StatusCode, Json<ErrorResponse>)> {
    let items = state.store.get_credential_types().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
    })?;
    Ok(Json(items))
}

/// POST /api/credential-types
pub async fn create_credential_type(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CredentialTypeCreate>,
) -> Result<(StatusCode, Json<crate::models::credential_type::CredentialType>), (StatusCode, Json<ErrorResponse>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Credential type name is required".to_string())),
        ));
    }
    let item = state.store.create_credential_type(payload).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
    })?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// GET /api/credential-types/:id
pub async fn get_credential_type(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<crate::models::credential_type::CredentialType>, (StatusCode, Json<ErrorResponse>)> {
    let item = state.store.get_credential_type(id).await.map_err(|e| {
        (StatusCode::NOT_FOUND, Json(ErrorResponse::new(e.to_string())))
    })?;
    Ok(Json(item))
}

/// PUT /api/credential-types/:id
pub async fn update_credential_type(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<CredentialTypeUpdate>,
) -> Result<Json<crate::models::credential_type::CredentialType>, (StatusCode, Json<ErrorResponse>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Credential type name is required".to_string())),
        ));
    }
    let item = state.store.update_credential_type(id, payload).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
    })?;
    Ok(Json(item))
}

/// DELETE /api/credential-types/:id
pub async fn delete_credential_type(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_credential_type(id).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/project/:project_id/credentials
pub async fn list_credential_instances(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<crate::models::credential_type::CredentialInstance>>, (StatusCode, Json<ErrorResponse>)> {
    let items = state.store.get_credential_instances(project_id).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
    })?;
    Ok(Json(items))
}

/// POST /api/project/:project_id/credentials
pub async fn create_credential_instance(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<CredentialInstanceCreate>,
) -> Result<(StatusCode, Json<crate::models::credential_type::CredentialInstance>), (StatusCode, Json<ErrorResponse>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Credential name is required".to_string())),
        ));
    }
    let item = state.store.create_credential_instance(project_id, payload).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
    })?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// DELETE /api/project/:project_id/credentials/:id
pub async fn delete_credential_instance(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_credential_instance(id, project_id).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
    })?;
    Ok(StatusCode::NO_CONTENT)
}
