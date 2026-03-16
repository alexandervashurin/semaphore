//! Tasks Handlers
//!
//! Обработчики запросов для управления задачами

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::api::state::AppState;
use crate::models::{Task, TaskWithTpl, TaskOutput, Inventory, Repository, Environment};
use crate::services::task_logger::{TaskStatus, BasicLogger, TaskLogger, LogListener};
use crate::services::local_job::LocalJob;
use crate::db_lib::AccessKeyInstallerImpl;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;
use crate::db::store::{TaskManager, TemplateManager, InventoryManager, RepositoryManager, EnvironmentManager, ProjectStore};

/// Получить все активные задачи всех проектов
///
/// GET /api/tasks
pub async fn get_all_tasks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TaskWithTpl>>, (StatusCode, Json<ErrorResponse>)> {
    // Получаем все проекты
    let projects = state.store.get_projects(None)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    // Собираем активные задачи из всех проектов
    let mut all_tasks = Vec::new();
    for project in projects {
        let tasks = state.store.get_tasks(project.id, None::<i32>)
            .await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            ))?;

        // Фильтруем только активные задачи
        for task_with_tpl in tasks {
            if task_with_tpl.task.status.is_active() {
                all_tasks.push(task_with_tpl);
            }
        }
    }

    // Сортируем по дате создания (новые первые)
    all_tasks.sort_by(|a, b| b.task.created.cmp(&a.task.created));

    Ok(Json(all_tasks))
}

/// Получить список задач проекта
///
/// GET /api/projects/:project_id/tasks
pub async fn get_tasks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<TaskWithTpl>>, (StatusCode, Json<ErrorResponse>)> {
    let tasks: Result<Vec<TaskWithTpl>, Error> = state.store
        .get_tasks(project_id, None::<i32>)
        .await;

    let tasks = tasks.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(e.to_string()))
    ))?;

    Ok(Json(tasks))
}

/// Создать задачу
///
/// POST /api/projects/:project_id/tasks
pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<TaskCreatePayload>,
) -> Result<(StatusCode, Json<Task>), (StatusCode, Json<ErrorResponse>)> {
    let task = Task {
        id: 0,
        template_id: payload.template_id,
        project_id,
        status: TaskStatus::Waiting,
        playbook: payload.playbook,
        environment: payload.environment,
        secret: None,
        arguments: payload.arguments,
        git_branch: payload.git_branch,
        user_id: payload.user_id,
        integration_id: None,
        schedule_id: None,
        created: Utc::now(),
        start: None,
        end: None,
        message: None,
        commit_hash: None,
        commit_message: None,
        build_task_id: payload.build_task_id,
        version: None,
        inventory_id: payload.inventory_id,
        repository_id: payload.repository_id,
        environment_id: payload.environment_id,
        params: None,
    };

    let created: Result<Task, Error> = state.store
        .create_task(task)
        .await;

    let created = created.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(e.to_string()))
    ))?;

    // Запускаем выполнение задачи в фоне
    let task_state = state.clone();
    let task_to_run = created.clone();
    tokio::spawn(async move {
        execute_task_background(task_state, task_to_run).await;
    });

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить задачу по ID
///
/// GET /api/projects/:project_id/tasks/:task_id
pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> Result<Json<Task>, (StatusCode, Json<ErrorResponse>)> {
    let task: Result<Task, Error> = state.store
        .get_task(project_id, task_id)
        .await;

    let task = task.map_err(|e| match e {
        Error::NotFound(_) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(e.to_string())),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        ),
    })?;

    Ok(Json(task))
}

/// Получить последние задачи проекта
///
/// GET /api/project/:project_id/tasks/last
pub async fn get_last_tasks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<TaskWithTpl>>, (StatusCode, Json<ErrorResponse>)> {
    let tasks: Result<Vec<TaskWithTpl>, Error> = state.store
        .get_tasks(project_id, None::<i32>)
        .await;

    let tasks = tasks.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(e.to_string()))
    ))?;

    let limited: Vec<TaskWithTpl> = tasks.into_iter().take(20).collect();
    Ok(Json(limited))
}

/// Удалить задачу
///
/// DELETE /api/projects/:project_id/tasks/:task_id
pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result: Result<(), Error> = state.store
        .delete_task(project_id, task_id)
        .await;

    result.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(e.to_string()))
    ))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Выполняет задачу в фоновом потоке
