//! LDAP Group → Teams автосинк — обработчики

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::LdapGroupMappingManager;
use crate::models::ldap_group::LdapGroupMappingCreate;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

/// GET /api/admin/ldap/group-mappings
pub async fn list_ldap_group_mappings(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.get_ldap_group_mappings().await {
        Ok(list) => (StatusCode::OK, Json(json!(list))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/admin/ldap/group-mappings
pub async fn create_ldap_group_mapping(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(body): Json<LdapGroupMappingCreate>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.create_ldap_group_mapping(body).await {
        Ok(m) => (StatusCode::CREATED, Json(json!(m))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// DELETE /api/admin/ldap/group-mappings/:id
pub async fn delete_ldap_group_mapping(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.delete_ldap_group_mapping(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
