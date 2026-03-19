//! Projects API - Backup/Restore Handler
//!
//! Экспорт и импорт всех сущностей проекта

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::Project;
use crate::api::middleware::ErrorResponse;
use crate::db::store::{
    ProjectStore, TemplateManager, RepositoryManager, AccessKeyManager,
    InventoryManager, EnvironmentManager, ScheduleManager,
    IntegrationManager, ViewManager, SecretStorageManager,
};

/// Формат бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFormat {
    pub meta: Project,
    pub templates: Vec<serde_json::Value>,
    pub repositories: Vec<serde_json::Value>,
    pub keys: Vec<serde_json::Value>,
    pub views: Vec<serde_json::Value>,
    pub inventories: Vec<serde_json::Value>,
    pub environments: Vec<serde_json::Value>,
    pub integrations: Vec<serde_json::Value>,
    pub schedules: Vec<serde_json::Value>,
    pub secret_storages: Vec<serde_json::Value>,
    pub roles: Vec<serde_json::Value>,
}

/// Получает бэкап проекта
pub async fn get_backup(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<BackupFormat>, (StatusCode, Json<ErrorResponse>)> {
    macro_rules! try_load {
        ($expr:expr) => {
            $expr.await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string()))))?
        };
    }

    let project = try_load!(state.store.get_project(project_id));
    let templates = try_load!(state.store.get_templates(project_id));
    let repositories = try_load!(state.store.get_repositories(project_id));
    let keys = try_load!(state.store.get_access_keys(project_id));
    let views = try_load!(state.store.get_views(project_id));
    let inventories = try_load!(state.store.get_inventories(project_id));
    let environments = try_load!(state.store.get_environments(project_id));
    let integrations = try_load!(state.store.get_integrations(project_id));
    let schedules = try_load!(state.store.get_schedules(project_id));
    let secret_storages = try_load!(state.store.get_secret_storages(project_id));

    // Маскируем секреты в ключах
    let keys_masked: Vec<serde_json::Value> = keys.into_iter().map(|mut k| {
        crate::services::key_encryption::mask_key_secrets(&mut k);
        serde_json::to_value(k).unwrap_or(serde_json::Value::Null)
    }).collect();

    let backup = BackupFormat {
        meta: project,
        templates: templates.into_iter().map(|t| serde_json::to_value(t).unwrap_or_default()).collect(),
        repositories: repositories.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect(),
        keys: keys_masked,
        views: views.into_iter().map(|v| serde_json::to_value(v).unwrap_or_default()).collect(),
        inventories: inventories.into_iter().map(|i| serde_json::to_value(i).unwrap_or_default()).collect(),
        environments: environments.into_iter().map(|e| serde_json::to_value(e).unwrap_or_default()).collect(),
        integrations: integrations.into_iter().map(|i| serde_json::to_value(i).unwrap_or_default()).collect(),
        schedules: schedules.into_iter().map(|s| serde_json::to_value(s).unwrap_or_default()).collect(),
        secret_storages: secret_storages.into_iter().map(|s| serde_json::to_value(s).unwrap_or_default()).collect(),
        roles: vec![],
    };

    Ok(Json(backup))
}