async fn execute_task_background(state: Arc<AppState>, task: Task) {
    println!("[task_runner] Starting task {} (template {})", task.id, task.template_id);
    let store = &state.store;

    match store.update_task_status(task.project_id, task.id, TaskStatus::Running).await {
        Ok(()) => println!("[task_runner] task {} status → Running", task.id),
        Err(e) => println!("[task_runner] task {} failed to set Running: {e}", task.id),
    }

    let template = match store.get_template(task.project_id, task.template_id).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[task_runner] task {}: failed to get template: {e}", task.id);
            let _ = store.update_task_status(task.project_id, task.id, TaskStatus::Error).await;
            return;
        }
    };

    let inventory_id = task.inventory_id.or(template.inventory_id);
    let inventory = match inventory_id {
        Some(id) => store.get_inventory(task.project_id, id).await.unwrap_or_default(),
        None => Inventory::default(),
    };

    let repository_id = task.repository_id.or(template.repository_id);
    let repository = match repository_id {
        Some(id) => store.get_repository(task.project_id, id).await.unwrap_or_default(),
        None => Repository::default(),
    };

    let environment_id = task.environment_id.or(template.environment_id);
    let environment = match environment_id {
        Some(id) => store.get_environment(task.project_id, id).await.unwrap_or_default(),
        None => Environment::default(),
    };

    let log_buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let buf_clone = log_buffer.clone();
    let logger = Arc::new(BasicLogger::new());
    logger.add_log_listener(Box::new(move |_time, msg| {
        let _ = buf_clone.lock().map(|mut v| v.push(msg));
    }));

    let work_dir = std::env::temp_dir().join(format!("semaphore_task_{}_{}", task.project_id, task.id));
    let tmp_dir = work_dir.join("tmp");

    if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
        eprintln!("[task_runner] task {}: failed to create workdir: {e}", task.id);
        let _ = store.update_task_status(task.project_id, task.id, TaskStatus::Error).await;
        return;
    }

    let key_installer = AccessKeyInstallerImpl::new();
    let mut job = LocalJob::new(
        task.clone(),
        template,
        inventory,
        repository,
        environment,
        logger,
        key_installer,
        work_dir,
        tmp_dir,
    );

    job.store = Some(Arc::new(state.store.clone()) as Arc<dyn crate::db::store::Store + Send + Sync>);
    let result = job.run("runner", None, "default").await;
    job.cleanup();

    let log_lines: Vec<String> = log_buffer.lock().map(|v| v.clone()).unwrap_or_default();
    for line in log_lines {
        let output = TaskOutput {
            id: 0,
            task_id: task.id,
            project_id: task.project_id,
            time: Utc::now(),
            output: line,
            stage_id: None,
        };
        let _ = store.create_task_output(output).await;
    }

    match result {
        Ok(()) => {
            let _ = store.update_task_status(task.project_id, task.id, TaskStatus::Success).await;
            println!("[task_runner] task {} completed successfully", task.id);
        }
        Err(e) => {
            eprintln!("[task_runner] task {} failed: {e}", task.id);
            let _ = store.update_task_status(task.project_id, task.id, TaskStatus::Error).await;
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Payload для создания задачи
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskCreatePayload {
    pub template_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playbook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_task_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<i32>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_create_payload_deserialize_required_only() {
        let json = r#"{"template_id": 1}"#;
        let payload: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.template_id, 1);
        assert_eq!(payload.playbook, None);
        assert_eq!(payload.environment, None);
    }

    #[test]
    fn test_task_create_payload_deserialize_all_fields() {
        let json = r#"{
            "template_id": 1,
            "playbook": "site.yml",
            "environment": "prod",
            "arguments": "--verbose",
            "git_branch": "main",
            "user_id": 5,
            "build_task_id": 10,
            "inventory_id": 3
        }"#;
        let payload: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.template_id, 1);
        assert_eq!(payload.playbook, Some("site.yml".to_string()));
        assert_eq!(payload.environment, Some("prod".to_string()));
        assert_eq!(payload.arguments, Some("--verbose".to_string()));
        assert_eq!(payload.git_branch, Some("main".to_string()));
        assert_eq!(payload.user_id, Some(5));
        assert_eq!(payload.build_task_id, Some(10));
        assert_eq!(payload.inventory_id, Some(3));
    }
}
