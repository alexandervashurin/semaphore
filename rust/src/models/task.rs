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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_serialization() {
        let task = Task {
            id: 1,
            template_id: 10,
            project_id: 5,
            status: TaskStatus::Waiting,
            playbook: Some("deploy.yml".to_string()),
            environment: None,
            secret: None,
            arguments: Some("--limit=web".to_string()),
            git_branch: Some("main".to_string()),
            user_id: Some(1),
            integration_id: None,
            schedule_id: None,
            created: Utc::now(),
            start: None,
            end: None,
            message: Some("Deploy to production".to_string()),
            commit_hash: Some("abc123".to_string()),
            commit_message: Some("Update config".to_string()),
            build_task_id: None,
            version: Some("1.0.0".to_string()),
            inventory_id: Some(2),
            repository_id: Some(3),
            environment_id: Some(4),
            params: Some(serde_json::json!({"debug": true})),
        };
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("\"template_id\":10"));
        assert!(json.contains("\"playbook\":\"deploy.yml\""));
        assert!(json.contains("\"git_branch\":\"main\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
    }

    #[test]
    fn test_task_skip_serializing_null_fields() {
        let task = Task {
            id: 1,
            template_id: 1,
            project_id: 1,
            status: TaskStatus::Waiting,
            playbook: None,
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: None,
            created: Utc::now(),
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
        };
        let json = serde_json::to_string(&task).unwrap();
        assert!(!json.contains("\"playbook\":"));
        assert!(!json.contains("\"environment\":"));
        assert!(!json.contains("\"secret\":"));
        assert!(!json.contains("\"arguments\":"));
        assert!(!json.contains("\"git_branch\":"));
    }

    #[test]
    fn test_task_clone() {
        let task = Task::default();
        let cloned = task.clone();
        assert_eq!(cloned.id, task.id);
        assert_eq!(cloned.template_id, task.template_id);
        assert_eq!(cloned.status, task.status);
    }

    #[test]
    fn test_task_debug_impl() {
        let task = Task::default();
        let debug_str = format!("{:?}", task);
        assert!(debug_str.contains("Task"));
        assert!(debug_str.contains("id"));
    }

    #[test]
    fn test_task_get_url() {
        let task = Task {
            id: 42,
            template_id: 10,
            project_id: 7,
            status: TaskStatus::Waiting,
            playbook: None,
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: None,
            created: Utc::now(),
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
        };
        assert_eq!(task.get_url(), "/project/7/tasks/42");
    }

    #[test]
    fn test_task_with_tpl_serialization() {
        let task = Task::default();
        let with_tpl = TaskWithTpl {
            task,
            tpl_playbook: Some("site.yml".to_string()),
            tpl_type: Some(TemplateType::Ansible),
            tpl_app: Some(TemplateApp::Ansible),
            user_name: Some("admin".to_string()),
            build_task: None,
        };
        let json = serde_json::to_string(&with_tpl).unwrap();
        assert!(json.contains("\"tpl_playbook\":\"site.yml\""));
        assert!(json.contains("\"user_name\":\"admin\""));
    }

    #[test]
    fn test_task_output_serialization() {
        let output = TaskOutput {
            id: 1,
            task_id: 100,
            project_id: 5,
            time: Utc::now(),
            output: "Task started".to_string(),
            stage_id: Some(1),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"task_id\":100"));
        assert!(json.contains("\"output\":\"Task started\""));
    }

    #[test]
    fn test_task_stage_type_display() {
        assert_eq!(TaskStageType::Init.to_string(), "init");
        assert_eq!(TaskStageType::TerraformPlan.to_string(), "terraform_plan");
        assert_eq!(TaskStageType::Running.to_string(), "running");
        assert_eq!(TaskStageType::PrintResult.to_string(), "print_result");
    }

    #[test]
    fn test_task_stage_type_serialization() {
        assert_eq!(serde_json::to_string(&TaskStageType::Init).unwrap(), "\"init\"");
        assert_eq!(serde_json::to_string(&TaskStageType::TerraformPlan).unwrap(), "\"terraform_plan\"");
        assert_eq!(serde_json::to_string(&TaskStageType::Running).unwrap(), "\"running\"");
        assert_eq!(serde_json::to_string(&TaskStageType::PrintResult).unwrap(), "\"print_result\"");
    }

    #[test]
    fn test_task_stage_serialization() {
        let stage = TaskStage {
            id: 1,
            task_id: 100,
            project_id: 5,
            start: Some(Utc::now()),
            end: None,
            r#type: TaskStageType::Running,
        };
        let json = serde_json::to_string(&stage).unwrap();
        assert!(json.contains("\"task_id\":100"));
        assert!(json.contains("\"type\":\"running\""));
    }

    #[test]
    fn test_ansible_task_params_serialization() {
        let params = AnsibleTaskParams {
            debug: true,
            debug_level: 2,
            dry_run: true,
            diff: false,
            limit: vec!["web".to_string(), "db".to_string()],
            tags: vec!["deploy".to_string()],
            skip_tags: vec!["test".to_string()],
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"debug\":true"));
        assert!(json.contains("\"dry_run\":true"));
        assert!(json.contains("\"limit\":[\"web\",\"db\"]"));
    }

    #[test]
    fn test_ansible_task_params_default() {
        let params = AnsibleTaskParams::default();
        assert!(!params.debug);
        assert_eq!(params.debug_level, 0);
        assert!(!params.dry_run);
        assert!(!params.diff);
        assert!(params.limit.is_empty());
        assert!(params.tags.is_empty());
        assert!(params.skip_tags.is_empty());
    }

    #[test]
    fn test_terraform_task_params_serialization() {
        let params = TerraformTaskParams {
            plan: true,
            destroy: false,
            auto_approve: true,
            upgrade: true,
            reconfigure: false,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"plan\":true"));
        assert!(json.contains("\"auto_approve\":true"));
        assert!(json.contains("\"upgrade\":true"));
    }

    #[test]
    fn test_terraform_task_params_default() {
        let params = TerraformTaskParams::default();
        assert!(!params.plan);
        assert!(!params.destroy);
        assert!(!params.auto_approve);
        assert!(!params.upgrade);
        assert!(!params.reconfigure);
    }

    #[test]
    fn test_default_task_params_serialization() {
        let params = DefaultTaskParams::default();
        let json = serde_json::to_string(&params).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_task_deserialization() {
        let json = r#"{"id":10,"template_id":5,"project_id":2,"status":"waiting","created":"2024-01-01T00:00:00Z"}"#;
        let task: Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.id, 10);
        assert_eq!(task.template_id, 5);
        assert_eq!(task.project_id, 2);
    }

    #[test]
    fn test_task_stage_with_result_serialization() {
        let stage = TaskStage {
            id: 1,
            task_id: 100,
            project_id: 5,
            start: Some(Utc::now()),
            end: Some(Utc::now()),
            r#type: TaskStageType::Init,
        };
        let stage_with_result = TaskStageWithResult {
            stage,
            start_output: Some("Starting...".to_string()),
            end_output: Some("Finished.".to_string()),
        };
        let json = serde_json::to_string(&stage_with_result).unwrap();
        assert!(json.contains("\"start_output\":\"Starting...\""));
        assert!(json.contains("\"end_output\":\"Finished.\""));
    }

    #[test]
    fn test_task_status_clone() {
        let status = TaskStatus::Waiting;
        let cloned = status.clone();
        assert_eq!(cloned, status);
    }

    #[test]
    fn test_task_stage_type_clone() {
        let stage_type = TaskStageType::TerraformPlan;
        let cloned = stage_type.clone();
        assert_eq!(cloned, stage_type);
    }
}
