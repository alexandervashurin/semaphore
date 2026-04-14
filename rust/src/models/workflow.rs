//! Модель Workflow - DAG автоматизация (граф шаблонов)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Workflow - DAG пайплайн из шаблонов
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Workflow {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// Данные для создания Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCreate {
    pub name: String,
    pub description: Option<String>,
}

/// Данные для обновления Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowUpdate {
    pub name: String,
    pub description: Option<String>,
}

/// Узел в DAG-графе workflow (ссылается на шаблон)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowNode {
    pub id: i32,
    pub workflow_id: i32,
    pub template_id: i32,
    pub name: String,
    pub pos_x: f64,
    pub pos_y: f64,
    /// Sync Wave (Argo CD): узлы одной волны выполняются параллельно.
    /// Волны выполняются по возрастанию: 0, 1, 2, ...
    #[serde(default)]
    pub wave: i32,
}

/// Данные для создания узла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNodeCreate {
    pub template_id: i32,
    pub name: String,
    pub pos_x: f64,
    pub pos_y: f64,
    #[serde(default)]
    pub wave: i32,
}

/// Данные для обновления узла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNodeUpdate {
    pub name: String,
    pub pos_x: f64,
    pub pos_y: f64,
    #[serde(default)]
    pub wave: i32,
}

/// Условие перехода по ребру DAG
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum EdgeCondition {
    Success,
    Failure,
    Always,
}

impl std::fmt::Display for EdgeCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeCondition::Success => write!(f, "success"),
            EdgeCondition::Failure => write!(f, "failure"),
            EdgeCondition::Always => write!(f, "always"),
        }
    }
}

/// Ребро в DAG-графе workflow
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowEdge {
    pub id: i32,
    pub workflow_id: i32,
    pub from_node_id: i32,
    pub to_node_id: i32,
    pub condition: String, // "success" | "failure" | "always"
}

/// Данные для создания ребра
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdgeCreate {
    pub from_node_id: i32,
    pub to_node_id: i32,
    pub condition: String,
}

/// Запуск workflow
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowRun {
    pub id: i32,
    pub workflow_id: i32,
    pub project_id: i32,
    pub status: String, // "pending" | "running" | "success" | "failed" | "cancelled"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub created: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished: Option<DateTime<Utc>>,
}

