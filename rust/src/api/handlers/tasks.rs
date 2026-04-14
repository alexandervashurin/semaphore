//! Tasks Handlers
//!
//! Обработчики запросов для управления задачами

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::{
    EnvironmentManager, InventoryManager, ProjectStore, RepositoryManager, TaskManager,
    TemplateManager,
};
use crate::db_lib::AccessKeyInstallerImpl;
use crate::error::Error;
use crate::models::{Environment, Inventory, Repository, Task, TaskOutput, TaskWithTpl};
use crate::services::local_job::LocalJob;
use crate::services::task_logger::{BasicLogger, LogListener, TaskLogger, TaskStatus};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Получить все активные задачи всех проектов
///
/// GET /api/tasks
pub async fn get_all_tasks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TaskWithTpl>>, (StatusCode, Json<ErrorResponse>)> {
    // Получаем все проекты
    let projects = state.store.get_projects(None).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    // Собираем активные задачи из всех проектов
    let mut all_tasks = Vec::new();
    for project in projects {
        let tasks = state
            .store
            .get_tasks(project.id, None::<i32>)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(e.to_string())),
                )
            })?;

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
    let tasks: Result<Vec<TaskWithTpl>, Error> =
        state.store.get_tasks(project_id, None::<i32>).await;

    let tasks = tasks.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

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

    let created: Result<Task, Error> = state.store.create_task(task).await;

    let created = created.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

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
    let task: Result<Task, Error> = state.store.get_task(project_id, task_id).await;

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
    let tasks: Result<Vec<TaskWithTpl>, Error> =
        state.store.get_tasks(project_id, None::<i32>).await;

    let tasks = tasks.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

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
    let result: Result<(), Error> = state.store.delete_task(project_id, task_id).await;

    result.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Выполняет задачу в фоновом потоке
async fn execute_task_background(state: Arc<AppState>, mut task: Task) {
    println!(
        "[task_runner] Starting task {} (template {})",
        task.id, task.template_id
    );
    let store = &state.store;

    match store
        .update_task_status(task.project_id, task.id, TaskStatus::Running)
        .await
    {
        Ok(()) => println!("[task_runner] task {} status → Running", task.id),
        Err(e) => println!("[task_runner] task {} failed to set Running: {e}", task.id),
    }

    let template = match store.get_template(task.project_id, task.template_id).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "[task_runner] task {}: failed to get template: {e}",
                task.id
            );
            let _ = store
                .update_task_status(task.project_id, task.id, TaskStatus::Error)
                .await;
            return;
        }
    };

    let inventory_id = task.inventory_id.or(template.inventory_id);
    let inventory = match inventory_id {
        Some(id) => store
            .get_inventory(task.project_id, id)
            .await
            .unwrap_or_default(),
        None => Inventory::default(),
    };

    let repository_id = task.repository_id.or(template.repository_id);
    let repository = match repository_id {
        Some(id) => store
            .get_repository(task.project_id, id)
            .await
            .unwrap_or_default(),
        None => Repository::default(),
    };

    let environment_id = task.environment_id.or(template.environment_id);
    let environment = match environment_id {
        Some(id) => store
            .get_environment(task.project_id, id)
            .await
            .unwrap_or_default(),
        None => Environment::default(),
    };

    let log_buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let buf_clone = log_buffer.clone();
    let logger = Arc::new(BasicLogger::new());
    logger.add_log_listener(Box::new(move |_time, msg| {
        let _ = buf_clone.lock().map(|mut v| v.push(msg));
    }));

    let work_dir =
        std::env::temp_dir().join(format!("semaphore_task_{}_{}", task.project_id, task.id));
    let tmp_dir = work_dir.join("tmp");

    if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
        eprintln!(
            "[task_runner] task {}: failed to create workdir: {e}",
            task.id
        );
        let _ = store
            .update_task_status(task.project_id, task.id, TaskStatus::Error)
            .await;
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

    job.store =
        Some(Arc::new(state.store.clone()) as Arc<dyn crate::db::store::Store + Send + Sync>);
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

    task.end = Some(Utc::now());
    match result {
        Ok(()) => {
            let _ = store
                .update_task_status(task.project_id, task.id, TaskStatus::Success)
                .await;
            println!("[task_runner] task {} completed successfully", task.id);
        }
        Err(e) => {
            eprintln!("[task_runner] task {} failed: {e}", task.id);
            let _ = store
                .update_task_status(task.project_id, task.id, TaskStatus::Error)
                .await;
        }
    }
}

