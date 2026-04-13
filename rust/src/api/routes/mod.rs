//! Модули маршрутов API
//!
//! Декомпозиция routes.rs на логические модули:
//! - auth — аутентификация, TOTP, OIDC, health checks
//! - users — управление пользователями
//! - projects — проекты, организации, брендинг, deployment environments
//! - templates — шаблоны, workflows, marketplace, survey forms
//! - playbooks — playbooks, inventories, запуски
//! - repositories — репозитории, ключи доступа, переменные окружения
//! - tasks — задачи, расписания, интеграции, вебхуки, backup/restore
//! - kubernetes — Kubernetes API (отдельный подмодуль, ~720 строк)
//! - static_files — статические файлы frontend

pub mod auth;
pub mod kubernetes;
pub mod playbooks;
pub mod projects;
pub mod repositories;
pub mod static_files;
pub mod tasks;
pub mod templates;
pub mod users;

pub use auth::auth_routes;
pub use kubernetes::kubernetes_routes;
pub use playbooks::playbook_routes;
pub use projects::project_routes;
pub use repositories::repository_routes;
pub use static_files::static_routes;
pub use tasks::task_routes;
pub use templates::template_routes;
pub use users::user_routes;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_module_exists() {
        let _ = auth::auth_routes;
    }

    #[test]
    fn test_users_module_exists() {
        let _ = users::user_routes;
    }

    #[test]
    fn test_projects_module_exists() {
        let _ = projects::project_routes;
    }

    #[test]
    fn test_templates_module_exists() {
        let _ = templates::template_routes;
    }

    #[test]
    fn test_playbooks_module_exists() {
        let _ = playbooks::playbook_routes;
    }

    #[test]
    fn test_repositories_module_exists() {
        let _ = repositories::repository_routes;
    }

    #[test]
    fn test_tasks_module_exists() {
        let _ = tasks::task_routes;
    }

    #[test]
    fn test_kubernetes_module_exists() {
        let _ = kubernetes::kubernetes_routes;
    }

    #[test]
    fn test_static_files_module_exists() {
        let _ = static_files::static_routes;
    }

    #[test]
    fn test_auth_routes_function_is_public() {
        let _: fn() -> axum::Router<std::sync::Arc<crate::api::state::AppState>> = auth_routes;
    }

    #[test]
    fn test_user_routes_function_is_public() {
        let _: fn() -> axum::Router<std::sync::Arc<crate::api::state::AppState>> = user_routes;
    }

    #[test]
    fn test_project_routes_function_is_public() {
        let _: fn() -> axum::Router<std::sync::Arc<crate::api::state::AppState>> = project_routes;
    }

    #[test]
    fn test_template_routes_function_is_public() {
        let _: fn() -> axum::Router<std::sync::Arc<crate::api::state::AppState>> = template_routes;
    }

    #[test]
    fn test_playbook_routes_function_is_public() {
        let _: fn() -> axum::Router<std::sync::Arc<crate::api::state::AppState>> = playbook_routes;
    }

    #[test]
    fn test_repository_routes_function_is_public() {
        let _: fn() -> axum::Router<std::sync::Arc<crate::api::state::AppState>> = repository_routes;
    }

    #[test]
    fn test_task_routes_function_is_public() {
        let _: fn() -> axum::Router<std::sync::Arc<crate::api::state::AppState>> = task_routes;
    }

    #[test]
    fn test_kubernetes_routes_function_is_public() {
        let _: fn() -> axum::Router<std::sync::Arc<crate::api::state::AppState>> = kubernetes_routes;
    }

    #[test]
    fn test_static_routes_function_is_public() {
        let _: fn() -> axum::Router<std::sync::Arc<crate::api::state::AppState>> = static_routes;
    }

    #[test]
    fn test_all_route_functions_have_same_signature() {
        // All route functions return Router<Arc<AppState>>
        type RouteFn = fn() -> axum::Router<std::sync::Arc<crate::api::state::AppState>>;
        let _fns: [RouteFn; 9] = [
            auth_routes,
            user_routes,
            project_routes,
            template_routes,
            playbook_routes,
            repository_routes,
            task_routes,
            kubernetes_routes,
            static_routes,
        ];
    }
}
