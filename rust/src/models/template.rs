//! Модель шаблона (Template)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, decode::Decode, encode::Encode, database::Database};

/// Тип шаблона
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateType {
    Default,
    Build,
    Deploy,
    Task,
    Ansible,
    Terraform,
    Shell,
}

impl std::fmt::Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateType::Default => write!(f, "default"),
            TemplateType::Build => write!(f, "build"),
            TemplateType::Deploy => write!(f, "deploy"),
            TemplateType::Task => write!(f, "task"),
            TemplateType::Ansible => write!(f, "ansible"),
            TemplateType::Terraform => write!(f, "terraform"),
            TemplateType::Shell => write!(f, "shell"),
        }
    }
}

impl std::str::FromStr for TemplateType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(TemplateType::Default),
            "build" => Ok(TemplateType::Build),
            "deploy" => Ok(TemplateType::Deploy),
            "task" => Ok(TemplateType::Task),
            "ansible" => Ok(TemplateType::Ansible),
            "terraform" => Ok(TemplateType::Terraform),
            "shell" => Ok(TemplateType::Shell),
            _ => Ok(TemplateType::Default),
        }
    }
}

impl<DB: Database> Type<DB> for TemplateType
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for TemplateType
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "build" => TemplateType::Build,
            "deploy" => TemplateType::Deploy,
            "task" => TemplateType::Task,
            "ansible" => TemplateType::Ansible,
            "terraform" => TemplateType::Terraform,
            "shell" => TemplateType::Shell,
            _ => TemplateType::Default,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for TemplateType
where
    DB: 'q,
    String: Encode<'q, DB>,
{
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            TemplateType::Build => "build",
            TemplateType::Deploy => "deploy",
            TemplateType::Task => "task",
            TemplateType::Ansible => "ansible",
            TemplateType::Terraform => "terraform",
            TemplateType::Shell => "shell",
            TemplateType::Default => "default",
        }.to_string();
        <String as Encode<'q, DB>>::encode(s, buf)
    }
}

/// Приложение, используемое шаблоном
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateApp {
    Ansible,
    Terraform,
    Tofu,
    Terragrunt,
    Bash,
    PowerShell,
    Python,
    Pulumi,
    Default,
}

impl std::fmt::Display for TemplateApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateApp::Ansible => write!(f, "ansible"),
            TemplateApp::Terraform => write!(f, "terraform"),
            TemplateApp::Tofu => write!(f, "tofu"),
            TemplateApp::Terragrunt => write!(f, "terragrunt"),
            TemplateApp::Bash => write!(f, "bash"),
            TemplateApp::PowerShell => write!(f, "powershell"),
            TemplateApp::Python => write!(f, "python"),
            TemplateApp::Pulumi => write!(f, "pulumi"),
            TemplateApp::Default => write!(f, "default"),
        }
    }
}

impl std::str::FromStr for TemplateApp {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "ansible" => TemplateApp::Ansible,
            "terraform" => TemplateApp::Terraform,
            "tofu" => TemplateApp::Tofu,
            "terragrunt" => TemplateApp::Terragrunt,
            "bash" => TemplateApp::Bash,
            "powershell" => TemplateApp::PowerShell,
            "python" => TemplateApp::Python,
            "pulumi" => TemplateApp::Pulumi,
            _ => TemplateApp::Default,
        })
    }
}

impl<DB: Database> Type<DB> for TemplateApp
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for TemplateApp
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "ansible" => TemplateApp::Ansible,
            "terraform" => TemplateApp::Terraform,
            "tofu" => TemplateApp::Tofu,
            "terragrunt" => TemplateApp::Terragrunt,
            "bash" => TemplateApp::Bash,
            "powershell" => TemplateApp::PowerShell,
            "python" => TemplateApp::Python,
            "pulumi" => TemplateApp::Pulumi,
            _ => TemplateApp::Default,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for TemplateApp
where
    DB: 'q,
    String: Encode<'q, DB>,
{
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            TemplateApp::Ansible => "ansible",
            TemplateApp::Terraform => "terraform",
            TemplateApp::Tofu => "tofu",
            TemplateApp::Terragrunt => "terragrunt",
            TemplateApp::Bash => "bash",
            TemplateApp::PowerShell => "powershell",
            TemplateApp::Python => "python",
            TemplateApp::Pulumi => "pulumi",
            TemplateApp::Default => "default",
        }.to_string();
        <String as Encode<'q, DB>>::encode(s, buf)
    }
}

/// Шаблон задачи
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Template {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Название шаблона
    pub name: String,

    /// Псевдоним шаблона
    pub playbook: String,

    /// Описание
    pub description: String,

    /// ID инвентаря
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_id: Option<i32>,

    /// ID репозитория
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<i32>,

    /// ID окружения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<i32>,

    /// Тип шаблона
    pub r#type: TemplateType,

    /// Приложение
    pub app: TemplateApp,

    /// Ветка Git по умолчанию
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Аргументы командной строки
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,

    /// ID ключа vault
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_key_id: Option<i32>,

    /// ID View (группа шаблонов)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_id: Option<i32>,

    /// ID шаблона сборки (для type=deploy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_template_id: Option<i32>,

    /// Автозапуск при успешном build
    #[serde(default)]
    pub autorun: bool,

    /// Разрешить переопределение аргументов при запуске
    #[serde(default)]
    pub allow_override_args_in_task: bool,

    /// Разрешить переопределение ветки при запуске
    #[serde(default)]
    pub allow_override_branch_in_task: bool,

    /// Разрешить смену инвентаря при запуске
    #[serde(default)]
    pub allow_inventory_in_task: bool,

    /// Разрешить параллельный запуск
    #[serde(default)]
    pub allow_parallel_tasks: bool,

    /// Подавлять уведомления при успехе
    #[serde(default)]
    pub suppress_success_alerts: bool,
}

/// Шаблон с правами доступа
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateWithPerms {
    #[serde(flatten)]
    pub template: Template,
    pub user_id: i32,
    pub role: String,
}

/// Разрешение для шаблона
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateRolePerm {
    pub id: i32,
    pub project_id: i32,
    pub template_id: i32,
    pub role_id: i32,
    pub role_slug: String,
}

/// Фильтр для шаблонов
#[derive(Debug, Clone, Default)]
pub struct TemplateFilter {
    pub project_id: Option<i32>,
    pub r#type: Option<TemplateType>,
    pub app: Option<TemplateApp>,
    pub view_id: Option<i32>,
}

impl Template {
    /// Создаёт новый шаблон с значениями по умолчанию
    pub fn default_template(project_id: i32, name: String, playbook: String) -> Self {
        Self {
            id: 0,
            project_id,
            name,
            playbook,
            description: String::new(),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            r#type: TemplateType::Default,
            app: TemplateApp::Default,
            git_branch: None,
            created: Utc::now(),
            arguments: None,
            vault_key_id: None,
            view_id: None,
            build_template_id: None,
            autorun: false,
            allow_override_args_in_task: false,
            allow_override_branch_in_task: false,
            allow_inventory_in_task: false,
            allow_parallel_tasks: false,
            suppress_success_alerts: false,
        }
    }
}

impl Default for Template {
    fn default() -> Self {
        Self::default_template(0, String::new(), String::new())
    }
}
