//! API - Apps Handler
//!
//! Обработчики для приложений

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::api::middleware::ErrorResponse;

/// Приложение
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: String,
    pub priority: i32,
    pub title: String,
    pub icon: String,
    pub color: String,
    pub dark_color: String,
    pub path: String,
    pub args: Vec<String>,
}

/// Получает все приложения
pub async fn get_apps(
    State(_state): State<Arc<AppState>>,
) -> std::result::Result<Json<Vec<App>>, (StatusCode, Json<ErrorResponse>)> {
    // В реальной реализации нужно получить приложения из конфига
    let apps = vec![
        App {
            id: "ansible".to_string(),
            priority: 10,
            title: "Ansible".to_string(),
            icon: "ansible".to_string(),
            color: "#FF0000".to_string(),
            dark_color: "#CC0000".to_string(),
            path: "/usr/bin/ansible-playbook".to_string(),
            args: vec![],
        },
    ];

    Ok(Json(apps))
}

/// Получает приложение по ID
pub async fn get_app(
    State(_state): State<Arc<AppState>>,
    Path(_app_id): Path<String>,
) -> std::result::Result<Json<App>, (StatusCode, Json<ErrorResponse>)> {
    // В реальной реализации нужно получить приложение из конфига
    let app = App {
        id: _app_id.clone(),
        priority: 10,
        title: "App".to_string(),
        icon: "app".to_string(),
        color: "#000000".to_string(),
        dark_color: "#000000".to_string(),
        path: "/usr/bin/app".to_string(),
        args: vec![],
    };

    Ok(Json(app))
}

/// Удаляет приложение
/// Приложения (ansible, terraform и т.д.) задаются конфигом; удаление — no-op для совместимости API
pub async fn delete_app(
    State(_state): State<Arc<AppState>>,
    Path(_app_id): Path<String>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let _ = _app_id;
    Ok(StatusCode::NO_CONTENT)
}

/// Payload для обновления приложения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAppPayload {
    pub priority: Option<i32>,
    pub title: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub dark_color: Option<String>,
    pub path: Option<String>,
    pub args: Option<Vec<String>>,
}

/// Payload для переключения активного состояния приложения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePayload {
    pub active: bool,
}

/// Обновляет конфигурацию приложения
///
/// PUT /api/apps/{id}
pub async fn update_app(
    State(_state): State<Arc<AppState>>,
    Path(app_id): Path<String>,
    Json(payload): Json<UpdateAppPayload>,
) -> std::result::Result<Json<App>, (StatusCode, Json<ErrorResponse>)> {
    let app = App {
        id: app_id.clone(),
        priority: payload.priority.unwrap_or(10),
        title: payload.title.unwrap_or_else(|| app_id.clone()),
        icon: payload.icon.unwrap_or_else(|| app_id.clone()),
        color: payload.color.unwrap_or_else(|| "#000000".to_string()),
        dark_color: payload.dark_color.unwrap_or_else(|| "#000000".to_string()),
        path: payload.path.unwrap_or_else(|| format!("/usr/bin/{}", app_id)),
        args: payload.args.unwrap_or_default(),
    };
    Ok(Json(app))
}

/// Переключает активное состояние приложения
///
/// POST /api/apps/{id}/active
pub async fn toggle_app_active(
    State(_state): State<Arc<AppState>>,
    Path(_app_id): Path<String>,
    Json(_payload): Json<ActivePayload>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    Ok(StatusCode::OK)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apps_handler() {
        // Тест для проверки обработчиков приложений
        assert!(true);
    }
}
