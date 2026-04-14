//! Task Snapshot & Rollback Handlers

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::{SnapshotManager, TaskManager};
use crate::models::snapshot::{RollbackRequest, TaskSnapshotCreate};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct SnapshotQuery {
    pub template_id: Option<i32>,
    pub limit: Option<i64>,
}

/// GET /api/project/{project_id}/snapshots
pub async fn list_snapshots(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
    Query(q): Query<SnapshotQuery>,
) -> impl IntoResponse {
    let store = state.store.store();
    let limit = q.limit.unwrap_or(50).min(200);
    match store.get_snapshots(project_id, q.template_id, limit).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/project/{project_id}/snapshots (manual snapshot creation)
pub async fn create_snapshot(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
    Json(body): Json<TaskSnapshotCreate>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.create_snapshot(project_id, body).await {
        Ok(s) => (StatusCode::CREATED, Json(json!(s))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// DELETE /api/project/{project_id}/snapshots/{id}
pub async fn delete_snapshot(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.delete_snapshot(id, project_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/project/{project_id}/snapshots/{id}/rollback
/// Создаёт новую задачу с параметрами из снапшота
pub async fn rollback_snapshot(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
    Json(body): Json<RollbackRequest>,
) -> impl IntoResponse {
    let store = state.store.store();

    // Get the snapshot
    let snap = match store.get_snapshot(id, project_id).await {
        Ok(s) => s,
        Err(e) => {
            return (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response();
        }
    };

    // Build task params from snapshot
    let rollback_msg = body
        .message
        .unwrap_or_else(|| format!("Rollback to snapshot #{} (task #{})", id, snap.task_id));

    let mut task = crate::models::Task {
        id: 0,
        template_id: snap.template_id,
        project_id,
        status: crate::services::task_logger::TaskStatus::Waiting,
        playbook: None,
        secret: None,
        arguments: snap.arguments.clone(),
        git_branch: snap.git_branch.clone(),
        user_id: Some(auth.user_id),
        integration_id: None,
        schedule_id: None,
        created: chrono::Utc::now(),
        start: None,
        end: None,
        message: Some(rollback_msg),
        commit_hash: None,
        commit_message: None,
        build_task_id: None,
        version: None,
        inventory_id: snap.inventory_id,
        repository_id: None,
        environment_id: snap.environment_id,
        params: None,
        environment: None,
    };

    match store.create_task(task).await {
        Ok(t) => (
            StatusCode::CREATED,
            Json(json!({
                "message": "Rollback task created",
                "task_id": t.id,
                "snapshot_id": id,
                "from_task": snap.task_id
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

/// POST /api/project/{project_id}/tasks/{task_id}/snapshot
/// Создаёт снапшот из конкретной задачи (вызывается после успешного завершения)
pub async fn snapshot_from_task(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, task_id)): Path<(i32, i32)>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let store = state.store.store();

    // Get the task
    let task = match store.get_task(task_id, project_id).await {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response();
        }
    };

    let label = body
        .get("label")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let payload = TaskSnapshotCreate {
        template_id: task.template_id,
        task_id,
        git_branch: task.git_branch.clone(),
        git_commit: task.commit_hash.clone(),
        arguments: task.arguments.clone(),
        inventory_id: task.inventory_id,
        environment_id: task.environment_id,
        message: task.message.clone(),
        label,
    };

    match store.create_snapshot(project_id, payload).await {
        Ok(s) => (StatusCode::CREATED, Json(json!(s))).into_response(),
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

    #[test]
    fn test_snapshot_query_defaults() {
        let query = SnapshotQuery {
            template_id: None,
            limit: None,
        };
        assert!(query.template_id.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_snapshot_query_with_params() {
        let query = SnapshotQuery {
            template_id: Some(5),
            limit: Some(100),
        };
        assert_eq!(query.template_id, Some(5));
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_task_snapshot_create() {
        let payload = TaskSnapshotCreate {
            template_id: 5,
            task_id: 10,
            git_branch: Some("main".to_string()),
            git_commit: Some("abc123".to_string()),
            arguments: Some("--limit=web".to_string()),
            inventory_id: Some(3),
            environment_id: Some(2),
            message: Some("Deploy success".to_string()),
            label: Some("stable".to_string()),
        };
        assert_eq!(payload.template_id, 5);
        assert_eq!(payload.task_id, 10);
        assert_eq!(payload.label, Some("stable".to_string()));
    }

    #[test]
    fn test_snapshot_query_deserialize_with_params() {
        let json = r#"{"template_id": 5, "limit": 100}"#;
        let query: SnapshotQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.template_id, Some(5));
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_snapshot_query_deserialize_empty() {
        let json = r#"{}"#;
        let query: SnapshotQuery = serde_json::from_str(json).unwrap();
        assert!(query.template_id.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_snapshot_query_deserialize_null_fields() {
        let json = r#"{"template_id": null, "limit": null}"#;
        let query: SnapshotQuery = serde_json::from_str(json).unwrap();
        assert!(query.template_id.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_snapshot_query_debug() {
        // SnapshotQuery doesn't derive Debug
        let query = SnapshotQuery {
            template_id: Some(10),
            limit: Some(50),
        };
        assert_eq!(query.template_id, Some(10));
        assert_eq!(query.limit, Some(50));
    }

    #[test]
    fn test_snapshot_query_clone() {
        // SnapshotQuery doesn't derive Clone
        let json = r#"{"template_id": 5, "limit": 100}"#;
        let q1: SnapshotQuery = serde_json::from_str(json).unwrap();
        let q2: SnapshotQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q1.template_id, q2.template_id);
        assert_eq!(q1.limit, q2.limit);
    }

    #[test]
    fn test_snapshot_query_zero_limit() {
        let query = SnapshotQuery {
            template_id: None,
            limit: Some(0),
        };
        assert_eq!(query.limit, Some(0));
    }

    #[test]
    fn test_rollback_request_serialize_roundtrip() {
        let original = RollbackRequest {
            message: Some("Manual rollback".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: RollbackRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.message, original.message);
    }

    #[test]
    fn test_rollback_request_debug() {
        let req = RollbackRequest {
            message: Some("Debug".to_string()),
        };
        let debug_str = format!("{:?}", req);
        assert!(debug_str.contains("RollbackRequest"));
    }

    #[test]
    fn test_rollback_request_clone() {
        let original = RollbackRequest {
            message: Some("Original".to_string()),
        };
        let cloned = original.clone();
        assert_eq!(cloned.message, original.message);
    }

    #[test]
    fn test_rollback_request_unicode() {
        let json = r#"{"message": "Откат к предыдущей версии"}"#;
        let req: RollbackRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.message, Some("Откат к предыдущей версии".to_string()));
    }

    #[test]
    fn test_task_snapshot_create_roundtrip() {
        let original = TaskSnapshotCreate {
            template_id: 1,
            task_id: 100,
            git_branch: Some("develop".to_string()),
            git_commit: Some("def456".to_string()),
            arguments: Some("--check".to_string()),
            inventory_id: Some(2),
            environment_id: Some(3),
            message: Some("Auto snapshot".to_string()),
            label: Some("v2.0".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: TaskSnapshotCreate = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.template_id, original.template_id);
        assert_eq!(restored.git_commit, original.git_commit);
        assert_eq!(restored.label, original.label);
    }

    #[test]
    fn test_task_snapshot_create_debug() {
        let payload = TaskSnapshotCreate {
            template_id: 1,
            task_id: 1,
            git_branch: None,
            git_commit: None,
            arguments: None,
            inventory_id: None,
            environment_id: None,
            message: None,
            label: None,
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("TaskSnapshotCreate"));
    }

    #[test]
    fn test_snapshot_query_clone_independence() {
        // SnapshotQuery doesn't derive Clone
        let json = r#"{"template_id": 5, "limit": 100}"#;
        let query: SnapshotQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.template_id, Some(5));
    }

    #[test]
    fn test_task_snapshot_create_unicode() {
        let payload = TaskSnapshotCreate {
            template_id: 1,
            task_id: 1,
            git_branch: Some("основная".to_string()),
            git_commit: None,
            arguments: None,
            inventory_id: None,
            environment_id: None,
            message: Some("Успешное развёртывание".to_string()),
            label: Some("стабильная".to_string()),
        };
        assert_eq!(payload.git_branch, Some("основная".to_string()));
        assert_eq!(payload.message, Some("Успешное развёртывание".to_string()));
    }

    #[test]
    fn test_rollback_request_clone_independence() {
        let mut original = RollbackRequest {
            message: Some("Original message".to_string()),
        };
        let cloned = original.clone();
        original.message = Some("Modified message".to_string());
        assert_eq!(cloned.message, Some("Original message".to_string()));
    }
}
