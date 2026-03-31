//! Kubernetes Runbook Integration
//!
//! Запуск задач Velum из Kubernetes объектов (Pod, Deployment, etc.)
//! для диагностики и восстановления

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::db::store::{TaskManager, TemplateManager};
use crate::error::{Error, Result};
use crate::models::{Task, Template};
use crate::services::task_logger::TaskStatus;

// ============================================================================
// Runbook Types
// ============================================================================

/// Запрос на запуск Runbook задачи из Kubernetes объекта
#[derive(Debug, Deserialize)]
pub struct RunbookRequest {
    /// ID шаблона задачи для запуска
    pub template_id: i32,
    
    /// Kubernetes объект (контекст)
    pub kubernetes_context: KubernetesContext,
    
    /// Дополнительные параметры задачи
    #[serde(default)]
    pub task_params: RunbookTaskParams,
    
    /// Сообщение для задачи
    #[serde(default)]
    pub message: Option<String>,
}

/// Контекст Kubernetes объекта
#[derive(Debug, Deserialize, Clone)]
pub struct KubernetesContext {
    /// Тип ресурса (Pod, Deployment, etc.)
    pub kind: String,
    
    /// Имя ресурса
    pub name: String,
    
    /// Namespace ресурса
    pub namespace: String,
    
    /// Cluster name (для multi-cluster)
    #[serde(default)]
    pub cluster: Option<String>,
    
    /// UID ресурса
    #[serde(default)]
    pub uid: Option<String>,
    
    /// Labels ресурса
    #[serde(default)]
    pub labels: Option<std::collections::HashMap<String, String>>,
}

/// Параметры для запуска задачи
#[derive(Debug, Deserialize, Default)]
pub struct RunbookTaskParams {
    /// Override arguments для ansible-playbook
    #[serde(default)]
    pub arguments: Option<String>,
    
    /// Git branch
    #[serde(default)]
    pub git_branch: Option<String>,
    
    /// Environment ID
    #[serde(default)]
    pub environment_id: Option<i32>,
    
    /// Inventory ID
    #[serde(default)]
    pub inventory_id: Option<i32>,
    
    /// Limit hosts
    #[serde(default)]
    pub limit: Option<String>,
    
    /// Tags
    #[serde(default)]
    pub tags: Option<String>,
    
    /// Skip tags
    #[serde(default)]
    pub skip_tags: Option<String>,
    
    /// Debug mode
    #[serde(default)]
    pub debug: bool,
    
    /// Dry run
    #[serde(default)]
    pub dry_run: bool,
    
    /// Diff mode
    #[serde(default)]
    pub diff: bool,
}

/// Ответ на запуск Runbook
#[derive(Debug, Serialize)]
pub struct RunbookResponse {
    /// ID созданной задачи
    pub task_id: i32,
    
    /// Статус задачи
    pub status: String,
    
    /// Сообщение
    pub message: String,
    
    /// Ссылка на задачу
    pub task_url: String,
}

/// Шаблон Runbook для Kubernetes объекта
#[derive(Debug, Serialize, Deserialize)]
pub struct KubernetesRunbookTemplate {
    /// ID шаблона
    pub id: i32,
    
    /// Название
    pub name: String,
    
    /// Описание
    pub description: String,
    
    /// Тип ресурса для которого применим
    pub resource_kinds: Vec<String>,
    
    /// Категория
    pub category: RunbookCategory,
    
    /// Автоматически заполняемые параметры
    pub auto_params: AutoRunbookParams,
}

/// Категория Runbook
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunbookCategory {
    Diagnostic,    // Диагностика
    Remediation,   // Восстановление
    Maintenance,   // Обслуживание
    Scaling,       // Масштабирование
    Backup,        // Бэкап
    Other,         // Другое
}

/// Автоматические параметры Runbook
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AutoRunbookParams {
    /// Подстановка имени ресурса в аргументы
    #[serde(default)]
    pub inject_resource_name: bool,
    
    /// Подстановка namespace в аргументы
    #[serde(default)]
    pub inject_namespace: bool,
    
    /// Подстановка labels как переменные
    #[serde(default)]
    pub inject_labels: bool,
    
    /// Шаблон аргументов
    #[serde(default)]
    pub arguments_template: Option<String>,
}

// ============================================================================
// API Handlers
// ============================================================================

/// Получить доступные Runbook шаблоны для Kubernetes объекта
pub async fn get_available_runbooks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Query(query): Query<RunbookResourceQuery>,
) -> Result<Json<Vec<KubernetesRunbookTemplate>>> {
    // Получаем все шаблоны проекта
    let templates = state.store.get_templates(project_id).await
        .map_err(|e| Error::Other(format!("Failed to get templates: {}", e)))?;
    
    // Фильтруем шаблоны с меткой "runbook"
    let runbooks = templates
        .into_iter()
        .filter(|tpl| {
            // Проверяем наличие метки runbook или специального префикса в названии
            tpl.name.contains("[Runbook]") || 
            tpl.name.contains("runbook") ||
            tpl.name.contains("diagnose") ||
            tpl.name.contains("remediate")
        })
        .map(|tpl| {
            // Определяем категорию по названию
            let category = if tpl.name.contains("diagnose") || tpl.name.contains("check") {
                RunbookCategory::Diagnostic
            } else if tpl.name.contains("fix") || tpl.name.contains("remediate") || tpl.name.contains("restore") {
                RunbookCategory::Remediation
            } else if tpl.name.contains("scale") {
                RunbookCategory::Scaling
            } else if tpl.name.contains("backup") {
                RunbookCategory::Backup
            } else {
                RunbookCategory::Other
            };
            
            // Определяем типы ресурсов по названию или описанию
            let resource_kinds = vec![
                "Pod".to_string(),
                "Deployment".to_string(),
                "StatefulSet".to_string(),
            ];
            
            KubernetesRunbookTemplate {
                id: tpl.id,
                name: tpl.name,
                description: tpl.playbook,
                resource_kinds,
                category,
                auto_params: AutoRunbookParams {
                    inject_resource_name: true,
                    inject_namespace: true,
                    inject_labels: false,
                    arguments_template: None,
                },
            }
        })
        .collect();
    
    Ok(Json(runbooks))
}

