//! Модель задачи (Task)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use crate::models::template::{TemplateType, TemplateApp};
use crate::services::task_logger::TaskStatus;

/// Задача - экземпляр выполнения шаблона
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID шаблона
    pub template_id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Статус задачи
    pub status: TaskStatus,

    /// Playbook (переопределение)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playbook: Option<String>,

    /// Окружение (переопределение)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,

    /// Секреты (не сериализуется)
    #[serde(skip_serializing, skip_deserializing)]
    pub secret: Option<String>,

    /// Аргументы
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,

    /// Ветка Git
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,

    /// ID пользователя
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,

    /// ID интеграции
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integration_id: Option<i32>,

    /// ID расписания
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_id: Option<i32>,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Время начала
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime<Utc>>,

    /// Время завершения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime<Utc>>,

    /// Сообщение
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Хэш коммита
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,

    /// Сообщение коммита
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_message: Option<String>,

    /// ID задачи сборки
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_task_id: Option<i32>,

    /// Версия
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// ID инвентаря
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_id: Option<i32>,

    /// Параметры задачи
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, serde_json::Value>>,
}

/// Задача с дополнительными полями шаблона
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskWithTpl {
    #[serde(flatten)]
    pub task: Task,

    /// Playbook шаблона
    pub tpl_playbook: String,

    /// Псевдоним шаблона
    pub tpl_alias: String,

    /// Тип шаблона
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpl_type: Option<TemplateType>,

    /// Приложение шаблона
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpl_app: Option<TemplateApp>,

    /// Имя пользователя
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,

    /// Задача сборки
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_task: Option<Box<Task>>,
}

/// Вывод задачи (лог)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskOutput {
    pub id: i32,
    pub task_id: i32,
    pub time: DateTime<Utc>,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<i32>,
}

/// Тип этапа задачи
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStageType {
    Init,
    TerraformPlan,
    Running,
    PrintResult,
}

/// Этап задачи
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskStage {
    pub id: i32,
    pub task_id: i32,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub r#type: TaskStageType,
}

/// Параметры задачи для Ansible
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnsibleTaskParams {
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub debug_level: i32,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub diff: bool,
    #[serde(default)]
    pub limit: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub skip_tags: Vec<String>,
}

/// Параметры задачи для Terraform/OpenTofu
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerraformTaskParams {
    #[serde(default)]
    pub plan: bool,
    #[serde(default)]
    pub destroy: bool,
    #[serde(default)]
    pub auto_approve: bool,
    #[serde(default)]
    pub upgrade: bool,
    #[serde(default)]
    pub reconfigure: bool,
}

/// Параметры задачи по умолчанию
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultTaskParams {}
