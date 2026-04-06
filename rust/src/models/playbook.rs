//! Модель Playbook - YAML файл с задачами Ansible/Terraform

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Playbook - YAML файл с автоматизацией
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Playbook {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Название плейбука
    pub name: String,

    /// YAML содержимое
    pub content: String,

    /// Описание
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Тип (ansible, terraform, shell)
    pub playbook_type: String,

    /// ID репозитория (опционально, если связан с git)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<i32>,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Дата обновления
    pub updated: DateTime<Utc>,
}

/// Playbook для создания
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookCreate {
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    pub playbook_type: String,
    pub repository_id: Option<i32>,
}

/// Playbook для обновления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookUpdate {
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    pub playbook_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playbook_serialization() {
        let playbook = Playbook {
            id: 1,
            project_id: 10,
            name: "deploy.yml".to_string(),
            content: "---\n- hosts: all\n  tasks: []".to_string(),
            description: Some("Main deploy playbook".to_string()),
            playbook_type: "ansible".to_string(),
            repository_id: Some(5),
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&playbook).unwrap();
        assert!(json.contains("\"name\":\"deploy.yml\""));
        assert!(json.contains("\"playbook_type\":\"ansible\""));
        assert!(json.contains("\"description\":\"Main deploy playbook\""));
    }

    #[test]
    fn test_playbook_skip_nulls() {
        let playbook = Playbook {
            id: 1,
            project_id: 10,
            name: "simple.yml".to_string(),
            content: "---".to_string(),
            description: None,
            playbook_type: "ansible".to_string(),
            repository_id: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&playbook).unwrap();
        assert!(!json.contains("\"description\":"));
        assert!(!json.contains("\"repository_id\":"));
    }

    #[test]
    fn test_playbook_create_serialization() {
        let create = PlaybookCreate {
            name: "new-playbook.yml".to_string(),
            content: "---\nhosts: localhost".to_string(),
            description: None,
            playbook_type: "shell".to_string(),
            repository_id: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"new-playbook.yml\""));
        assert!(json.contains("\"playbook_type\":\"shell\""));
    }

    #[test]
    fn test_playbook_update_serialization() {
        let update = PlaybookUpdate {
            name: "updated.yml".to_string(),
            content: "---\nupdated content".to_string(),
            description: Some("Updated description".to_string()),
            playbook_type: "terraform".to_string(),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"updated.yml\""));
        assert!(json.contains("\"playbook_type\":\"terraform\""));
    }
}
