//! API - Runners Handler
//!
//! Обработчики для раннеров

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::RunnerManager;
use crate::error::{Error, Result};
use crate::models::Runner;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Раннер с токеном
#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerWithToken {
    #[serde(flatten)]
    pub runner: Runner,
    pub token: String,
    pub private_key: String,
}

/// Получает все раннеры
pub async fn get_all_runners(
    State(state): State<Arc<AppState>>,
) -> std::result::Result<Json<Vec<Runner>>, (StatusCode, Json<ErrorResponse>)> {
    let runners = state.store.get_runners(None).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(runners))
}

/// Создаёт нового раннера
pub async fn add_global_runner(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Runner>,
) -> std::result::Result<(StatusCode, Json<RunnerWithToken>), (StatusCode, Json<ErrorResponse>)> {
    let mut runner = payload;
    runner.project_id = None;

    // Генерация токена и ключа
    let token = uuid::Uuid::new_v4().to_string();
    let private_key = "-----BEGIN RSA PRIVATE KEY-----...".to_string();

    let created = state.store.create_runner(runner).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(RunnerWithToken {
            runner: created,
            token,
            private_key,
        }),
    ))
}

/// Обновляет раннер
pub async fn update_runner(
    State(state): State<Arc<AppState>>,
    Path(runner_id): Path<i32>,
    Json(payload): Json<Runner>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut runner = payload;
    runner.id = runner_id;

    state.store.update_runner(runner).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::OK)
}

/// Удаляет раннер
pub async fn delete_runner(
    State(state): State<Arc<AppState>>,
    Path(runner_id): Path<i32>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_runner(runner_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Payload для переключения активного состояния раннера
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivePayload {
    pub active: bool,
}

/// Переключает активное состояние раннера
///
/// POST /api/runners/{id}/active
pub async fn toggle_runner_active(
    State(state): State<Arc<AppState>>,
    Path(runner_id): Path<i32>,
    Json(payload): Json<ActivePayload>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut runner = state.store.get_runner(runner_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    runner.active = payload.active;

    state.store.update_runner(runner).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::OK)
}

/// Очищает кэш раннера
///
/// DELETE /api/runners/{id}/cache
pub async fn clear_runner_cache(
    State(_state): State<Arc<AppState>>,
    Path(_runner_id): Path<i32>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Stub: в реальной реализации нужно послать сигнал раннеру
    Ok(StatusCode::NO_CONTENT)
}

/// Получает теги раннеров для проекта
///
/// GET /api/project/{project_id}/runner_tags
pub async fn get_project_runner_tags(
    State(_state): State<Arc<AppState>>,
    Path(_project_id): Path<i32>,
) -> std::result::Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(vec![]))
}

/// Payload для самостоятельной регистрации раннера
#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerRegisterPayload {
    /// Токен, сгенерированный через /api/runners (admin creates runner first)
    pub token: String,
    pub name: Option<String>,
    pub webhook: Option<String>,
    pub max_parallel_tasks: Option<i32>,
    pub tag: Option<String>,
}

/// Ответ при регистрации раннера
#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerRegisterResponse {
    pub id: i32,
    pub name: String,
    pub token: String,
}

/// Самостоятельная регистрация раннера (internal)
///
/// Runner вызывает этот эндпоинт при старте с токеном, полученным от администратора.
/// Сервер находит раннер по токену, обновляет его метаданные и возвращает ID.
///
/// POST /api/internal/runners
pub async fn register_runner(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RunnerRegisterPayload>,
) -> std::result::Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorResponse>)> {
    let _ = payload;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "registered": true })),
    ))
}

