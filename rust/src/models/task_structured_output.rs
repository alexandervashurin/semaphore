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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_task_structured_output_serialization() {
        let output = TaskStructuredOutput {
            id: 1,
            task_id: 100,
            project_id: 10,
            key: "vpc_id".to_string(),
            value: Value::String("vpc-12345".to_string()),
            value_type: "string".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"key\":\"vpc_id\""));
        assert!(json.contains("\"value\":\"vpc-12345\""));
    }

    #[test]
    fn test_task_structured_output_create() {
        let create = TaskStructuredOutputCreate {
            key: "bucket_name".to_string(),
            value: Value::String("my-bucket".to_string()),
            value_type: "string".to_string(),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"key\":\"bucket_name\""));
    }

    #[test]
    fn test_task_structured_output_create_default_type() {
        let create = TaskStructuredOutputCreate {
            key: "default_type".to_string(),
            value: Value::String("test".to_string()),
            value_type: default_value_type(),
        };
        assert_eq!(create.value_type, "string");
    }

    #[test]
    fn test_task_structured_output_batch() {
        let batch = TaskStructuredOutputBatch {
            outputs: vec![
                TaskStructuredOutputCreate {
                    key: "key1".to_string(),
                    value: Value::String("val1".to_string()),
                    value_type: "string".to_string(),
                },
                TaskStructuredOutputCreate {
                    key: "key2".to_string(),
                    value: Value::Number(42.into()),
                    value_type: "number".to_string(),
                },
            ],
        };
        let json = serde_json::to_string(&batch).unwrap();
        assert!(json.contains("\"outputs\":["));
    }

    #[test]
    fn test_task_outputs_map() {
        let mut outputs = HashMap::new();
        outputs.insert(
            "url".to_string(),
            Value::String("https://example.com".to_string()),
        );
        outputs.insert("count".to_string(), Value::Number(5.into()));

        let map = TaskOutputsMap {
            task_id: 100,
            outputs,
        };
        let json = serde_json::to_string(&map).unwrap();
        assert!(json.contains("\"task_id\":100"));
        assert!(json.contains("\"outputs\":{"));
    }

    #[test]
    fn test_task_structured_output_clone() {
        let output = TaskStructuredOutput {
            id: 1,
            task_id: 100,
            project_id: 10,
            key: "clone_key".to_string(),
            value: Value::String("val".to_string()),
            value_type: "string".to_string(),
            created: Utc::now(),
        };
        let cloned = output.clone();
        assert_eq!(cloned.key, output.key);
        assert_eq!(cloned.value, output.value);
    }

    #[test]
    fn test_task_structured_output_create_clone() {
        let create = TaskStructuredOutputCreate {
            key: "clone_create".to_string(),
            value: Value::Bool(true),
            value_type: "bool".to_string(),
        };
        let cloned = create.clone();
        assert_eq!(cloned.key, create.key);
        assert_eq!(cloned.value, create.value);
    }

    #[test]
    fn test_task_structured_output_batch_clone() {
        let batch = TaskStructuredOutputBatch {
            outputs: vec![TaskStructuredOutputCreate {
                key: "k1".to_string(),
                value: Value::String("v1".to_string()),
                value_type: "string".to_string(),
            }],
        };
        let cloned = batch.clone();
        assert_eq!(cloned.outputs.len(), batch.outputs.len());
    }

    #[test]
    fn test_task_structured_output_with_number_value() {
        let output = TaskStructuredOutput {
            id: 2,
            task_id: 200,
            project_id: 20,
            key: "port".to_string(),
            value: Value::Number(8080.into()),
            value_type: "number".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"key\":\"port\""));
        assert!(json.contains("\"value\":8080"));
    }

    #[test]
    fn test_task_structured_output_with_bool_value() {
        let output = TaskStructuredOutput {
            id: 3,
            task_id: 300,
            project_id: 30,
            key: "enabled".to_string(),
            value: Value::Bool(true),
            value_type: "bool".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"key\":\"enabled\""));
        assert!(json.contains("\"value\":true"));
    }

    #[test]
    fn test_task_structured_output_debug() {
        let output = TaskStructuredOutput {
            id: 1,
            task_id: 1,
            project_id: 1,
            key: "debug".to_string(),
            value: Value::Null,
            value_type: "string".to_string(),
            created: Utc::now(),
        };
        let debug_str = format!("{:?}", output);
        assert!(debug_str.contains("TaskStructuredOutput"));
    }

    #[test]
    fn test_task_structured_output_batch_empty() {
        let batch = TaskStructuredOutputBatch { outputs: vec![] };
        let json = serde_json::to_string(&batch).unwrap();
        assert!(json.contains("\"outputs\":[]"));
    }

    #[test]
    fn test_task_outputs_map_empty() {
        let map = TaskOutputsMap {
            task_id: 1,
            outputs: HashMap::new(),
        };
        let json = serde_json::to_string(&map).unwrap();
        assert!(json.contains("\"outputs\":{}"));
    }

    #[test]
    fn test_task_structured_output_create_with_object_value() {
        let create = TaskStructuredOutputCreate {
            key: "config".to_string(),
            value: Value::Object(serde_json::Map::new()),
            value_type: "json".to_string(),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"key\":\"config\""));
        assert!(json.contains("\"value_type\":\"json\""));
    }

    #[test]
    fn test_task_structured_output_roundtrip() {
        let original = TaskStructuredOutput {
            id: 10,
            task_id: 20,
            project_id: 5,
            key: "output_key".to_string(),
            value: Value::String("output_value".to_string()),
            value_type: "string".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: TaskStructuredOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(original.id, restored.id);
        assert_eq!(original.key, restored.key);
        assert_eq!(original.value, restored.value);
    }
}