/// Query параметры для get_available_runbooks
#[derive(Debug, Deserialize)]
pub struct RunbookResourceQuery {
    pub kind: String,
    pub namespace: Option<String>,
}

/// Запустить Runbook задачу для Kubernetes объекта
pub async fn execute_runbook(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<RunbookRequest>,
) -> Result<Json<RunbookResponse>> {
    // Проверяем существование шаблона
    let template = state.store.get_template(project_id, payload.template_id).await
        .map_err(|e| match e {
            Error::NotFound(_) => Error::NotFound("Template not found".to_string()),
            _ => Error::Other(format!("Failed to get template: {}", e)),
        })?;
    
    // Формируем аргументы с учётом Kubernetes контекста
    let arguments = build_runbook_arguments(&payload, &template.playbook);
    
    // Создаём задачу
    let task = Task {
        id: 0,
        template_id: payload.template_id,
        project_id,
        status: TaskStatus::Waiting,
        playbook: Some(template.playbook.clone()),
        environment: payload.task_params.environment_id.map(|id| id.to_string()),
        secret: None,
        arguments,
        git_branch: payload.task_params.git_branch.or(template.git_branch),
        user_id: None, // TODO: получить из сессии
        integration_id: None,
        schedule_id: None,
        created: chrono::Utc::now(),
        start: None,
        end: None,
        message: payload.message.or_else(|| Some(format!(
            "Runbook для {} {} в namespace {}",
            payload.kubernetes_context.kind,
            payload.kubernetes_context.name,
            payload.kubernetes_context.namespace
        ))),
        commit_hash: None,
        commit_message: None,
        build_task_id: None,
        version: None,
        inventory_id: payload.task_params.inventory_id,
        repository_id: None,
        environment_id: payload.task_params.environment_id,
        params: None, // TODO: добавить параметры
    };
    
    // Сохраняем задачу
    let created_task = state.store.create_task(task).await
        .map_err(|e| Error::Other(format!("Failed to create task: {}", e)))?;
    
    // Запускаем задачу (отправляем в очередь)
    // TODO: интеграция с task runner service
    
    Ok(Json(RunbookResponse {
        task_id: created_task.id,
        status: "waiting".to_string(),
        message: "Задача создана и будет выполнена".to_string(),
        task_url: format!("/api/project/{}/tasks/{}", project_id, created_task.id),
    }))
}

/// Построить аргументы для задачи на основе Kubernetes контекста
fn build_runbook_arguments(
    payload: &RunbookRequest,
    playbook: &str,
) -> Option<String> {
    let ctx = &payload.kubernetes_context;
    let params = &payload.task_params;
    
    // Базовые аргументы из template
    let mut args = params.arguments.clone().unwrap_or_default();
    
    // Авто-подстановка имени ресурса
    if payload.task_params.arguments.is_none() {
        // Добавляем переменные для ansible
        args.push_str(&format!(" -e target_resource={}", ctx.name));
        args.push_str(&format!(" -e target_namespace={}", ctx.namespace));
        args.push_str(&format!(" -e target_kind={}", ctx.kind));
        
        if let Some(cluster) = &ctx.cluster {
            args.push_str(&format!(" -e target_cluster={}", cluster));
        }
        
        // Добавляем labels как переменные
        if let Some(labels) = &ctx.labels {
            for (key, value) in labels {
                args.push_str(&format!(" -e label_{}={}", key, value));
            }
        }
    }
    
    if args.is_empty() {
        None
    } else {
        Some(args)
    }
}

/// Получить статус выполнения Runbook задачи
pub async fn get_runbook_status(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> Result<Json<RunbookResponse>> {
    let task = state.store.get_task(project_id, task_id).await
        .map_err(|e| match e {
            Error::NotFound(_) => Error::NotFound("Task not found".to_string()),
            _ => Error::Other(format!("Failed to get task: {}", e)),
        })?;
    
    let status_str = match task.status {
        TaskStatus::Waiting => "waiting",
        TaskStatus::Running => "running",
        TaskStatus::Success => "success",
        TaskStatus::Error => "error",
        TaskStatus::Stopped => "stopped",
        _ => "unknown",
    }.to_string();
    
    Ok(Json(RunbookResponse {
        task_id: task.id,
        status: status_str,
        message: task.message.unwrap_or_default(),
        task_url: format!("/api/project/{}/tasks/{}", project_id, task.id),
    }))
}
