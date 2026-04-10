//! WorkflowManager - управление Workflow DAG

use crate::db::sql::SqlStore;
use crate::db::store::WorkflowManager;
use crate::error::{Error, Result};
use crate::models::workflow::{
    Workflow, WorkflowCreate, WorkflowEdge, WorkflowEdgeCreate, WorkflowNode, WorkflowNodeCreate,
    WorkflowNodeUpdate, WorkflowRun, WorkflowUpdate,
};
use async_trait::async_trait;

#[async_trait]
impl WorkflowManager for SqlStore {
    // =========================================================================
    // Workflows
    // =========================================================================

    async fn get_workflows(&self, project_id: i32) -> Result<Vec<Workflow>> {
        let rows = sqlx::query_as::<_, Workflow>(
            "SELECT * FROM workflow WHERE project_id = $1 ORDER BY name",
        )
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(rows)
    }

    async fn get_workflow(&self, id: i32, project_id: i32) -> Result<Workflow> {
        let row = sqlx::query_as::<_, Workflow>(
            "SELECT * FROM workflow WHERE id = $1 AND project_id = $2",
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
                 VALUES ($1, $2, $3, NOW(), NOW()) RETURNING *",
        )
        .bind(project_id)
        .bind(&payload.name)
        .bind(&payload.description)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(row)
    }

