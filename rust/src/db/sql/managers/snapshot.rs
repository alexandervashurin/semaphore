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
    use crate::models::snapshot::{TaskSnapshot, TaskSnapshotCreate, RollbackRequest};

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
}
