//! Маршруты задач и сервисов проекта
//!
//! Tasks, Schedules, Integrations, Secret Storages, Project Users,
//! Analytics, Views, Notifications, Drift, Credentials, Backup/Restore

use crate::api::handlers;
use crate::api::handlers::projects::{
    integration as project_integration, invites, notifications,
    schedules, secret_storages, tasks, users as project_users, views,
};
use crate::api::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

/// Создаёт маршруты задач и сервисов проекта
pub fn task_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Задачи
        .route("/api/tasks", get(handlers::get_all_tasks))
        .route("/api/projects/{project_id}/tasks", get(handlers::get_tasks))
        .route(
            "/api/projects/{project_id}/tasks",
            post(handlers::create_task),
        )
        .route(
            "/api/projects/{project_id}/tasks/{id}",
            get(handlers::get_task),
        )
        .route(
            "/api/projects/{project_id}/tasks/{id}",
            delete(handlers::delete_task),
        )
        // Vue-алиасы
        .route("/api/project/{project_id}/tasks", get(handlers::get_tasks))
        .route(
            "/api/project/{project_id}/tasks",
            post(handlers::create_task),
        )
        .route(
            "/api/project/{project_id}/tasks/{id}",
            get(handlers::get_task),
        )
        .route(
            "/api/project/{project_id}/tasks/{id}",
            delete(handlers::delete_task),
        )
        // Последние задачи проекта (History)
        .route(
            "/api/project/{project_id}/tasks/last",
            get(tasks::get_last_tasks),
        )
        // Задачи - дополнительные endpoints
        .route(
            "/api/projects/{project_id}/tasks/{id}/stop",
            post(tasks::stop_task),
        )
        .route(
            "/api/projects/{project_id}/tasks/{id}/confirm",
            post(tasks::confirm_task),
        )
        .route(
            "/api/projects/{project_id}/tasks/{id}/reject",
            post(tasks::reject_task),
        )
        .route(
            "/api/projects/{project_id}/tasks/{id}/output",
            get(tasks::get_task_output),
        )
        .route(
            "/api/project/{project_id}/tasks/{id}/stop",
            post(tasks::stop_task),
        )
        .route(
            "/api/project/{project_id}/tasks/{id}/confirm",
            post(tasks::confirm_task),
        )
        .route(
            "/api/project/{project_id}/tasks/{id}/reject",
            post(tasks::reject_task),
        )
        .route(
            "/api/project/{project_id}/tasks/{id}/output",
            get(tasks::get_task_output),
        )
        .route(
            "/api/project/{project_id}/tasks/{id}/raw_output",
            get(tasks::get_task_raw_output),
        )
        .route(
            "/api/project/{project_id}/tasks/{id}/stages",
            get(tasks::get_task_stages),
        )
        .route(
            "/api/projects/{project_id}/tasks/{id}/raw_output",
            get(tasks::get_task_raw_output),
        )
        .route(
            "/api/projects/{project_id}/tasks/{id}/stages",
            get(tasks::get_task_stages),
        )
        // Schedules
        .route(
            "/api/projects/{project_id}/schedules",
            get(schedules::get_project_schedules),
        )
        .route(
            "/api/projects/{project_id}/schedules",
            post(schedules::add_schedule),
        )
        .route(
            "/api/projects/{project_id}/schedules/{id}",
            get(schedules::get_schedule),
        )
        .route(
            "/api/projects/{project_id}/schedules/{id}",
            put(schedules::update_schedule),
        )
        .route(
            "/api/projects/{project_id}/schedules/{id}",
            delete(schedules::delete_schedule),
        )
        .route(
            "/api/projects/{project_id}/schedules/validate",
            post(schedules::validate_schedule_cron_format),
        )
        .route(
            "/api/project/{project_id}/schedules",
            get(schedules::get_project_schedules),
        )
        .route(
            "/api/project/{project_id}/schedules",
            post(schedules::add_schedule),
        )
        .route(
            "/api/project/{project_id}/schedules/{id}",
            get(schedules::get_schedule),
        )
        .route(
            "/api/project/{project_id}/schedules/{id}",
            put(schedules::update_schedule),
        )
        .route(
            "/api/project/{project_id}/schedules/{id}",
            delete(schedules::delete_schedule),
        )
        .route(
            "/api/project/{project_id}/schedules/validate",
            post(schedules::validate_schedule_cron_format),
        )
        // Analytics
        .route(
            "/api/project/{project_id}/analytics",
            get(handlers::analytics::get_project_analytics),
        )
        .route(
            "/api/project/{project_id}/analytics/tasks-chart",
            get(handlers::analytics::get_tasks_chart),
        )
        .route(
            "/api/project/{project_id}/analytics/status-distribution",
            get(handlers::analytics::get_status_distribution),
        )
        .route(
            "/api/analytics/system",
            get(handlers::analytics::get_system_analytics),
        )
        // Views
        .route("/api/projects/{project_id}/views", get(views::get_views))
        .route("/api/projects/{project_id}/views", post(views::add_view))
        .route(
            "/api/projects/{project_id}/views/{id}",
            get(views::get_view),
        )
        .route(
            "/api/projects/{project_id}/views/{id}",
            put(views::update_view),
        )
        .route(
            "/api/projects/{project_id}/views/{id}",
            delete(views::delete_view),
        )
        .route(
            "/api/projects/{project_id}/views/positions",
            post(views::set_view_positions),
        )
        .route("/api/project/{project_id}/views", get(views::get_views))
        .route("/api/project/{project_id}/views", post(views::add_view))
        .route("/api/project/{project_id}/views/{id}", get(views::get_view))
        .route(
            "/api/project/{project_id}/views/{id}",
            put(views::update_view),
        )
        .route(
            "/api/project/{project_id}/views/{id}",
            delete(views::delete_view),
        )
        .route(
            "/api/project/{project_id}/views/positions",
            post(views::set_view_positions),
        )
        // Integrations
        .route(
            "/api/projects/{project_id}/integrations",
            get(project_integration::get_integrations),
        )
        .route(
            "/api/projects/{project_id}/integrations",
            post(project_integration::add_integration),
        )
        .route(
            "/api/projects/{project_id}/integrations/{id}",
            get(project_integration::get_integration),
        )
        .route(
            "/api/projects/{project_id}/integrations/{id}",
            put(project_integration::update_integration),
        )
        .route(
            "/api/projects/{project_id}/integrations/{id}",
            delete(project_integration::delete_integration),
        )
        .route(
            "/api/project/{project_id}/integrations",
            get(project_integration::get_integrations),
        )
        .route(
            "/api/project/{project_id}/integrations",
            post(project_integration::add_integration),
        )
        .route(
            "/api/project/{project_id}/integrations/{id}",
            get(project_integration::get_integration),
        )
        .route(
            "/api/project/{project_id}/integrations/{id}",
            put(project_integration::update_integration),
        )
        .route(
            "/api/project/{project_id}/integrations/{id}",
            delete(project_integration::delete_integration),
        )
        // Integration Aliases - заглушки
        // .route(
        //     "/api/project/{project_id}/integrations/aliases",
        //     get(project_integration::get_integration_aliases),
        // )
        // .route(
        //     "/api/project/{project_id}/integrations/aliases",
        //     post(project_integration::add_integration_alias),
        // )
        // .route(
        //     "/api/project/{project_id}/integrations/aliases/{id}",
        //     get(project_integration::get_integration_alias),
        // )
        // .route(
        //     "/api/project/{project_id}/integrations/aliases/{id}",
        //     put(project_integration::update_integration_alias),
        // )
        // .route(
        //     "/api/project/{project_id}/integrations/aliases/{id}",
        //     delete(project_integration::delete_integration_alias),
        // )
        // Secret Storages
        .route(
            "/api/projects/{project_id}/secret_storages",
            get(secret_storages::get_secret_storages),
        )
        .route(
            "/api/projects/{project_id}/secret_storages",
            post(secret_storages::add_secret_storage),
        )
        .route(
            "/api/projects/{project_id}/secret_storages/{id}",
            get(secret_storages::get_secret_storage),
        )
        .route(
            "/api/projects/{project_id}/secret_storages/{id}",
            put(secret_storages::update_secret_storage),
        )
        .route(
            "/api/projects/{project_id}/secret_storages/{id}",
            delete(secret_storages::delete_secret_storage),
        )
        .route(
            "/api/project/{project_id}/secret_storages",
            get(secret_storages::get_secret_storages),
        )
        .route(
            "/api/project/{project_id}/secret_storages",
            post(secret_storages::add_secret_storage),
        )
        .route(
            "/api/project/{project_id}/secret_storages/{id}",
            get(secret_storages::get_secret_storage),
        )
        .route(
            "/api/project/{project_id}/secret_storages/{id}",
            put(secret_storages::update_secret_storage),
        )
        .route(
            "/api/project/{project_id}/secret_storages/{id}",
            delete(secret_storages::delete_secret_storage),
        )
        // Secret Storages — дополнительные endpoints
        .route(
            "/api/project/{project_id}/secret_storages/{id}/sync",
            post(secret_storages::sync_secret_storage),
        )
        .route(
            "/api/project/{project_id}/secret_storages/{id}/refs",
            get(secret_storages::get_secret_storage_refs),
        )
        .route(
            "/api/projects/{project_id}/secret_storages/{id}/sync",
            post(secret_storages::sync_secret_storage),
        )
        .route(
            "/api/projects/{project_id}/secret_storages/{id}/refs",
            get(secret_storages::get_secret_storage_refs),
        )
        // Project Users
        .route(
            "/api/projects/{project_id}/users",
            get(project_users::get_users),
        )
        .route(
            "/api/projects/{project_id}/users",
            post(project_users::add_user),
        )
        .route(
            "/api/projects/{project_id}/users/{user_id}",
            put(project_users::update_user_role),
        )
        .route(
            "/api/projects/{project_id}/users/{user_id}",
            delete(project_users::delete_user),
        )
        .route(
            "/api/project/{project_id}/users",
            get(project_users::get_users),
        )
        .route(
            "/api/project/{project_id}/users",
            post(project_users::add_user),
        )
        .route(
            "/api/project/{project_id}/users/{user_id}",
            put(project_users::update_user_role),
        )
        .route(
            "/api/project/{project_id}/users/{user_id}",
            delete(project_users::delete_user),
        )
        // Project Invites - заглушки
        // .route(
        //     "/api/project/{project_id}/invites",
        //     get(invites::get_project_invites),
        // )
        // .route(
        //     "/api/project/{project_id}/invites",
        //     post(invites::create_project_invite),
        // )
        // .route(
        //     "/api/project/{project_id}/invites/{id}",
        //     delete(invites::delete_project_invite),
        // )
        // .route(
        //     "/api/project/{project_id}/invites/accept/{token}",
        //     post(invites::accept_invite),
        // )
        // Notification Policies
        .route(
            "/api/project/{project_id}/notifications",
            get(handlers::notification::list_notification_policies),
        )
        .route(
            "/api/project/{project_id}/notifications",
            post(handlers::notification::create_notification_policy),
        )
        .route(
            "/api/project/{project_id}/notifications/{id}",
            get(handlers::notification::get_notification_policy),
        )
        .route(
            "/api/project/{project_id}/notifications/{id}",
            put(handlers::notification::update_notification_policy),
        )
        .route(
            "/api/project/{project_id}/notifications/{id}",
            delete(handlers::notification::delete_notification_policy),
        )
        .route(
            "/api/project/{project_id}/notifications/{id}/test",
            post(handlers::notification::test_notification_policy),
        )
        // Backup & Restore - заглушки, т.к. handlers::backup не существует
        // .route(
        //     "/api/project/{project_id}/backup",
        //     get(handlers::backup::create_backup),
        // )
}
