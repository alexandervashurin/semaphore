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

    #[test]
    fn test_playbook_clone() {
        let playbook = Playbook {
            id: 1, project_id: 10, name: "clone.yml".to_string(),
            content: "---".to_string(), description: None,
            playbook_type: "ansible".to_string(), repository_id: None,
            created: Utc::now(), updated: Utc::now(),
        };
        let cloned = playbook.clone();
        assert_eq!(cloned.name, playbook.name);
        assert_eq!(cloned.playbook_type, playbook.playbook_type);
    }

    #[test]
    fn test_playbook_debug() {
        let playbook = Playbook {
            id: 1, project_id: 1, name: "debug.yml".to_string(),
            content: "---".to_string(), description: None,
            playbook_type: "ansible".to_string(), repository_id: None,
            created: Utc::now(), updated: Utc::now(),
        };
        let debug_str = format!("{:?}", playbook);
        assert!(debug_str.contains("Playbook"));
        assert!(debug_str.contains("debug.yml"));
    }

    #[test]
    fn test_playbook_create_clone() {
        let create = PlaybookCreate {
            name: "clone-create.yml".to_string(), content: "---".to_string(),
            description: None, playbook_type: "shell".to_string(), repository_id: None,
        };
        let cloned = create.clone();
        assert_eq!(cloned.name, create.name);
        assert_eq!(cloned.playbook_type, create.playbook_type);
    }

    #[test]
    fn test_playbook_update_clone() {
        let update = PlaybookUpdate {
            name: "clone-update.yml".to_string(), content: "---".to_string(),
            description: Some("Clone".to_string()), playbook_type: "ansible".to_string(),
        };
        let cloned = update.clone();
        assert_eq!(cloned.name, update.name);
        assert_eq!(cloned.description, update.description);
    }

    #[test]
    fn test_playbook_deserialization() {
        let json = r#"{"id":5,"project_id":20,"name":"deser.yml","content":"---","description":"Desc","playbook_type":"ansible","repository_id":null,"created":"2024-01-01T00:00:00Z","updated":"2024-01-01T00:00:00Z"}"#;
        let playbook: Playbook = serde_json::from_str(json).unwrap();
        assert_eq!(playbook.id, 5);
        assert_eq!(playbook.name, "deser.yml");
        assert_eq!(playbook.playbook_type, "ansible");
    }

    #[test]
    fn test_playbook_create_deserialization() {
        let json = r#"{"name":"create.yml","content":"---","description":null,"playbook_type":"terraform","repository_id":null}"#;
        let create: PlaybookCreate = serde_json::from_str(json).unwrap();
        assert_eq!(create.name, "create.yml");
        assert_eq!(create.playbook_type, "terraform");
    }

    #[test]
    fn test_playbook_empty_description() {
        let playbook = Playbook {
            id: 1, project_id: 1, name: "empty.yml".to_string(),
            content: "---".to_string(), description: Some("".to_string()),
            playbook_type: "ansible".to_string(), repository_id: None,
            created: Utc::now(), updated: Utc::now(),
        };
        let json = serde_json::to_string(&playbook).unwrap();
        assert!(json.contains("\"description\":\"\""));
    }

    #[test]
    fn test_playbook_all_types() {
        let types = ["ansible", "terraform", "shell"];
        for pt in types {
            let playbook = Playbook {
                id: 1, project_id: 1, name: "test.yml".to_string(),
                content: "---".to_string(), description: None,
                playbook_type: pt.to_string(), repository_id: None,
                created: Utc::now(), updated: Utc::now(),
            };
            let json = serde_json::to_string(&playbook).unwrap();
            assert!(json.contains(&format!("\"playbook_type\":\"{}\"", pt)));
        }
    }
}
