//! Integration Matcher - операции с IntegrationMatcher
//!
//! Аналог db/sql/integration.go из Go версии (часть 2: IntegrationMatcher)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_matcher(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает все matcher'ы для интеграции
    pub async fn get_integration_matchers(
        &self,
        project_id: i32,
        integration_id: i32,
    ) -> Result<Vec<IntegrationMatcher>> {
        let rows = sqlx::query(
            "SELECT * FROM integration_matcher WHERE integration_id = $1 AND project_id = $2",
        )
        .bind(integration_id)
        .bind(project_id)
        .fetch_all(self.pg_pool_matcher()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| IntegrationMatcher {
                id: row.get("id"),
                integration_id: row.get("integration_id"),
                project_id: row.get("project_id"),
                name: row.try_get("name").ok().unwrap_or_default(),
                body_data_type: row.try_get("body_data_type").ok().unwrap_or_default(),
                key: row.try_get("key").ok().flatten(),
                matcher_type: row.get("matcher_type"),
                matcher_value: row.get("matcher_value"),
                method: row.try_get("method").ok().unwrap_or_default(),
            })
            .collect())
    }

    /// Создаёт IntegrationMatcher
    pub async fn create_integration_matcher(
        &self,
        mut matcher: IntegrationMatcher,
    ) -> Result<IntegrationMatcher> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO integration_matcher \
             (integration_id, project_id, name, body_data_type, key, matcher_type, matcher_value, method) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(matcher.integration_id)
        .bind(matcher.project_id)
        .bind(&matcher.name)
        .bind(&matcher.body_data_type)
        .bind(&matcher.key)
        .bind(&matcher.matcher_type)
        .bind(&matcher.matcher_value)
        .bind(&matcher.method)
        .fetch_one(self.pg_pool_matcher()?)
        .await
        .map_err(Error::Database)?;

        matcher.id = id;
        Ok(matcher)
    }

    /// Обновляет IntegrationMatcher
    pub async fn update_integration_matcher(&self, matcher: IntegrationMatcher) -> Result<()> {
        sqlx::query(
            "UPDATE integration_matcher SET name = $1, body_data_type = $2, key = $3, \
             matcher_type = $4, matcher_value = $5, method = $6 \
             WHERE id = $7 AND integration_id = $8 AND project_id = $9",
        )
        .bind(&matcher.name)
        .bind(&matcher.body_data_type)
        .bind(&matcher.key)
        .bind(&matcher.matcher_type)
        .bind(&matcher.matcher_value)
        .bind(&matcher.method)
        .bind(matcher.id)
        .bind(matcher.integration_id)
        .bind(matcher.project_id)
        .execute(self.pg_pool_matcher()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет IntegrationMatcher
    pub async fn delete_integration_matcher(
        &self,
        project_id: i32,
        integration_id: i32,
        matcher_id: i32,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM integration_matcher WHERE id = $1 AND integration_id = $2 AND project_id = $3"
        )
        .bind(matcher_id)
        .bind(integration_id)
        .bind(project_id)
        .execute(self.pg_pool_matcher()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_matcher_struct_fields() {
        let matcher = IntegrationMatcher {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Event Matcher".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.event".to_string()),
            matcher_type: "equals".to_string(),
            matcher_value: "task_started".to_string(),
            method: "POST".to_string(),
        };
        assert_eq!(matcher.id, 1);
        assert_eq!(matcher.name, "Event Matcher");
        assert_eq!(matcher.matcher_type, "equals");
        assert_eq!(matcher.method, "POST");
    }

    #[test]
    fn test_integration_matcher_clone() {
        let matcher = IntegrationMatcher {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Clone Matcher".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.event".to_string()),
            matcher_type: "equals".to_string(),
            matcher_value: "deploy".to_string(),
            method: "POST".to_string(),
        };
        let cloned = matcher.clone();
        assert_eq!(cloned.id, matcher.id);
        assert_eq!(cloned.name, matcher.name);
        assert_eq!(cloned.matcher_type, matcher.matcher_type);
    }

    #[test]
    fn test_integration_matcher_serialization() {
        let matcher = IntegrationMatcher {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Serialize Matcher".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.action".to_string()),
            matcher_type: "regex".to_string(),
            matcher_value: "deploy.*".to_string(),
            method: "POST".to_string(),
        };
        let json = serde_json::to_string(&matcher).unwrap();
        assert!(json.contains("\"name\":\"Serialize Matcher\""));
        assert!(json.contains("\"matcher_type\":\"regex\""));
        assert!(json.contains("\"method\":\"POST\""));
    }

    #[test]
    fn test_integration_matcher_deserialization() {
        let json = r#"{"id":5,"integration_id":20,"project_id":10,"name":"Deserialized","body_data_type":"json","key":"$.id","matcher_type":"equals","matcher_value":"123","method":"GET"}"#;
        let matcher: IntegrationMatcher = serde_json::from_str(json).unwrap();
        assert_eq!(matcher.id, 5);
        assert_eq!(matcher.name, "Deserialized");
        assert_eq!(matcher.method, "GET");
    }

    #[test]
    fn test_integration_matcher_null_key() {
        let matcher = IntegrationMatcher {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Null Key".to_string(),
            body_data_type: "text".to_string(),
            key: None,
            matcher_type: "contains".to_string(),
            matcher_value: "substring".to_string(),
            method: "POST".to_string(),
        };
        let json = serde_json::to_string(&matcher).unwrap();
        assert!(json.contains("\"key\":null"));
    }

    #[test]
    fn test_integration_matcher_matcher_types() {
        let types = ["equals", "contains", "regex", "starts_with", "ends_with", "not_equals"];
        for mt in &types {
            let matcher = IntegrationMatcher {
                id: 1, integration_id: 1, project_id: 1, name: "Test".to_string(),
                body_data_type: "json".to_string(), key: None,
                matcher_type: mt.to_string(), matcher_value: "val".to_string(),
                method: "POST".to_string(),
            };
            assert_eq!(matcher.matcher_type, *mt);
        }
    }

    #[test]
    fn test_integration_matcher_http_methods() {
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"];
        for method in &methods {
            let matcher = IntegrationMatcher {
                id: 1, integration_id: 1, project_id: 1, name: "Test".to_string(),
                body_data_type: "json".to_string(), key: None,
                matcher_type: "equals".to_string(), matcher_value: "val".to_string(),
                method: method.to_string(),
            };
            assert_eq!(matcher.method, *method);
        }
    }

    #[test]
    fn test_integration_matcher_zero_values() {
        let matcher = IntegrationMatcher {
            id: 0,
            integration_id: 0,
            project_id: 0,
            name: String::new(),
            body_data_type: String::new(),
            key: None,
            matcher_type: String::new(),
            matcher_value: String::new(),
            method: String::new(),
        };
        assert_eq!(matcher.id, 0);
        assert!(matcher.name.is_empty());
        assert!(matcher.key.is_none());
    }

    #[test]
    fn test_integration_matcher_vec_serialization() {
        let matchers = vec![
            IntegrationMatcher { id: 1, integration_id: 10, project_id: 5, name: "A".to_string(), body_data_type: "json".to_string(), key: None, matcher_type: "equals".to_string(), matcher_value: "a".to_string(), method: "POST".to_string() },
            IntegrationMatcher { id: 2, integration_id: 10, project_id: 5, name: "B".to_string(), body_data_type: "text".to_string(), key: Some("X".to_string()), matcher_type: "contains".to_string(), matcher_value: "b".to_string(), method: "GET".to_string() },
        ];
        let json = serde_json::to_string(&matchers).unwrap();
        assert!(json.contains("\"A\""));
        assert!(json.contains("\"B\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"id\":2"));
    }

    #[test]
    fn test_integration_matcher_debug() {
        let matcher = IntegrationMatcher {
            id: 1, integration_id: 10, project_id: 5, name: "Debug".to_string(),
            body_data_type: "json".to_string(), key: None,
            matcher_type: "equals".to_string(), matcher_value: "test".to_string(),
            method: "POST".to_string(),
        };
        let debug_str = format!("{:?}", matcher);
        assert!(debug_str.contains("Debug"));
        assert!(debug_str.contains("IntegrationMatcher"));
    }
}
