//! Task Snapshot SQL Manager

use crate::db::sql::SqlStore;
use crate::db::store::SnapshotManager;
use crate::error::{Error, Result};
use crate::models::snapshot::{TaskSnapshot, TaskSnapshotCreate};
use async_trait::async_trait;

#[async_trait]
impl SnapshotManager for SqlStore {
    async fn get_snapshots(
        &self,
        project_id: i32,
        template_id: Option<i32>,
        limit: i64,
    ) -> Result<Vec<TaskSnapshot>> {
        let rows = if let Some(tpl_id) = template_id {
            sqlx::query_as::<_, TaskSnapshot>(
                r#"SELECT s.*, COALESCE(t.name,'') AS template_name
                       FROM task_snapshot s
                       LEFT JOIN template t ON t.id = s.template_id
                       WHERE s.project_id = $1 AND s.template_id = $2
                       ORDER BY s.id DESC LIMIT $3"#,
            )
            .bind(project_id)
            .bind(tpl_id)
            .bind(limit)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?
        } else {
            sqlx::query_as::<_, TaskSnapshot>(
                r#"SELECT s.*, COALESCE(t.name,'') AS template_name
                       FROM task_snapshot s
                       LEFT JOIN template t ON t.id = s.template_id
                       WHERE s.project_id = $1
                       ORDER BY s.id DESC LIMIT $2"#,
            )
            .bind(project_id)
            .bind(limit)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?
        };
        Ok(rows)
    }

    async fn create_snapshot(
        &self,
        project_id: i32,
        payload: TaskSnapshotCreate,
    ) -> Result<TaskSnapshot> {
        let row = sqlx::query_as::<_, TaskSnapshot>(
                r#"INSERT INTO task_snapshot (project_id, template_id, task_id, git_branch, git_commit, arguments, inventory_id, environment_id, message, label)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                   RETURNING *, '' AS template_name"#
            )
            .bind(project_id)
            .bind(payload.template_id)
            .bind(payload.task_id)
            .bind(&payload.git_branch)
            .bind(&payload.git_commit)
            .bind(&payload.arguments)
            .bind(payload.inventory_id)
            .bind(payload.environment_id)
            .bind(&payload.message)
            .bind(&payload.label)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(row)
    }

    async fn get_snapshot(&self, id: i32, project_id: i32) -> Result<TaskSnapshot> {
        sqlx::query_as::<_, TaskSnapshot>(
            r#"SELECT s.*, COALESCE(t.name,'') AS template_name
                   FROM task_snapshot s LEFT JOIN template t ON t.id = s.template_id
                   WHERE s.id = $1 AND s.project_id = $2"#,
        )
        .bind(id)
        .bind(project_id)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(|_| Error::NotFound(format!("Snapshot {} not found", id)))
    }

