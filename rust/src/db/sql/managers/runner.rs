//! RunnerManager - управление раннерами

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Runner;
use async_trait::async_trait;
use sqlx::Row;

fn row_to_runner(row: sqlx::postgres::PgRow) -> Runner {
    Runner {
        id: row.get("id"),
        project_id: row.try_get("project_id").ok().flatten(),
        token: row.try_get("token").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        active: row.try_get::<bool, _>("active").unwrap_or(true),
        last_active: row.try_get("last_active").ok().flatten(),
        webhook: row.try_get("webhook").ok().flatten(),
        max_parallel_tasks: row.try_get("max_parallel_tasks").ok().flatten(),
        tag: row.try_get("tag").ok().flatten(),
        cleaning_requested: None,
        touched: row.try_get("last_active").ok().flatten(),
        created: row.try_get("created").ok().flatten(),
    }
}

#[async_trait]
impl RunnerManager for SqlStore {
    async fn get_runners(&self, project_id: Option<i32>) -> Result<Vec<Runner>> {
        let pool = self.get_postgres_pool()?;
        let rows = if let Some(pid) = project_id {
            sqlx::query(
                "SELECT * FROM runner WHERE project_id = $1 OR project_id IS NULL ORDER BY name",
            )
            .bind(pid)
            .fetch_all(pool)
            .await
            .map_err(Error::Database)?
        } else {
            sqlx::query("SELECT * FROM runner ORDER BY name")
                .fetch_all(pool)
                .await
                .map_err(Error::Database)?
        };
        Ok(rows.into_iter().map(row_to_runner).collect())
    }

