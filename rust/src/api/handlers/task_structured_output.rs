//! Task Structured Output Handlers (FI-PUL-1 — Pulumi Outputs)
//!
//! Именованные key-value выходы задачи.
//! Позволяют передавать output одного шаблона как input другого.
//! Парсятся из stdout по маркеру: VELUM_OUTPUT: {"key":"value"}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::StructuredOutputManager;
use crate::models::{TaskStructuredOutputBatch, TaskStructuredOutputCreate};

/// GET /api/project/{project_id}/tasks/{task_id}/outputs
pub async fn get_task_outputs(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let outputs = state
        .store
        .get_task_structured_outputs(task_id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(outputs)))
}

/// GET /api/project/{project_id}/tasks/{task_id}/outputs/map
/// Возвращает outputs как плоский map {key: value} — для использования в extra_vars
pub async fn get_task_outputs_map(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let map = state
        .store
        .get_task_outputs_map(task_id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(map)))
}

/// POST /api/project/{project_id}/tasks/{task_id}/outputs
pub async fn create_task_output(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
    _auth: AuthUser,
    Json(payload): Json<TaskStructuredOutputCreate>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let output = state
        .store
        .create_task_structured_output(task_id, project_id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok((StatusCode::CREATED, Json(json!(output))))
}

/// POST /api/project/{project_id}/tasks/{task_id}/outputs/batch
/// Batch-запись нескольких outputs за раз
pub async fn create_task_outputs_batch(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
    _auth: AuthUser,
    Json(payload): Json<TaskStructuredOutputBatch>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    state
        .store
        .create_task_structured_outputs_batch(task_id, project_id, payload.outputs)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/project/{project_id}/templates/{template_id}/last-outputs
/// Outputs последней успешной задачи шаблона — для ссылок между шаблонами
pub async fn get_template_last_outputs(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let map = state
        .store
        .get_template_last_outputs(template_id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(map)))
}

#[cfg(test)]
mod tests {
    use crate::models::task_structured_output::{
        TaskOutputsMap, TaskStructuredOutput, TaskStructuredOutputBatch, TaskStructuredOutputCreate,
    };
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_task_structured_output_create_serialization() {
        let create = TaskStructuredOutputCreate {
            key: "vpc_id".to_string(),
            value: json!("vpc-12345"),
            value_type: "string".to_string(),
        };
        let json_str = serde_json::to_string(&create).unwrap();
        assert!(json_str.contains("\"key\":\"vpc_id\""));
        assert!(json_str.contains("\"value\":\"vpc-12345\""));
    }

    #[test]
    fn test_task_structured_output_create_with_number() {
        let create = TaskStructuredOutputCreate {
            key: "instance_count".to_string(),
            value: json!(5),
            value_type: "number".to_string(),
        };
        let json_str = serde_json::to_string(&create).unwrap();
        assert!(json_str.contains("\"value\":5"));
        assert!(json_str.contains("\"value_type\":\"number\""));
    }

    #[test]
    fn test_task_structured_output_create_with_bool() {
        let create = TaskStructuredOutputCreate {
            key: "enabled".to_string(),
            value: json!(true),
            value_type: "bool".to_string(),
        };
        let json_str = serde_json::to_string(&create).unwrap();
        assert!(json_str.contains("\"value\":true"));
        assert!(json_str.contains("\"value_type\":\"bool\""));
    }

    #[test]
    fn test_task_structured_output_create_with_object() {
        let create = TaskStructuredOutputCreate {
            key: "config".to_string(),
            value: json!({"region": "us-east-1", "size": "large"}),
            value_type: "json".to_string(),
        };
        let json_str = serde_json::to_string(&create).unwrap();
        assert!(json_str.contains("\"config\""));
        assert!(json_str.contains("\"region\":\"us-east-1\""));
    }

    #[test]
    fn test_task_structured_output_create_deserialization() {
        let json_str = r#"{"key":"bucket","value":"my-bucket","value_type":"string"}"#;
        let create: TaskStructuredOutputCreate = serde_json::from_str(json_str).unwrap();
        assert_eq!(create.key, "bucket");
        assert_eq!(create.value, json!("my-bucket"));
    }

    #[test]
    fn test_task_structured_output_full() {
        let output = TaskStructuredOutput {
            id: 1,
            task_id: 100,
            project_id: 10,
            key: "subnet_id".to_string(),
            value: json!("subnet-abc123"),
            value_type: "string".to_string(),
            created: Utc::now(),
        };
        let json_str = serde_json::to_string(&output).unwrap();
        assert!(json_str.contains("\"key\":\"subnet_id\""));
        assert!(json_str.contains("\"task_id\":100"));
        let deserialized: TaskStructuredOutput = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.key, "subnet_id");
    }

    #[test]
    fn test_task_structured_output_batch_serialization() {
        let batch = TaskStructuredOutputBatch {
            outputs: vec![
                TaskStructuredOutputCreate {
                    key: "key1".to_string(),
                    value: json!("val1"),
                    value_type: "string".to_string(),
                },
                TaskStructuredOutputCreate {
                    key: "key2".to_string(),
                    value: json!(42),
                    value_type: "number".to_string(),
                },
            ],
        };
        let json_str = serde_json::to_string(&batch).unwrap();
        assert!(json_str.contains("\"outputs\":["));
        assert!(json_str.contains("\"key1\""));
        assert!(json_str.contains("\"key2\""));
    }

    #[test]
    fn test_task_structured_output_batch_deserialization() {
        let json_str = r#"{"outputs":[{"key":"a","value":1,"value_type":"number"},{"key":"b","value":2,"value_type":"number"}]}"#;
        let batch: TaskStructuredOutputBatch = serde_json::from_str(json_str).unwrap();
        assert_eq!(batch.outputs.len(), 2);
        assert_eq!(batch.outputs[0].key, "a");
        assert_eq!(batch.outputs[1].key, "b");
    }

    #[test]
    fn test_task_outputs_map_serialization() {
        let mut outputs = HashMap::new();
        outputs.insert("url".to_string(), json!("https://example.com"));
        outputs.insert("count".to_string(), json!(10));

        let map = TaskOutputsMap {
            task_id: 42,
            outputs,
        };
        let json_str = serde_json::to_string(&map).unwrap();
        assert!(json_str.contains("\"task_id\":42"));
        assert!(json_str.contains("\"outputs\":"));
    }

    #[test]
    fn test_task_outputs_map_deserialization() {
        let json_str = r#"{"task_id":1,"outputs":{"key1":"val1","key2":123}}"#;
        let map: TaskOutputsMap = serde_json::from_str(json_str).unwrap();
        assert_eq!(map.task_id, 1);
        assert_eq!(map.outputs["key1"], json!("val1"));
        assert_eq!(map.outputs["key2"], json!(123));
    }

    #[test]
    fn test_task_structured_output_clone() {
        let output = TaskStructuredOutput {
            id: 1,
            task_id: 100,
            project_id: 10,
            key: "clone_key".to_string(),
            value: json!("clone_val"),
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
            key: "clone".to_string(),
            value: json!("test"),
            value_type: "string".to_string(),
        };
        let cloned = create.clone();
        assert_eq!(cloned.key, create.key);
    }

    #[test]
    fn test_task_structured_output_batch_clone() {
        let batch = TaskStructuredOutputBatch {
            outputs: vec![TaskStructuredOutputCreate {
                key: "k".to_string(),
                value: json!("v"),
                value_type: "string".to_string(),
            }],
        };
        let cloned = batch.clone();
        assert_eq!(cloned.outputs.len(), batch.outputs.len());
    }

    #[test]
    fn test_task_outputs_map_clone() {
        let mut outputs = HashMap::new();
        outputs.insert("test".to_string(), json!(1));
        let map = TaskOutputsMap {
            task_id: 1,
            outputs,
        };
        let cloned = map.clone();
        assert_eq!(cloned.task_id, map.task_id);
    }

    #[test]
    fn test_task_structured_output_create_array_value() {
        let create = TaskStructuredOutputCreate {
            key: "host_list".to_string(),
            value: json!(["host1", "host2", "host3"]),
            value_type: "json".to_string(),
        };
        let json_str = serde_json::to_string(&create).unwrap();
        assert!(json_str.contains("\"host_list\""));
        assert!(json_str.contains("[\"host1\",\"host2\",\"host3\"]"));
    }
}