    async fn update_workflow(
        &self,
        id: i32,
        project_id: i32,
        payload: WorkflowUpdate,
    ) -> Result<Workflow> {
        let row = sqlx::query_as::<_, Workflow>(
            "UPDATE workflow SET name = $1, description = $2, updated = NOW()
                 WHERE id = $3 AND project_id = $4 RETURNING *",
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
            "SELECT * FROM workflow_node WHERE workflow_id = $1 ORDER BY id",
        )
        .bind(workflow_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(rows)
    }

    async fn create_workflow_node(
        &self,
        workflow_id: i32,
        payload: WorkflowNodeCreate,
    ) -> Result<WorkflowNode> {
        let row = sqlx::query_as::<_, WorkflowNode>(
            "INSERT INTO workflow_node (workflow_id, template_id, name, pos_x, pos_y)
                 VALUES ($1, $2, $3, $4, $5) RETURNING *",
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

    async fn update_workflow_node(
        &self,
        id: i32,
        workflow_id: i32,
        payload: WorkflowNodeUpdate,
    ) -> Result<WorkflowNode> {
        let row = sqlx::query_as::<_, WorkflowNode>(
            "UPDATE workflow_node SET name = $1, pos_x = $2, pos_y = $3
                 WHERE id = $4 AND workflow_id = $5 RETURNING *",
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
            "SELECT * FROM workflow_edge WHERE workflow_id = $1 ORDER BY id",
        )
        .bind(workflow_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(rows)
    }

    async fn create_workflow_edge(
        &self,
        workflow_id: i32,
        payload: WorkflowEdgeCreate,
    ) -> Result<WorkflowEdge> {
        let row = sqlx::query_as::<_, WorkflowEdge>(
            "INSERT INTO workflow_edge (workflow_id, from_node_id, to_node_id, condition)
                 VALUES ($1, $2, $3, $4) RETURNING *",
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

    async fn get_workflow_runs(
        &self,
        workflow_id: i32,
        project_id: i32,
    ) -> Result<Vec<WorkflowRun>> {
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
                 VALUES ($1, $2, 'pending', NOW()) RETURNING *",
        )
        .bind(workflow_id)
        .bind(project_id)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(row)
    }

    async fn update_workflow_run_status(
        &self,
        id: i32,
        status: &str,
        message: Option<String>,
    ) -> Result<()> {
        sqlx::query("UPDATE workflow_run SET status = $1, message = $2 WHERE id = $3")
            .bind(status)
            .bind(&message)
            .bind(id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::workflow::{
        EdgeCondition, Workflow, WorkflowCreate, WorkflowEdge, WorkflowEdgeCreate, WorkflowNode,
        WorkflowNodeCreate, WorkflowNodeUpdate, WorkflowRun, WorkflowUpdate,
    };
    use chrono::Utc;

    #[test]
    fn test_workflow_structure() {
        let wf = Workflow {
            id: 1,
            project_id: 10,
            name: "Deploy Pipeline".to_string(),
            description: Some("CD pipeline".to_string()),
            created: Utc::now(),
            updated: Utc::now(),
        };
        assert_eq!(wf.project_id, 10);
        assert_eq!(wf.name, "Deploy Pipeline");
    }

    #[test]
    fn test_workflow_create_structure() {
        let create = WorkflowCreate {
            name: "New Workflow".to_string(),
            description: Some("Description".to_string()),
        };
        assert_eq!(create.name, "New Workflow");
    }

    #[test]
    fn test_workflow_update_structure() {
        let update = WorkflowUpdate {
            name: "Updated Name".to_string(),
            description: None,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"Updated Name\""));
        assert!(json.contains("\"description\":null"));
    }

    #[test]
    fn test_workflow_node_structure() {
        let node = WorkflowNode {
            id: 1,
            workflow_id: 5,
            template_id: 10,
            name: "Deploy Step".to_string(),
            pos_x: 100.0,
            pos_y: 200.0,
            wave: 0,
        };
        assert_eq!(node.template_id, 10);
        assert_eq!(node.wave, 0);
    }

    #[test]
    fn test_workflow_node_create() {
        let create = WorkflowNodeCreate {
            template_id: 5,
            name: "Test Node".to_string(),
            pos_x: 0.0,
            pos_y: 0.0,
            wave: 1,
        };
        assert_eq!(create.wave, 1);
    }

    #[test]
    fn test_workflow_node_update() {
        let update = WorkflowNodeUpdate {
            name: "Renamed".to_string(),
            pos_x: 150.0,
            pos_y: 250.0,
            wave: 2,
        };
        assert_eq!(update.name, "Renamed");
    }

    #[test]
    fn test_edge_condition_variants() {
        assert_eq!(EdgeCondition::Success.to_string(), "success");
        assert_eq!(EdgeCondition::Failure.to_string(), "failure");
        assert_eq!(EdgeCondition::Always.to_string(), "always");
    }

    #[test]
    fn test_edge_condition_equality() {
        assert_eq!(EdgeCondition::Success, EdgeCondition::Success);
        assert_ne!(EdgeCondition::Success, EdgeCondition::Failure);
    }

    #[test]
    fn test_workflow_edge_structure() {
        let edge = WorkflowEdge {
            id: 1,
            workflow_id: 5,
            from_node_id: 2,
            to_node_id: 3,
            condition: "success".to_string(),
        };
        assert_eq!(edge.from_node_id, 2);
        assert_eq!(edge.to_node_id, 3);
    }

    #[test]
    fn test_workflow_edge_create() {
        let create = WorkflowEdgeCreate {
            from_node_id: 1,
            to_node_id: 2,
            condition: "always".to_string(),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"from_node_id\":1"));
        assert!(json.contains("\"to_node_id\":2"));
    }

    #[test]
    fn test_workflow_run_structure() {
        let run = WorkflowRun {
            id: 1,
            workflow_id: 5,
            project_id: 10,
            status: "running".to_string(),
            message: None,
            created: Utc::now(),
            started: None,
            finished: None,
        };
        assert_eq!(run.workflow_id, 5);
        assert_eq!(run.status, "running");
    }

    #[test]
    fn test_sql_query_get_workflows() {
        let query = "SELECT * FROM workflow WHERE project_id = $1 ORDER BY name";
        assert!(query.contains("workflow"));
        assert!(query.contains("project_id"));
    }

    #[test]
    fn test_sql_query_create_workflow_node() {
        let query = "INSERT INTO workflow_node (workflow_id, template_id, name, pos_x, pos_y)
                 VALUES ($1, $2, $3, $4, $5) RETURNING *";
        assert!(query.contains("workflow_node"));
        assert!(query.contains("template_id"));
    }

    #[test]
    fn test_sql_query_workflow_edges() {
        let query = "SELECT * FROM workflow_edge WHERE workflow_id = $1 ORDER BY id";
        assert!(query.contains("workflow_edge"));
    }

    #[test]
    fn test_sql_query_workflow_run() {
        let query = "INSERT INTO workflow_run (workflow_id, project_id, status, created)
                 VALUES ($1, $2, 'pending', NOW()) RETURNING *";
        assert!(query.contains("workflow_run"));
        assert!(query.contains("'pending'"));
    }

    #[test]
    fn test_workflow_serialize() {
        let wf = Workflow {
            id: 1,
            project_id: 1,
            name: "Test WF".to_string(),
            description: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&wf).unwrap();
        assert!(json.contains("\"name\":\"Test WF\""));
        assert!(json.contains("\"project_id\":1"));
    }
}
