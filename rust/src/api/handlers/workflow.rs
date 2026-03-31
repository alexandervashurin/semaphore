//! Handlers для Workflow DAG API

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::WorkflowManager;
use crate::models::workflow::{
    WorkflowCreate, WorkflowEdgeCreate, WorkflowFull, WorkflowNodeCreate, WorkflowNodeUpdate,
    WorkflowUpdate,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

/// GET /api/project/{project_id}/workflows
pub async fn get_workflows(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<crate::models::workflow::Workflow>>, (StatusCode, Json<ErrorResponse>)> {
    let workflows = state.store.get_workflows(project_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    Ok(Json(workflows))
}

/// POST /api/project/{project_id}/workflows
pub async fn create_workflow(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<WorkflowCreate>,
) -> Result<(StatusCode, Json<crate::models::workflow::Workflow>), (StatusCode, Json<ErrorResponse>)>
{
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Workflow name is required".to_string())),
        ));
    }
    let workflow = state
        .store
        .create_workflow(project_id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok((StatusCode::CREATED, Json(workflow)))
}

/// GET /api/project/{project_id}/workflows/{id}
/// Returns WorkflowFull with nodes and edges
pub async fn get_workflow(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<Json<WorkflowFull>, (StatusCode, Json<ErrorResponse>)> {
    let workflow = state
        .store
        .get_workflow(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    let nodes = state.store.get_workflow_nodes(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    let edges = state.store.get_workflow_edges(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    Ok(Json(WorkflowFull {
        workflow,
        nodes,
        edges,
    }))
}

/// PUT /api/project/{project_id}/workflows/{id}
pub async fn update_workflow(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    Json(payload): Json<WorkflowUpdate>,
) -> Result<Json<crate::models::workflow::Workflow>, (StatusCode, Json<ErrorResponse>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Workflow name is required".to_string())),
        ));
    }
    let workflow = state
        .store
        .update_workflow(id, project_id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(Json(workflow))
}

/// DELETE /api/project/{project_id}/workflows/{id}
pub async fn delete_workflow(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_workflow(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/project/{project_id}/workflows/{id}/nodes
pub async fn add_workflow_node(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    Json(payload): Json<WorkflowNodeCreate>,
) -> Result<
    (StatusCode, Json<crate::models::workflow::WorkflowNode>),
    (StatusCode, Json<ErrorResponse>),
> {
    // Verify workflow belongs to project
    state
        .store
        .get_workflow(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    let node = state
        .store
        .create_workflow_node(id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok((StatusCode::CREATED, Json(node)))
}

/// PUT /api/project/{project_id}/workflows/{id}/nodes/{node_id}
pub async fn update_workflow_node(
    State(state): State<Arc<AppState>>,
    Path((project_id, id, node_id)): Path<(i32, i32, i32)>,
    Json(payload): Json<WorkflowNodeUpdate>,
) -> Result<Json<crate::models::workflow::WorkflowNode>, (StatusCode, Json<ErrorResponse>)> {
    // Verify workflow belongs to project
    state
        .store
        .get_workflow(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    let node = state
        .store
        .update_workflow_node(node_id, id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(Json(node))
}

/// DELETE /api/project/{project_id}/workflows/{id}/nodes/{node_id}
pub async fn delete_workflow_node(
    State(state): State<Arc<AppState>>,
    Path((project_id, id, node_id)): Path<(i32, i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Verify workflow belongs to project
    state
        .store
        .get_workflow(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    state
        .store
        .delete_workflow_node(node_id, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/project/{project_id}/workflows/{id}/edges
pub async fn add_workflow_edge(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    Json(payload): Json<WorkflowEdgeCreate>,
) -> Result<
    (StatusCode, Json<crate::models::workflow::WorkflowEdge>),
    (StatusCode, Json<ErrorResponse>),
> {
    // Verify workflow belongs to project
    state
        .store
        .get_workflow(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    let valid_conditions = ["success", "failure", "always"];
    if !valid_conditions.contains(&payload.condition.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!(
                "Invalid condition '{}'. Must be one of: success, failure, always",
                payload.condition
            ))),
        ));
    }
    let edge = state
        .store
        .create_workflow_edge(id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok((StatusCode::CREATED, Json(edge)))
}

/// DELETE /api/project/{project_id}/workflows/{id}/edges/{edge_id}
pub async fn delete_workflow_edge(
    State(state): State<Arc<AppState>>,
    Path((project_id, id, edge_id)): Path<(i32, i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Verify workflow belongs to project
    state
        .store
        .get_workflow(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    state
        .store
        .delete_workflow_edge(edge_id, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/project/{project_id}/workflows/{id}/run
/// Запускает workflow DAG execution
pub async fn run_workflow(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<
    (StatusCode, Json<crate::models::workflow::WorkflowRun>),
    (StatusCode, Json<ErrorResponse>),
> {
    // Verify workflow belongs to project
    state
        .store
        .get_workflow(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    // Запустить workflow executor
    let run = crate::services::workflow_executor::run_workflow(state.clone(), id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok((StatusCode::ACCEPTED, Json(run)))
}

/// GET /api/project/{project_id}/workflows/{id}/runs
pub async fn get_workflow_runs(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<Json<Vec<crate::models::workflow::WorkflowRun>>, (StatusCode, Json<ErrorResponse>)> {
    // Verify workflow belongs to project
    state
        .store
        .get_workflow(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    let runs = state
        .store
        .get_workflow_runs(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(Json(runs))
}
