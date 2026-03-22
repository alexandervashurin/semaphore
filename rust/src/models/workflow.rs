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
}

/// Данные для создания узла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNodeCreate {
    pub template_id: i32,
    pub name: String,
    pub pos_x: f64,
    pub pos_y: f64,
}

/// Данные для обновления узла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNodeUpdate {
    pub name: String,
    pub pos_x: f64,
    pub pos_y: f64,
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
