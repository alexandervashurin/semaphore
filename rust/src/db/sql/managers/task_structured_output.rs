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
