//! GraphQL типы — полный набор для публичного API

use async_graphql::SimpleObject;

// ── Core domain ──────────────────────────────────────────────────────────────

/// Пользователь
#[derive(SimpleObject, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub name: String,
    pub email: String,
    pub admin: bool,
}

/// Проект
#[derive(SimpleObject, Debug, Clone)]
pub struct Project {
    pub id: i32,
    pub name: String,
}

/// Шаблон (Template)
#[derive(SimpleObject, Debug, Clone)]
pub struct Template {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub playbook: String,
}

/// Задача (Task)
#[derive(SimpleObject, Debug, Clone)]
pub struct Task {
    pub id: i32,
    pub template_id: i32,
    pub project_id: i32,
    pub status: String,
    pub created: String,
}

/// Инвентарь
#[derive(SimpleObject, Debug, Clone)]
pub struct Inventory {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub r#type: String,
}

/// Репозиторий
#[derive(SimpleObject, Debug, Clone)]
pub struct Repository {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub git_url: String,
    pub git_branch: String,
}

/// Переменные окружения
#[derive(SimpleObject, Debug, Clone)]
pub struct Environment {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
}

/// Расписание (Schedule)
#[derive(SimpleObject, Debug, Clone)]
pub struct Schedule {
    pub id: i32,
    pub project_id: i32,
    pub template_id: i32,
    pub name: String,
    pub cron_format: String,
    pub active: bool,
}

/// Раннер (Runner)
#[derive(SimpleObject, Debug, Clone)]
pub struct Runner {
    pub id: i32,
    pub name: String,
    pub active: bool,
    pub webhook: String,
}

/// Событие аудит-лога
#[derive(SimpleObject, Debug, Clone)]
pub struct AuditEvent {
    pub id: i32,
    pub project_id: Option<i32>,
    pub object_type: String,
    pub object_id: i32,
    pub description: String,
    pub created: String,
}

// ── Kubernetes ────────────────────────────────────────────────────────────────

/// Kubernetes namespace
#[derive(SimpleObject, Debug, Clone)]
pub struct KubernetesNamespace {
    pub name: String,
    pub status: String,
    pub labels: Vec<String>,
}

/// Kubernetes node
#[derive(SimpleObject, Debug, Clone)]
pub struct KubernetesNode {
    pub name: String,
    pub status: String,
    pub roles: Vec<String>,
    pub version: String,
    pub os_image: String,
}

/// Kubernetes cluster info
#[derive(SimpleObject, Debug, Clone)]
pub struct KubernetesClusterInfo {
    pub server_url: String,
    pub version: String,
    pub namespace: String,
}

// ── Subscription event types ──────────────────────────────────────────────────

/// Строка лога задачи — для subscription taskOutput
#[derive(SimpleObject, Debug, Clone)]
pub struct TaskOutputLine {
    pub task_id: i32,
    pub line: String,
    pub timestamp: String,
    /// Уровень: "info" | "warning" | "error" | "debug"
    pub level: String,
}

/// Изменение статуса задачи — для subscription taskStatus
#[derive(SimpleObject, Debug, Clone)]
pub struct TaskStatusEvent {
    pub task_id: i32,
    pub project_id: i32,
    pub status: String,
    pub updated_at: String,
}