    async fn delete_snapshot(&self, id: i32, project_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM task_snapshot WHERE id = $1 AND project_id = $2")
            .bind(id)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::snapshot::{RollbackRequest, TaskSnapshot, TaskSnapshotCreate};

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
            arguments: None,
            inventory_id: None,
            environment_id: None,
            message: None,
            label: None,
            created_at: "2024-01-01".to_string(),
            template_name: "Deploy".to_string(),
        };
        let cloned = snapshot.clone();
        assert_eq!(cloned.git_commit, snapshot.git_commit);
        assert_eq!(cloned.template_name, snapshot.template_name);
    }

    #[test]
    fn test_task_snapshot_create_clone() {
        let create = TaskSnapshotCreate {
            template_id: 5,
            task_id: 100,
            git_branch: None,
            git_commit: None,
            arguments: None,
            inventory_id: None,
            environment_id: None,
            message: Some("Auto".to_string()),
            label: None,
        };
        let cloned = create.clone();
        assert_eq!(cloned.template_id, create.template_id);
        assert_eq!(cloned.message, create.message);
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
            created_at: "2024-01-01".to_string(),
            template_name: String::new(),
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("\"git_branch\":null"));
        assert!(json.contains("\"label\":null"));
    }

    #[test]
    fn test_task_snapshot_create_all_null_fields() {
        let create = TaskSnapshotCreate {
            template_id: 1,
            task_id: 1,
            git_branch: None,
            git_commit: None,
            arguments: None,
            inventory_id: None,
            environment_id: None,
            message: None,
            label: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"template_id\":1"));
        assert!(json.contains("\"task_id\":1"));
    }

    #[test]
    fn test_rollback_request_clone() {
        let req = RollbackRequest {
            message: Some("Clone rollback".to_string()),
        };
        let cloned = req.clone();
        assert_eq!(cloned.message, req.message);
    }

    #[test]
    fn test_task_snapshot_debug_format() {
        let snapshot = TaskSnapshot {
            id: 1,
            project_id: 10,
            template_id: 5,
            task_id: 100,
            git_branch: Some("feature".to_string()),
            git_commit: None,
            arguments: None,
            inventory_id: None,
            environment_id: None,
            message: None,
            label: None,
            created_at: "2024-01-01".to_string(),
            template_name: "Template".to_string(),
        };
        let debug = format!("{:?}", snapshot);
        assert!(debug.contains("TaskSnapshot"));
        assert!(debug.contains("feature"));
    }

    #[test]
    fn test_task_snapshot_create_debug_format() {
        let create = TaskSnapshotCreate {
            template_id: 10,
            task_id: 50,
            git_branch: Some("main".to_string()),
            git_commit: Some("commit123".to_string()),
            arguments: Some("--check".to_string()),
            inventory_id: Some(1),
            environment_id: Some(2),
            message: Some("Create message".to_string()),
            label: Some("label1".to_string()),
        };
        let debug = format!("{:?}", create);
        assert!(debug.contains("TaskSnapshotCreate"));
        assert!(debug.contains("commit123"));
    }

    #[test]
    fn test_task_snapshot_unicode_values() {
        let snapshot = TaskSnapshot {
            id: 1,
            project_id: 1,
            template_id: 1,
            task_id: 1,
            git_branch: Some("ветка".to_string()),
            git_commit: Some("коммит".to_string()),
            arguments: Some("аргументы".to_string()),
            inventory_id: None,
            environment_id: None,
            message: Some("сообщение".to_string()),
            label: Some("метка".to_string()),
            created_at: "2024-01-01".to_string(),
            template_name: "Шаблон".to_string(),
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("ветка"));
        assert!(json.contains("сообщение"));

        let deserialized: TaskSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.git_branch, Some("ветка".to_string()));
    }

    #[test]
    fn test_task_snapshot_deserialization() {
        let json = r#"{
            "id": 42,
            "project_id": 10,
            "template_id": 5,
            "task_id": 200,
            "git_branch": "release",
            "git_commit": "deadbeef",
            "arguments": null,
            "inventory_id": null,
            "environment_id": null,
            "message": "Release snapshot",
            "label": "v2.0",
            "created_at": "2024-06-15T10:00:00Z",
            "template_name": "Release Template"
        }"#;
        let snapshot: TaskSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.id, 42);
        assert_eq!(snapshot.task_id, 200);
    }

    #[test]
    fn test_task_snapshot_create_deserialization() {
        let json = r#"{
            "template_id": 3,
            "task_id": 30,
            "git_branch": "staging",
            "git_commit": "staging123",
            "arguments": "--dry-run",
            "inventory_id": 2,
            "environment_id": 1,
            "message": "Staging deploy",
            "label": "staging-v1"
        }"#;
        let create: TaskSnapshotCreate = serde_json::from_str(json).unwrap();
        assert_eq!(create.template_id, 3);
        assert_eq!(create.git_branch, Some("staging".to_string()));
    }

    #[test]
    fn test_task_snapshot_empty_string_fields() {
        let snapshot = TaskSnapshot {
            id: 1,
            project_id: 1,
            template_id: 1,
            task_id: 1,
            git_branch: Some(String::new()),
            git_commit: Some(String::new()),
            arguments: Some(String::new()),
            inventory_id: None,
            environment_id: None,
            message: Some(String::new()),
            label: Some(String::new()),
            created_at: String::new(),
            template_name: String::new(),
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("\"git_branch\":\"\""));
        assert!(json.contains("\"created_at\":\"\""));
    }

    #[test]
    fn test_task_snapshot_large_ids() {
        let snapshot = TaskSnapshot {
            id: i32::MAX,
            project_id: i32::MAX,
            template_id: i32::MAX,
            task_id: i32::MAX,
            git_branch: None,
            git_commit: None,
            arguments: None,
            inventory_id: Some(i32::MAX),
            environment_id: Some(i32::MAX),
            message: None,
            label: None,
            created_at: "2024-01-01".to_string(),
            template_name: "Max".to_string(),
        };
        assert_eq!(snapshot.id, i32::MAX);
        assert_eq!(snapshot.project_id, i32::MAX);
    }

    #[test]
    fn test_rollback_request_deserialization() {
        let json = r#"{"message": "Please rollback this deployment"}"#;
        let req: RollbackRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            req.message,
            Some("Please rollback this deployment".to_string())
        );
    }

    #[test]
    fn test_task_snapshot_json_roundtrip() {
        let original = TaskSnapshot {
            id: 77,
            project_id: 7,
            template_id: 77,
            task_id: 777,
            git_branch: Some("roundtrip".to_string()),
            git_commit: Some("roundtrip_commit".to_string()),
            arguments: Some("--roundtrip".to_string()),
            inventory_id: Some(7),
            environment_id: Some(7),
            message: Some("Roundtrip test".to_string()),
            label: Some("roundtrip-label".to_string()),
            created_at: "2024-12-01T00:00:00Z".to_string(),
            template_name: "Roundtrip Template".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: TaskSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, original.id);
        assert_eq!(restored.git_branch, original.git_branch);
        assert_eq!(restored.label, original.label);
    }

    #[test]
    fn test_task_snapshot_create_json_roundtrip() {
        let original = TaskSnapshotCreate {
            template_id: 55,
            task_id: 555,
            git_branch: Some("roundtrip".to_string()),
            git_commit: Some("commit_rt".to_string()),
            arguments: Some("--rt".to_string()),
            inventory_id: Some(5),
            environment_id: Some(5),
            message: Some("Roundtrip create".to_string()),
            label: Some("rt-label".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: TaskSnapshotCreate = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.template_id, original.template_id);
        assert_eq!(restored.git_commit, original.git_commit);
    }
}
