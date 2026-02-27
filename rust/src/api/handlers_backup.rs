//! Обработчики HTTP-запросов

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::*;
use crate::models::template::{TemplateType, TemplateApp};
use crate::models::inventory::InventoryType;
use crate::models::access_key::AccessKeyType;
use crate::models::user::UserTotp;
use crate::db::store::RetrieveQueryParams;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;

/// Health check
pub async fn health() -> &'static str {
    "OK"
}

// ==================== Аутентификация ====================

/// Вход в систему
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    use crate::api::auth::{AuthService, verify_password};
    use crate::services::totp::verify_totp_code;

    // Находим пользователя
    let user = state.store.get_user_by_login_or_email(&payload.username, &payload.username)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Неверный логин или пароль")
                    .with_code("INVALID_CREDENTIALS")),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Ошибка сервера")),
            ),
        })?;

    // Проверяем пароль
    if !verify_password(&payload.password, &user.password) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Неверный логин или пароль")
                .with_code("INVALID_CREDENTIALS")),
        ));
    }

    // Проверяем TOTP, если настроен
    if let Some(ref totp) = user.totp {
        let totp_code = payload.totp_code
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Требуется TOTP код")
                    .with_code("TOTP_REQUIRED")),
            ))?;

        if !verify_totp_code(&totp.url, &totp_code) {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Неверный TOTP код")
                    .with_code("INVALID_TOTP")),
            ));
        }
    }

    // Генерируем токен
    let auth_service = AuthService::new();
    let token_info = auth_service.generate_token(&user)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка генерации токена: {}", e))
                .with_code("TOKEN_GENERATION_ERROR")),
        ))?;

    Ok(Json(LoginResponse {
        token: token_info.token,
        token_type: token_info.token_type,
        expires_in: token_info.expires_in,
        totp_required: None,
    }))
}

/// Выход из системы
pub async fn logout(
    State(_state): State<Arc<AppState>>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Реализовать выход (добавление токена в чёрный список)
    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub totp_code: Option<String>,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp_required: Option<bool>,
}

#[derive(serde::Serialize)]
pub struct TotpSetupResponse {
    pub secret: String,
    pub url: String,
    pub recovery_code: String,
}

// ==================== Пользователи ====================

/// Получить список пользователей
pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let users = state.store.get_users(RetrieveQueryParams::default())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(users))
}

/// Получить пользователя по ID
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user = state.store.get_user(user_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    Ok(Json(user))
}

/// Обновить пользователя
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Json(payload): Json<UserUpdatePayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut user = state.store.get_user(user_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    if let Some(username) = payload.username {
        user.username = username;
    }
    if let Some(name) = payload.name {
        user.name = name;
    }
    if let Some(email) = payload.email {
        user.email = email;
    }
    
    state.store.update_user(user)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::OK)
}

/// Удалить пользователя
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.store.delete_user(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize)]
pub struct UserUpdatePayload {
    pub username: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
}

// ==================== Проекты ====================

/// Получить список проектов
pub async fn get_projects(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Project>>, (StatusCode, String)> {
    let projects = state.store.get_projects(None)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(projects))
}

/// Создать проект
pub async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProjectCreatePayload>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let project = Project::new(payload.name);
    
    let created = state.store.create_project(project)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(created))
}

/// Получить проект по ID
pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let project = state.store.get_project(project_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    Ok(Json(project))
}

/// Обновить проект
pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<ProjectUpdatePayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut project = state.store.get_project(project_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    if let Some(name) = payload.name {
        project.name = name;
    }
    if let Some(alert) = payload.alert {
        project.alert = Some(alert);
    }
    
    state.store.update_project(project)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::OK)
}

