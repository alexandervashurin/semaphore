//! Deployment Environment Handlers (FI-GL-1 — GitLab Environments)
//!
//! Реестр окружений деплоя: production/staging/dev/review
//! История деплоев, статусы, URL живого окружения.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::DeploymentEnvironmentManager;
use crate::models::{DeploymentEnvironmentCreate, DeploymentEnvironmentUpdate};

/// GET /api/project/{project_id}/deploy-environments
pub async fn list_deploy_environments(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let envs = state
        .store
        .get_deployment_environments(project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(envs)))
}

/// GET /api/project/{project_id}/deploy-environments/{id}
pub async fn get_deploy_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let env = state
        .store
        .get_deployment_environment(id, project_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))))?;
    Ok(Json(json!(env)))
}

/// POST /api/project/{project_id}/deploy-environments
pub async fn create_deploy_environment(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    _auth: AuthUser,
    Json(payload): Json<DeploymentEnvironmentCreate>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let env = state
        .store
        .create_deployment_environment(project_id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok((StatusCode::CREATED, Json(json!(env))))
}

/// PUT /api/project/{project_id}/deploy-environments/{id}
pub async fn update_deploy_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    _auth: AuthUser,
    Json(payload): Json<DeploymentEnvironmentUpdate>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let env = state
        .store
        .update_deployment_environment(id, project_id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(env)))
}

/// DELETE /api/project/{project_id}/deploy-environments/{id}
pub async fn delete_deploy_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    state
        .store
        .delete_deployment_environment(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/project/{project_id}/deploy-environments/{id}/history
pub async fn get_deploy_history(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let history = state
        .store
        .get_deployment_history(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(history)))
}

#[cfg(test)]
mod tests {
    use crate::models::deployment_environment::{
        DeployEnvironmentStatus, DeploymentEnvironment, DeploymentEnvironmentCreate,
        DeploymentEnvironmentUpdate, DeploymentRecord, EnvironmentTier,
    };
    use chrono::Utc;

