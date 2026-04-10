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

#[cfg(test)]
mod tests {
    use super::*;

    // ── LdapGroupMappingCreate payload tests ─────────────────────────────

    #[test]
    fn test_ldap_create_payload_deserialize() {
        let body = json!({
            "ldap_group_dn": "CN=devops,OU=Groups,DC=example,DC=com",
            "project_id": 10,
            "role": "manager"
        });
        let payload: LdapGroupMappingCreate = serde_json::from_value(body).unwrap();
        assert_eq!(
            payload.ldap_group_dn,
            "CN=devops,OU=Groups,DC=example,DC=com"
        );
        assert_eq!(payload.project_id, 10);
        assert_eq!(payload.role, "manager");
    }

    #[test]
    fn test_ldap_create_payload_serialize() {
        let payload = LdapGroupMappingCreate {
            ldap_group_dn: "CN=admin,DC=test,DC=com".to_string(),
            project_id: 1,
            role: "owner".to_string(),
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["ldap_group_dn"], "CN=admin,DC=test,DC=com");
        assert_eq!(json["project_id"], 1);
        assert_eq!(json["role"], "owner");
    }

    #[test]
    fn test_ldap_role_owner() {
        let payload = LdapGroupMappingCreate {
            ldap_group_dn: "CN=owners,DC=x,DC=y".to_string(),
            project_id: 42,
            role: "owner".to_string(),
        };
        assert_eq!(payload.role, "owner");
    }

    #[test]
    fn test_ldap_role_manager() {
        let payload = LdapGroupMappingCreate {
            ldap_group_dn: "CN=managers,DC=x,DC=y".to_string(),
            project_id: 7,
            role: "manager".to_string(),
        };
        assert_eq!(payload.role, "manager");
    }

    #[test]
    fn test_ldap_role_task_runner() {
        let payload = LdapGroupMappingCreate {
            ldap_group_dn: "CN=runners,DC=x,DC=y".to_string(),
            project_id: 3,
            role: "task_runner".to_string(),
        };
        assert_eq!(payload.role, "task_runner");
    }

    #[test]
    fn test_ldap_group_dn_complex_dn() {
        let dn = "CN=velum-admins,OU=IT,OU=Groups,DC=company,DC=local";
        let payload = LdapGroupMappingCreate {
            ldap_group_dn: dn.to_string(),
            project_id: 100,
            role: "owner".to_string(),
        };
        assert!(payload.ldap_group_dn.contains("velum-admins"));
        assert!(payload.ldap_group_dn.contains("OU=IT"));
    }

    #[test]
    fn test_ldap_create_payload_clone() {
        let original = LdapGroupMappingCreate {
            ldap_group_dn: "CN=test,DC=example,DC=com".to_string(),
            project_id: 5,
            role: "manager".to_string(),
        };
        let cloned = original.clone();
        assert_eq!(cloned.ldap_group_dn, original.ldap_group_dn);
        assert_eq!(cloned.project_id, original.project_id);
        assert_eq!(cloned.role, original.role);
    }

    #[test]
    fn test_ldap_create_payload_debug() {
        let payload = LdapGroupMappingCreate {
            ldap_group_dn: "CN=debug,DC=test,DC=com".to_string(),
            project_id: 1,
            role: "owner".to_string(),
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("LdapGroupMappingCreate"));
        assert!(debug_str.contains("debug"));
    }

    // ── Error response shape tests ───────────────────────────────────────

    #[test]
    fn test_error_response_shape() {
        let err_json = json!({"error": "some error message"});
        assert!(err_json.get("error").is_some());
        assert_eq!(err_json["error"], "some error message");
    }

    #[test]
    fn test_success_list_response_shape() {
        let empty_list: Vec<serde_json::Value> = vec![];
        let resp = json!({"mappings": empty_list});
        assert!(resp.get("mappings").is_some());
    }

    #[test]
    fn test_status_code_ok() {
        assert_eq!(StatusCode::OK, 200);
    }

    #[test]
    fn test_status_code_created() {
        assert_eq!(StatusCode::CREATED, 201);
    }

    #[test]
    fn test_status_code_no_content() {
        assert_eq!(StatusCode::NO_CONTENT, 204);
    }

    #[test]
    fn test_status_code_internal_error() {
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, 500);
    }
}
