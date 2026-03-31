//! Deployment Environment — реестр окружений деплоя (GitLab Environments)
//!
//! Отслеживает production/staging/dev окружения: кто, когда и что задеплоил.
//! **Отличается** от `Environment` (Ansible env vars) — это deployment tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Tier окружения (уровень)
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum EnvironmentTier {
    Production,
    Staging,
    Development,
    Review,
    #[default]
    Other,
}

impl std::fmt::Display for EnvironmentTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Production => write!(f, "production"),
            Self::Staging => write!(f, "staging"),
            Self::Development => write!(f, "development"),
            Self::Review => write!(f, "review"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Статус окружения
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum DeployEnvironmentStatus {
    Active,
    Stopped,
    #[default]
    Unknown,
}

/// Deployment Environment — запись об окружении (production, staging, dev, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeploymentEnvironment {
    pub id: i32,
    pub project_id: i32,

    /// Имя окружения (уникально в проекте)
    pub name: String,

    /// URL живого окружения (e.g. https://app.example.com)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Уровень (production/staging/development/review/other)
    pub tier: String,

    /// Статус (active/stopped/unknown)
    pub status: String,

    /// Шаблон, который деплоит в это окружение
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<i32>,

    /// ID последней задачи (деплоя)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_task_id: Option<i32>,

    /// Версия/тег последнего деплоя
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_deploy_version: Option<String>,

    /// Кто задеплоил последний раз (user_id)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_deployed_by: Option<i32>,

    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// Payload для создания окружения деплоя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentEnvironmentCreate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default = "default_tier")]
    pub tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<i32>,
}

fn default_tier() -> String {
    "other".to_string()
}

/// Payload для обновления окружения деплоя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentEnvironmentUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<i32>,
}

/// История деплоев для одного окружения
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeploymentRecord {
    pub id: i32,
    pub deploy_environment_id: i32,
    pub task_id: i32,
    pub project_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployed_by: Option<i32>,
    pub status: String,
    pub created: DateTime<Utc>,
}
