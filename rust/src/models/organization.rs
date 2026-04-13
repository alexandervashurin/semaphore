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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_default() {
        let org = Organization::default();
        assert_eq!(org.id, 0);
        assert!(org.name.is_empty());
        assert!(org.slug.is_empty());
        assert!(org.active);
        assert!(org.quota_max_projects.is_none());
    }

    #[test]
    fn test_organization_serialization() {
        let org = Organization {
            id: 1,
            name: "Acme Corp".to_string(),
            slug: "acme".to_string(),
            description: Some("Test company".to_string()),
            settings: None,
            quota_max_projects: Some(10),
            quota_max_users: Some(50),
            quota_max_tasks_per_month: Some(1000),
            active: true,
            created: Utc::now(),
            updated: None,
        };
        let json = serde_json::to_string(&org).unwrap();
        assert!(json.contains("\"name\":\"Acme Corp\""));
        assert!(json.contains("\"slug\":\"acme\""));
        assert!(json.contains("\"quota_max_projects\":10"));
    }

    #[test]
    fn test_organization_create_serialization() {
        let create = OrganizationCreate {
            name: "New Org".to_string(),
            slug: Some("new-org".to_string()),
            description: Some("New organization".to_string()),
            settings: None,
            quota_max_projects: Some(5),
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"New Org\""));
        assert!(json.contains("\"slug\":\"new-org\""));
    }

    #[test]
    fn test_organization_update_skip_nulls() {
        let update = OrganizationUpdate {
            name: Some("Updated Name".to_string()),
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
            active: Some(false),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"Updated Name\""));
        assert!(json.contains("\"active\":false"));
        assert!(!json.contains("\"description\":"));
        assert!(!json.contains("\"settings\":"));
    }

    #[test]
    fn test_organization_user_serialization() {
        let user = OrganizationUser {
            id: 1,
            org_id: 10,
            user_id: 5,
            role: "admin".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"org_id\":10"));
        assert!(json.contains("\"role\":\"admin\""));
    }

    #[test]
    fn test_organization_user_create_serialization() {
        let create = OrganizationUserCreate {
            org_id: 10,
            user_id: 5,
            role: "member".to_string(),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"org_id\":10"));
        assert!(json.contains("\"role\":\"member\""));
    }

    #[test]
    fn test_organization_clone() {
        let org = Organization {
            id: 1,
            name: "Clone Org".to_string(),
            slug: "clone-org".to_string(),
            description: Some("Clone Test".to_string()),
            settings: None,
            quota_max_projects: Some(10),
            quota_max_users: Some(50),
            quota_max_tasks_per_month: Some(1000),
            active: true,
            created: Utc::now(),
            updated: None,
        };
        let cloned = org.clone();
        assert_eq!(cloned.name, org.name);
        assert_eq!(cloned.slug, org.slug);
    }

    #[test]
    fn test_organization_user_clone() {
        let user = OrganizationUser {
            id: 1,
            org_id: 10,
            user_id: 5,
            role: "owner".to_string(),
            created: Utc::now(),
        };
        let cloned = user.clone();
        assert_eq!(cloned.role, user.role);
        assert_eq!(cloned.org_id, user.org_id);
    }

    #[test]
    fn test_organization_update_default() {
        let update = OrganizationUpdate::default();
        assert!(update.name.is_none());
        assert!(update.active.is_none());
        assert!(update.quota_max_projects.is_none());
    }

    #[test]
    fn test_organization_create_minimal() {
        let create = OrganizationCreate {
            name: "Minimal Org".to_string(),
            slug: None,
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"Minimal Org\""));
        assert!(json.contains("\"slug\":null"));
    }

    #[test]
    fn test_organization_unicode_name_and_description() {
        let org = Organization {
            id: 1,
            name: "Организация".to_string(),
            slug: "org".to_string(),
            description: Some("Тестовая организация".to_string()),
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
            active: true,
            created: Utc::now(),
            updated: None,
        };
        let json = serde_json::to_string(&org).unwrap();
        let restored: Organization = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "Организация");
        assert_eq!(restored.description, Some("Тестовая организация".to_string()));
    }

    #[test]
    fn test_organization_clone_independence() {
        let mut org = Organization::default();
        org.name = "Original".to_string();
        let cloned = org.clone();
        org.name = "Modified".to_string();
        assert_eq!(cloned.name, "Original");
    }

    #[test]
    fn test_organization_settings_json() {
        let mut org = Organization::default();
        org.settings = Some(serde_json::json!({"theme": "dark", "lang": "ru"}));
        let json = serde_json::to_string(&org).unwrap();
        assert!(json.contains("\"theme\":\"dark\""));
    }

    #[test]
    fn test_organization_user_roles() {
        for role in &["owner", "admin", "member"] {
            let user = OrganizationUser {
                id: 1, org_id: 1, user_id: 1, role: role.to_string(),
                created: Utc::now(),
            };
            let json = serde_json::to_string(&user).unwrap();
            assert!(json.contains(role));
        }
    }

    #[test]
    fn test_organization_update_all_fields() {
        let update = OrganizationUpdate {
            name: Some("New Name".to_string()),
            description: Some("New Desc".to_string()),
            settings: Some(serde_json::json!({})),
            quota_max_projects: Some(100),
            quota_max_users: Some(200),
            quota_max_tasks_per_month: Some(5000),
            active: Some(true),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"New Name\""));
        assert!(json.contains("\"quota_max_projects\":100"));
    }
}
