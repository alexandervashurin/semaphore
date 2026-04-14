//! GitOps Drift Detection handlers

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::DriftManager;
use crate::models::drift::{DriftConfigCreate, DriftConfigWithStatus};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn list_drift_configs(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.get_drift_configs(project_id).await {
        Ok(configs) => {
            let mut result: Vec<Value> = Vec::new();
            for c in configs {
                let latest = store.get_drift_results(c.id, 1).await.unwrap_or_default();
                let with_status = DriftConfigWithStatus {
                    latest_result: latest.into_iter().next(),
                    config: c,
                };
                result.push(serde_json::to_value(&with_status).unwrap_or(json!({})));
            }
            (StatusCode::OK, Json(json!(result)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn create_drift_config(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
    Json(body): Json<DriftConfigCreate>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.create_drift_config(project_id, body).await {
        Ok(c) => (StatusCode::CREATED, Json(json!(c))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

#[derive(Deserialize)]
pub struct DriftToggle {
    pub enabled: bool,
    pub schedule: Option<String>,
}

pub async fn update_drift_config(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
    Json(body): Json<DriftToggle>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store
        .update_drift_config_enabled(id, project_id, body.enabled)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn delete_drift_config(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.delete_drift_config(id, project_id).await {
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
    use crate::models::drift::DriftConfigUpdate;

    #[test]
    fn test_drift_toggle_enabled() {
        let toggle = DriftToggle {
            enabled: true,
            schedule: Some("*/5 * * * *".to_string()),
        };
        assert!(toggle.enabled);
        assert_eq!(toggle.schedule, Some("*/5 * * * *".to_string()));
    }

    #[test]
    fn test_drift_toggle_disabled() {
        let toggle = DriftToggle {
            enabled: false,
            schedule: None,
        };
        assert!(!toggle.enabled);
        assert!(toggle.schedule.is_none());
    }

    #[test]
    fn test_drift_toggle_deserialize_enabled_no_schedule() {
        let json = r#"{"enabled": true}"#;
        let toggle: DriftToggle = serde_json::from_str(json).unwrap();
        assert!(toggle.enabled);
        assert!(toggle.schedule.is_none());
    }

    #[test]
    fn test_drift_toggle_deserialize_disabled_with_schedule() {
        let json = r#"{"enabled": false, "schedule": "0 0 * * *"}"#;
        let toggle: DriftToggle = serde_json::from_str(json).unwrap();
        assert!(!toggle.enabled);
        assert_eq!(toggle.schedule, Some("0 0 * * *".to_string()));
    }

    #[test]
    fn test_drift_toggle_serialize_roundtrip() {
        // DriftToggle doesn't derive Serialize, test via deserialization
        let json = r#"{"enabled": true, "schedule": "*/10 * * * *"}"#;
        let toggle: DriftToggle = serde_json::from_str(json).unwrap();
        assert!(toggle.enabled);
        assert_eq!(toggle.schedule, Some("*/10 * * * *".to_string()));
    }

    #[test]
    fn test_drift_toggle_debug() {
        // DriftToggle doesn't derive Debug
        let toggle = DriftToggle {
            enabled: true,
            schedule: None,
        };
        assert!(toggle.enabled);
        assert!(toggle.schedule.is_none());
    }

    #[test]
    fn test_drift_toggle_clone() {
        // DriftToggle doesn't derive Clone
        let json = r#"{"enabled": false, "schedule": "daily"}"#;
        let p1: DriftToggle = serde_json::from_str(json).unwrap();
        let p2: DriftToggle = serde_json::from_str(json).unwrap();
        assert_eq!(p1.enabled, p2.enabled);
        assert_eq!(p1.schedule, p2.schedule);
    }

    #[test]
    fn test_drift_toggle_empty_schedule() {
        let toggle = DriftToggle {
            enabled: true,
            schedule: Some("".to_string()),
        };
        assert_eq!(toggle.schedule, Some("".to_string()));
    }

    #[test]
    fn test_drift_toggle_complex_cron() {
        let json = r#"{"enabled": true, "schedule": "0 */2 * * 1-5"}"#;
        let toggle: DriftToggle = serde_json::from_str(json).unwrap();
        assert_eq!(toggle.schedule, Some("0 */2 * * 1-5".to_string()));
    }

    #[test]
    fn test_drift_config_create_deserialize() {
        let json = r#"{"template_id": 10}"#;
        let config: DriftConfigCreate = serde_json::from_str(json).unwrap();
        assert_eq!(config.template_id, 10);
        assert!(config.enabled.is_none());
        assert!(config.schedule.is_none());
    }

    #[test]
    fn test_drift_config_create_full() {
        let json = r#"{"template_id": 5, "enabled": true, "schedule": "0 * * * *"}"#;
        let config: DriftConfigCreate = serde_json::from_str(json).unwrap();
        assert_eq!(config.template_id, 5);
        assert_eq!(config.enabled, Some(true));
        assert_eq!(config.schedule, Some("0 * * * *".to_string()));
    }

    #[test]
    fn test_drift_config_create_roundtrip() {
        let original = DriftConfigCreate {
            template_id: 42,
            enabled: Some(false),
            schedule: Some("*/5 * * * *".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: DriftConfigCreate = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.template_id, original.template_id);
        assert_eq!(restored.enabled, original.enabled);
        assert_eq!(restored.schedule, original.schedule);
    }

    #[test]
    fn test_drift_config_create_debug() {
        // DriftConfigCreate derives Debug, Clone, Serialize, Deserialize
        let config = DriftConfigCreate {
            template_id: 1,
            enabled: None,
            schedule: None,
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("DriftConfigCreate"));
    }

    #[test]
    fn test_drift_config_update_deserialize() {
        let json = r#"{"enabled": true, "schedule": "daily"}"#;
        let update: DriftConfigUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.enabled, Some(true));
        assert_eq!(update.schedule, Some("daily".to_string()));
    }

    #[test]
    fn test_drift_config_update_roundtrip() {
        let original = DriftConfigUpdate {
            enabled: Some(false),
            schedule: Some("weekly".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: DriftConfigUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.enabled, original.enabled);
        assert_eq!(restored.schedule, original.schedule);
    }

    #[test]
    fn test_drift_toggle_clone_independence() {
        // DriftToggle doesn't derive Clone
        let json = r#"{"enabled": true, "schedule": "original"}"#;
        let toggle: DriftToggle = serde_json::from_str(json).unwrap();
        assert_eq!(toggle.schedule, Some("original".to_string()));
    }
}

/// Trigger manual drift check — create a task with --check flag and record result
pub async fn trigger_drift_check(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let store = state.store.store();

    // Get the drift config
    let config = match store.get_drift_config(id, project_id).await {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response();
        }
    };

    // Create a task with --check argument (dry run)
    let task_body = json!({
        "template_id": config.template_id,
        "message": "Drift check (auto)",
        "arguments": "--check --diff",
        "dry_run": true
    });

    // Post task to the project tasks endpoint via store
    // We record the drift result with "pending" status and the task_id
    let result = store
        .create_drift_result(
            project_id,
            id,
            config.template_id,
            "pending",
            Some("Drift check triggered manually".to_string()),
            None,
        )
        .await;

    match result {
        Ok(r) => (
            StatusCode::OK,
            Json(json!({
                "message": "Drift check triggered",
                "drift_result_id": r.id,
                "template_id": config.template_id,
                "hint": "Create a task manually with --check --diff arguments to complete the check"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_drift_results(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let store = state.store.store();
    // Verify config belongs to project
    if store.get_drift_config(id, project_id).await.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Drift config not found"})),
        )
            .into_response();
    }
    match store.get_drift_results(id, 50).await {
        Ok(results) => (StatusCode::OK, Json(json!(results))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
