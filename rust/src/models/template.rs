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
}

impl<DB: Database> Type<DB> for TemplateType {
    fn type_info() -> DB::TypeInfo {
        String::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        String::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for TemplateType {
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = String::decode(value)?;
        Ok(match s.as_str() {
            "build" => TemplateType::Build,
            _ => TemplateType::Default,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for TemplateType
where
    DB: 'q,
{
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s: String = match self {
            TemplateType::Build => "build",
            TemplateType::Default => "default",
        }.to_string();
        Encode::encode(s, buf)
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

impl<DB: Database> Type<DB> for TemplateApp {
    fn type_info() -> DB::TypeInfo {
        String::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        String::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for TemplateApp {
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = String::decode(value)?;
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
{
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s: String = match self {
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
        Encode::encode(s, buf)
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
