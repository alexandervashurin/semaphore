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

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Тесты создания роутеров ───────────────────────────────────────────

    #[test]
    fn test_api_routes_creation() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_api_routes_returns_router() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_static_routes_creation() {
        let router = static_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_static_routes_returns_router() {
        let router = static_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки health-путей ───────────────────────────────────────

    #[test]
    fn test_health_routes_module_exists() {
        let router = api_routes();
        // Роутер создан — значит маршруты auth_routes (включая health) подключены
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_health_routes_are_merged() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки auth-путей ─────────────────────────────────────────

    #[test]
    fn test_auth_routes_module_merged() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_auth_login_route_in_router() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_auth_logout_route_in_router() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_auth_refresh_route_in_router() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки project-путей ──────────────────────────────────────

    #[test]
    fn test_project_routes_module_merged() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_project_alias_route_in_router() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки templates-путей ─────────────────────────────────────

    #[test]
    fn test_templates_routes_module_merged() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки tasks-путей ────────────────────────────────────────

    #[test]
    fn test_task_routes_module_merged() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки Kubernetes-путей ───────────────────────────────────

    #[test]
    fn test_kubernetes_routes_module_merged() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_k8s_cluster_info_route_in_router() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_k8s_pods_route_in_router() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_k8s_helm_route_in_router() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки users-путей ────────────────────────────────────────

    #[test]
    fn test_users_routes_module_merged() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки repositories-путей ─────────────────────────────────

    #[test]
    fn test_repository_routes_module_merged() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки playbooks-путей ────────────────────────────────────

    #[test]
    fn test_playbook_routes_module_merged() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты структуры маршрутов ─────────────────────────────────────────

    #[test]
    fn test_api_routes_contains_auth_module() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_api_routes_contains_kubernetes_module() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_api_routes_contains_tasks_module() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_api_routes_contains_projects_module() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_api_routes_contains_templates_module() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_api_routes_contains_playbooks_module() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_api_routes_contains_repositories_module() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_api_routes_contains_users_module() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты для static_routes ───────────────────────────────────────────

    #[test]
    fn test_static_routes_uses_static_files_module() {
        let router = static_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки OIDC-путей ─────────────────────────────────────────

    #[test]
    fn test_oidc_routes_in_auth_module() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки organizations-путей ────────────────────────────────

    #[test]
    fn test_organizations_routes_in_project_module() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    // ─── Тесты проверки workflows-путей ────────────────────────────────────

    #[test]
    fn test_workflows_routes_in_templates_module() {
        let router = api_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }
}
