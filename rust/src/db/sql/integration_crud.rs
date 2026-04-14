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
        let rows = sqlx::query("SELECT * FROM integration WHERE project_id = $1 ORDER BY name")
            .bind(project_id)
            .fetch_all(self.pg_pool_integration()?)
            .await
            .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| Integration {
                id: row.get("id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                template_id: row.get("template_id"),
                auth_method: row.try_get("auth_method").ok().unwrap_or_default(),
                auth_header: row.try_get("auth_header").ok().flatten(),
                auth_secret_id: row.try_get("auth_secret_id").ok().flatten(),
            })
            .collect())
    }

    /// Получает интеграцию по ID
    pub async fn get_integration(
        &self,
        project_id: i32,
        integration_id: i32,
    ) -> Result<Integration> {
        let row = sqlx::query("SELECT * FROM integration WHERE id = $1 AND project_id = $2")
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
             auth_header = $4, auth_secret_id = $5 WHERE id = $6 AND project_id = $7",
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

    #[test]
    fn test_integration_struct_fields() {
        let integration = Integration {
            id: 1,
            project_id: 10,
            name: "GitHub Webhook".to_string(),
            template_id: 5,
            auth_method: "hmac".to_string(),
            auth_header: Some("X-Hub-Signature".to_string()),
            auth_secret_id: Some(3),
        };
        assert_eq!(integration.id, 1);
        assert_eq!(integration.name, "GitHub Webhook");
        assert_eq!(integration.auth_method, "hmac");
    }

    #[test]
    fn test_integration_clone() {
        let integration = Integration {
            id: 1,
            project_id: 10,
            name: "Clone Test".to_string(),
            template_id: 5,
            auth_method: "token".to_string(),
            auth_header: None,
            auth_secret_id: None,
        };
        let cloned = integration.clone();
        assert_eq!(cloned.id, integration.id);
        assert_eq!(cloned.name, integration.name);
    }

    #[test]
    fn test_integration_serialization() {
        let integration = Integration {
            id: 1,
            project_id: 10,
            name: "Serialize Test".to_string(),
            template_id: 5,
            auth_method: "none".to_string(),
            auth_header: None,
            auth_secret_id: None,
        };
        let json = serde_json::to_string(&integration).unwrap();
        assert!(json.contains("\"name\":\"Serialize Test\""));
        assert!(json.contains("\"auth_method\":\"none\""));
    }

    #[test]
    fn test_integration_deserialization() {
        let json = r#"{"id":10,"project_id":20,"name":"Deserialized","template_id":15,"auth_method":"token","auth_header":"Authorization","auth_secret_id":5}"#;
        let integration: Integration = serde_json::from_str(json).unwrap();
        assert_eq!(integration.id, 10);
        assert_eq!(integration.name, "Deserialized");
    }

    #[test]
    fn test_integration_skip_nulls() {
        let integration = Integration {
            id: 1,
            project_id: 10,
            name: "Null Test".to_string(),
            template_id: 5,
            auth_method: "none".to_string(),
            auth_header: None,
            auth_secret_id: None,
        };
        let json = serde_json::to_string(&integration).unwrap();
        assert!(!json.contains("auth_header"));
        assert!(!json.contains("auth_secret_id"));
    }

    #[test]
    fn test_integration_auth_methods() {
        let methods = ["none", "hmac", "token", "basic", "oauth2"];
        for method in &methods {
            let integration = Integration {
                id: 1,
                project_id: 1,
                name: "Test".to_string(),
                template_id: 1,
                auth_method: method.to_string(),
                auth_header: None,
                auth_secret_id: None,
            };
            assert_eq!(integration.auth_method, *method);
        }
    }

    #[test]
    fn test_integration_with_all_fields() {
        let integration = Integration {
            id: 1,
            project_id: 10,
            name: "Full Auth".to_string(),
            template_id: 5,
            auth_method: "hmac".to_string(),
            auth_header: Some("X-Signature-256".to_string()),
            auth_secret_id: Some(42),
        };
        assert_eq!(integration.auth_header, Some("X-Signature-256".to_string()));
        assert_eq!(integration.auth_secret_id, Some(42));
    }

    #[test]
    fn test_integration_zero_values() {
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
        assert!(integration.name.is_empty());
    }

    #[test]
    fn test_integration_vec_serialization() {
        let integrations = vec![
            Integration {
                id: 1,
                project_id: 10,
                name: "A".to_string(),
                template_id: 1,
                auth_method: "none".to_string(),
                auth_header: None,
                auth_secret_id: None,
            },
            Integration {
                id: 2,
                project_id: 10,
                name: "B".to_string(),
                template_id: 2,
                auth_method: "token".to_string(),
                auth_header: Some("Auth".to_string()),
                auth_secret_id: Some(1),
            },
        ];
        let json = serde_json::to_string(&integrations).unwrap();
        assert!(json.contains("\"A\""));
        assert!(json.contains("\"B\""));
    }

    #[test]
    fn test_integration_debug() {
        let integration = Integration {
            id: 1,
            project_id: 10,
            name: "Debug".to_string(),
            template_id: 5,
            auth_method: "hmac".to_string(),
            auth_header: None,
            auth_secret_id: None,
        };
        let debug_str = format!("{:?}", integration);
        assert!(debug_str.contains("Debug"));
        assert!(debug_str.contains("Integration"));
    }
}
