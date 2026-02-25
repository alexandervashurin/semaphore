//! Модель шаблона (Template)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Тип шаблона
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateType {
    Default,
    Build,
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
    Pulumi,
    Default,
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
