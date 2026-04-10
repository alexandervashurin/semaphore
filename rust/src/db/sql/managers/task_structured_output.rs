//! StructuredOutputManager — именованные key-value outputs задачи (FI-PUL-1)

use crate::db::sql::SqlStore;
use crate::db::store::StructuredOutputManager;
use crate::error::{Error, Result};
use crate::models::{TaskOutputsMap, TaskStructuredOutput, TaskStructuredOutputCreate};
use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashMap;

#[async_trait]
impl StructuredOutputManager for SqlStore {
    async fn get_task_structured_outputs(
        &self,
        task_id: i32,
        project_id: i32,
    ) -> Result<Vec<TaskStructuredOutput>> {
        let rows = sqlx::query(
            "SELECT * FROM task_structured_output WHERE task_id = $1 AND project_id = $2 ORDER BY id"
        )
        .bind(task_id)
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.iter().map(row_to_output).collect())
    }

    async fn get_task_outputs_map(&self, task_id: i32, project_id: i32) -> Result<TaskOutputsMap> {
        let outputs = self
            .get_task_structured_outputs(task_id, project_id)
            .await?;
        let map: HashMap<_, _> = outputs.into_iter().map(|o| (o.key, o.value)).collect();
        Ok(TaskOutputsMap {
            task_id,
            outputs: map,
        })
    }

    async fn create_task_structured_output(
        &self,
        task_id: i32,
        project_id: i32,
        payload: TaskStructuredOutputCreate,
    ) -> Result<TaskStructuredOutput> {
        let row = sqlx::query(
            "INSERT INTO task_structured_output (task_id, project_id, key, value, value_type, created) \
             VALUES ($1, $2, $3, $4::jsonb, $5, NOW()) \
             ON CONFLICT (task_id, key) DO UPDATE SET value = EXCLUDED.value, value_type = EXCLUDED.value_type \
             RETURNING *"
        )
        .bind(task_id)
        .bind(project_id)
        .bind(&payload.key)
        .bind(payload.value.to_string())
        .bind(&payload.value_type)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(row_to_output(&row))
    }

    async fn create_task_structured_outputs_batch(
        &self,
        task_id: i32,
        project_id: i32,
        outputs: Vec<TaskStructuredOutputCreate>,
    ) -> Result<()> {
        for output in outputs {
            self.create_task_structured_output(task_id, project_id, output)
                .await?;
        }
        Ok(())
    }

    async fn delete_task_structured_outputs(&self, task_id: i32, project_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM task_structured_output WHERE task_id = $1 AND project_id = $2")
            .bind(task_id)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_template_last_outputs(
        &self,
        template_id: i32,
        project_id: i32,
    ) -> Result<TaskOutputsMap> {
        // Находим последнюю успешную задачу шаблона
        let last_task_id: Option<i32> = sqlx::query_scalar(
            "SELECT id FROM task WHERE template_id = $1 AND project_id = $2 AND status = 'success' \
             ORDER BY created DESC LIMIT 1"
        )
        .bind(template_id)
        .bind(project_id)
        .fetch_optional(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        match last_task_id {
            Some(task_id) => self.get_task_outputs_map(task_id, project_id).await,
            None => Ok(TaskOutputsMap {
                task_id: 0,
                outputs: HashMap::new(),
            }),
        }
    }
}

fn row_to_output(row: &sqlx::postgres::PgRow) -> TaskStructuredOutput {
    let value_str: String = row.try_get("value").unwrap_or_else(|_| "null".into());
    let value = serde_json::from_str(&value_str).unwrap_or(serde_json::Value::Null);
    TaskStructuredOutput {
        id: row.get("id"),
        task_id: row.get("task_id"),
        project_id: row.get("project_id"),
        key: row.get("key"),
        value,
        value_type: row
            .try_get("value_type")
            .unwrap_or_else(|_| "string".into()),
        created: row.get("created"),
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{TaskOutputsMap, TaskStructuredOutput, TaskStructuredOutputBatch, TaskStructuredOutputCreate};
    use chrono::Utc;
    use serde_json::Value;
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
            value_type: "string".to_string(),
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
        outputs.insert("url".to_string(), Value::String("https://example.com".to_string()));
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
    fn test_task_structured_output_with_number_value() {
        let output = TaskStructuredOutput {
            id: 2,
            task_id: 200,
            project_id: 10,
            key: "port".to_string(),
            value: Value::Number(8080.into()),
            value_type: "number".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"value\":8080"));
        assert!(json.contains("\"value_type\":\"number\""));
    }

    #[test]
    fn test_task_structured_output_with_bool_value() {
        let output = TaskStructuredOutput {
            id: 3,
            task_id: 300,
            project_id: 10,
            key: "enabled".to_string(),
            value: Value::Bool(true),
            value_type: "bool".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"value\":true"));
        assert!(json.contains("\"value_type\":\"bool\""));
    }

    #[test]
    fn test_task_structured_output_with_object_value() {
        let output = TaskStructuredOutput {
            id: 4,
            task_id: 400,
            project_id: 10,
            key: "config".to_string(),
            value: serde_json::json!({"region": "us-east-1", "size": "t2.micro"}),
            value_type: "json".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"key\":\"config\""));
        assert!(json.contains("\"region\":\"us-east-1\""));
    }

    #[test]
    fn test_task_structured_output_clone() {
        let output = TaskStructuredOutput {
            id: 1,
            task_id: 100,
            project_id: 10,
            key: "clone_key".to_string(),
            value: Value::String("clone_val".to_string()),
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
            value: Value::Null,
            value_type: "string".to_string(),
        };
        let cloned = create.clone();
        assert_eq!(cloned.key, create.key);
    }

    #[test]
    fn test_task_outputs_map_empty() {
        let map = TaskOutputsMap {
            task_id: 1,
            outputs: HashMap::new(),
        };
        let json = serde_json::to_string(&map).unwrap();
        assert!(json.contains("\"task_id\":1"));
        assert!(json.contains("\"outputs\":{}"));
    }

    #[test]
    fn test_task_structured_output_batch_empty() {
        let batch = TaskStructuredOutputBatch {
            outputs: vec![],
        };
        let json = serde_json::to_string(&batch).unwrap();
        assert!(json.contains("\"outputs\":[]"));
    }

    #[test]
    fn test_task_structured_output_value_types() {
        let value_types = vec!["string", "number", "bool", "json"];
        for vt in value_types {
            let create = TaskStructuredOutputCreate {
                key: "test".to_string(),
                value: Value::String("x".to_string()),
                value_type: vt.to_string(),
            };
            let json = serde_json::to_string(&create).unwrap();
            assert!(json.contains(&format!("\"value_type\":\"{}\"", vt)));
        }
    }

    #[test]
    fn test_task_structured_output_with_null_value() {
        let output = TaskStructuredOutput {
            id: 5,
            task_id: 500,
            project_id: 10,
            key: "nothing".to_string(),
            value: Value::Null,
            value_type: "string".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"value\":null"));
    }
}
