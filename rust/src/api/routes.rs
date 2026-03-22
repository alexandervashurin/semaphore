//! Маршруты API

use axum::{Router, routing::{get, post, put, delete}};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::api::handlers;
use crate::api::websocket::websocket_handler;
use crate::api::handlers::projects::{schedules, views, integration as project_integration, integration_alias, secret_storages, users as project_users, tasks, templates, repository, notifications, backup_restore, refs, invites, roles};
use crate::api::{events, apps, options, runners, cache, system_info, user, graphql, mcp};
use crate::api::handlers::totp;
use tower_http::services::{ServeDir, ServeFile};

/// Создаёт маршруты API
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Health checks
        .route("/api/health", get(handlers::health))
        .route("/api/health/live", get(handlers::health_live))
        .route("/api/health/ready", get(handlers::health_ready))

        // Аутентификация
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/login", get(handlers::get_login_metadata))
        .route("/api/auth/logout", post(handlers::logout))
        .route("/api/auth/refresh", post(handlers::refresh_token))
        .route("/api/auth/verify", post(handlers::verify_session))
        .route("/api/auth/recovery", post(handlers::recovery_session))
        
        // OIDC
        .route("/api/auth/oidc/{provider}", get(handlers::oidc_login))
        .route("/api/auth/oidc/{provider}/callback", get(handlers::oidc_callback))

        // TOTP
        .route("/api/auth/totp/start", post(totp::start_totp_setup))
        .route("/api/auth/totp/confirm", post(totp::confirm_totp_setup))
        .route("/api/auth/totp/disable", post(totp::disable_totp))

        // Текущий пользователь
        .route("/api/user", get(handlers::get_current_user))

        // Пользователи
        .route("/api/users", get(handlers::get_users))
        .route("/api/users", post(handlers::create_user))
        .route("/api/users/{id}", get(handlers::get_user))
        .route("/api/users/{id}", put(handlers::update_user))
        .route("/api/users/{id}", delete(handlers::delete_user))
        .route("/api/users/{id}/password", post(handlers::update_user_password))

        // Проекты
        .route("/api/projects", get(handlers::get_projects))
        .route("/api/projects", post(handlers::add_project))
        .route("/api/projects/restore", post(handlers::restore_project))
        .route("/api/projects/{id}", get(handlers::get_project))
        .route("/api/projects/{id}", put(handlers::update_project))
        .route("/api/projects/{id}", delete(handlers::delete_project))

        // Алиасы для Vue upstream (singular /api/project/ вместо /api/projects/)
        .route("/api/project/{id}", get(handlers::get_project))
        .route("/api/project/{id}", put(handlers::update_project))
        .route("/api/project/{id}", delete(handlers::delete_project))

        // Leave project + Project stats
        .route("/api/project/{project_id}/me", delete(handlers::projects::project::leave_project))
        .route("/api/projects/{project_id}/me", delete(handlers::projects::project::leave_project))
        .route("/api/project/{project_id}/stats", get(handlers::projects::project::get_project_stats))
        .route("/api/projects/{project_id}/stats", get(handlers::projects::project::get_project_stats))

        // Шаблоны
        .route("/api/projects/{project_id}/templates", get(handlers::get_templates))
        .route("/api/projects/{project_id}/templates", post(handlers::create_template))
        .route("/api/projects/{project_id}/templates/{id}", get(handlers::get_template))
        .route("/api/projects/{project_id}/templates/{id}", put(handlers::update_template))
        .route("/api/projects/{project_id}/templates/{id}", delete(handlers::delete_template))
        .route("/api/project/{project_id}/templates", get(handlers::get_templates))
        .route("/api/project/{project_id}/templates", post(handlers::create_template))
        .route("/api/project/{project_id}/templates/{id}", get(handlers::get_template))
        .route("/api/project/{project_id}/templates/{id}", put(handlers::update_template))
        .route("/api/project/{project_id}/templates/{id}", delete(handlers::delete_template))
        .route("/api/project/{project_id}/templates/{id}/stop_all_tasks", post(handlers::stop_all_template_tasks))

        // Задачи
        .route("/api/tasks", get(handlers::get_all_tasks))
        .route("/api/projects/{project_id}/tasks", get(handlers::get_tasks))
        .route("/api/projects/{project_id}/tasks", post(handlers::create_task))
        .route("/api/projects/{project_id}/tasks/{id}", get(handlers::get_task))
        .route("/api/projects/{project_id}/tasks/{id}", delete(handlers::delete_task))
        // Vue-алиасы
        .route("/api/project/{project_id}/tasks", get(handlers::get_tasks))
        .route("/api/project/{project_id}/tasks", post(handlers::create_task))
        .route("/api/project/{project_id}/tasks/{id}", get(handlers::get_task))
        .route("/api/project/{project_id}/tasks/{id}", delete(handlers::delete_task))
        // Последние задачи проекта (History)
        .route("/api/project/{project_id}/tasks/last", get(tasks::get_last_tasks))

        // Инвентари
        .route("/api/projects/{project_id}/inventories", get(handlers::get_inventories))
        .route("/api/projects/{project_id}/inventories", post(handlers::create_inventory))
        .route("/api/projects/{project_id}/inventories/{id}", get(handlers::get_inventory))
        .route("/api/projects/{project_id}/inventories/{id}", put(handlers::update_inventory))
        .route("/api/projects/{project_id}/inventories/{id}", delete(handlers::delete_inventory))

        // Алиас Vue: /api/project/{id}/inventory
        .route("/api/project/{project_id}/inventory", get(handlers::get_inventories))
        .route("/api/project/{project_id}/inventory", post(handlers::create_inventory))
        .route("/api/project/{project_id}/inventory/{id}", get(handlers::get_inventory))
        .route("/api/project/{project_id}/inventory/{id}", put(handlers::update_inventory))
        .route("/api/project/{project_id}/inventory/{id}", delete(handlers::delete_inventory))

        // Playbooks endpoint (из upstream)
        .route("/api/projects/{project_id}/inventories/playbooks", get(handlers::get_playbooks))
        
        // Playbooks - новые endpoints
        .route("/api/project/{project_id}/playbooks", get(handlers::playbook::get_project_playbooks))
        .route("/api/project/{project_id}/playbooks", post(handlers::playbook::create_playbook))
        .route("/api/project/{project_id}/playbooks/{id}", get(handlers::playbook::get_playbook))
        .route("/api/project/{project_id}/playbooks/{id}", put(handlers::playbook::update_playbook))
        .route("/api/project/{project_id}/playbooks/{id}", delete(handlers::playbook::delete_playbook))
        .route("/api/project/{project_id}/playbooks/{id}/sync", post(handlers::playbook::sync_playbook))
        .route("/api/project/{project_id}/playbooks/{id}/preview", get(handlers::playbook::preview_playbook))
        .route("/api/project/{project_id}/playbooks/{id}/run", post(handlers::playbook::run_playbook))
        
        // Playbook Runs - история запусков
        .route("/api/project/{project_id}/playbook-runs", get(handlers::playbook_runs::get_playbook_runs))
        .route("/api/project/{project_id}/playbook-runs/{id}", get(handlers::playbook_runs::get_playbook_run))
        .route("/api/project/{project_id}/playbook-runs/{id}", delete(handlers::playbook_runs::delete_playbook_run))
        .route("/api/project/{project_id}/playbooks/{playbook_id}/runs/stats", get(handlers::playbook_runs::get_playbook_run_stats))

        // Workflows (DAG)
        .route("/api/project/{project_id}/workflows", get(handlers::workflow::get_workflows))
        .route("/api/project/{project_id}/workflows", post(handlers::workflow::create_workflow))
        .route("/api/project/{project_id}/workflows/{id}", get(handlers::workflow::get_workflow))
        .route("/api/project/{project_id}/workflows/{id}", put(handlers::workflow::update_workflow))
        .route("/api/project/{project_id}/workflows/{id}", delete(handlers::workflow::delete_workflow))
        .route("/api/project/{project_id}/workflows/{id}/nodes", post(handlers::workflow::add_workflow_node))
        .route("/api/project/{project_id}/workflows/{id}/nodes/{node_id}", put(handlers::workflow::update_workflow_node))
        .route("/api/project/{project_id}/workflows/{id}/nodes/{node_id}", delete(handlers::workflow::delete_workflow_node))
        .route("/api/project/{project_id}/workflows/{id}/edges", post(handlers::workflow::add_workflow_edge))
        .route("/api/project/{project_id}/workflows/{id}/edges/{edge_id}", delete(handlers::workflow::delete_workflow_edge))
        .route("/api/project/{project_id}/workflows/{id}/run", post(handlers::workflow::run_workflow))
        .route("/api/project/{project_id}/workflows/{id}/runs", get(handlers::workflow::get_workflow_runs))

        // AI Integration
        .route("/api/ai/settings", get(handlers::ai::get_ai_settings))
        .route("/api/ai/settings", put(handlers::ai::update_ai_settings))
        .route("/api/ai/analyze", post(handlers::ai::analyze_failure))
        .route("/api/ai/generate", post(handlers::ai::generate_playbook))

        // Репозитории
        .route("/api/projects/{project_id}/repositories", get(handlers::get_repositories))
        .route("/api/projects/{project_id}/repositories", post(handlers::create_repository))
        .route("/api/projects/{project_id}/repositories/{id}", get(handlers::get_repository))
        .route("/api/projects/{project_id}/repositories/{id}", put(handlers::update_repository))
        .route("/api/projects/{project_id}/repositories/{id}", delete(handlers::delete_repository))
        .route("/api/project/{project_id}/repositories", get(handlers::get_repositories))
        .route("/api/project/{project_id}/repositories", post(handlers::create_repository))
        .route("/api/project/{project_id}/repositories/{id}", get(handlers::get_repository))
        .route("/api/project/{project_id}/repositories/{id}", put(handlers::update_repository))
        .route("/api/project/{project_id}/repositories/{id}", delete(handlers::delete_repository))

        // Окружения
        .route("/api/projects/{project_id}/environments", get(handlers::get_environments))
        .route("/api/projects/{project_id}/environments", post(handlers::create_environment))
        .route("/api/projects/{project_id}/environments/{id}", get(handlers::get_environment))
        .route("/api/projects/{project_id}/environments/{id}", put(handlers::update_environment))
        .route("/api/projects/{project_id}/environments/{id}", delete(handlers::delete_environment))
        // Алиас Vue: /api/project/{id}/environment
        .route("/api/project/{project_id}/environment", get(handlers::get_environments))
        .route("/api/project/{project_id}/environment", post(handlers::create_environment))
        .route("/api/project/{project_id}/environment/{id}", get(handlers::get_environment))
        .route("/api/project/{project_id}/environment/{id}", put(handlers::update_environment))
        .route("/api/project/{project_id}/environment/{id}", delete(handlers::delete_environment))

        // Ключи доступа
        .route("/api/projects/{project_id}/keys", get(handlers::get_access_keys))
        .route("/api/projects/{project_id}/keys", post(handlers::create_access_key))
        .route("/api/projects/{project_id}/keys/{id}", get(handlers::get_access_key))
        .route("/api/projects/{project_id}/keys/{id}", put(handlers::update_access_key))
        .route("/api/projects/{project_id}/keys/{id}", delete(handlers::delete_access_key))
        .route("/api/project/{project_id}/keys", get(handlers::get_access_keys))
        .route("/api/project/{project_id}/keys", post(handlers::create_access_key))
        .route("/api/project/{project_id}/keys/{id}", get(handlers::get_access_key))
        .route("/api/project/{project_id}/keys/{id}", put(handlers::update_access_key))
        .route("/api/project/{project_id}/keys/{id}", delete(handlers::delete_access_key))

        // Расписания (Schedules)
        .route("/api/projects/{project_id}/schedules", get(schedules::get_project_schedules))
        .route("/api/projects/{project_id}/schedules", post(schedules::add_schedule))
        .route("/api/projects/{project_id}/schedules/{id}", get(schedules::get_schedule))
        .route("/api/projects/{project_id}/schedules/{id}", put(schedules::update_schedule))
        .route("/api/projects/{project_id}/schedules/{id}", delete(schedules::delete_schedule))
        .route("/api/projects/{project_id}/schedules/validate", post(schedules::validate_schedule_cron_format))
        .route("/api/project/{project_id}/schedules", get(schedules::get_project_schedules))
        .route("/api/project/{project_id}/schedules", post(schedules::add_schedule))
        .route("/api/project/{project_id}/schedules/{id}", get(schedules::get_schedule))
        .route("/api/project/{project_id}/schedules/{id}", put(schedules::update_schedule))
        .route("/api/project/{project_id}/schedules/{id}", delete(schedules::delete_schedule))
        .route("/api/project/{project_id}/schedules/validate", post(schedules::validate_schedule_cron_format))

        // Analytics
        .route("/api/project/{project_id}/analytics", get(handlers::analytics::get_project_analytics))
        .route("/api/project/{project_id}/analytics/tasks-chart", get(handlers::analytics::get_tasks_chart))
        .route("/api/project/{project_id}/analytics/status-distribution", get(handlers::analytics::get_status_distribution))
        .route("/api/analytics/system", get(handlers::analytics::get_system_analytics))

        // Представления (Views)
        .route("/api/projects/{project_id}/views", get(views::get_views))
        .route("/api/projects/{project_id}/views", post(views::add_view))
        .route("/api/projects/{project_id}/views/{id}", get(views::get_view))
        .route("/api/projects/{project_id}/views/{id}", put(views::update_view))
        .route("/api/projects/{project_id}/views/{id}", delete(views::delete_view))
        .route("/api/projects/{project_id}/views/positions", post(views::set_view_positions))
        .route("/api/project/{project_id}/views", get(views::get_views))
        .route("/api/project/{project_id}/views", post(views::add_view))
        .route("/api/project/{project_id}/views/{id}", get(views::get_view))
        .route("/api/project/{project_id}/views/{id}", put(views::update_view))
        .route("/api/project/{project_id}/views/{id}", delete(views::delete_view))
        .route("/api/project/{project_id}/views/positions", post(views::set_view_positions))

        // Интеграции (Integrations)
        .route("/api/projects/{project_id}/integrations", get(project_integration::get_integrations))
        .route("/api/projects/{project_id}/integrations", post(project_integration::add_integration))
        .route("/api/projects/{project_id}/integrations/{id}", get(project_integration::get_integration))
        .route("/api/projects/{project_id}/integrations/{id}", put(project_integration::update_integration))
        .route("/api/projects/{project_id}/integrations/{id}", delete(project_integration::delete_integration))
        .route("/api/project/{project_id}/integrations", get(project_integration::get_integrations))
        .route("/api/project/{project_id}/integrations", post(project_integration::add_integration))
        .route("/api/project/{project_id}/integrations/{id}", get(project_integration::get_integration))
        .route("/api/project/{project_id}/integrations/{id}", put(project_integration::update_integration))
        .route("/api/project/{project_id}/integrations/{id}", delete(project_integration::delete_integration))

        // Хранилища секретов (Secret Storages)
        .route("/api/projects/{project_id}/secret_storages", get(secret_storages::get_secret_storages))
        .route("/api/projects/{project_id}/secret_storages", post(secret_storages::add_secret_storage))
        .route("/api/projects/{project_id}/secret_storages/{id}", get(secret_storages::get_secret_storage))
        .route("/api/projects/{project_id}/secret_storages/{id}", put(secret_storages::update_secret_storage))
        .route("/api/projects/{project_id}/secret_storages/{id}", delete(secret_storages::delete_secret_storage))
        .route("/api/project/{project_id}/secret_storages", get(secret_storages::get_secret_storages))
        .route("/api/project/{project_id}/secret_storages", post(secret_storages::add_secret_storage))
        .route("/api/project/{project_id}/secret_storages/{id}", get(secret_storages::get_secret_storage))
        .route("/api/project/{project_id}/secret_storages/{id}", put(secret_storages::update_secret_storage))
        .route("/api/project/{project_id}/secret_storages/{id}", delete(secret_storages::delete_secret_storage))
        // Secret Storages — дополнительные endpoints (B-BE-06/07)
        .route("/api/project/{project_id}/secret_storages/{id}/sync", post(secret_storages::sync_secret_storage))
        .route("/api/project/{project_id}/secret_storages/{id}/refs", get(secret_storages::get_secret_storage_refs))
        .route("/api/projects/{project_id}/secret_storages/{id}/sync", post(secret_storages::sync_secret_storage))
        .route("/api/projects/{project_id}/secret_storages/{id}/refs", get(secret_storages::get_secret_storage_refs))

        // Пользователи проекта (Project Users)
        .route("/api/projects/{project_id}/users", get(project_users::get_users))
        .route("/api/projects/{project_id}/users", post(project_users::add_user))
        .route("/api/projects/{project_id}/users/{user_id}", put(project_users::update_user_role))
        .route("/api/projects/{project_id}/users/{user_id}", delete(project_users::delete_user))
        .route("/api/project/{project_id}/users", get(project_users::get_users))
        .route("/api/project/{project_id}/users", post(project_users::add_user))
        .route("/api/project/{project_id}/users/{user_id}", put(project_users::update_user_role))
        .route("/api/project/{project_id}/users/{user_id}", delete(project_users::delete_user))

        // Задачи (Tasks) - дополнительные endpoints
        .route("/api/projects/{project_id}/tasks/{id}/stop", post(tasks::stop_task))
        .route("/api/projects/{project_id}/tasks/{id}/confirm", post(tasks::confirm_task))
        .route("/api/projects/{project_id}/tasks/{id}/reject", post(tasks::reject_task))
        .route("/api/projects/{project_id}/tasks/{id}/output", get(tasks::get_task_output))
        .route("/api/project/{project_id}/tasks/{id}/stop", post(tasks::stop_task))
        .route("/api/project/{project_id}/tasks/{id}/confirm", post(tasks::confirm_task))
        .route("/api/project/{project_id}/tasks/{id}/reject", post(tasks::reject_task))
        .route("/api/project/{project_id}/tasks/{id}/output", get(tasks::get_task_output))

        // Роль пользователя в проекте
        .route("/api/projects/{project_id}/role", get(handlers::get_user_role))
        .route("/api/project/{project_id}/role", get(handlers::get_user_role))

        // Кастомные роли (Custom Roles)
        .route("/api/project/{project_id}/roles/all", get(roles::get_all_roles))
        .route("/api/project/{project_id}/roles", get(roles::get_roles))
        .route("/api/project/{project_id}/roles", post(roles::create_role))
        .route("/api/project/{project_id}/roles/{id}", get(roles::get_role))
        .route("/api/project/{project_id}/roles/{id}", put(roles::update_role))
        .route("/api/project/{project_id}/roles/{id}", delete(roles::delete_role))

        // Backup/Restore
        .route("/api/project/{project_id}/backup", get(backup_restore::get_backup))
        .route("/api/project/{project_id}/backup", post(backup_restore::restore_backup))
        .route("/api/backup/verify", post(backup_restore::verify_backup))

        // Refs (keys, repositories, inventory, templates, integrations)
        .route("/api/project/{project_id}/keys/{key_id}/refs", get(refs::get_key_refs))
        .route("/api/project/{project_id}/repositories/{repository_id}/refs", get(refs::get_repository_refs))
        .route("/api/project/{project_id}/inventory/{inventory_id}/refs", get(refs::get_inventory_refs))
        .route("/api/project/{project_id}/templates/{template_id}/refs", get(refs::get_template_refs))
        .route("/api/project/{project_id}/integrations/{integration_id}/refs", get(refs::get_integration_refs))

        // Integration aliases
        .route("/api/project/{project_id}/integrations/aliases", get(integration_alias::get_integration_aliases))
        .route("/api/project/{project_id}/integrations/aliases", post(integration_alias::add_integration_alias))
        .route("/api/project/{project_id}/integrations/aliases/{alias_id}", delete(integration_alias::delete_integration_alias))

        // Invites
        .route("/api/project/{project_id}/invites", get(invites::get_invites))
        .route("/api/project/{project_id}/invites", post(invites::create_invite))
        .route("/api/project/{project_id}/invites/{invite_id}", delete(invites::delete_invite))
        .route("/api/invites/accept/{token}", post(invites::accept_invite))

        // Уведомления (Notifications)
        .route("/api/projects/{project_id}/notifications/test", post(notifications::send_test_notification))
        .route("/api/project/{project_id}/notifications/test", post(notifications::send_test_notification))

        // WebSocket
        .route("/api/ws", get(websocket_handler))

        // События (Events)
        .route("/api/events", get(events::get_all_events))
        .route("/api/events/last", get(events::get_last_events))
        .route("/api/projects/{project_id}/events", get(events::get_project_events))
        .route("/api/project/{project_id}/events", get(events::get_project_events))

        // Приложения (Apps)
        .route("/api/apps", get(apps::get_apps))
        .route("/api/apps/{id}", get(apps::get_app))
        .route("/api/apps/{id}", put(apps::update_app))
        .route("/api/apps/{id}", delete(apps::delete_app))
        // Apps - дополнительные endpoints (B-BE-04/05)
        .route("/api/apps/{id}/active", post(apps::toggle_app_active))

        // Опции (Options) - admin only
        .route("/api/options", get(options::get_options))
        .route("/api/options", post(options::set_option))

        // Mailer - admin only
        .route("/api/admin/mail/test", post(handlers::send_test_email))

        // Раннеры (Runners) - admin only
        .route("/api/runners", get(runners::get_all_runners))
        .route("/api/runners", post(runners::add_global_runner))
        .route("/api/runners/{id}", put(runners::update_runner))
        .route("/api/runners/{id}", delete(runners::delete_runner))
        // Раннеры - дополнительные endpoints (B-BE-01/02/03)
        .route("/api/runners/{id}/active", post(runners::toggle_runner_active))
        .route("/api/runners/{id}/cache", delete(runners::clear_runner_cache))
        .route("/api/project/{project_id}/runner_tags", get(runners::get_project_runner_tags))
        .route("/api/internal/runners", post(runners::register_runner))
        .route("/api/internal/runners/{id}", post(runners::runner_heartbeat))

        // Кэш (Cache) - admin only
        .route("/api/cache", delete(cache::clear_cache))

        // Кэш проекта (B-BE-24)
        .route("/api/project/{id}/cache", delete(cache::clear_project_cache))

        // Системная информация (System Info)
        .route("/api/info", get(system_info::get_system_info))

        // Prometheus Metrics
        .route("/api/metrics", get(handlers::metrics::get_metrics))
        .route("/api/metrics/json", get(handlers::metrics::get_metrics_json))

        // Audit Log - admin only
        .route("/api/audit-log", get(handlers::audit_log::get_audit_logs))
        .route("/api/audit-log/clear", delete(handlers::audit_log::clear_audit_log))
        .route("/api/audit-log/expiry", delete(handlers::audit_log::delete_old_audit_logs))
        .route("/api/audit-log/{id}", get(handlers::audit_log::get_audit_log))
        .route("/api/project/{project_id}/audit-log", get(handlers::audit_log::get_project_audit_logs))

        // Пользовательские API токены (User Tokens)
        .route("/api/user/tokens", get(user::get_api_tokens))
        .route("/api/user/tokens", post(user::create_api_token))
        .route("/api/user/tokens/{id}", delete(user::delete_api_token))

        // Все задачи (Global Tasks List) (B-BE-15) — registered above via handlers::get_all_tasks

        // Шаблоны - дополнительные endpoints (B-BE-17/18)
        // stop_all_tasks для /api/project/ регистрирован выше (line 64)
        .route("/api/project/{project_id}/templates/{id}/description", put(handlers::projects::templates::update_template_description))
        .route("/api/projects/{project_id}/templates/{id}/stop_all_tasks", post(handlers::projects::templates::stop_all_template_tasks))
        .route("/api/projects/{project_id}/templates/{id}/description", put(handlers::projects::templates::update_template_description))

        // Integration Matchers CRUD (B-BE-20)
        .route("/api/project/{project_id}/integrations/{integration_id}/matchers", get(project_integration::get_integration_matchers))
        .route("/api/project/{project_id}/integrations/{integration_id}/matchers", post(project_integration::add_integration_matcher))
        .route("/api/project/{project_id}/integrations/{integration_id}/matchers/{matcher_id}", put(project_integration::update_integration_matcher))
        .route("/api/project/{project_id}/integrations/{integration_id}/matchers/{matcher_id}", delete(project_integration::delete_integration_matcher))

        // Integration Extract Values CRUD (B-BE-21)
        .route("/api/project/{project_id}/integrations/{integration_id}/extractvalues", get(project_integration::get_integration_extract_values))
        .route("/api/project/{project_id}/integrations/{integration_id}/extractvalues", post(project_integration::add_integration_extract_value))
        .route("/api/project/{project_id}/integrations/{integration_id}/extractvalues/{value_id}", put(project_integration::update_integration_extract_value))
        .route("/api/project/{project_id}/integrations/{integration_id}/extractvalues/{value_id}", delete(project_integration::delete_integration_extract_value))
        // Aliases for Go-compat: /values = extractvalues
        .route("/api/project/{project_id}/integrations/{integration_id}/values", get(project_integration::get_integration_extract_values))
        .route("/api/project/{project_id}/integrations/{integration_id}/values", post(project_integration::add_integration_extract_value))
        .route("/api/project/{project_id}/integrations/{integration_id}/values/{value_id}", put(project_integration::update_integration_extract_value))
        .route("/api/project/{project_id}/integrations/{integration_id}/values/{value_id}", delete(project_integration::delete_integration_extract_value))

        // Расписание — toggle active
        .route("/api/project/{project_id}/schedules/{id}/active", put(schedules::toggle_schedule_active))
        .route("/api/projects/{project_id}/schedules/{id}/active", put(schedules::toggle_schedule_active))

        // Templates — дополнительные endpoints
        .route("/api/project/{project_id}/templates/{id}/schedules", get(templates::get_template_schedules))
        .route("/api/project/{project_id}/templates/{id}/tasks", get(templates::get_template_tasks))
        .route("/api/project/{project_id}/templates/{id}/tasks/last", get(templates::get_template_last_task))
        .route("/api/project/{project_id}/templates/{id}/stats", get(templates::get_template_stats))
        .route("/api/projects/{project_id}/templates/{id}/schedules", get(templates::get_template_schedules))
        .route("/api/projects/{project_id}/templates/{id}/tasks", get(templates::get_template_tasks))
        .route("/api/projects/{project_id}/templates/{id}/tasks/last", get(templates::get_template_last_task))
        .route("/api/projects/{project_id}/templates/{id}/stats", get(templates::get_template_stats))

        // Repository — branches (refs covered by refs.rs)
        .route("/api/project/{project_id}/repositories/{id}/branches", get(repository::get_repository_branches))
        .route("/api/projects/{project_id}/repositories/{id}/branches", get(repository::get_repository_branches))

        // Tasks — raw output + stages
        .route("/api/project/{project_id}/tasks/{id}/raw_output", get(tasks::get_task_raw_output))
        .route("/api/project/{project_id}/tasks/{id}/stages", get(tasks::get_task_stages))
        .route("/api/projects/{project_id}/tasks/{id}/raw_output", get(tasks::get_task_raw_output))
        .route("/api/projects/{project_id}/tasks/{id}/stages", get(tasks::get_task_stages))

        // ── MCP (Model Context Protocol) — embedded AI gateway ──────────────
        // POST /mcp  — JSON-RPC 2.0 endpoint (Claude Desktop/Code connects here)
        .route("/mcp", post(mcp::mcp_endpoint))
        // REST settings & tool catalog (for the Settings UI page)
        .route("/api/mcp/settings", get(mcp::get_mcp_settings))
        .route("/api/mcp/settings", put(mcp::update_mcp_settings))
        .route("/api/mcp/tools", get(mcp::get_mcp_tools))

        // Notification Policies
        .route("/api/project/{project_id}/notifications", get(handlers::notification::list_notification_policies))
        .route("/api/project/{project_id}/notifications", post(handlers::notification::create_notification_policy))
        .route("/api/project/{project_id}/notifications/{id}", get(handlers::notification::get_notification_policy))
        .route("/api/project/{project_id}/notifications/{id}", put(handlers::notification::update_notification_policy))
        .route("/api/project/{project_id}/notifications/{id}", delete(handlers::notification::delete_notification_policy))
        .route("/api/project/{project_id}/notifications/{id}/test", post(handlers::notification::test_notification_policy))

        // GitOps Drift Detection
        .route("/api/project/{project_id}/drift", get(handlers::drift::list_drift_configs))
        .route("/api/project/{project_id}/drift", post(handlers::drift::create_drift_config))
        .route("/api/project/{project_id}/drift/{id}", put(handlers::drift::update_drift_config))
        .route("/api/project/{project_id}/drift/{id}", delete(handlers::drift::delete_drift_config))
        .route("/api/project/{project_id}/drift/{id}/check", post(handlers::drift::trigger_drift_check))
        .route("/api/project/{project_id}/drift/{id}/results", get(handlers::drift::get_drift_results))

        // Custom Credential Types (global)
        .route("/api/credential-types", get(handlers::credential_type::list_credential_types))
        .route("/api/credential-types", post(handlers::credential_type::create_credential_type))
        .route("/api/credential-types/{id}", get(handlers::credential_type::get_credential_type))
        .route("/api/credential-types/{id}", put(handlers::credential_type::update_credential_type))
        .route("/api/credential-types/{id}", delete(handlers::credential_type::delete_credential_type))

        // Credential Instances (per-project)
        .route("/api/project/{project_id}/credentials", get(handlers::credential_type::list_credential_instances))
        .route("/api/project/{project_id}/credentials", post(handlers::credential_type::create_credential_instance))
        .route("/api/project/{project_id}/credentials/{id}", delete(handlers::credential_type::delete_credential_instance))

        // Snapshots & Rollback
        .route("/api/project/{project_id}/snapshots", get(handlers::snapshot::list_snapshots))
        .route("/api/project/{project_id}/snapshots", post(handlers::snapshot::create_snapshot))
        .route("/api/project/{project_id}/snapshots/{id}", delete(handlers::snapshot::delete_snapshot))
        .route("/api/project/{project_id}/snapshots/{id}/rollback", post(handlers::snapshot::rollback_snapshot))
        .route("/api/project/{project_id}/tasks/{task_id}/snapshot", post(handlers::snapshot::snapshot_from_task))

        // LDAP Group → Teams mapping (admin)
        .route("/api/admin/ldap/group-mappings", get(handlers::ldap_groups::list_ldap_group_mappings))
        .route("/api/admin/ldap/group-mappings", post(handlers::ldap_groups::create_ldap_group_mapping))
        .route("/api/admin/ldap/group-mappings/{id}", delete(handlers::ldap_groups::delete_ldap_group_mapping))

        // Terraform Cost Estimates (Infracost)
        .route("/api/project/{project_id}/costs", get(handlers::cost_estimate::list_cost_estimates))
        .route("/api/project/{project_id}/costs/summary", get(handlers::cost_estimate::cost_summary))
        .route("/api/project/{project_id}/tasks/{task_id}/cost", get(handlers::cost_estimate::get_task_cost))
        .route("/api/project/{project_id}/tasks/{task_id}/cost", post(handlers::cost_estimate::create_task_cost))
}

