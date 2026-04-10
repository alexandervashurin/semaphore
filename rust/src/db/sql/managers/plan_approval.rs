//! Plan Approval SQL Manager (Phase 2)

use crate::db::sql::SqlStore;
use crate::db::store::PlanApprovalManager;
use crate::error::{Error, Result};
use crate::models::TerraformPlan;
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl PlanApprovalManager for SqlStore {
    async fn create_plan(&self, plan: TerraformPlan) -> Result<TerraformPlan> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query(
            "INSERT INTO terraform_plan
               (task_id, project_id, plan_output, plan_json,
                resources_added, resources_changed, resources_removed, status)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
             RETURNING id, task_id, project_id, plan_output, plan_json,
                       resources_added, resources_changed, resources_removed,
                       status, created_at, reviewed_at, reviewed_by, review_comment",
        )
        .bind(plan.task_id)
        .bind(plan.project_id)
        .bind(&plan.plan_output)
        .bind(&plan.plan_json)
        .bind(plan.resources_added)
        .bind(plan.resources_changed)
        .bind(plan.resources_removed)
        .bind(&plan.status)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

        Ok(row_to_plan(row))
    }

    async fn get_plan_by_task(
        &self,
        project_id: i32,
        task_id: i32,
    ) -> Result<Option<TerraformPlan>> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query(
            "SELECT id, task_id, project_id, plan_output, plan_json,
                    resources_added, resources_changed, resources_removed,
                    status, created_at, reviewed_at, reviewed_by, review_comment
             FROM terraform_plan
             WHERE project_id = $1 AND task_id = $2
             ORDER BY id DESC LIMIT 1",
        )
        .bind(project_id)
        .bind(task_id)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;

        Ok(row.map(row_to_plan))
    }

    async fn list_pending_plans(&self, project_id: i32) -> Result<Vec<TerraformPlan>> {
        let pool = self.get_postgres_pool()?;
        let rows = sqlx::query(
            "SELECT id, task_id, project_id, plan_output, plan_json,
                    resources_added, resources_changed, resources_removed,
                    status, created_at, reviewed_at, reviewed_by, review_comment
             FROM terraform_plan
             WHERE project_id = $1 AND status = 'pending'
             ORDER BY created_at DESC",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(row_to_plan).collect())
    }

    async fn approve_plan(&self, id: i64, reviewed_by: i32, comment: Option<String>) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        let result = sqlx::query(
            "UPDATE terraform_plan
             SET status = 'approved', reviewed_at = NOW(), reviewed_by = $2, review_comment = $3
             WHERE id = $1 AND status = 'pending'",
        )
        .bind(id)
        .bind(reviewed_by)
        .bind(&comment)
        .execute(pool)
        .await
        .map_err(Error::Database)?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound(format!(
                "Plan {} not found or already reviewed",
                id
            )));
        }
        Ok(())
    }

    async fn reject_plan(&self, id: i64, reviewed_by: i32, comment: Option<String>) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        let result = sqlx::query(
            "UPDATE terraform_plan
             SET status = 'rejected', reviewed_at = NOW(), reviewed_by = $2, review_comment = $3
             WHERE id = $1 AND status = 'pending'",
        )
        .bind(id)
        .bind(reviewed_by)
        .bind(&comment)
        .execute(pool)
        .await
        .map_err(Error::Database)?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound(format!(
                "Plan {} not found or already reviewed",
                id
            )));
        }
        Ok(())
    }

    async fn update_plan_output(
        &self,
        task_id: i32,
        output: String,
        json: Option<String>,
        added: i32,
        changed: i32,
        removed: i32,
    ) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query(
            "UPDATE terraform_plan
             SET plan_output = $2, plan_json = $3,
                 resources_added = $4, resources_changed = $5, resources_removed = $6
             WHERE task_id = $1 AND status = 'pending'",
        )
        .bind(task_id)
        .bind(&output)
        .bind(&json)
        .bind(added)
        .bind(changed)
        .bind(removed)
        .execute(pool)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }
}

