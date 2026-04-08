//! Task Snapshot Model
//!
//! Снапшот успешного запуска шаблона — зафиксированные параметры деплоя.
//! Позволяет "откатиться" к предыдущей версии одним кликом.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Снапшот успешного запуска
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskSnapshot {
    pub id: i32,
    pub project_id: i32,
    pub template_id: i32,
    /// ID исходной задачи
    pub task_id: i32,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub arguments: Option<String>,
    pub inventory_id: Option<i32>,
    pub environment_id: Option<i32>,
    pub message: Option<String>,
    /// Пользовательская метка (необязательно)
    pub label: Option<String>,
    pub created_at: String,
    /// Название шаблона (joined)
    #[sqlx(default)]
    pub template_name: String,
}

/// Создание нового снапшота (обычно автоматически после успешной задачи)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSnapshotCreate {
    pub template_id: i32,
    pub task_id: i32,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub arguments: Option<String>,
    pub inventory_id: Option<i32>,
    pub environment_id: Option<i32>,
    pub message: Option<String>,
    pub label: Option<String>,
}

/// Запрос на откат (создание новой задачи из снапшота)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRequest {
    /// Пользовательское сообщение для задачи отката
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_snapshot_serialization() {
        let snapshot = TaskSnapshot {
            id: 1,
            project_id: 10,
            template_id: 5,
            task_id: 100,
            git_branch: Some("main".to_string()),
            git_commit: Some("abc123".to_string()),
            arguments: Some("--limit=web".to_string()),
            inventory_id: Some(3),
            environment_id: Some(2),
            message: Some("Successful deploy".to_string()),
            label: Some("v1.2.3".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            template_name: "Deploy".to_string(),
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("\"label\":\"v1.2.3\""));
        assert!(json.contains("\"git_commit\":\"abc123\""));
    }

    #[test]
    fn test_task_snapshot_create_serialization() {
        let create = TaskSnapshotCreate {
            template_id: 5,
            task_id: 100,
            git_branch: Some("develop".to_string()),
            git_commit: Some("def456".to_string()),
            arguments: None,
            inventory_id: None,
            environment_id: None,
            message: None,
            label: Some("test-snapshot".to_string()),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"template_id\":5"));
        assert!(json.contains("\"label\":\"test-snapshot\""));
        // TaskSnapshotCreate doesn't have skip_serializing_if
        assert!(json.contains("\"arguments\":null"));
    }

    #[test]
    fn test_rollback_request_serialization() {
        let req = RollbackRequest {
            message: Some("Rolling back due to issues".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"message\":\"Rolling back due to issues\""));
    }

    #[test]
    fn test_rollback_request_empty() {
        let req = RollbackRequest { message: None };
        let json = serde_json::to_string(&req).unwrap();
        // RollbackRequest doesn't have skip_serializing_if
        assert!(json.contains("\"message\":null"));
    }

    #[test]
    fn test_task_snapshot_clone() {
        let snapshot = TaskSnapshot {
            id: 1,
            project_id: 10,
            template_id: 5,
            task_id: 100,
            git_branch: Some("main".to_string()),
            git_commit: Some("abc123".to_string()),
            arguments: Some("--limit=web".to_string()),
            inventory_id: Some(3),
            environment_id: Some(2),
            message: Some("Deploy snapshot".to_string()),
            label: Some("v1.0.0".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            template_name: "Deploy".to_string(),
        };
        let cloned = snapshot.clone();
        assert_eq!(cloned.git_commit, snapshot.git_commit);
        assert_eq!(cloned.label, snapshot.label);
    }

    #[test]
    fn test_task_snapshot_create_clone() {
        let create = TaskSnapshotCreate {
            template_id: 5,
            task_id: 100,
            git_branch: Some("develop".to_string()),
            git_commit: Some("def456".to_string()),
            arguments: None,
            inventory_id: None,
            environment_id: None,
            message: Some("Auto snapshot".to_string()),
            label: None,
        };
        let cloned = create.clone();
        assert_eq!(cloned.template_id, create.template_id);
        assert_eq!(cloned.git_branch, create.git_branch);
    }

    #[test]
    fn test_task_snapshot_all_null_fields() {
        let snapshot = TaskSnapshot {
            id: 1,
            project_id: 1,
            template_id: 1,
            task_id: 1,
            git_branch: None,
            git_commit: None,
            arguments: None,
            inventory_id: None,
            environment_id: None,
            message: None,
            label: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            template_name: String::new(),
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("\"git_branch\":null"));
        assert!(json.contains("\"label\":null"));
        assert!(json.contains("\"message\":null"));
    }
}