/// Полный workflow с узлами и рёбрами для рендера canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFull {
    #[serde(flatten)]
    pub workflow: Workflow,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_condition_display() {
        assert_eq!(EdgeCondition::Success.to_string(), "success");
        assert_eq!(EdgeCondition::Failure.to_string(), "failure");
        assert_eq!(EdgeCondition::Always.to_string(), "always");
    }

    #[test]
    fn test_workflow_create_serialization() {
        let create = WorkflowCreate {
            name: "Test Workflow".to_string(),
            description: Some("Test description".to_string()),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"Test Workflow\""));
        assert!(json.contains("\"description\":\"Test description\""));
    }

    #[test]
    fn test_workflow_create_serialization_no_description() {
        let create = WorkflowCreate {
            name: "Minimal Workflow".to_string(),
            description: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"Minimal Workflow\""));
    }

    #[test]
    fn test_workflow_node_default_wave() {
        let node = WorkflowNodeCreate {
            template_id: 1,
            name: "Deploy".to_string(),
            pos_x: 100.0,
            pos_y: 200.0,
            wave: 0,
        };
        assert_eq!(node.wave, 0);
    }

    #[test]
    fn test_workflow_edge_create() {
        let edge = WorkflowEdgeCreate {
            from_node_id: 1,
            to_node_id: 2,
            condition: "success".to_string(),
        };
        assert_eq!(edge.from_node_id, 1);
        assert_eq!(edge.to_node_id, 2);
        assert_eq!(edge.condition, "success");
    }

    #[test]
    fn test_workflow_run_serialization() {
        let run = WorkflowRun {
            id: 1,
            workflow_id: 10,
            project_id: 5,
            status: "running".to_string(),
            message: Some("In progress".to_string()),
            created: Utc::now(),
            started: Some(Utc::now()),
            finished: None,
        };
        let json = serde_json::to_string(&run).unwrap();
        assert!(json.contains("\"status\":\"running\""));
        assert!(json.contains("\"message\":\"In progress\""));
    }

    #[test]
    fn test_workflow_full_serialization() {
        let workflow = Workflow {
            id: 1,
            project_id: 10,
            name: "Full Workflow".to_string(),
            description: Some("Complete workflow".to_string()),
            created: Utc::now(),
            updated: Utc::now(),
        };
        let node = WorkflowNode {
            id: 1,
            workflow_id: 1,
            template_id: 100,
            name: "Deploy".to_string(),
            pos_x: 0.0,
            pos_y: 0.0,
            wave: 0,
        };
        let edge = WorkflowEdge {
            id: 1,
            workflow_id: 1,
            from_node_id: 1,
            to_node_id: 2,
            condition: "success".to_string(),
        };
        let full = WorkflowFull {
            workflow,
            nodes: vec![node],
            edges: vec![edge],
        };
        let json = serde_json::to_string(&full).unwrap();
        assert!(json.contains("\"name\":\"Full Workflow\""));
        assert!(json.contains("\"nodes\":["));
        assert!(json.contains("\"edges\":["));
    }

    #[test]
    fn test_workflow_update_serialization() {
        let update = WorkflowUpdate {
            name: "Updated Name".to_string(),
            description: Some("Updated desc".to_string()),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"Updated Name\""));
    }

    #[test]
    fn test_edge_condition_clone() {
        let cond = EdgeCondition::Success;
        let cloned = cond.clone();
        assert_eq!(cloned, cond);
    }

    #[test]
    fn test_edge_condition_equality() {
        assert_eq!(EdgeCondition::Success, EdgeCondition::Success);
        assert_ne!(EdgeCondition::Failure, EdgeCondition::Always);
    }

    #[test]
    fn test_workflow_node_serialization() {
        let node = WorkflowNode {
            id: 5,
            workflow_id: 10,
            template_id: 20,
            name: "Test Node".to_string(),
            pos_x: 100.5,
            pos_y: 200.5,
            wave: 2,
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"name\":\"Test Node\""));
        assert!(json.contains("\"pos_x\":100.5"));
        assert!(json.contains("\"wave\":2"));
    }

    #[test]
    fn test_workflow_node_create_serialization() {
        let create = WorkflowNodeCreate {
            template_id: 1,
            name: "Create Node".to_string(),
            pos_x: 50.0,
            pos_y: 75.0,
            wave: 1,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"Create Node\""));
        assert!(json.contains("\"wave\":1"));
    }

    #[test]
    fn test_workflow_node_update_serialization() {
        let update = WorkflowNodeUpdate {
            name: "Updated Node".to_string(),
            pos_x: 150.0,
            pos_y: 250.0,
            wave: 3,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"Updated Node\""));
    }

    #[test]
    fn test_workflow_edge_serialization() {
        let edge = WorkflowEdge {
            id: 1,
            workflow_id: 1,
            from_node_id: 1,
            to_node_id: 2,
            condition: "failure".to_string(),
        };
        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("\"condition\":\"failure\""));
    }

    #[test]
    fn test_workflow_edge_create_clone() {
        let create = WorkflowEdgeCreate {
            from_node_id: 1,
            to_node_id: 3,
            condition: "always".to_string(),
        };
        let cloned = create.clone();
        assert_eq!(cloned.from_node_id, create.from_node_id);
    }

    #[test]
    fn test_workflow_run_clone() {
        let run = WorkflowRun {
            id: 1,
            workflow_id: 10,
            project_id: 5,
            status: "running".to_string(),
            message: Some("In progress".to_string()),
            created: Utc::now(),
            started: Some(Utc::now()),
            finished: None,
        };
        let cloned = run.clone();
        assert_eq!(cloned.status, run.status);
    }

    #[test]
    fn test_workflow_clone() {
        let workflow = Workflow {
            id: 1,
            project_id: 10,
            name: "Clone WF".to_string(),
            description: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let cloned = workflow.clone();
        assert_eq!(cloned.name, workflow.name);
    }

    #[test]
    fn test_workflow_create_clone() {
        let create = WorkflowCreate {
            name: "Clone Create".to_string(),
            description: None,
        };
        let cloned = create.clone();
        assert_eq!(cloned.name, create.name);
    }
}
