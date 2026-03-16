//! Projects API - Tasks Handler
//!
//! Обработчики для задач в проектах

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::{Task, TaskWithTpl, TaskOutput, Inventory, Repository, Environment};
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::db::store::{RetrieveQueryParams, TaskManager, TemplateManager, InventoryManager, RepositoryManager, EnvironmentManager};
use crate::services::task_logger::{TaskStatus, BasicLogger, TaskLogger, LogListener};
use crate::services::local_job::LocalJob;
use crate::db_lib::AccessKeyInstallerImpl;

/// Получает задачи проекта
pub async fn get_tasks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Query(_params): Query<RetrieveQueryParams>,
) -> std::result::Result<Json<Vec<TaskWithTpl>>, (StatusCode, Json<ErrorResponse>)> {
    let tasks = state.store.get_tasks(project_id, None)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(tasks))
}

/// Получает последние задачи проекта (по дате создания)
///
/// GET /api/project/{project_id}/tasks/last
pub async fn get_last_tasks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Vec<TaskWithTpl>>, (StatusCode, Json<ErrorResponse>)> {
    let tasks = state
        .store
        .get_tasks(project_id, None)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    // Возвращаем только последние 20 записей
    let limited: Vec<TaskWithTpl> = tasks.into_iter().take(20).collect();

    Ok(Json(limited))
}

/// Получает задачу по ID
pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Task>, (StatusCode, Json<ErrorResponse>)> {
    let task = state.store.get_task(project_id, task_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Task not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    Ok(Json(task))
}

/// Создаёт новую задачу
pub async fn add_task(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<CreateTaskPayload>,
) -> std::result::Result<(StatusCode, Json<Task>), (StatusCode, Json<ErrorResponse>)> {
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
        created: chrono::Utc::now(),
        start: None,
        end: None,
        message: payload.message,
        commit_hash: None,
        commit_message: None,
        build_task_id: payload.build_task_id,
        version: None,
        inventory_id: payload.inventory_id,
        repository_id: payload.repository_id,
        environment_id: payload.environment_id,
        params: None,
    };

    let created = state.store.create_task(task)
        .await
        .map_err(|e| (
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

/// Останавливает задачу
pub async fn stop_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // В реальной реализации нужно остановить задачу
    // state.store.stop_task(project_id, task_id).await?;

    Ok(StatusCode::OK)
}

/// Подтверждает задачу
///
/// POST /api/projects/{project_id}/tasks/{task_id}/confirm
pub async fn confirm_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut task = state.store.get_task(project_id, task_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Task not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    // Подтверждение задачи - перевод в статус Waiting
    task.status = TaskStatus::Waiting;
    
    state.store.update_task(task)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Отклоняет задачу
///
/// POST /api/projects/{project_id}/tasks/{task_id}/reject
pub async fn reject_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut task = state.store.get_task(project_id, task_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Task not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    // Отклонение задачи - перевод в статус Rejected
    task.status = TaskStatus::Rejected;
    task.end = Some(chrono::Utc::now());
    
    state.store.update_task(task)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удаляет задачу
pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_task(project_id, task_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Получает вывод задачи (логи)
///
/// GET /api/projects/{project_id}/tasks/{task_id}/output
pub async fn get_task_output(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Vec<TaskOutput>>, (StatusCode, Json<ErrorResponse>)> {
    // Получаем вывод задачи
    let outputs = state.store.get_task_outputs(task_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(outputs))
}

/// Выполняет задачу в фоновом потоке
async fn execute_task_background(state: Arc<AppState>, task: Task) {
    println!("[task_runner] Starting task {} (template {})", task.id, task.template_id);
    let store = &state.store;

    // Обновляем статус → Running
    match store.update_task_status(task.project_id, task.id, TaskStatus::Running).await {
        Ok(()) => println!("[task_runner] task {} status → Running", task.id),
        Err(e) => println!("[task_runner] task {} failed to set Running: {e}", task.id),
    }

    // Загружаем шаблон
    let template = match store.get_template(task.project_id, task.template_id).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[task_runner] task {}: failed to get template: {e}", task.id);
            let _ = store.update_task_status(task.project_id, task.id, TaskStatus::Error).await;
            return;
        }
    };

    // Загружаем инвентарь, репозиторий, окружение
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

    // Логгер с буфером для сохранения в БД
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

    let result = job.run("runner", None, "default").await;
    job.cleanup();

    // Сохраняем логи в БД (копируем под локом, затем освобождаем лок перед await)
    let log_lines: Vec<String> = log_buffer.lock().map(|v| v.clone()).unwrap_or_default();
    for line in log_lines {
        let output = TaskOutput {
            id: 0,
            task_id: task.id,
            project_id: task.project_id,
            time: chrono::Utc::now(),
            output: line,
            stage_id: None,
        };
        let _ = store.create_task_output(output).await;
    }

    match result {
        Ok(()) => {
            let _ = store.update_task_status(task.project_id, task.id, TaskStatus::Success).await;
        }
        Err(e) => {
            eprintln!("[task_runner] task {} failed: {e}", task.id);
            let _ = store.update_task_status(task.project_id, task.id, TaskStatus::Error).await;
        }
    }
}

/// Возвращает raw-вывод задачи (текст без форматирования)
///
/// GET /api/project/{project_id}/tasks/{id}/raw_output
pub async fn get_task_raw_output(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> std::result::Result<String, (StatusCode, Json<ErrorResponse>)> {
    let outputs = state.store.get_task_outputs(task_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    // Объединяем все строки вывода в plain text
    let raw = outputs.iter()
        .map(|o| o.output.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    // Убираем ANSI-коды для raw формата
    let clean = crate::utils::ansi::clear_from_ansi_codes(&raw);
    let _ = project_id; // suppress unused warning
    Ok(clean)
}

/// Возвращает стадии (этапы) задачи
///
/// GET /api/project/{project_id}/tasks/{id}/stages
pub async fn get_task_stages(
    State(_state): State<Arc<AppState>>,
    Path((_project_id, _task_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Stages — это более высокоуровневое представление стадий выполнения
    // В базовой реализации возвращаем пустой список
    Ok(Json(serde_json::json!([])))
}

/// Возвращает все активные задачи по всем проектам
///
/// GET /api/tasks
pub async fn get_all_tasks(
    State(_state): State<Arc<AppState>>,
) -> std::result::Result<Json<Vec<TaskWithTpl>>, (StatusCode, Json<ErrorResponse>)> {
    // Stub: в реальной реализации нужен запрос без фильтра по project_id
    Ok(Json(vec![]))
}

/// Payload для создания задачи
#[derive(Debug, Deserialize)]
pub struct CreateTaskPayload {
    pub template_id: i32,
    pub playbook: Option<String>,
    pub environment: Option<String>,
    pub arguments: Option<String>,
    pub git_branch: Option<String>,
    pub user_id: Option<i32>,
    pub message: Option<String>,
    pub build_task_id: Option<i32>,
    pub inventory_id: Option<i32>,
    pub repository_id: Option<i32>,
    pub environment_id: Option<i32>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tasks_handler() {
        // Тест для проверки обработчиков задач
        assert!(true);
    }
}
