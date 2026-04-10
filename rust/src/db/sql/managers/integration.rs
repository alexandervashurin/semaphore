//! IntegrationManager - управление интеграциями

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Integration;
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl IntegrationManager for SqlStore {
    async fn get_integrations(&self, project_id: i32) -> Result<Vec<Integration>> {
        let pool = self.get_postgres_pool()?;
        let rows = sqlx::query(
            "SELECT id, project_id, name, template_id, auth_method, auth_header, auth_secret_id FROM integration WHERE project_id = $1 ORDER BY name"
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| Integration {
                id: row.get("id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                template_id: row.try_get("template_id").unwrap_or(0),
                auth_method: row.try_get("auth_method").unwrap_or_default(),
                auth_header: row.try_get("auth_header").ok().flatten(),
                auth_secret_id: row.try_get("auth_secret_id").ok().flatten(),
            })
            .collect())
    }

    async fn get_integration(&self, project_id: i32, integration_id: i32) -> Result<Integration> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query(
            "SELECT id, project_id, name, template_id, auth_method, auth_header, auth_secret_id FROM integration WHERE id = $1 AND project_id = $2"
        )
        .bind(integration_id)
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?
        .ok_or_else(|| Error::NotFound("Интеграция не найдена".to_string()))?;

        Ok(Integration {
            id: row.get("id"),
            project_id: row.get("project_id"),
            name: row.get("name"),
            template_id: row.try_get("template_id").unwrap_or(0),
            auth_method: row.try_get("auth_method").unwrap_or_default(),
            auth_header: row.try_get("auth_header").ok().flatten(),
            auth_secret_id: row.try_get("auth_secret_id").ok().flatten(),
        })
    }

    async fn create_integration(&self, mut integration: Integration) -> Result<Integration> {
        let pool = self.get_postgres_pool()?;
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO integration (project_id, name, template_id, auth_method, auth_header, auth_secret_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(integration.project_id)
        .bind(&integration.name)
        .bind(integration.template_id)
        .bind(&integration.auth_method)
        .bind(&integration.auth_header)
        .bind(integration.auth_secret_id)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

        integration.id = id;
        Ok(integration)
    }

    async fn update_integration(&self, integration: Integration) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query(
            "UPDATE integration SET name = $1, template_id = $2, auth_method = $3, auth_header = $4, auth_secret_id = $5 WHERE id = $6 AND project_id = $7"
        )
        .bind(&integration.name)
        .bind(integration.template_id)
        .bind(&integration.auth_method)
        .bind(&integration.auth_header)
        .bind(integration.auth_secret_id)
        .bind(integration.id)
        .bind(integration.project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_integration(&self, project_id: i32, integration_id: i32) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query("DELETE FROM integration WHERE id = $1 AND project_id = $2")
            .bind(integration_id)
            .bind(project_id)
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::Integration;

    #[test]
    fn test_integration_serialization() {
        let integration = Integration {
            id: 1,
            project_id: 10,
            name: "GitHub Integration".to_string(),
            template_id: 5,
            auth_method: "hmac".to_string(),
            auth_header: Some("X-Hub-Signature".to_string()),
            auth_secret_id: Some(3),
        };
        let json = serde_json::to_string(&integration).unwrap();
        assert!(json.contains("\"name\":\"GitHub Integration\""));
        assert!(json.contains("\"auth_method\":\"hmac\""));
    }

    #[test]
    fn test_integration_no_auth() {
        let integration = Integration {
            id: 2,
            project_id: 1,
            name: "Public Webhook".to_string(),
            template_id: 1,
            auth_method: "none".to_string(),
            auth_header: None,
            auth_secret_id: None,
        };
        let json = serde_json::to_string(&integration).unwrap();
        assert!(!json.contains("auth_header"));
        assert!(!json.contains("auth_secret_id"));
    }

    #[test]
    fn test_integration_default_values() {
        let integration = Integration {
            id: 0,
            project_id: 0,
            name: String::new(),
            template_id: 0,
            auth_method: String::new(),
            auth_header: None,
            auth_secret_id: None,
        };
        assert_eq!(integration.id, 0);
        assert_eq!(integration.template_id, 0);
    }

    #[test]
    fn test_integration_clone() {
        let integration = Integration {
            id: 1,
            project_id: 10,
            name: "Clone Test".to_string(),
            template_id: 5,
            auth_method: "token".to_string(),
            auth_header: Some("Authorization".to_string()),
            auth_secret_id: Some(1),
        };
        let cloned = integration.clone();
        assert_eq!(cloned.name, integration.name);
        assert_eq!(cloned.auth_method, integration.auth_method);
    }

    #[test]
    fn test_integration_deserialization() {
        let json = r#"{"id":5,"project_id":20,"name":"Test Integration","template_id":10,"auth_method":"none","auth_header":null,"auth_secret_id":null}"#;
        let integration: Integration = serde_json::from_str(json).unwrap();
        assert_eq!(integration.id, 5);
        assert_eq!(integration.name, "Test Integration");
        assert_eq!(integration.auth_method, "none");
    }

    #[test]
    fn test_integration_with_token_auth() {
        let integration = Integration {
            id: 3,
            project_id: 2,
            name: "Token Auth Integration".to_string(),
            template_id: 7,
            auth_method: "token".to_string(),
            auth_header: Some("X-Auth-Token".to_string()),
            auth_secret_id: Some(42),
        };
        let json = serde_json::to_string(&integration).unwrap();
        assert!(json.contains("\"auth_method\":\"token\""));
        assert!(json.contains("\"auth_header\":\"X-Auth-Token\""));
    }

    #[test]
    fn test_integration_project_id() {
        let integration = Integration {
            id: 1,
            project_id: 100,
            name: "Project Test".to_string(),
            template_id: 1,
            auth_method: String::new(),
            auth_header: None,
            auth_secret_id: None,
        };
        assert_eq!(integration.project_id, 100);
    }

    #[test]
    fn test_integration_name_empty_deserialize() {
        let json = r#"{"id":1,"project_id":1,"name":"","template_id":1,"auth_method":"","auth_header":null,"auth_secret_id":null}"#;
        let integration: Integration = serde_json::from_str(json).unwrap();
        assert!(integration.name.is_empty());
    }
}
