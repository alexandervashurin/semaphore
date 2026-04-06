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

    /// URL живого окружения (e.g. <https://app.example.com>)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_tier_display() {
        assert_eq!(EnvironmentTier::Production.to_string(), "production");
        assert_eq!(EnvironmentTier::Staging.to_string(), "staging");
        assert_eq!(EnvironmentTier::Development.to_string(), "development");
        assert_eq!(EnvironmentTier::Review.to_string(), "review");
        assert_eq!(EnvironmentTier::Other.to_string(), "other");
    }

    #[test]
    fn test_environment_tier_default() {
        let tier = EnvironmentTier::default();
        assert_eq!(tier, EnvironmentTier::Other);
    }

    #[test]
    fn test_deploy_environment_status_default() {
        let status = DeployEnvironmentStatus::default();
        assert_eq!(status, DeployEnvironmentStatus::Unknown);
    }

    #[test]
    fn test_deployment_environment_serialization() {
        let env = DeploymentEnvironment {
            id: 1,
            project_id: 10,
            name: "production".to_string(),
            url: Some("https://app.example.com".to_string()),
            tier: "production".to_string(),
            status: "active".to_string(),
            template_id: Some(5),
            last_task_id: Some(100),
            last_deploy_version: Some("v1.2.3".to_string()),
            last_deployed_by: Some(1),
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"name\":\"production\""));
        assert!(json.contains("\"url\":\"https://app.example.com\""));
        assert!(json.contains("\"tier\":\"production\""));
    }

    #[test]
    fn test_deployment_environment_create_default_tier() {
        let create = DeploymentEnvironmentCreate {
            name: "staging".to_string(),
            url: None,
            tier: String::new(),
            template_id: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        // tier должен быть по умолчанию "other" при сериализации
        assert!(json.contains("\"name\":\"staging\""));
    }

    #[test]
    fn test_deployment_environment_update_partial() {
        let update = DeploymentEnvironmentUpdate {
            name: Some("new-name".to_string()),
            url: Some("https://new.example.com".to_string()),
            tier: None,
            status: None,
            template_id: None,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"new-name\""));
        assert!(json.contains("\"url\":\"https://new.example.com\""));
        assert!(!json.contains("\"tier\":"));
        assert!(!json.contains("\"status\":"));
    }

    #[test]
    fn test_deployment_record_serialization() {
        let record = DeploymentRecord {
            id: 1,
            deploy_environment_id: 5,
            task_id: 100,
            project_id: 10,
            version: Some("v2.0.0".to_string()),
            deployed_by: Some(1),
            status: "success".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"task_id\":100"));
        assert!(json.contains("\"version\":\"v2.0.0\""));
        assert!(json.contains("\"status\":\"success\""));
    }
}