/// Удалить проект
pub async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.store.delete_project(project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize)]
pub struct ProjectCreatePayload {
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct ProjectUpdatePayload {
    pub name: Option<String>,
    pub alert: Option<bool>,
}

// ==================== Шаблоны ====================

pub async fn get_templates(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Template>>, (StatusCode, String)> {
    let templates = state.store.get_templates(project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(templates))
}

pub async fn create_template(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<TemplateCreatePayload>,
) -> Result<Json<Template>, (StatusCode, String)> {
    let template = Template {
        id: 0,
        project_id,
        name: payload.name,
        playbook: payload.playbook,
        description: String::new(),
        inventory_id: payload.inventory_id,
        repository_id: payload.repository_id,
        environment_id: payload.environment_id,
        r#type: TemplateType::Default,
        app: TemplateApp::Ansible,
        git_branch: "main".to_string(),
        deleted: false,
        created: chrono::Utc::now(),
    };
    
    let created = state.store.create_template(template)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(created))
}

pub async fn get_template(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> Result<Json<Template>, (StatusCode, String)> {
    let template = state.store.get_template(project_id, template_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    Ok(Json(template))
}

pub async fn update_template(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
    Json(payload): Json<TemplateUpdatePayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut template = state.store.get_template(project_id, template_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    if let Some(name) = payload.name {
        template.name = name;
    }
    if let Some(playbook) = payload.playbook {
        template.playbook = playbook;
    }
    if let Some(description) = payload.description {
        template.description = description;
    }
    
    state.store.update_template(template)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::OK)
}

pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.store.delete_template(project_id, template_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize)]
pub struct TemplateCreatePayload {
    pub name: String,
    pub playbook: String,
    pub inventory_id: i32,
    pub repository_id: i32,
    pub environment_id: i32,
}

#[derive(serde::Deserialize)]
pub struct TemplateUpdatePayload {
    pub name: Option<String>,
    pub playbook: Option<String>,
    pub description: Option<String>,
}

// ==================== Задачи ====================

pub async fn get_tasks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<TaskWithTpl>>, (StatusCode, String)> {
    let tasks = state.store.get_tasks(project_id, None)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(tasks))
}

pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<TaskCreatePayload>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let task = Task {
        id: 0,
        template_id: payload.template_id,
        project_id,
        status: crate::services::task_logger::TaskStatus::Waiting,
        playbook: payload.playbook,
        environment: payload.environment,
        secret: None,
        arguments: payload.arguments,
        git_branch: payload.git_branch,
        user_id: payload.user_id,
        integration_id: None,
        schedule_id: None,
        created: chrono::Utc::now(),
        start: None,
        end: None,
        message: None,
        commit_hash: None,
        commit_message: None,
        build_task_id: payload.build_task_id,
        version: None,
        inventory_id: payload.inventory_id,
        params: None,
    };
    
    let created = state.store.create_task(task)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(created))
}

pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let task = state.store.get_task(project_id, task_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    Ok(Json(task))
}

pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.store.delete_task(project_id, task_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize)]
pub struct TaskCreatePayload {
    pub template_id: i32,
    pub playbook: Option<String>,
    pub environment: Option<String>,
    pub arguments: Option<String>,
    pub git_branch: Option<String>,
    pub user_id: Option<i32>,
    pub build_task_id: Option<i32>,
    pub inventory_id: Option<i32>,
}

// ==================== Инвентари ====================

pub async fn get_inventories(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Inventory>>, (StatusCode, String)> {
    let inventories = state.store.get_inventories(project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(inventories))
}

pub async fn create_inventory(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<InventoryCreatePayload>,
) -> Result<Json<Inventory>, (StatusCode, String)> {
    let inventory = Inventory::new(
        project_id,
        payload.name,
        payload.inventory_type,
    );
    
    let created = state.store.create_inventory(inventory)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(created))
}

