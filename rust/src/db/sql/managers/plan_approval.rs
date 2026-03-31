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
