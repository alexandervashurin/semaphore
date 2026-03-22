//! Task Output - операции с выводами задач
//!
//! Аналог db/sql/task.go из Go версии (часть 2: TaskOutput)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_task_output(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает выводы задачи
    pub async fn get_task_outputs(&self, project_id: i32, task_id: i32, params: &RetrieveQueryParams) -> Result<Vec<TaskOutput>> {
        let limit = params.count.unwrap_or(100) as i64;
        let offset = params.offset as i64;

        let rows = sqlx::query(
            "SELECT * FROM task_output WHERE task_id = $1 AND project_id = $2 \
             ORDER BY time ASC LIMIT $3 OFFSET $4"
        )
        .bind(task_id)
        .bind(project_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pg_pool_task_output()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| TaskOutput {
            id: row.get("id"),
            task_id: row.get("task_id"),
            project_id: row.get("project_id"),
            time: row.get("time"),
            output: row.get("output"),
            stage_id: row.try_get("stage_id").ok().flatten(),
        }).collect())
    }

    /// Создаёт вывод задачи
    pub async fn create_task_output(&self, mut output: TaskOutput) -> Result<TaskOutput> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO task_output (task_id, project_id, time, output, stage_id) \
             VALUES ($1, $2, $3, $4, $5) RETURNING id"
        )
        .bind(output.task_id)
        .bind(output.project_id)
        .bind(output.time)
        .bind(&output.output)
        .bind(output.stage_id)
        .fetch_one(self.pg_pool_task_output()?)
        .await
        .map_err(Error::Database)?;

        output.id = id;
        Ok(output)
    }

    /// Создаёт несколько выводов задачи (batch)
    pub async fn create_task_output_batch(&self, outputs: Vec<TaskOutput>) -> Result<()> {
        for output in outputs {
            self.create_task_output(output).await?;
        }
        Ok(())
    }

    /// Удаляет выводы задачи
    pub async fn delete_task_output(&self, project_id: i32, task_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM task_output WHERE task_id = $1 AND project_id = $2")
            .bind(task_id)
            .bind(project_id)
            .execute(self.pg_pool_task_output()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    /// Получает количество выводов задачи
    pub async fn get_task_output_count(&self, project_id: i32, task_id: i32) -> Result<usize> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM task_output WHERE task_id = $1 AND project_id = $2"
        )
        .bind(task_id)
        .bind(project_id)
        .fetch_one(self.pg_pool_task_output()?)
        .await
        .map_err(Error::Database)?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    struct TestDb {
        db: SqlDb,
        _temp: tempfile::NamedTempFile,
    }

    async fn create_test_db() -> TestDb {
        let (db_path, temp) = crate::db::sql::init::test_sqlite_url();

        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();

        // Создаём таблицу task_output
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task_output (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                output TEXT NOT NULL,
                time DATETIME NOT NULL,
                stage_id INTEGER
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();

        TestDb { db, _temp: temp }
    }

    #[tokio::test]
    async fn test_create_and_get_task_output() {
        let TestDb { db, _temp } = create_test_db().await;

        let output = TaskOutput {
            id: 0,
            task_id: 1,
            project_id: 1,
            output: "Test output line 1".to_string(),
            time: Utc::now(),
            stage_id: None,
        };

        let created = db.create_task_output(output.clone()).await.unwrap();
        assert!(created.id > 0);

        let params = RetrieveQueryParams {
            offset: 0,
            count: Some(10),
            sort_by: None,
            sort_inverted: false,
            filter: None,
        };

        let outputs = db.get_task_outputs(1, 1, &params).await.unwrap();
        assert!(outputs.len() >= 1);
        assert_eq!(outputs[0].output, "Test output line 1");

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_create_task_output_batch() {
        let TestDb { db, _temp } = create_test_db().await;

        let outputs = vec![
            TaskOutput {
                id: 0,
                task_id: 1,
                project_id: 1,
                output: "Output 1".to_string(),
                time: Utc::now(),
                stage_id: None,
            },
            TaskOutput {
                id: 0,
                task_id: 1,
                project_id: 1,
                output: "Output 2".to_string(),
                time: Utc::now(),
                stage_id: None,
            },
            TaskOutput {
                id: 0,
                task_id: 1,
                project_id: 1,
                output: "Output 3".to_string(),
                time: Utc::now(),
                stage_id: None,
            },
        ];

        db.create_task_output_batch(outputs).await.unwrap();

        let params = RetrieveQueryParams {
            offset: 0,
            count: Some(10),
            sort_by: None,
            sort_inverted: false,
            filter: None,
        };

        let outputs = db.get_task_outputs(1, 1, &params).await.unwrap();
        assert_eq!(outputs.len(), 3);

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_task_output() {
        let TestDb { db, _temp } = create_test_db().await;

        let output = TaskOutput {
            id: 0,
            task_id: 1,
            project_id: 1,
            output: "Test output".to_string(),
            time: Utc::now(),
            stage_id: None,
        };

        db.create_task_output(output).await.unwrap();

        db.delete_task_output(1, 1).await.unwrap();

        let params = RetrieveQueryParams {
            offset: 0,
            count: Some(10),
            sort_by: None,
            sort_inverted: false,
            filter: None,
        };

        let outputs = db.get_task_outputs(1, 1, &params).await.unwrap();
        assert!(outputs.is_empty());

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_get_task_output_count() {
        let TestDb { db, _temp } = create_test_db().await;

        let outputs = vec![
            TaskOutput {
                id: 0,
                task_id: 1,
                project_id: 1,
                output: "Output 1".to_string(),
                time: Utc::now(),
                stage_id: None,
            },
            TaskOutput {
                id: 0,
                task_id: 1,
                project_id: 1,
                output: "Output 2".to_string(),
                time: Utc::now(),
                stage_id: None,
            },
        ];

        for output in outputs {
            db.create_task_output(output).await.unwrap();
        }

        let count = db.get_task_output_count(1, 1).await.unwrap();
        assert_eq!(count, 2);

        // Cleanup
        let _ = db.close().await;
    }
}
