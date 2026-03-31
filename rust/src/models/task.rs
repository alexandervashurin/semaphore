//! Модель задачи (Task)

use crate::models::template::{TemplateApp, TemplateType};
use crate::services::task_logger::TaskStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Задача - экземпляр выполнения шаблона
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// ID репозитория
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<i32>,

    /// ID окружения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<i32>,

    /// Параметры задачи
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl Task {
    /// Получает URL задачи
    pub fn get_url(&self) -> String {
        format!("/project/{}/tasks/{}", self.project_id, self.id)
    }
}

#[cfg(test)]
impl Default for Task {
    fn default() -> Self {
        Self {
            id: 0,
            template_id: 0,
            project_id: 0,
            status: TaskStatus::Waiting,
            created: Utc::now(),
            playbook: None,
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: None,
            start: None,
            end: None,
            message: None,
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            params: None,
        }
    }
}

/// Задача с дополнительными полями шаблона
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskWithTpl {
    #[serde(flatten)]
    pub task: Task,

    /// Playbook шаблона
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpl_playbook: Option<String>,

    /// Тип шаблона
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpl_type: Option<TemplateType>,

    /// Приложение шаблона
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpl_app: Option<TemplateApp>,

    /// Имя пользователя
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,

    /// Задача сборки (игнорируется для SQLx)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_task: Option<Box<Task>>,
}

/// Вывод задачи (лог)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskOutput {
    pub id: i32,
    pub task_id: i32,
    pub project_id: i32,
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

impl std::fmt::Display for TaskStageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStageType::Init => write!(f, "init"),
            TaskStageType::TerraformPlan => write!(f, "terraform_plan"),
            TaskStageType::Running => write!(f, "running"),
            TaskStageType::PrintResult => write!(f, "print_result"),
        }
    }
}

/// Этап задачи
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskStage {
    pub id: i32,
    pub task_id: i32,
    pub project_id: i32,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub r#type: TaskStageType,
}

/// Этап задачи с результатом
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskStageWithResult {
    #[serde(flatten)]
    pub stage: TaskStage,
    pub start_output: Option<String>,
    pub end_output: Option<String>,
}

/// Результат этапа задачи
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskStageResult {
    pub id: i32,
    pub stage_id: i32,
    pub task_id: i32,
    pub project_id: i32,
    pub result: String,
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

// ============================================================================
// SQLx реализации для TaskStatus
// ============================================================================

impl<DB: sqlx::database::Database> sqlx::Type<DB> for TaskStatus
where
    String: sqlx::Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as sqlx::Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as sqlx::Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: sqlx::database::Database> sqlx::Decode<'r, DB> for TaskStatus
where
    String: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<'r, DB>>::decode(value)?;
        Ok(s.parse().unwrap_or(TaskStatus::Waiting))
    }
}

impl<'q, DB: sqlx::database::Database> sqlx::Encode<'q, DB> for TaskStatus
where
    DB: 'q,
    String: sqlx::Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, DB>>::encode(s, buf)
    }
}

// ============================================================================
// SQLx реализации для TaskStageType
// ============================================================================

impl<DB: sqlx::database::Database> sqlx::Type<DB> for TaskStageType
where
    String: sqlx::Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as sqlx::Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as sqlx::Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: sqlx::database::Database> sqlx::Decode<'r, DB> for TaskStageType
where
    String: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "init" => TaskStageType::Init,
            "terraform_plan" => TaskStageType::TerraformPlan,
            "running" => TaskStageType::Running,
            "print_result" => TaskStageType::PrintResult,
            _ => TaskStageType::Init,
        })
    }
}

impl<'q, DB: sqlx::database::Database> sqlx::Encode<'q, DB> for TaskStageType
where
    DB: 'q,
    String: sqlx::Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            TaskStageType::Init => "init",
            TaskStageType::TerraformPlan => "terraform_plan",
            TaskStageType::Running => "running",
            TaskStageType::PrintResult => "print_result",
        }
        .to_string();
        <String as sqlx::Encode<'q, DB>>::encode(s, buf)
    }
}