pub async fn get_inventory(
    State(state): State<Arc<AppState>>,
    Path((project_id, inventory_id)): Path<(i32, i32)>,
) -> Result<Json<Inventory>, (StatusCode, String)> {
    let inventory = state.store.get_inventory(project_id, inventory_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    Ok(Json(inventory))
}

pub async fn update_inventory(
    State(state): State<Arc<AppState>>,
    Path((project_id, inventory_id)): Path<(i32, i32)>,
    Json(payload): Json<InventoryUpdatePayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut inventory = state.store.get_inventory(project_id, inventory_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    if let Some(name) = payload.name {
        inventory.name = name;
    }
    if let Some(data) = payload.inventory_data {
        inventory.inventory_data = data;
    }
    
    state.store.update_inventory(inventory)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::OK)
}

pub async fn delete_inventory(
    State(state): State<Arc<AppState>>,
    Path((project_id, inventory_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.store.delete_inventory(project_id, inventory_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize)]
pub struct InventoryCreatePayload {
    pub name: String,
    #[serde(rename = "inventory")]
    pub inventory_type: InventoryType,
}

#[derive(serde::Deserialize)]
pub struct InventoryUpdatePayload {
    pub name: Option<String>,
    pub inventory_data: Option<String>,
}

// ==================== Репозитории ====================

pub async fn get_repositories(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Repository>>, (StatusCode, String)> {
    let repositories = state.store.get_repositories(project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(repositories))
}

pub async fn create_repository(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<RepositoryCreatePayload>,
) -> Result<Json<Repository>, (StatusCode, String)> {
    let repository = Repository::new(
        project_id,
        payload.name,
        payload.git_url,
    );
    
    let created = state.store.create_repository(repository)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(created))
}

pub async fn get_repository(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
) -> Result<Json<Repository>, (StatusCode, String)> {
    let repository = state.store.get_repository(project_id, repository_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    Ok(Json(repository))
}

pub async fn update_repository(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
    Json(payload): Json<RepositoryUpdatePayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut repository = state.store.get_repository(project_id, repository_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    if let Some(name) = payload.name {
        repository.name = name;
    }
    if let Some(git_url) = payload.git_url {
        repository.git_url = git_url;
    }
    
    state.store.update_repository(repository)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::OK)
}

pub async fn delete_repository(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.store.delete_repository(project_id, repository_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize)]
pub struct RepositoryCreatePayload {
    pub name: String,
    pub git_url: String,
}

#[derive(serde::Deserialize)]
pub struct RepositoryUpdatePayload {
    pub name: Option<String>,
    pub git_url: Option<String>,
}

// ==================== Окружения ====================

pub async fn get_environments(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Environment>>, (StatusCode, String)> {
    let environments = state.store.get_environments(project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(environments))
}

pub async fn create_environment(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<EnvironmentCreatePayload>,
) -> Result<Json<Environment>, (StatusCode, String)> {
    let environment = Environment::new(
        project_id,
        payload.name,
        payload.json,
    );
    
    let created = state.store.create_environment(environment)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(created))
}

pub async fn get_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, environment_id)): Path<(i32, i32)>,
) -> Result<Json<Environment>, (StatusCode, String)> {
    let environment = state.store.get_environment(project_id, environment_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    Ok(Json(environment))
}

pub async fn update_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, environment_id)): Path<(i32, i32)>,
    Json(payload): Json<EnvironmentUpdatePayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut environment = state.store.get_environment(project_id, environment_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    if let Some(name) = payload.name {
        environment.name = name;
    }
    if let Some(json) = payload.json {
        environment.json = json;
    }
    
    state.store.update_environment(environment)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::OK)
}

pub async fn delete_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, environment_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.store.delete_environment(project_id, environment_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize)]
pub struct EnvironmentCreatePayload {
    pub name: String,
    pub json: String,
}

#[derive(serde::Deserialize)]
pub struct EnvironmentUpdatePayload {
    pub name: Option<String>,
    pub json: Option<String>,
}

// ==================== Ключи доступа ====================

