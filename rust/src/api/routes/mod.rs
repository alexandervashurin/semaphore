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