    #[test]
    fn test_deployment_environment_create_serialization() {
        let create = DeploymentEnvironmentCreate {
            name: "production".to_string(),
            url: Some("https://prod.example.com".to_string()),
            tier: "production".to_string(),
            template_id: Some(5),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"production\""));
        assert!(json.contains("\"tier\":\"production\""));
    }

    #[test]
    fn test_deployment_environment_create_minimal() {
        let create = DeploymentEnvironmentCreate {
            name: "dev".to_string(),
            url: None,
            tier: String::new(),
            template_id: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "dev");
        assert!(parsed["url"].is_null());
        assert!(parsed["template_id"].is_null());
    }

    #[test]
    fn test_deployment_environment_create_deserialization() {
        let json = r#"{"name":"staging","url":"https://staging.example.com","tier":"staging","template_id":10}"#;
        let create: DeploymentEnvironmentCreate = serde_json::from_str(json).unwrap();
        assert_eq!(create.name, "staging");
        assert_eq!(create.tier, "staging");
        assert_eq!(create.template_id, Some(10));
    }

    #[test]
    fn test_deployment_environment_update_partial() {
        let update = DeploymentEnvironmentUpdate {
            name: Some("renamed-env".to_string()),
            url: None,
            tier: None,
            status: None,
            template_id: None,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"renamed-env\""));
        assert!(!json.contains("\"url\":"));
        assert!(!json.contains("\"status\":"));
    }

    #[test]
    fn test_deployment_environment_update_full() {
        let update = DeploymentEnvironmentUpdate {
            name: Some("full-update".to_string()),
            url: Some("https://new.example.com".to_string()),
            tier: Some("staging".to_string()),
            status: Some("active".to_string()),
            template_id: Some(3),
        };
        let json = serde_json::to_string(&update).unwrap();
        let parsed: DeploymentEnvironmentUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, Some("full-update".to_string()));
        assert_eq!(parsed.tier, Some("staging".to_string()));
    }

    #[test]
    fn test_deployment_environment_update_deserialization() {
        let json = r#"{"name":"test","url":"https://test.com"}"#;
        let update: DeploymentEnvironmentUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.name, Some("test".to_string()));
        assert_eq!(update.url, Some("https://test.com".to_string()));
    }

    #[test]
    fn test_environment_tier_display() {
        assert_eq!(EnvironmentTier::Production.to_string(), "production");
        assert_eq!(EnvironmentTier::Staging.to_string(), "staging");
        assert_eq!(EnvironmentTier::Development.to_string(), "development");
        assert_eq!(EnvironmentTier::Review.to_string(), "review");
        assert_eq!(EnvironmentTier::Other.to_string(), "other");
    }

    #[test]
    fn test_deploy_environment_status_display() {
        // DeployEnvironmentStatus doesn't implement Display, but serializes via serde
        let active = DeployEnvironmentStatus::Active;
        let json = serde_json::to_string(&active).unwrap();
        // Serde serializes enum variant names (Active, Stopped, Unknown) by default
        assert!(json.contains("Active"));
        let stopped = DeployEnvironmentStatus::Stopped;
        let json = serde_json::to_string(&stopped).unwrap();
        assert!(json.contains("Stopped"));
        let unknown = DeployEnvironmentStatus::Unknown;
        let json = serde_json::to_string(&unknown).unwrap();
        assert!(json.contains("Unknown"));
    }

    #[test]
    fn test_deployment_environment_full_serialization() {
        let env = DeploymentEnvironment {
            id: 1,
            project_id: 10,
            name: "prod".to_string(),
            url: Some("https://prod.example.com".to_string()),
            tier: "production".to_string(),
            status: "active".to_string(),
            template_id: Some(1),
            last_task_id: Some(100),
            last_deploy_version: Some("v2.0.0".to_string()),
            last_deployed_by: Some(5),
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"name\":\"prod\""));
        assert!(json.contains("\"last_deploy_version\":\"v2.0.0\""));
    }

    #[test]
    fn test_deployment_record_serialization() {
        let record = DeploymentRecord {
            id: 1,
            deploy_environment_id: 5,
            task_id: 100,
            project_id: 10,
            version: Some("v1.0.0".to_string()),
            deployed_by: Some(1),
            status: "success".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"task_id\":100"));
        assert!(json.contains("\"version\":\"v1.0.0\""));
    }

    #[test]
    fn test_deployment_record_null_fields() {
        let record = DeploymentRecord {
            id: 2,
            deploy_environment_id: 5,
            task_id: 101,
            project_id: 10,
            version: None,
            deployed_by: None,
            status: "running".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(!json.contains("\"deployed_by\":"));
    }

    #[test]
    fn test_deployment_environment_create_clone() {
        let create = DeploymentEnvironmentCreate {
            name: "clone".to_string(),
            url: Some("https://clone.example.com".to_string()),
            tier: "review".to_string(),
            template_id: Some(7),
        };
        let cloned = create.clone();
        assert_eq!(cloned.name, create.name);
        assert_eq!(cloned.url, create.url);
    }

    #[test]
    fn test_deployment_environment_update_clone() {
        let update = DeploymentEnvironmentUpdate {
            name: Some("clone-update".to_string()),
            url: None,
            tier: None,
            status: Some("stopped".to_string()),
            template_id: None,
        };
        let cloned = update.clone();
        assert_eq!(cloned.name, update.name);
        assert_eq!(cloned.status, update.status);
    }

    #[test]
    fn test_environment_tier_equality() {
        assert_eq!(EnvironmentTier::Production, EnvironmentTier::Production);
        assert_ne!(EnvironmentTier::Production, EnvironmentTier::Staging);
    }

    #[test]
    fn test_deployment_status_equality() {
        assert_eq!(
            DeployEnvironmentStatus::Active,
            DeployEnvironmentStatus::Active
        );
        assert_ne!(
            DeployEnvironmentStatus::Active,
            DeployEnvironmentStatus::Stopped
        );
    }
}
