//! Маршруты API

use axum::{Router, routing::{get, post, put, delete}};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::api::handlers;

/// Создаёт маршруты API
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Health check
        .route("/api/health", get(handlers::health))
        
        // Аутентификация
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/logout", post(handlers::logout))
        
        // Пользователи
        .route("/api/users", get(handlers::get_users))
        .route("/api/users/:id", get(handlers::get_user))
        .route("/api/users/:id", put(handlers::update_user))
        .route("/api/users/:id", delete(handlers::delete_user))
        
        // Проекты
        .route("/api/projects", get(handlers::get_projects))
        .route("/api/projects", post(handlers::create_project))
        .route("/api/projects/:id", get(handlers::get_project))
        .route("/api/projects/:id", put(handlers::update_project))
        .route("/api/projects/:id", delete(handlers::delete_project))
        
        // Шаблоны
        .route("/api/projects/:project_id/templates", get(handlers::get_templates))
        .route("/api/projects/:project_id/templates", post(handlers::create_template))
        .route("/api/projects/:project_id/templates/:id", get(handlers::get_template))
        .route("/api/projects/:project_id/templates/:id", put(handlers::update_template))
        .route("/api/projects/:project_id/templates/:id", delete(handlers::delete_template))
        
        // Задачи
        .route("/api/projects/:project_id/tasks", get(handlers::get_tasks))
        .route("/api/projects/:project_id/tasks", post(handlers::create_task))
        .route("/api/projects/:project_id/tasks/:id", get(handlers::get_task))
        .route("/api/projects/:project_id/tasks/:id", delete(handlers::delete_task))
        
        // Инвентари
        .route("/api/projects/:project_id/inventories", get(handlers::get_inventories))
        .route("/api/projects/:project_id/inventories", post(handlers::create_inventory))
        .route("/api/projects/:project_id/inventories/:id", get(handlers::get_inventory))
        .route("/api/projects/:project_id/inventories/:id", put(handlers::update_inventory))
        .route("/api/projects/:project_id/inventories/:id", delete(handlers::delete_inventory))
        
        // Репозитории
        .route("/api/projects/:project_id/repositories", get(handlers::get_repositories))
        .route("/api/projects/:project_id/repositories", post(handlers::create_repository))
        .route("/api/projects/:project_id/repositories/:id", get(handlers::get_repository))
        .route("/api/projects/:project_id/repositories/:id", put(handlers::update_repository))
        .route("/api/projects/:project_id/repositories/:id", delete(handlers::delete_repository))
        
        // Окружения
        .route("/api/projects/:project_id/environments", get(handlers::get_environments))
        .route("/api/projects/:project_id/environments", post(handlers::create_environment))
        .route("/api/projects/:project_id/environments/:id", get(handlers::get_environment))
        .route("/api/projects/:project_id/environments/:id", put(handlers::update_environment))
        .route("/api/projects/:project_id/environments/:id", delete(handlers::delete_environment))
        
        // Ключи доступа
        .route("/api/projects/:project_id/keys", get(handlers::get_access_keys))
        .route("/api/projects/:project_id/keys", post(handlers::create_access_key))
        .route("/api/projects/:project_id/keys/:id", get(handlers::get_access_key))
        .route("/api/projects/:project_id/keys/:id", put(handlers::update_access_key))
        .route("/api/projects/:project_id/keys/:id", delete(handlers::delete_access_key))
}

/// Создаёт маршруты для статических файлов
pub fn static_routes() -> Router<Arc<AppState>> {
    // TODO: Добавить раздачу статических файлов (frontend)
    Router::new()
}
