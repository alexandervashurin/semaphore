//! Task Structured Output — именованные key-value выходы задачи (Pulumi Outputs)
//!
//! Позволяет передавать output одного шаблона как input другого.
//! Парсится из stdout задачи по маркеру `VELUM_OUTPUT: {"key":"value"}`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

/// Структурированный output задачи (одна пара key=value)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskStructuredOutput {
    pub id: i32,
    pub task_id: i32,
    pub project_id: i32,
    /// Имя ключа (e.g. "vpc_id", "bucket_name")
    pub key: String,
    /// Значение (JSON — может быть строкой, числом, объектом)
    pub value: Value,
    /// Тип данных: string | number | bool | json
    pub value_type: String,
    pub created: DateTime<Utc>,
}

/// Payload для создания structured output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStructuredOutputCreate {
    pub key: String,
    pub value: Value,
    #[serde(default = "default_value_type")]
    pub value_type: String,
}

fn default_value_type() -> String {
    "string".to_string()
}

/// Batch payload — несколько outputs за раз
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStructuredOutputBatch {
    pub outputs: Vec<TaskStructuredOutputCreate>,
}

/// Ответ с outputs в виде плоского map {key: value}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutputsMap {
    pub task_id: i32,
    pub outputs: std::collections::HashMap<String, Value>,
}
