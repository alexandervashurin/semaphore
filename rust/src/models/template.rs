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

impl<DB: Database> Type<DB> for TemplateType {
    fn type_info() -> DB::TypeInfo {
        <&str as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <&str as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for TemplateType
where
    &'r str: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as Decode<'r, DB>>::decode(value)?;
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
    for<'a> &'a str: Encode<'q, DB>,
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
        };
        <&str as Encode<'q, DB>>::encode(&s, buf)
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

impl<DB: Database> Type<DB> for TemplateApp {
    fn type_info() -> DB::TypeInfo {
        <&str as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <&str as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for TemplateApp
where
    &'r str: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as Decode<'r, DB>>::decode(value)?;
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
    for<'a> &'a str: Encode<'q, DB>,
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
        };
        <&str as Encode<'q, DB>>::encode(&s, buf)
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
    pub inventory_id: i32,

    /// ID репозитория
    pub repository_id: i32,

    /// ID окружения
    pub environment_id: i32,

    /// Тип шаблона
    pub r#type: TemplateType,

    /// Приложение
    pub app: TemplateApp,

    /// Ветка Git по умолчанию
    pub git_branch: String,

    /// Флаг удаления
    #[serde(skip_serializing)]
    pub deleted: bool,

    /// Дата создания
    pub created: DateTime<Utc>,
    
    /// Аргументы командной строки
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    
    /// Тип шаблона (для совместимости)
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_type: Option<TemplateType>,
    
    /// Начальная версия
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_version: Option<String>,
    
    /// Версия сборки
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_version: Option<String>,
    
    /// Переменные опроса
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub survey_vars: Option<String>,
    
    /// Хранилища секретов
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaults: Option<String>,
    
    /// Количество задач
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<i32>,
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
    pub deleted: Option<bool>,
}