/// Восстанавливает проект из бэкапа
pub async fn restore_backup(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<BackupFormat>,
) -> std::result::Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorResponse>)> {
    let mut n_views = 0usize;
    let mut n_keys = 0usize;
    let mut n_repos = 0usize;
    let mut n_envs = 0usize;
    let mut n_invs = 0usize;
    let mut n_tmpls = 0usize;
    let mut n_scheds = 0usize;
    let mut n_integs = 0usize;
    let mut errors: Vec<String> = vec![];

    for v in &payload.views {
        if let Ok(mut view) = serde_json::from_value::<crate::models::View>(v.clone()) {
            view.id = 0;
            view.project_id = project_id;
            match state.store.create_view(view).await {
                Ok(_) => n_views += 1,
                Err(e) => errors.push(format!("view: {}", e)),
            }
        }
    }

    for k in &payload.keys {
        if let Ok(mut key) = serde_json::from_value::<crate::models::AccessKey>(k.clone()) {
            key.id = 0;
            key.project_id = Some(project_id);
            if key.ssh_key.as_deref() == Some("**SECRET**") { key.ssh_key = None; }
            if key.login_password_password.as_deref() == Some("**SECRET**") { key.login_password_password = None; }
            if key.access_key_secret_key.as_deref() == Some("**SECRET**") { key.access_key_secret_key = None; }
            match state.store.create_access_key(key).await {
                Ok(_) => n_keys += 1,
                Err(e) => errors.push(format!("key: {}", e)),
            }
        }
    }

    for r in &payload.repositories {
        if let Ok(mut repo) = serde_json::from_value::<crate::models::Repository>(r.clone()) {
            repo.id = 0;
            repo.project_id = project_id;
            match state.store.create_repository(repo).await {
                Ok(_) => n_repos += 1,
                Err(e) => errors.push(format!("repository: {}", e)),
            }
        }
    }

    for e in &payload.environments {
        if let Ok(mut env) = serde_json::from_value::<crate::models::Environment>(e.clone()) {
            env.id = 0;
            env.project_id = project_id;
            match state.store.create_environment(env).await {
                Ok(_) => n_envs += 1,
                Err(e) => errors.push(format!("environment: {}", e)),
            }
        }
    }

    for i in &payload.inventories {
        if let Ok(mut inv) = serde_json::from_value::<crate::models::Inventory>(i.clone()) {
            inv.id = 0;
            inv.project_id = project_id;
            match state.store.create_inventory(inv).await {
                Ok(_) => n_invs += 1,
                Err(e) => errors.push(format!("inventory: {}", e)),
            }
        }
    }

    for t in &payload.templates {
        if let Ok(mut tmpl) = serde_json::from_value::<crate::models::Template>(t.clone()) {
            tmpl.id = 0;
            tmpl.project_id = project_id;
            match state.store.create_template(tmpl).await {
                Ok(_) => n_tmpls += 1,
                Err(e) => errors.push(format!("template: {}", e)),
            }
        }
    }

    for s in &payload.schedules {
        if let Ok(mut sched) = serde_json::from_value::<crate::models::Schedule>(s.clone()) {
            sched.id = 0;
            sched.project_id = project_id;
            match state.store.create_schedule(sched).await {
                Ok(_) => n_scheds += 1,
                Err(e) => errors.push(format!("schedule: {}", e)),
            }
        }
    }

    for i in &payload.integrations {
        if let Ok(mut integ) = serde_json::from_value::<crate::models::Integration>(i.clone()) {
            integ.id = 0;
            integ.project_id = project_id;
            match state.store.create_integration(integ).await {
                Ok(_) => n_integs += 1,
                Err(e) => errors.push(format!("integration: {}", e)),
            }
        }
    }

    let result = serde_json::json!({
        "views": n_views, "keys": n_keys, "repositories": n_repos,
        "environments": n_envs, "inventories": n_invs, "templates": n_tmpls,
        "schedules": n_scheds, "integrations": n_integs,
        "errors": errors
    });

    Ok((StatusCode::OK, Json(result)))
}

/// Проверяет бэкап без применения
pub async fn verify_backup(
    State(_state): State<Arc<AppState>>,
    Path(_project_id): Path<i32>,
    Json(payload): Json<BackupFormat>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let result = serde_json::json!({
        "valid": true,
        "project": payload.meta.name,
        "templates": payload.templates.len(),
        "repositories": payload.repositories.len(),
        "keys": payload.keys.len(),
        "inventories": payload.inventories.len(),
        "environments": payload.environments.len(),
        "integrations": payload.integrations.len(),
        "schedules": payload.schedules.len(),
    });
    Ok(Json(result))
}