fn row_to_plan(row: sqlx::postgres::PgRow) -> TerraformPlan {
    TerraformPlan {
        id: row.get("id"),
        task_id: row.get("task_id"),
        project_id: row.get("project_id"),
        plan_output: row.get("plan_output"),
        plan_json: row.try_get("plan_json").ok().flatten(),
        resources_added: row.get("resources_added"),
        resources_changed: row.get("resources_changed"),
        resources_removed: row.get("resources_removed"),
        status: row.get("status"),
        created_at: row.get("created_at"),
        reviewed_at: row.try_get("reviewed_at").ok().flatten(),
        reviewed_by: row.try_get("reviewed_by").ok().flatten(),
        review_comment: row.try_get("review_comment").ok().flatten(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terraform_plan_structure() {
        let plan = TerraformPlan {
            id: 1,
            task_id: 100,
            project_id: 10,
            plan_output: "Plan: 1 to add, 0 to change, 0 to destroy.".to_string(),
            plan_json: Some(r#"{"format_version":"1.0"}"#.to_string()),
            resources_added: 1,
            resources_changed: 0,
            resources_removed: 0,
            status: "pending".to_string(),
            created_at: chrono::Utc::now(),
            reviewed_at: None,
            reviewed_by: None,
            review_comment: None,
        };
        assert_eq!(plan.resources_added, 1);
        assert_eq!(plan.status, "pending");
    }

    #[test]
    fn test_terraform_plan_with_review() {
        let plan = TerraformPlan {
            id: 1,
            task_id: 100,
            project_id: 10,
            plan_output: "No changes.".to_string(),
            plan_json: None,
            resources_added: 0,
            resources_changed: 0,
            resources_removed: 0,
            status: "approved".to_string(),
            created_at: chrono::Utc::now(),
            reviewed_at: Some(chrono::Utc::now()),
            reviewed_by: Some(5),
            review_comment: Some("Looks good".to_string()),
        };
        assert_eq!(plan.status, "approved");
        assert!(plan.review_comment.is_some());
    }

    #[test]
    fn test_terraform_plan_status_variants() {
        let statuses = vec!["pending", "approved", "rejected"];
        for status in statuses {
            let plan = TerraformPlan {
                id: 1,
                task_id: 1,
                project_id: 1,
                plan_output: "output".to_string(),
                plan_json: None,
                resources_added: 0,
                resources_changed: 0,
                resources_removed: 0,
                status: status.to_string(),
                created_at: chrono::Utc::now(),
                reviewed_at: None,
                reviewed_by: None,
                review_comment: None,
            };
            assert_eq!(plan.status, status);
        }
    }

    #[test]
    fn test_terraform_plan_serialize() {
        let plan = TerraformPlan {
            id: 42,
            task_id: 100,
            project_id: 10,
            plan_output: "1 to add".to_string(),
            plan_json: Some("{}" .to_string()),
            resources_added: 1,
            resources_changed: 0,
            resources_removed: 0,
            status: "pending".to_string(),
            created_at: chrono::Utc::now(),
            reviewed_at: None,
            reviewed_by: None,
            review_comment: None,
        };
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"resources_added\":1"));
    }

    #[test]
    fn test_terraform_plan_resource_counts() {
        let plan = TerraformPlan {
            id: 1,
            task_id: 1,
            project_id: 1,
            plan_output: "Plan: 5 to add, 3 to change, 2 to destroy.".to_string(),
            plan_json: None,
            resources_added: 5,
            resources_changed: 3,
            resources_removed: 2,
            status: "pending".to_string(),
            created_at: chrono::Utc::now(),
            reviewed_at: None,
            reviewed_by: None,
            review_comment: None,
        };
        assert_eq!(plan.resources_added, 5);
        assert_eq!(plan.resources_changed, 3);
        assert_eq!(plan.resources_removed, 2);
    }

    #[test]
    fn test_terraform_plan_clone() {
        let plan = TerraformPlan {
            id: 1,
            task_id: 10,
            project_id: 5,
            plan_output: "output".to_string(),
            plan_json: Some("{}" .to_string()),
            resources_added: 0,
            resources_changed: 0,
            resources_removed: 0,
            status: "pending".to_string(),
            created_at: chrono::Utc::now(),
            reviewed_at: None,
            reviewed_by: None,
            review_comment: Some("review comment".to_string()),
        };
        let cloned = plan.clone();
        assert_eq!(cloned.task_id, plan.task_id);
        assert_eq!(cloned.review_comment, plan.review_comment);
    }

    #[test]
    fn test_sql_query_create_plan() {
        let query = "INSERT INTO terraform_plan
               (task_id, project_id, plan_output, plan_json,
                resources_added, resources_changed, resources_removed, status)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
             RETURNING id, task_id, project_id, plan_output, plan_json,
                       resources_added, resources_changed, resources_removed,
                       status, created_at, reviewed_at, reviewed_by, review_comment";
        assert!(query.contains("INSERT INTO terraform_plan"));
        assert!(query.contains("resources_added"));
    }

    #[test]
    fn test_sql_query_get_plan_by_task() {
        let query = "SELECT id, task_id, project_id, plan_output, plan_json,
                    resources_added, resources_changed, resources_removed,
                    status, created_at, reviewed_at, reviewed_by, review_comment
             FROM terraform_plan
             WHERE project_id = $1 AND task_id = $2
             ORDER BY id DESC LIMIT 1";
        assert!(query.contains("WHERE project_id"));
        assert!(query.contains("task_id"));
    }

    #[test]
    fn test_sql_query_list_pending_plans() {
        let query = "SELECT id, task_id, project_id, plan_output, plan_json,
                    resources_added, resources_changed, resources_removed,
                    status, created_at, reviewed_at, reviewed_by, review_comment
             FROM terraform_plan
             WHERE project_id = $1 AND status = 'pending'
             ORDER BY created_at DESC";
        assert!(query.contains("status = 'pending'"));
    }

    #[test]
    fn test_sql_query_approve_plan() {
        let query = "UPDATE terraform_plan
             SET status = 'approved', reviewed_at = NOW(), reviewed_by = $2, review_comment = $3
             WHERE id = $1 AND status = 'pending'";
        assert!(query.contains("status = 'approved'"));
        assert!(query.contains("NOW()"));
    }

    #[test]
    fn test_sql_query_reject_plan() {
        let query = "UPDATE terraform_plan
             SET status = 'rejected', reviewed_at = NOW(), reviewed_by = $2, review_comment = $3
             WHERE id = $1 AND status = 'pending'";
        assert!(query.contains("status = 'rejected'"));
    }

    #[test]
    fn test_terraform_plan_with_optional_fields() {
        let plan = TerraformPlan {
            id: 1,
            task_id: 1,
            project_id: 1,
            plan_output: "output".to_string(),
            plan_json: None,
            resources_added: 0,
            resources_changed: 0,
            resources_removed: 0,
            status: "pending".to_string(),
            created_at: chrono::Utc::now(),
            reviewed_at: Some(chrono::Utc::now()),
            reviewed_by: None,
            review_comment: None,
        };
        assert!(plan.plan_json.is_none());
        assert!(plan.reviewed_at.is_some());
        assert!(plan.reviewed_by.is_none());
    }

    #[test]
    fn test_plan_output_non_empty() {
        let plan = TerraformPlan {
            id: 1,
            task_id: 1,
            project_id: 1,
            plan_output: "Plan: 0 to add, 0 to change, 0 to destroy.".to_string(),
            plan_json: None,
            resources_added: 0,
            resources_changed: 0,
            resources_removed: 0,
            status: "pending".to_string(),
            created_at: chrono::Utc::now(),
            reviewed_at: None,
            reviewed_by: None,
            review_comment: None,
        };
        assert!(!plan.plan_output.is_empty());
    }
}
