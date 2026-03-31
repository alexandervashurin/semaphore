//! Organization Model - Модель организации
//!
//! Поддержка Multi-Tenancy через организации

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Организация (Multi-Tenancy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// ID организации
    pub id: i32,

    /// Название организации
    pub name: String,

    /// Уникальный slug для URL
    pub slug: String,

    /// Описание организации
    pub description: Option<String>,

    /// Настройки организации (JSON)
    pub settings: Option<serde_json::Value>,

    /// Квота: максимальное количество проектов
    pub quota_max_projects: Option<i32>,

    /// Квота: максимальное количество пользователей
    pub quota_max_users: Option<i32>,

    /// Квота: максимальное количество задач в месяц
    pub quota_max_tasks_per_month: Option<i32>,

    /// Включена ли организация
    pub active: bool,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Дата обновления
    pub updated: Option<DateTime<Utc>>,
}

/// Создание организации
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OrganizationCreate {
    /// Название организации
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Slug (опционально, генерируется автоматически)
    pub slug: Option<String>,

    /// Описание
    pub description: Option<String>,

    /// Настройки
    pub settings: Option<serde_json::Value>,

    /// Квота проектов
    pub quota_max_projects: Option<i32>,

    /// Квота пользователей
    pub quota_max_users: Option<i32>,

    /// Квота задач
    pub quota_max_tasks_per_month: Option<i32>,
}

/// Обновление организации
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrganizationUpdate {
    /// Название организации
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Описание
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Настройки
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,

    /// Квота проектов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_max_projects: Option<i32>,

    /// Квота пользователей
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_max_users: Option<i32>,

    /// Квота задач
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_max_tasks_per_month: Option<i32>,

    /// Активность
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Связь пользователя с организацией
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationUser {
    /// ID записи
    pub id: i32,

    /// ID организации
    pub org_id: i32,

    /// ID пользователя
    pub user_id: i32,

    /// Роль в организации (owner, admin, member)
    pub role: String,

    /// Дата создания
    pub created: DateTime<Utc>,
}

/// Создание связи пользователя с организацией
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationUserCreate {
    /// ID организации
    pub org_id: i32,

    /// ID пользователя
    pub user_id: i32,

    /// Роль
    pub role: String,
}

impl Default for Organization {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            slug: String::new(),
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
            active: true,
            created: Utc::now(),
            updated: None,
        }
    }
}
