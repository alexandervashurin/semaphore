//! Integration Extract Value - операции с IntegrationExtractValue
//!
//! Аналог db/sql/integration.go из Go версии (часть 3: IntegrationExtractValue)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_extract(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает все extract values для интеграции
    pub async fn get_integration_extract_values(
        &self,
        project_id: i32,
        integration_id: i32,
    ) -> Result<Vec<IntegrationExtractValue>> {
        let rows = sqlx::query(
            "SELECT * FROM integration_extract_value WHERE integration_id = $1 AND project_id = $2",
        )
        .bind(integration_id)
        .bind(project_id)
        .fetch_all(self.pg_pool_extract()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| IntegrationExtractValue {
                id: row.get("id"),
                integration_id: row.get("integration_id"),
                project_id: row.get("project_id"),
                name: row.try_get("name").ok().unwrap_or_default(),
                value_source: row.get("value_source"),
                body_data_type: row.try_get("body_data_type").ok().unwrap_or_default(),
                key: row.try_get("key").ok().flatten(),
                variable: row.try_get("variable").ok().flatten(),
                value_name: row.get("value_name"),
                value_type: row.get("value_type"),
            })
            .collect())
    }

    /// Создаёт IntegrationExtractValue
    pub async fn create_integration_extract_value(
        &self,
        mut value: IntegrationExtractValue,
    ) -> Result<IntegrationExtractValue> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO integration_extract_value \
             (integration_id, project_id, name, value_source, body_data_type, key, variable, value_name, value_type) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"
        )
        .bind(value.integration_id)
        .bind(value.project_id)
        .bind(&value.name)
        .bind(&value.value_source)
        .bind(&value.body_data_type)
        .bind(&value.key)
        .bind(&value.variable)
        .bind(&value.value_name)
        .bind(&value.value_type)
        .fetch_one(self.pg_pool_extract()?)
        .await
        .map_err(Error::Database)?;

        value.id = id;
        Ok(value)
    }

    /// Обновляет IntegrationExtractValue
    pub async fn update_integration_extract_value(
        &self,
        value: IntegrationExtractValue,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE integration_extract_value SET name = $1, value_source = $2, \
             body_data_type = $3, key = $4, variable = $5, value_name = $6, value_type = $7 \
             WHERE id = $8 AND integration_id = $9 AND project_id = $10",
        )
        .bind(&value.name)
        .bind(&value.value_source)
        .bind(&value.body_data_type)
        .bind(&value.key)
        .bind(&value.variable)
        .bind(&value.value_name)
        .bind(&value.value_type)
        .bind(value.id)
        .bind(value.integration_id)
        .bind(value.project_id)
        .execute(self.pg_pool_extract()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет IntegrationExtractValue
    pub async fn delete_integration_extract_value(
        &self,
        project_id: i32,
        integration_id: i32,
        value_id: i32,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM integration_extract_value WHERE id = $1 AND integration_id = $2 AND project_id = $3"
        )
        .bind(value_id)
        .bind(integration_id)
        .bind(project_id)
        .execute(self.pg_pool_extract()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_extract_value_struct_fields() {
        let value = IntegrationExtractValue {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Extract URL".to_string(),
            value_source: "body".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.url".to_string()),
            variable: Some("DEPLOY_URL".to_string()),
            value_name: "url".to_string(),
            value_type: "string".to_string(),
        };
        assert_eq!(value.id, 1);
        assert_eq!(value.integration_id, 10);
        assert_eq!(value.name, "Extract URL");
        assert_eq!(value.value_source, "body");
        assert_eq!(value.value_type, "string");
    }

    #[test]
    fn test_integration_extract_value_clone() {
        let value = IntegrationExtractValue {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Clone Extract".to_string(),
            value_source: "header".to_string(),
            body_data_type: "text".to_string(),
            key: None,
            variable: None,
            value_name: "header_val".to_string(),
            value_type: "string".to_string(),
        };
        let cloned = value.clone();
        assert_eq!(cloned.id, value.id);
        assert_eq!(cloned.name, value.name);
        assert_eq!(cloned.key, value.key);
    }

    #[test]
    fn test_integration_extract_value_serialization() {
        let value = IntegrationExtractValue {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Test Extract".to_string(),
            value_source: "body".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.data.id".to_string()),
            variable: Some("DATA_ID".to_string()),
            value_name: "data_id".to_string(),
            value_type: "integer".to_string(),
        };
        let json = serde_json::to_string(&value).unwrap();
        assert!(json.contains("\"name\":\"Test Extract\""));
        assert!(json.contains("\"key\":\"$.data.id\""));
        assert!(json.contains("\"value_type\":\"integer\""));
    }

    #[test]
    fn test_integration_extract_value_deserialization() {
        let json = r#"{"id":5,"integration_id":20,"project_id":10,"name":"Deserialized","value_source":"body","body_data_type":"json","key":"$.name","variable":"NAME","value_name":"name","value_type":"string"}"#;
        let value: IntegrationExtractValue = serde_json::from_str(json).unwrap();
        assert_eq!(value.id, 5);
        assert_eq!(value.name, "Deserialized");
        assert_eq!(value.key, Some("$.name".to_string()));
    }

    #[test]
    fn test_integration_extract_value_null_fields() {
        let value = IntegrationExtractValue {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Null Fields".to_string(),
            value_source: "body".to_string(),
            body_data_type: "json".to_string(),
            key: None,
            variable: None,
            value_name: "test".to_string(),
            value_type: "string".to_string(),
        };
        let json = serde_json::to_string(&value).unwrap();
        assert!(json.contains("\"key\":null"));
        assert!(json.contains("\"variable\":null"));
    }

    #[test]
    fn test_integration_extract_value_all_value_sources() {
        let sources = ["body", "header", "query", "cookie", "path"];
        for source in &sources {
            let value = IntegrationExtractValue {
                id: 1, integration_id: 1, project_id: 1, name: "Test".to_string(),
                value_source: source.to_string(), body_data_type: "json".to_string(),
                key: None, variable: None, value_name: "test".to_string(),
                value_type: "string".to_string(),
            };
            let json = serde_json::to_string(&value).unwrap();
            assert!(json.contains(source));
        }
    }

    #[test]
    fn test_integration_extract_value_all_data_types() {
        let data_types = ["json", "xml", "text", "form", "yaml"];
        for dt in &data_types {
            let value = IntegrationExtractValue {
                id: 1, integration_id: 1, project_id: 1, name: "Test".to_string(),
                value_source: "body".to_string(), body_data_type: dt.to_string(),
                key: None, variable: None, value_name: "test".to_string(),
                value_type: "string".to_string(),
            };
            assert_eq!(value.body_data_type, *dt);
        }
    }

    #[test]
    fn test_integration_extract_value_zero_id() {
        let value = IntegrationExtractValue {
            id: 0,
            integration_id: 0,
            project_id: 0,
            name: String::new(),
            value_source: String::new(),
            body_data_type: String::new(),
            key: None,
            variable: None,
            value_name: String::new(),
            value_type: String::new(),
        };
        assert_eq!(value.id, 0);
        assert!(value.name.is_empty());
    }

    #[test]
    fn test_integration_extract_value_vec_serialization() {
        let values = vec![
            IntegrationExtractValue { id: 1, integration_id: 10, project_id: 5, name: "A".to_string(), value_source: "body".to_string(), body_data_type: "json".to_string(), key: None, variable: None, value_name: "a".to_string(), value_type: "string".to_string() },
            IntegrationExtractValue { id: 2, integration_id: 10, project_id: 5, name: "B".to_string(), value_source: "header".to_string(), body_data_type: "text".to_string(), key: Some("X-Val".to_string()), variable: None, value_name: "b".to_string(), value_type: "string".to_string() },
        ];
        let json = serde_json::to_string(&values).unwrap();
        assert!(json.contains("\"A\""));
        assert!(json.contains("\"B\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"id\":2"));
    }

    #[test]
    fn test_integration_extract_value_debug() {
        let value = IntegrationExtractValue {
            id: 1, integration_id: 10, project_id: 5, name: "Debug".to_string(),
            value_source: "body".to_string(), body_data_type: "json".to_string(),
            key: None, variable: None, value_name: "debug".to_string(),
            value_type: "string".to_string(),
        };
        let debug_str = format!("{:?}", value);
        assert!(debug_str.contains("Debug"));
        assert!(debug_str.contains("IntegrationExtractValue"));
    }
}