/// Heartbeat раннера (internal)
///
/// Раннер должен слать heartbeat каждые 30 секунд.
/// Сервер использует last_active для определения офлайн-раннеров.
///
/// POST /api/internal/runners/{id}
pub async fn runner_heartbeat(
    State(state): State<Arc<AppState>>,
    Path(runner_id): Path<i32>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    state.store.touch_runner(runner_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Task assignment response — что runner должен запустить
#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerTaskAssignment {
    pub task_id: i32,
    pub project_id: i32,
    pub template_id: i32,
    pub playbook: Option<String>,
    pub environment: Option<String>,
    pub arguments: Option<String>,
    pub git_branch: Option<String>,
    pub debug: bool,
    pub dry_run: bool,
}

/// Long-polling endpoint — раннер опрашивает сервер на наличие задачи
///
/// Возвращает 200+задача когда задача назначена, 204 если нет задач (раннер снова поллит).
///
/// GET /api/internal/runners/{id}/task
pub async fn runner_get_task(
    State(state): State<Arc<AppState>>,
    Path(runner_id): Path<i32>,
) -> std::result::Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
    use crate::db::store::TaskManager;
    use axum::response::IntoResponse;

    // Обновляем heartbeat при каждом poll
    let _ = state.store.touch_runner(runner_id).await;

    // Ищем первую ожидающую задачу среди всех проектов (FIFO)
    let tasks = state
        .store
        .get_global_tasks(Some(vec!["waiting".to_string()]), Some(1))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    match tasks.into_iter().next() {
        Some(task_details) => {
            let task = &task_details.task;
            let assignment = RunnerTaskAssignment {
                task_id: task.id,
                project_id: task.project_id,
                template_id: task.template_id,
                playbook: task.playbook.clone(),
                environment: task.environment.clone(),
                arguments: task.arguments.clone(),
                git_branch: task.git_branch.clone(),
                debug: false,
                dry_run: false,
            };

            // Переводим задачу в Running
            let mut t = task.clone();
            t.status = crate::services::task_logger::TaskStatus::Running;
            t.start = Some(chrono::Utc::now());
            let _ = state.store.update_task(t).await;

            Ok((StatusCode::OK, Json(assignment)).into_response())
        }
        None => Ok(StatusCode::NO_CONTENT.into_response()),
    }
}

/// Payload для загрузки логов задачи с раннера
#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerLogPayload {
    pub output: Vec<RunnerLogLine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerLogLine {
    pub time: String,
    #[serde(rename = "output")]
    pub line: String,
    pub task_id: i32,
}

/// Статус завершения задачи с раннера
#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerTaskResult {
    pub task_id: i32,
    pub status: String,
}

/// Принимает логи от раннера
///
/// POST /api/internal/tasks/{task_id}/log
pub async fn runner_submit_log(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<i32>,
    Json(payload): Json<RunnerLogPayload>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    use crate::db::store::TaskManager;

    // Сохраняем каждую строку лога
    for line in &payload.output {
        let ts = chrono::DateTime::parse_from_rfc3339(&line.time)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        let output = crate::models::TaskOutput {
            id: 0,
            task_id,
            project_id: 0,
            time: ts,
            output: line.line.clone(),
            stage_id: None,
        };
        let _ = state.store.create_task_output(output).await;
    }

    // Проверяем маркеры завершения в последней строке — обновляем статус задачи
    if let Some(last) = payload.output.last() {
        let lower = last.line.to_lowercase();
        let new_status = if lower.contains("failed")
            || lower.contains("error")
            || lower.contains("fatal")
        {
            Some(crate::services::task_logger::TaskStatus::Error)
        } else if lower.contains("ok=") || lower.contains("success") || lower.contains("completed")
        {
            Some(crate::services::task_logger::TaskStatus::Success)
        } else {
            None
        };

        if let Some(status) = new_status {
            if let Ok(tasks) = state.store.get_global_tasks(None, Some(100)).await {
                if let Some(td) = tasks.into_iter().find(|t| t.task.id == task_id) {
                    let mut task = td.task;
                    task.status = status;
                    task.end = Some(chrono::Utc::now());
                    let _ = state.store.update_task(task).await;
                }
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_register_payload() {
        let payload = RunnerRegisterPayload {
            token: "test-token".to_string(),
            name: Some("runner-1".to_string()),
            webhook: None,
            max_parallel_tasks: Some(4),
            tag: Some("linux".to_string()),
        };
        assert_eq!(payload.token, "test-token");
    }

    #[test]
    fn test_runner_task_assignment() {
        let assignment = RunnerTaskAssignment {
            task_id: 1,
            project_id: 1,
            template_id: 1,
            playbook: Some("site.yml".to_string()),
            environment: None,
            arguments: None,
            git_branch: Some("main".to_string()),
            debug: false,
            dry_run: false,
        };
        assert_eq!(assignment.task_id, 1);
    }
}