/// Создаёт маршруты для статических файлов
pub fn static_routes() -> Router<Arc<AppState>> {
    use axum::http::StatusCode;
    use axum::response::{Response, IntoResponse};
    use axum::middleware::{self, Next};
    
    // Путь к директории с frontend: SEMAPHORE_WEB_PATH или относительно Cargo.toml (rust/../web/public)
    let web_path = std::env::var("SEMAPHORE_WEB_PATH")
        .unwrap_or_else(|_| {
            let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let path = manifest_dir.join("..").join("web").join("public");
            // Канонический путь для корректной работы на Windows
            path.canonicalize()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| path.to_string_lossy().to_string())
        });

    // Проверяем существование директории
    let path = std::path::Path::new(&web_path);
    if !path.exists() || !path.is_dir() {
        tracing::warn!("Web path {} does not exist, static files will not be served", web_path);
        return Router::new();
    }
    tracing::info!("Serving static files from {}", web_path);

    // Middleware для проверки пути - API маршруты не обрабатываются
    async fn check_api_path(req: axum::http::Request<axum::body::Body>, next: Next) -> Result<Response, StatusCode> {
        // Если путь начинается с /api/, возвращаем 404 чтобы обработал API роутер
        if req.uri().path().starts_with("/api/") {
            return Err(StatusCode::NOT_FOUND);
        }
        Ok(next.run(req).await)
    }

    // ServeDir для раздачи статических файлов с fallback на index.html для SPA
    let serve_dir = ServeDir::new(&web_path)
        .not_found_service(ServeFile::new(format!("{web_path}/index.html")));

    Router::new()
        // В axum 0.8 используем fallback_service вместо nest_service
        .fallback_service(serve_dir)
        .layer(middleware::from_fn(check_api_path))
}