pub async fn get_access_keys(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<AccessKey>>, (StatusCode, String)> {
    let keys = state.store.get_access_keys(project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(keys))
}

pub async fn create_access_key(
    State(state): State<Arc<AppState>>,
    Path(_project_id): Path<i32>,
    Json(payload): Json<AccessKeyCreatePayload>,
) -> Result<Json<AccessKey>, (StatusCode, String)> {
    let key = AccessKey::new(
        payload.name,
        payload.key_type,
    );

    let created = state.store.create_access_key(key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(created))
}

pub async fn get_access_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
) -> Result<Json<AccessKey>, (StatusCode, String)> {
    let key = state.store.get_access_key(project_id, key_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    Ok(Json(key))
}

pub async fn update_access_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
    Json(payload): Json<AccessKeyUpdatePayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut key = state.store.get_access_key(project_id, key_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    
    if let Some(name) = payload.name {
        key.name = name;
    }
    
    state.store.update_access_key(key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::OK)
}

pub async fn delete_access_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.store.delete_access_key(project_id, key_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== TOTP (2FA) ====================

/// Начать настройку TOTP
pub async fn start_totp_setup(
    State(state): State<Arc<AppState>>,
    auth_user: crate::api::extractors::AuthUser,
) -> Result<Json<TotpSetupResponse>, (StatusCode, Json<ErrorResponse>)> {
    use crate::services::totp::generate_totp_secret;

    // Получаем пользователя
    let user = state.store.get_user(auth_user.user_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка: {}", e))),
        ))?;

    // Если TOTP уже настроен, возвращаем ошибку
    if user.totp.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("TOTP уже настроен")
                .with_code("TOTP_ALREADY_ENABLED")),
        ));
    }

    // Генерируем секрет
    let totp_secret = generate_totp_secret(&user, "Semaphore UI")
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка генерации секрета: {}", e))),
        ))?;

    Ok(Json(TotpSetupResponse {
        secret: totp_secret.secret,
        url: totp_secret.url,
        recovery_code: totp_secret.recovery_code,
    }))
}

/// Подтвердить настройку TOTP
pub async fn confirm_totp_setup(
    State(state): State<Arc<AppState>>,
    auth_user: crate::api::extractors::AuthUser,
    Json(payload): Json<TotpConfirmPayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    use crate::services::totp::{generate_totp_secret, verify_totp_code};
    use chrono::Utc;

    // Получаем пользователя
    let user = state.store.get_user(auth_user.user_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка: {}", e))),
        ))?;

    // Если TOTP уже настроен, возвращаем ошибку
    if user.totp.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("TOTP уже настроен")
                .with_code("TOTP_ALREADY_ENABLED")),
        ));
    }

    // Генерируем секрет (временно)
    let totp_secret = generate_totp_secret(&user, "Semaphore UI")
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка генерации секрета: {}", e))),
        ))?;

    // Проверяем код
    if !verify_totp_code(&totp_secret.secret, &payload.code) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Неверный TOTP код")
                .with_code("INVALID_TOTP")),
        ));
    }

    // Сохраняем TOTP в БД
    let _totp = UserTotp {
        id: 0,
        created: Utc::now(),
        user_id: user.id,
        url: totp_secret.url,
        recovery_hash: totp_secret.recovery_hash,
        recovery_code: None,
    };

    // TODO: Реализовать сохранение TOTP в store
    // state.store.create_totp(totp).await?;

    Ok(StatusCode::OK)
}

/// Отключить TOTP
pub async fn disable_totp(
    State(state): State<Arc<AppState>>,
    auth_user: crate::api::extractors::AuthUser,
    Json(payload): Json<TotpDisablePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    use crate::services::totp::verify_recovery_code;

    // Получаем пользователя
    let user = state.store.get_user(auth_user.user_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Ошибка: {}", e))),
        ))?;

    // Проверяем, что TOTP настроен
    let totp = user.totp
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("TOTP не настроен")
                .with_code("TOTP_NOT_ENABLED")),
        ))?;

    // Проверяем код восстановления
    if !verify_recovery_code(&payload.recovery_code, &totp.recovery_hash) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Неверный код восстановления")
                .with_code("INVALID_RECOVERY_CODE")),
        ));
    }

    // TODO: Реализовать удаление TOTP из store
    // state.store.delete_totp(user.id).await?;

    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct AccessKeyCreatePayload {
    pub name: String,
    #[serde(rename = "type")]
    pub key_type: AccessKeyType,
}

#[derive(serde::Deserialize)]
pub struct AccessKeyUpdatePayload {
    pub name: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct TotpConfirmPayload {
    pub code: String,
}

#[derive(serde::Deserialize)]
pub struct TotpDisablePayload {
    pub recovery_code: String,
}
