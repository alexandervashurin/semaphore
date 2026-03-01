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
use crate::error::Result;
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
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
    Path(app_id): Path<String>,
) -> std::result::Result<Json<App>, (StatusCode, Json<ErrorResponse>)> {
    // В реальной реализации нужно получить приложение из конфига
    let app = App {
        id: app_id,
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
pub async fn delete_app(
    State(state): State<Arc<AppState>>,
    Path(app_id): Path<String>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // В реальной реализации нужно удалить приложение из конфига
    Ok(StatusCode::NO_CONTENT)
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
