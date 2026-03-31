//! Маршруты API
//!
//! Декомпозированная версия - маршруты разделены по модулям:
//! - auth — аутентификация, health checks
//! - users — пользователи
//! - projects — проекты, организации
//! - templates — шаблоны, workflows
//! - playbooks — playbooks, inventories, runs
//! - repositories — репозитории, ключи, переменные
//! - tasks — задачи, расписания, интеграции, backup
//! - kubernetes — Kubernetes API (отдельный модуль)
//! - static — статические файлы frontend

use crate::api::routes;
use crate::api::state::AppState;
use axum::Router;
use std::sync::Arc;

/// Создаёт маршруты API
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Auth & Health
        .merge(routes::auth::auth_routes())
        // Users
        .merge(routes::users::user_routes())
        // Projects & Organizations
        .merge(routes::projects::project_routes())
        // Templates & Workflows
        .merge(routes::templates::template_routes())
        // Playbooks & Inventories
        .merge(routes::playbooks::playbook_routes())
        // Repositories, Keys, Environments
        .merge(routes::repositories::repository_routes())
        // Tasks, Schedules, Integrations, Backup
        .merge(routes::tasks::task_routes())
        // Kubernetes API
        .merge(routes::kubernetes::kubernetes_routes())
}

/// Создаёт маршруты для статических файлов
pub fn static_routes() -> Router<Arc<AppState>> {
    crate::api::routes::static_files::static_routes()
}
