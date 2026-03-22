//! WorkflowManager - управление Workflow DAG

use crate::db::sql::SqlStore;
use crate::db::store::WorkflowManager;
use crate::error::{Error, Result};
use crate::models::workflow::{
    Workflow, WorkflowCreate, WorkflowUpdate,
    WorkflowNode, WorkflowNodeCreate, WorkflowNodeUpdate,
    WorkflowEdge, WorkflowEdgeCreate,
    WorkflowRun,
};
use async_trait::async_trait;

#[async_trait]
impl WorkflowManager for SqlStore {
    // =========================================================================
    // Workflows
    // =========================================================================

    async fn get_workflows(&self, project_id: i32) -> Result<Vec<Workflow>> {
        let rows = sqlx::query_as::<_, Workflow>(
                "SELECT * FROM workflow WHERE project_id = $1 ORDER BY name"
            )
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(rows)
    }

    async fn get_workflow(&self, id: i32, project_id: i32) -> Result<Workflow> {
        let row = sqlx::query_as::<_, Workflow>(
                "SELECT * FROM workflow WHERE id = $1 AND project_id = $2"
            )
            .bind(id)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn create_workflow(&self, project_id: i32, payload: WorkflowCreate) -> Result<Workflow> {
        let row = sqlx::query_as::<_, Workflow>(
                "INSERT INTO workflow (project_id, name, description, created, updated)
                 VALUES ($1, $2, $3, NOW(), NOW()) RETURNING *"
            )
            .bind(project_id)
            .bind(&payload.name)
            .bind(&payload.description)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn update_workflow(&self, id: i32, project_id: i32, payload: WorkflowUpdate) -> Result<Workflow> {
        let row = sqlx::query_as::<_, Workflow>(
                "UPDATE workflow SET name = $1, description = $2, updated = NOW()
                 WHERE id = $3 AND project_id = $4 RETURNING *"
            )
            .bind(&payload.name)
            .bind(&payload.description)
            .bind(id)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn delete_workflow(&self, id: i32, project_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM workflow WHERE id = $1 AND project_id = $2")
                .bind(id)
                .bind(project_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    // =========================================================================
    // Workflow Nodes
    // =========================================================================

    async fn get_workflow_nodes(&self, workflow_id: i32) -> Result<Vec<WorkflowNode>> {
        let rows = sqlx::query_as::<_, WorkflowNode>(
                "SELECT * FROM workflow_node WHERE workflow_id = $1 ORDER BY id"
            )
            .bind(workflow_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(rows)
    }

    async fn create_workflow_node(&self, workflow_id: i32, payload: WorkflowNodeCreate) -> Result<WorkflowNode> {
        let row = sqlx::query_as::<_, WorkflowNode>(
                "INSERT INTO workflow_node (workflow_id, template_id, name, pos_x, pos_y)
                 VALUES ($1, $2, $3, $4, $5) RETURNING *"
            )
            .bind(workflow_id)
            .bind(payload.template_id)
            .bind(&payload.name)
            .bind(payload.pos_x)
            .bind(payload.pos_y)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn update_workflow_node(&self, id: i32, workflow_id: i32, payload: WorkflowNodeUpdate) -> Result<WorkflowNode> {
        let row = sqlx::query_as::<_, WorkflowNode>(
                "UPDATE workflow_node SET name = $1, pos_x = $2, pos_y = $3
                 WHERE id = $4 AND workflow_id = $5 RETURNING *"
            )
            .bind(&payload.name)
            .bind(payload.pos_x)
            .bind(payload.pos_y)
            .bind(id)
            .bind(workflow_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn delete_workflow_node(&self, id: i32, workflow_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM workflow_node WHERE id = $1 AND workflow_id = $2")
                .bind(id)
                .bind(workflow_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    // =========================================================================
    // Workflow Edges
    // =========================================================================

    async fn get_workflow_edges(&self, workflow_id: i32) -> Result<Vec<WorkflowEdge>> {
        let rows = sqlx::query_as::<_, WorkflowEdge>(
                "SELECT * FROM workflow_edge WHERE workflow_id = $1 ORDER BY id"
            )
            .bind(workflow_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(rows)
    }

    async fn create_workflow_edge(&self, workflow_id: i32, payload: WorkflowEdgeCreate) -> Result<WorkflowEdge> {
        let row = sqlx::query_as::<_, WorkflowEdge>(
                "INSERT INTO workflow_edge (workflow_id, from_node_id, to_node_id, condition)
                 VALUES ($1, $2, $3, $4) RETURNING *"
            )
            .bind(workflow_id)
            .bind(payload.from_node_id)
            .bind(payload.to_node_id)
            .bind(&payload.condition)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn delete_workflow_edge(&self, id: i32, workflow_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM workflow_edge WHERE id = $1 AND workflow_id = $2")
                .bind(id)
                .bind(workflow_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    // =========================================================================
    // Workflow Runs
    // =========================================================================

    async fn get_workflow_runs(&self, workflow_id: i32, project_id: i32) -> Result<Vec<WorkflowRun>> {
        let rows = sqlx::query_as::<_, WorkflowRun>(
                "SELECT * FROM workflow_run WHERE workflow_id = $1 AND project_id = $2 ORDER BY created DESC"
            )
            .bind(workflow_id)
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(rows)
    }

    async fn create_workflow_run(&self, workflow_id: i32, project_id: i32) -> Result<WorkflowRun> {
        let row = sqlx::query_as::<_, WorkflowRun>(
                "INSERT INTO workflow_run (workflow_id, project_id, status, created)
                 VALUES ($1, $2, 'pending', NOW()) RETURNING *"
            )
            .bind(workflow_id)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn update_workflow_run_status(&self, id: i32, status: &str, message: Option<String>) -> Result<()> {
        sqlx::query(
                "UPDATE workflow_run SET status = $1, message = $2 WHERE id = $3"
            )
            .bind(status)
            .bind(&message)
            .bind(id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}
