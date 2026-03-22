//! Integration CRUD - операции с интеграциями
//!
//! Аналог db/sql/integration.go из Go версии (часть 1: CRUD)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_integration(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает все интеграции проекта
    pub async fn get_integrations(&self, project_id: i32) -> Result<Vec<Integration>> {
        let rows = sqlx::query(
            "SELECT * FROM integration WHERE project_id = $1 ORDER BY name"
        )
        .bind(project_id)
        .fetch_all(self.pg_pool_integration()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| Integration {
            id: row.get("id"),
            project_id: row.get("project_id"),
            name: row.get("name"),
            template_id: row.get("template_id"),
            auth_method: row.try_get("auth_method").ok().unwrap_or_default(),
            auth_header: row.try_get("auth_header").ok().flatten(),
            auth_secret_id: row.try_get("auth_secret_id").ok().flatten(),
        }).collect())
    }

    /// Получает интеграцию по ID
    pub async fn get_integration(&self, project_id: i32, integration_id: i32) -> Result<Integration> {
        let row = sqlx::query(
            "SELECT * FROM integration WHERE id = $1 AND project_id = $2"
        )
        .bind(integration_id)
        .bind(project_id)
        .fetch_one(self.pg_pool_integration()?)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Интеграция не найдена".to_string()),
            _ => Error::Database(e),
        })?;

        Ok(Integration {
            id: row.get("id"),
            project_id: row.get("project_id"),
            name: row.get("name"),
            template_id: row.get("template_id"),
            auth_method: row.try_get("auth_method").ok().unwrap_or_default(),
            auth_header: row.try_get("auth_header").ok().flatten(),
            auth_secret_id: row.try_get("auth_secret_id").ok().flatten(),
        })
    }

    /// Создаёт новую интеграцию
    pub async fn create_integration(&self, mut integration: Integration) -> Result<Integration> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO integration (project_id, name, template_id, auth_method, auth_header, auth_secret_id) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(integration.project_id)
        .bind(&integration.name)
        .bind(integration.template_id)
        .bind(&integration.auth_method)
        .bind(&integration.auth_header)
        .bind(integration.auth_secret_id)
        .fetch_one(self.pg_pool_integration()?)
        .await
        .map_err(Error::Database)?;

        integration.id = id;
        Ok(integration)
    }

    /// Обновляет интеграцию
    pub async fn update_integration(&self, integration: Integration) -> Result<()> {
        sqlx::query(
            "UPDATE integration SET name = $1, template_id = $2, auth_method = $3, \
             auth_header = $4, auth_secret_id = $5 WHERE id = $6 AND project_id = $7"
        )
        .bind(&integration.name)
        .bind(integration.template_id)
        .bind(&integration.auth_method)
        .bind(&integration.auth_header)
        .bind(integration.auth_secret_id)
        .bind(integration.id)
        .bind(integration.project_id)
        .execute(self.pg_pool_integration()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет интеграцию
    pub async fn delete_integration(&self, project_id: i32, integration_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM integration WHERE id = $1 AND project_id = $2")
            .bind(integration_id)
            .bind(project_id)
            .execute(self.pg_pool_integration()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDb {
        db: SqlDb,
        _temp: tempfile::NamedTempFile,
    }

    async fn create_test_db() -> TestDb {
        let (db_path, temp) = crate::db::sql::init::test_sqlite_url();

        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();

        // Создаём таблицу integration
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS integration (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                template_id INTEGER,
                auth_method TEXT NOT NULL DEFAULT 'none',
                auth_header TEXT,
                auth_secret_id INTEGER
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();

        TestDb { db, _temp: temp }
    }

    #[tokio::test]
    async fn test_create_and_get_integration() {
        let TestDb { db, _temp } = create_test_db().await;

        let integration = Integration {
            id: 0,
            project_id: 1,
            name: "Test Integration".to_string(),
            template_id: 1,
            auth_method: "none".to_string(),
            auth_header: None,
            auth_secret_id: None,
        };

        let created = db.create_integration(integration.clone()).await.unwrap();
        assert!(created.id > 0);

        let retrieved = db.get_integration(1, created.id).await.unwrap();
        assert_eq!(retrieved.name, "Test Integration");

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_get_integrations() {
        let TestDb { db, _temp } = create_test_db().await;

        // Создаём несколько интеграций
        for i in 0..5 {
            let integration = Integration {
                id: 0,
                project_id: 1,
                name: format!("Integration {}", i),
                template_id: 1,
                auth_method: "none".to_string(),
                auth_header: None,
                auth_secret_id: None,
            };
            db.create_integration(integration).await.unwrap();
        }

        let integrations = db.get_integrations(1).await.unwrap();
        assert!(integrations.len() >= 5);

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_integration() {
        let TestDb { db, _temp } = create_test_db().await;

        let integration = Integration {
            id: 0,
            project_id: 1,
            name: "Test Integration".to_string(),
            template_id: 1,
            auth_method: "none".to_string(),
            auth_header: None,
            auth_secret_id: None,
        };

        let created = db.create_integration(integration).await.unwrap();

        let mut updated = created.clone();
        updated.name = "Updated Integration".to_string();

        db.update_integration(updated).await.unwrap();

        let retrieved = db.get_integration(1, created.id).await.unwrap();
        assert_eq!(retrieved.name, "Updated Integration");

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_integration() {
        let TestDb { db, _temp } = create_test_db().await;

        let integration = Integration {
            id: 0,
            project_id: 1,
            name: "Test Integration".to_string(),
            template_id: 1,
            auth_method: "none".to_string(),
            auth_header: None,
            auth_secret_id: None,
        };

        let created = db.create_integration(integration).await.unwrap();

        db.delete_integration(1, created.id).await.unwrap();

        let result = db.get_integration(1, created.id).await;
        assert!(result.is_err());

        // Cleanup
        let _ = db.close().await;
    }
}