/// Выполняет задачу с переданными параметрами и возвращает результат
/// Используется workflow executor для выполнения узлов DAG
pub async fn execute_task_background_with_template(
    state: Arc<AppState>,
    task: Task,
    template: crate::models::template::Template,
    inventory: Inventory,
    repository: Repository,
    environment: Environment,
) -> TaskStatus {
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::services::local_job::LocalJob;
    use crate::services::task_logger::BasicLogger;
    use std::sync::Mutex;

    println!(
        "[workflow_task] Starting task {} (template {})",
        task.id, task.template_id
    );
    let store = &state.store;

    match store
        .update_task_status(task.project_id, task.id, TaskStatus::Running)
        .await
    {
        Ok(()) => println!("[workflow_task] task {} status → Running", task.id),
        Err(e) => println!(
            "[workflow_task] task {} failed to set Running: {e}",
            task.id
        ),
    }

    let log_buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let buf_clone = log_buffer.clone();
    let logger = Arc::new(BasicLogger::new());
    logger.add_log_listener(Box::new(move |_time, msg| {
        let _ = buf_clone.lock().map(|mut v| v.push(msg));
    }));

    let work_dir =
        std::env::temp_dir().join(format!("semaphore_task_{}_{}", task.project_id, task.id));
    let tmp_dir = work_dir.join("tmp");

    if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
        eprintln!(
            "[workflow_task] task {}: failed to create workdir: {e}",
            task.id
        );
        let _ = store
            .update_task_status(task.project_id, task.id, TaskStatus::Error)
            .await;
        return TaskStatus::Error;
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

    job.store =
        Some(Arc::new(state.store.clone()) as Arc<dyn crate::db::store::Store + Send + Sync>);
    let result = job.run("runner", None, "default").await;
    job.cleanup();

    // Сохранить логи
    let log_lines: Vec<String> = log_buffer.lock().map(|v| v.clone()).unwrap_or_default();
    for line in log_lines {
        let output = crate::models::task::TaskOutput {
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
            let _ = store
                .update_task_status(task.project_id, task.id, TaskStatus::Success)
                .await;
            println!("[workflow_task] task {} completed successfully", task.id);
            TaskStatus::Success
        }
        Err(e) => {
            eprintln!("[workflow_task] task {} failed: {e}", task.id);
            let _ = store
                .update_task_status(task.project_id, task.id, TaskStatus::Error)
                .await;
            TaskStatus::Error
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

    #[test]
    fn test_task_create_payload_serialize_skip_none() {
        let payload = TaskCreatePayload {
            template_id: 1,
            playbook: None,
            environment: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            build_task_id: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(!json.contains("playbook"));
        assert!(!json.contains("environment"));
        assert!(!json.contains("arguments"));
        assert!(json.contains("\"template_id\":1"));
    }

    #[test]
    fn test_task_create_payload_serialize_with_values() {
        let payload = TaskCreatePayload {
            template_id: 5,
            playbook: Some("deploy.yml".to_string()),
            environment: None,
            arguments: Some("-e env=prod".to_string()),
            git_branch: Some("develop".to_string()),
            user_id: Some(3),
            build_task_id: None,
            inventory_id: Some(2),
            repository_id: Some(1),
            environment_id: Some(4),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"template_id\":5"));
        assert!(json.contains("\"playbook\":\"deploy.yml\""));
        assert!(json.contains("\"git_branch\":\"develop\""));
        assert!(json.contains("\"inventory_id\":2"));
        assert!(json.contains("\"repository_id\":1"));
        assert!(json.contains("\"environment_id\":4"));
        assert!(!json.contains("\"environment\":"));
    }

    #[test]
    fn test_task_create_payload_roundtrip() {
        let original = TaskCreatePayload {
            template_id: 42,
            playbook: Some("rollback.yml".to_string()),
            environment: Some("staging".to_string()),
            arguments: None,
            git_branch: Some("release/v2".to_string()),
            user_id: Some(7),
            build_task_id: Some(99),
            inventory_id: None,
            repository_id: Some(3),
            environment_id: Some(5),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: TaskCreatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.template_id, original.template_id);
        assert_eq!(restored.playbook, original.playbook);
        assert_eq!(restored.environment, original.environment);
        assert_eq!(restored.git_branch, original.git_branch);
        assert_eq!(restored.user_id, original.user_id);
        assert_eq!(restored.build_task_id, original.build_task_id);
        assert_eq!(restored.repository_id, original.repository_id);
        assert_eq!(restored.environment_id, original.environment_id);
    }

    #[test]
    fn test_task_create_payload_empty_string_fields() {
        let json = r#"{
            "template_id": 1,
            "playbook": "",
            "environment": "",
            "arguments": "",
            "git_branch": ""
        }"#;
        let payload: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.playbook, Some("".to_string()));
        assert_eq!(payload.environment, Some("".to_string()));
        assert_eq!(payload.arguments, Some("".to_string()));
        assert_eq!(payload.git_branch, Some("".to_string()));
    }

    #[test]
    fn test_task_create_payload_unicode_values() {
        let json = r#"{
            "template_id": 1,
            "playbook": "развертывание.yml",
            "environment": "Продакшн",
            "arguments": "--comment 'Обновление'"
        }"#;
        let payload: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.playbook, Some("развертывание.yml".to_string()));
        assert_eq!(payload.environment, Some("Продакшн".to_string()));
        assert_eq!(
            payload.arguments,
            Some("--comment 'Обновление'".to_string())
        );
    }

    #[test]
    fn test_task_create_payload_special_chars() {
        let json = r#"{
            "template_id": 1,
            "playbook": "path/to/deploy.yml",
            "arguments": "-e \"key=value with spaces\"",
            "git_branch": "feature/fix-bug#123"
        }"#;
        let payload: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.playbook, Some("path/to/deploy.yml".to_string()));
        assert_eq!(
            payload.arguments,
            Some("-e \"key=value with spaces\"".to_string())
        );
        assert_eq!(payload.git_branch, Some("feature/fix-bug#123".to_string()));
    }

    #[test]
    fn test_task_create_payload_debug() {
        let payload = TaskCreatePayload {
            template_id: 1,
            playbook: Some("test.yml".to_string()),
            environment: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            build_task_id: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("TaskCreatePayload"));
        assert!(debug_str.contains("test.yml"));
    }

    #[test]
    fn test_task_create_payload_clone_independence() {
        // TaskCreatePayload doesn't derive Clone
        let json = r#"{"template_id": 1, "playbook": "original.yml"}"#;
        let p1: TaskCreatePayload = serde_json::from_str(json).unwrap();
        let p2: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(p1.playbook, p2.playbook);
    }

    #[test]
    fn test_task_create_payload_negative_ids() {
        let payload = TaskCreatePayload {
            template_id: -1,
            playbook: None,
            environment: None,
            arguments: None,
            git_branch: None,
            user_id: Some(-5),
            build_task_id: Some(-10),
            inventory_id: Some(-3),
            repository_id: None,
            environment_id: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let restored: TaskCreatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.template_id, -1);
        assert_eq!(restored.user_id, Some(-5));
    }

    #[test]
    fn test_task_create_payload_deserialize_all_optionals_null() {
        let json = r#"{
            "template_id": 1,
            "playbook": null,
            "environment": null,
            "arguments": null,
            "git_branch": null,
            "user_id": null,
            "build_task_id": null,
            "inventory_id": null,
            "repository_id": null,
            "environment_id": null
        }"#;
        let payload: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.template_id, 1);
        assert!(payload.playbook.is_none());
        assert!(payload.environment.is_none());
        assert!(payload.arguments.is_none());
    }

    #[test]
    fn test_task_create_payload_large_template_id() {
        let json = r#"{"template_id": 2147483647}"#;
        let payload: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.template_id, 2147483647);
    }

    #[test]
    fn test_task_create_payload_newline_in_playbook() {
        let payload = TaskCreatePayload {
            template_id: 1,
            playbook: Some("line1\nline2".to_string()),
            environment: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            build_task_id: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let restored: TaskCreatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.playbook, Some("line1\nline2".to_string()));
    }

    #[test]
    fn test_task_create_payload_all_fields_set() {
        let json = r#"{
            "template_id": 10,
            "playbook": "full.yml",
            "environment": "env1",
            "arguments": "--arg1",
            "git_branch": "branch1",
            "user_id": 1,
            "build_task_id": 2,
            "inventory_id": 3,
            "repository_id": 4,
            "environment_id": 5
        }"#;
        let payload: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.template_id, 10);
        assert_eq!(payload.playbook, Some("full.yml".to_string()));
        assert_eq!(payload.environment, Some("env1".to_string()));
        assert_eq!(payload.arguments, Some("--arg1".to_string()));
        assert_eq!(payload.git_branch, Some("branch1".to_string()));
        assert_eq!(payload.user_id, Some(1));
        assert_eq!(payload.build_task_id, Some(2));
        assert_eq!(payload.inventory_id, Some(3));
        assert_eq!(payload.repository_id, Some(4));
        assert_eq!(payload.environment_id, Some(5));
    }

    #[test]
    fn test_task_create_payload_tab_and_special_whitespace() {
        let json = r#"{
            "template_id": 1,
            "arguments": "\t--verbose\t--debug"
        }"#;
        let payload: TaskCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.arguments, Some("\t--verbose\t--debug".to_string()));
    }

    #[test]
    fn test_task_create_payload_serialize_all_fields_present() {
        let payload = TaskCreatePayload {
            template_id: 1,
            playbook: Some("p.yml".to_string()),
            environment: Some("e".to_string()),
            arguments: Some("a".to_string()),
            git_branch: Some("g".to_string()),
            user_id: Some(1),
            build_task_id: Some(2),
            inventory_id: Some(3),
            repository_id: Some(4),
            environment_id: Some(5),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("playbook"));
        assert!(json.contains("environment"));
        assert!(json.contains("arguments"));
        assert!(json.contains("git_branch"));
        assert!(json.contains("user_id"));
        assert!(json.contains("build_task_id"));
        assert!(json.contains("inventory_id"));
        assert!(json.contains("repository_id"));
        assert!(json.contains("environment_id"));
    }
}