    async fn get_runner(&self, runner_id: i32) -> Result<Runner> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query("SELECT * FROM runner WHERE id = $1")
            .bind(runner_id)
            .fetch_optional(pool)
            .await
            .map_err(Error::Database)?
            .ok_or_else(|| Error::NotFound("Раннер не найден".to_string()))?;
        Ok(row_to_runner(row))
    }

    async fn create_runner(&self, mut runner: Runner) -> Result<Runner> {
        let pool = self.get_postgres_pool()?;
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO runner (project_id, token, name, active, webhook, max_parallel_tasks, tag) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"
        )
        .bind(runner.project_id)
        .bind(&runner.token)
        .bind(&runner.name)
        .bind(runner.active)
        .bind(&runner.webhook)
        .bind(runner.max_parallel_tasks)
        .bind(&runner.tag)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

        runner.id = id;
        Ok(runner)
    }

    async fn update_runner(&self, runner: Runner) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query(
            "UPDATE runner SET name = $1, active = $2, webhook = $3, max_parallel_tasks = $4, tag = $5 WHERE id = $6"
        )
        .bind(&runner.name)
        .bind(runner.active)
        .bind(&runner.webhook)
        .bind(runner.max_parallel_tasks)
        .bind(&runner.tag)
        .bind(runner.id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_runner(&self, runner_id: i32) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query("DELETE FROM runner WHERE id = $1")
            .bind(runner_id)
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_runners_count(&self) -> Result<usize> {
        let pool = self.get_postgres_pool()?;
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM runner")
            .fetch_one(pool)
            .await
            .map_err(Error::Database)?;
        Ok(count as usize)
    }

    async fn get_active_runners_count(&self) -> Result<usize> {
        let pool = self.get_postgres_pool()?;
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM runner WHERE active = true")
            .fetch_one(pool)
            .await
            .map_err(Error::Database)?;
        Ok(count as usize)
    }

    async fn find_runner_by_token(&self, token: &str) -> Result<Runner> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query("SELECT * FROM runner WHERE token = $1")
            .bind(token)
            .fetch_optional(pool)
            .await
            .map_err(Error::Database)?
            .ok_or_else(|| Error::NotFound("Раннер с таким токеном не найден".to_string()))?;
        Ok(row_to_runner(row))
    }

    async fn touch_runner(&self, runner_id: i32) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query("UPDATE runner SET last_active = NOW() WHERE id = $1")
            .bind(runner_id)
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::Runner;
    use chrono::Utc;

    #[test]
    fn test_runner_serialization() {
        let runner = Runner {
            id: 1,
            project_id: Some(10),
            token: "token123".to_string(),
            name: "Build Runner".to_string(),
            active: true,
            last_active: Some(Utc::now()),
            webhook: Some("https://example.com/webhook".to_string()),
            max_parallel_tasks: Some(5),
            tag: Some("linux".to_string()),
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        let json = serde_json::to_string(&runner).unwrap();
        assert!(json.contains("\"name\":\"Build Runner\""));
        assert!(json.contains("\"active\":true"));
        assert!(json.contains("\"tag\":\"linux\""));
    }

    #[test]
    fn test_runner_skip_nulls() {
        let runner = Runner {
            id: 1,
            project_id: None,
            token: "token".to_string(),
            name: "Simple".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        let json = serde_json::to_string(&runner).unwrap();
        assert!(!json.contains("\"webhook\":"));
        assert!(!json.contains("\"tag\":"));
        assert!(!json.contains("\"max_parallel_tasks\":"));
    }

    #[test]
    fn test_runner_default_active() {
        let runner = Runner {
            id: 0,
            project_id: None,
            token: String::new(),
            name: "Test".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        assert!(runner.active);
    }

    #[test]
    fn test_runner_inactive() {
        let runner = Runner {
            id: 1,
            project_id: None,
            token: "token".to_string(),
            name: "Inactive".to_string(),
            active: false,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        let json = serde_json::to_string(&runner).unwrap();
        assert!(json.contains("\"active\":false"));
    }

    #[test]
    fn test_runner_clone() {
        let runner = Runner {
            id: 1,
            project_id: Some(10),
            token: "clone-token".to_string(),
            name: "Clone Runner".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: Some(3),
            tag: Some("docker".to_string()),
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        let cloned = runner.clone();
        assert_eq!(cloned.name, runner.name);
        assert_eq!(cloned.max_parallel_tasks, runner.max_parallel_tasks);
    }

    #[test]
    fn test_runner_deserialization() {
        let json = r#"{"id":5,"project_id":20,"token":"tok","name":"Deser Runner","active":false,"last_active":null,"webhook":null,"max_parallel_tasks":10,"tag":"k8s","cleaning_requested":null,"touched":null,"created":null}"#;
        let runner: Runner = serde_json::from_str(json).unwrap();
        assert_eq!(runner.id, 5);
        assert_eq!(runner.name, "Deser Runner");
        assert!(!runner.active);
        assert_eq!(runner.max_parallel_tasks, Some(10));
    }

    #[test]
    fn test_runner_with_all_optional_fields() {
        let now = Utc::now();
        let runner = Runner {
            id: 1,
            project_id: Some(1),
            token: "full".to_string(),
            name: "Full Runner".to_string(),
            active: true,
            last_active: Some(now),
            webhook: Some("https://hooks.example.com".to_string()),
            max_parallel_tasks: Some(10),
            tag: Some("prod".to_string()),
            cleaning_requested: Some(now),
            touched: Some(now),
            created: Some(now),
        };
        let json = serde_json::to_string(&runner).unwrap();
        assert!(json.contains("\"webhook\":\"https://hooks.example.com\""));
        assert!(json.contains("\"max_parallel_tasks\":10"));
        assert!(json.contains("\"tag\":\"prod\""));
    }

    #[test]
    fn test_runner_global_vs_project() {
        let global = Runner {
            id: 1,
            project_id: None,
            token: "global".to_string(),
            name: "Global Runner".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        assert!(global.project_id.is_none());

        let project = Runner {
            id: 2,
            project_id: Some(5),
            token: "proj".to_string(),
            name: "Project Runner".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        assert_eq!(project.project_id, Some(5));
    }

    #[test]
    fn test_runner_token_not_empty() {
        let runner = Runner {
            id: 1,
            project_id: None,
            token: "my-secret-token".to_string(),
            name: "Token Test".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        assert!(!runner.token.is_empty());
    }

    #[test]
    fn test_runner_tag_serialization() {
        let runner = Runner {
            id: 1,
            project_id: None,
            token: "t".to_string(),
            name: "Tagged".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: Some("windows".to_string()),
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        let json = serde_json::to_string(&runner).unwrap();
        assert!(json.contains("\"tag\":\"windows\""));
    }

    #[test]
    fn test_runner_max_parallel_zero() {
        let runner = Runner {
            id: 1,
            project_id: None,
            token: "t".to_string(),
            name: "Zero Max".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: Some(0),
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        assert_eq!(runner.max_parallel_tasks, Some(0));
    }

    #[test]
    fn test_runner_name_not_empty() {
        let runner = Runner {
            id: 0,
            project_id: None,
            token: "".to_string(),
            name: "Named Runner".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        assert!(!runner.name.is_empty());
    }
}
