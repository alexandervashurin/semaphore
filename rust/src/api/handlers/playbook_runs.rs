//! Handlers для истории запусков Playbook

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::PlaybookRunManager;
use crate::models::playbook_run_history::{PlaybookRun, PlaybookRunFilter, PlaybookRunStats};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;

/// GET /api/project/{project_id}/playbook-runs
/// Получить список запусков playbook
pub async fn get_playbook_runs(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Query(params): Query<PlaybookRunFilterQuery>,
) -> Result<Json<Vec<PlaybookRun>>, (StatusCode, Json<ErrorResponse>)> {
    let filter = PlaybookRunFilter {
        project_id: Some(project_id),
        playbook_id: params.playbook_id,
        status: params.status,
        user_id: params.user_id,
        date_from: None,
        date_to: None,
        limit: params.limit,
        offset: params.offset,
    };

    let runs = state.store.get_playbook_runs(filter).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(runs))
}

/// GET /api/project/{project_id}/playbook-runs/{id}
/// Получить запуск playbook по ID
pub async fn get_playbook_run(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<Json<PlaybookRun>, (StatusCode, Json<ErrorResponse>)> {
    let run = state
        .store
        .get_playbook_run(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(run))
}

/// GET /api/project/{project_id}/playbooks/{playbook_id}/runs/stats
/// Получить статистику запусков playbook
pub async fn get_playbook_run_stats(
    State(state): State<Arc<AppState>>,
    Path((project_id, playbook_id)): Path<(i32, i32)>,
) -> Result<Json<PlaybookRunStats>, (StatusCode, Json<ErrorResponse>)> {
    let stats = state
        .store
        .get_playbook_run_stats(playbook_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(stats))
}

/// DELETE /api/project/{project_id}/playbook-runs/{id}
pub async fn delete_playbook_run(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_playbook_run(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Параметры запроса для фильтрации
#[derive(Debug, Deserialize)]
pub struct PlaybookRunFilterQuery {
    pub playbook_id: Option<i32>,
    pub status: Option<crate::models::playbook_run_history::PlaybookRunStatus>,
    pub user_id: Option<i32>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(test)]
mod tests {
    use crate::models::playbook_run_history::{
        PlaybookRun, PlaybookRunCreate, PlaybookRunFilter, PlaybookRunStats, PlaybookRunStatus,
        PlaybookRunUpdate,
    };
    use chrono::Utc;

    #[test]
    fn test_playbook_run_status_display() {
        assert_eq!(PlaybookRunStatus::Waiting.to_string(), "waiting");
        assert_eq!(PlaybookRunStatus::Running.to_string(), "running");
        assert_eq!(PlaybookRunStatus::Success.to_string(), "success");
        assert_eq!(PlaybookRunStatus::Failed.to_string(), "failed");
        assert_eq!(PlaybookRunStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_playbook_run_status_equality() {
        assert_eq!(PlaybookRunStatus::Success, PlaybookRunStatus::Success);
        assert_ne!(PlaybookRunStatus::Success, PlaybookRunStatus::Failed);
    }

    #[test]
    fn test_playbook_run_full_serialization() {
        let run = PlaybookRun {
            id: 1,
            project_id: 10,
            playbook_id: 5,
            task_id: Some(100),
            template_id: Some(3),
            status: PlaybookRunStatus::Success,
            inventory_id: None,
            environment_id: None,
            extra_vars: None,
            limit_hosts: None,
            tags: Some("deploy".to_string()),
            skip_tags: None,
            start_time: Some(Utc::now()),
            end_time: Some(Utc::now()),
            duration_seconds: Some(120),
            hosts_total: Some(5),
            hosts_changed: Some(2),
            hosts_unreachable: Some(0),
            hosts_failed: Some(0),
            output: Some("PLAY RECAP".to_string()),
            error_message: None,
            user_id: Some(1),
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&run).unwrap();
        assert!(json.contains("\"duration_seconds\":120"));
        assert!(json.contains("\"hosts_total\":5"));
        assert!(json.contains("\"tags\":\"deploy\""));
    }

    #[test]
    fn test_playbook_run_skip_nulls() {
        let run = PlaybookRun {
            id: 1,
            project_id: 10,
            playbook_id: 5,
            task_id: None,
            template_id: None,
            status: PlaybookRunStatus::Waiting,
            inventory_id: None,
            environment_id: None,
            extra_vars: None,
            limit_hosts: None,
            tags: None,
            skip_tags: None,
            start_time: None,
            end_time: None,
            duration_seconds: None,
            hosts_total: None,
            hosts_changed: None,
            hosts_unreachable: None,
            hosts_failed: None,
            output: None,
            error_message: None,
            user_id: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&run).unwrap();
        assert!(!json.contains("\"task_id\":"));
        assert!(!json.contains("\"template_id\":"));
        assert!(!json.contains("\"output\":"));
    }

    #[test]
    fn test_playbook_run_create_serialization() {
        let create = PlaybookRunCreate {
            project_id: 10,
            playbook_id: 5,
            task_id: None,
            template_id: None,
            inventory_id: Some(3),
            environment_id: Some(2),
            extra_vars: None,
            limit_hosts: Some("web".to_string()),
            tags: Some("deploy".to_string()),
            skip_tags: None,
            user_id: Some(1),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"project_id\":10"));
        assert!(json.contains("\"limit_hosts\":\"web\""));
        assert!(json.contains("\"tags\":\"deploy\""));
    }

    #[test]
    fn test_playbook_run_create_deserialization() {
        let json = r#"{"project_id":1,"playbook_id":1,"task_id":null,"template_id":null,"inventory_id":null,"environment_id":null,"extra_vars":null,"limit_hosts":null,"tags":null,"skip_tags":null,"user_id":null}"#;
        let create: PlaybookRunCreate = serde_json::from_str(json).unwrap();
        assert_eq!(create.project_id, 1);
        assert_eq!(create.playbook_id, 1);
    }

    #[test]
    fn test_playbook_run_update_serialization() {
        let update = PlaybookRunUpdate {
            status: Some(PlaybookRunStatus::Running),
            start_time: Some(Utc::now()),
            end_time: None,
            duration_seconds: None,
            hosts_total: Some(10),
            hosts_changed: None,
            hosts_unreachable: None,
            hosts_failed: None,
            output: Some("running...".to_string()),
            error_message: None,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"hosts_total\":10"));
        assert!(json.contains("\"output\":\"running...\""));
        assert!(!json.contains("\"hosts_changed\":"));
    }

    #[test]
    fn test_playbook_run_update_default() {
        let update = PlaybookRunUpdate::default();
        assert!(update.status.is_none());
        assert!(update.start_time.is_none());
        assert!(update.output.is_none());
    }

    #[test]
    fn test_playbook_run_stats_serialization() {
        let stats = PlaybookRunStats {
            total_runs: 100,
            success_runs: 85,
            failed_runs: 10,
            avg_duration_seconds: Some(95.5),
            last_run: Some(Utc::now()),
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_runs\":100"));
        assert!(json.contains("\"avg_duration_seconds\":95.5"));
        assert!(json.contains("\"success_runs\":85"));
    }

    #[test]
    fn test_playbook_run_stats_null_avg() {
        let stats = PlaybookRunStats {
            total_runs: 0,
            success_runs: 0,
            failed_runs: 0,
            avg_duration_seconds: None,
            last_run: None,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_runs\":0"));
        assert!(json.contains("\"avg_duration_seconds\":null"));
    }

    #[test]
    fn test_playbook_run_filter_default() {
        let filter = PlaybookRunFilter::default();
        assert!(filter.project_id.is_none());
        assert!(filter.playbook_id.is_none());
        assert!(filter.status.is_none());
        assert!(filter.user_id.is_none());
        assert!(filter.limit.is_none());
        assert!(filter.offset.is_none());
    }

    #[test]
    fn test_playbook_run_filter_with_values() {
        let filter = PlaybookRunFilter {
            project_id: Some(10),
            playbook_id: Some(5),
            status: Some(PlaybookRunStatus::Success),
            user_id: Some(1),
            date_from: None,
            date_to: None,
            limit: Some(50),
            offset: Some(100),
        };
        assert_eq!(filter.project_id, Some(10));
        assert_eq!(filter.status, Some(PlaybookRunStatus::Success));
        assert_eq!(filter.limit, Some(50));
        assert_eq!(filter.offset, Some(100));
    }

    #[test]
    fn test_playbook_run_clone() {
        let run = PlaybookRun {
            id: 42,
            project_id: 10,
            playbook_id: 5,
            task_id: Some(200),
            template_id: None,
            status: PlaybookRunStatus::Running,
            inventory_id: None,
            environment_id: None,
            extra_vars: None,
            limit_hosts: Some("localhost".to_string()),
            tags: None,
            skip_tags: None,
            start_time: Some(Utc::now()),
            end_time: None,
            duration_seconds: None,
            hosts_total: None,
            hosts_changed: None,
            hosts_unreachable: None,
            hosts_failed: None,
            output: None,
            error_message: None,
            user_id: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let cloned = run.clone();
        assert_eq!(cloned.id, run.id);
        assert_eq!(cloned.status, run.status);
        assert_eq!(cloned.limit_hosts, run.limit_hosts);
    }

    #[test]
    fn test_playbook_run_create_clone() {
        let create = PlaybookRunCreate {
            project_id: 1,
            playbook_id: 1,
            task_id: None,
            template_id: None,
            inventory_id: Some(1),
            environment_id: None,
            extra_vars: None,
            limit_hosts: Some("all".to_string()),
            tags: Some("all".to_string()),
            skip_tags: None,
            user_id: Some(1),
        };
        let cloned = create.clone();
        assert_eq!(cloned.limit_hosts, create.limit_hosts);
        assert_eq!(cloned.tags, create.tags);
    }

    #[test]
    fn test_playbook_run_status_serialize_all_variants() {
        let statuses = vec![
            PlaybookRunStatus::Waiting,
            PlaybookRunStatus::Running,
            PlaybookRunStatus::Success,
            PlaybookRunStatus::Failed,
            PlaybookRunStatus::Cancelled,
        ];
        for status in &statuses {
            let result = serde_json::to_string(status);
            assert!(result.is_ok(), "Failed to serialize {:?}", status);
        }
    }
}
