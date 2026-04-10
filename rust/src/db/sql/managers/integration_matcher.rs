//! IntegrationMatcherManager + IntegrationExtractValueManager

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::{IntegrationExtractValue, IntegrationMatcher};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl IntegrationMatcherManager for SqlStore {
    async fn get_integration_matchers(
        &self,
        project_id: i32,
        integration_id: i32,
    ) -> Result<Vec<IntegrationMatcher>> {
        let rows = sqlx::query(
            "SELECT * FROM integration_matcher WHERE integration_id = $1 AND project_id = $2",
        )
        .bind(integration_id)
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(rows
            .into_iter()
            .map(|row| IntegrationMatcher {
                id: row.get("id"),
                integration_id: row.get("integration_id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                body_data_type: row.get("body_data_type"),
                key: row.get("key"),
                matcher_type: row.get("matcher_type"),
                matcher_value: row.get("matcher_value"),
                method: row.get("method"),
            })
            .collect())
    }

    async fn create_integration_matcher(
        &self,
        mut m: IntegrationMatcher,
    ) -> Result<IntegrationMatcher> {
        let row = sqlx::query("INSERT INTO integration_matcher (integration_id, project_id, name, body_data_type, key, matcher_type, matcher_value, method) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id")
                .bind(m.integration_id).bind(m.project_id).bind(&m.name)
                .bind(&m.body_data_type).bind(&m.key).bind(&m.matcher_type)
                .bind(&m.matcher_value).bind(&m.method)
                .fetch_one(self.get_postgres_pool()?)
                .await.map_err(Error::Database)?;
        m.id = row.get("id");
        Ok(m)
    }

    async fn update_integration_matcher(&self, m: IntegrationMatcher) -> Result<()> {
        sqlx::query("UPDATE integration_matcher SET name=$1, body_data_type=$2, key=$3, matcher_type=$4, matcher_value=$5, method=$6 WHERE id=$7 AND project_id=$8")
                .bind(&m.name).bind(&m.body_data_type).bind(&m.key)
                .bind(&m.matcher_type).bind(&m.matcher_value).bind(&m.method)
                .bind(m.id).bind(m.project_id)
                .execute(self.get_postgres_pool()?)
                .await.map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_integration_matcher(
        &self,
        project_id: i32,
        _integration_id: i32,
        matcher_id: i32,
    ) -> Result<()> {
        sqlx::query("DELETE FROM integration_matcher WHERE id=$1 AND project_id=$2")
            .bind(matcher_id)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[async_trait]
impl IntegrationExtractValueManager for SqlStore {
    async fn get_integration_extract_values(
        &self,
        project_id: i32,
        integration_id: i32,
    ) -> Result<Vec<IntegrationExtractValue>> {
        let rows = sqlx::query(
            "SELECT * FROM integration_extract_value WHERE integration_id=$1 AND project_id=$2",
        )
        .bind(integration_id)
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(rows
            .into_iter()
            .map(|row| IntegrationExtractValue {
                id: row.get("id"),
                integration_id: row.get("integration_id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                value_source: row.get("value_source"),
                body_data_type: row.get("body_data_type"),
                key: row.get("key"),
                variable: row.get("variable"),
                value_name: row.get("value_name"),
                value_type: row.get("value_type"),
            })
            .collect())
    }

    async fn create_integration_extract_value(
        &self,
        mut v: IntegrationExtractValue,
    ) -> Result<IntegrationExtractValue> {
        let row = sqlx::query("INSERT INTO integration_extract_value (integration_id, project_id, name, value_source, body_data_type, key, variable, value_name, value_type) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) RETURNING id")
                .bind(v.integration_id).bind(v.project_id).bind(&v.name)
                .bind(&v.value_source).bind(&v.body_data_type).bind(&v.key)
                .bind(&v.variable).bind(&v.value_name).bind(&v.value_type)
                .fetch_one(self.get_postgres_pool()?)
                .await.map_err(Error::Database)?;
        v.id = row.get("id");
        Ok(v)
    }

    async fn update_integration_extract_value(&self, v: IntegrationExtractValue) -> Result<()> {
        sqlx::query("UPDATE integration_extract_value SET name=$1,value_source=$2,body_data_type=$3,key=$4,variable=$5,value_name=$6,value_type=$7 WHERE id=$8 AND project_id=$9")
                .bind(&v.name).bind(&v.value_source).bind(&v.body_data_type)
                .bind(&v.key).bind(&v.variable).bind(&v.value_name).bind(&v.value_type)
                .bind(v.id).bind(v.project_id)
                .execute(self.get_postgres_pool()?)
                .await.map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_integration_extract_value(
        &self,
        project_id: i32,
        _integration_id: i32,
        value_id: i32,
    ) -> Result<()> {
        sqlx::query("DELETE FROM integration_extract_value WHERE id=$1 AND project_id=$2")
            .bind(value_id)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{IntegrationExtractValue, IntegrationMatcher};

    #[test]
    fn test_integration_matcher_serialization() {
        let matcher = IntegrationMatcher {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Task Started".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.event".to_string()),
            matcher_type: "equals".to_string(),
            matcher_value: "task_started".to_string(),
            method: "POST".to_string(),
        };
        let json = serde_json::to_string(&matcher).unwrap();
        assert!(json.contains("\"name\":\"Task Started\""));
        assert!(json.contains("\"method\":\"POST\""));
    }

    #[test]
    fn test_integration_matcher_default_values() {
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
        let json = serde_json::to_string(&matcher).unwrap();
        assert!(json.contains("\"key\":null"));
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
            matcher_type: "contains".to_string(),
            matcher_value: "deploy".to_string(),
            method: "GET".to_string(),
        };
        let cloned = matcher.clone();
        assert_eq!(cloned.name, matcher.name);
        assert_eq!(cloned.matcher_type, matcher.matcher_type);
    }

    #[test]
    fn test_integration_matcher_methods() {
        let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH"];
        for method in methods {
            let matcher = IntegrationMatcher {
                id: 0,
                integration_id: 0,
                project_id: 0,
                name: "test".to_string(),
                body_data_type: "json".to_string(),
                key: None,
                matcher_type: "equals".to_string(),
                matcher_value: "x".to_string(),
                method: method.to_string(),
            };
            let json = serde_json::to_string(&matcher).unwrap();
            assert!(json.contains(&format!("\"method\":\"{}\"", method)));
        }
    }

    #[test]
    fn test_integration_matcher_body_data_types() {
        let types = vec!["json", "xml", "form", "text"];
        for body_type in types {
            let matcher = IntegrationMatcher {
                id: 0,
                integration_id: 0,
                project_id: 0,
                name: "test".to_string(),
                body_data_type: body_type.to_string(),
                key: None,
                matcher_type: "equals".to_string(),
                matcher_value: "x".to_string(),
                method: "POST".to_string(),
            };
            let json = serde_json::to_string(&matcher).unwrap();
            assert!(json.contains(&format!("\"body_data_type\":\"{}\"", body_type)));
        }
    }

    #[test]
    fn test_integration_extract_value_serialization() {
        let ev = IntegrationExtractValue {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Deploy URL".to_string(),
            value_source: "body".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.url".to_string()),
            variable: Some("DEPLOY_URL".to_string()),
            value_name: "url".to_string(),
            value_type: "string".to_string(),
        };
        let json = serde_json::to_string(&ev).unwrap();
        assert!(json.contains("\"name\":\"Deploy URL\""));
        assert!(json.contains("\"key\":\"$.url\""));
    }

    #[test]
    fn test_integration_extract_value_default_values() {
        let ev = IntegrationExtractValue {
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
        let json = serde_json::to_string(&ev).unwrap();
        assert!(json.contains("\"key\":null"));
        assert!(json.contains("\"variable\":null"));
    }

    #[test]
    fn test_integration_extract_value_clone() {
        let ev = IntegrationExtractValue {
            id: 1,
            integration_id: 10,
            project_id: 5,
            name: "Clone EV".to_string(),
            value_source: "header".to_string(),
            body_data_type: "text".to_string(),
            key: Some("X-Key".to_string()),
            variable: Some("VAR".to_string()),
            value_name: "key".to_string(),
            value_type: "string".to_string(),
        };
        let cloned = ev.clone();
        assert_eq!(cloned.name, ev.name);
        assert_eq!(cloned.value_source, ev.value_source);
    }

    #[test]
    fn test_integration_extract_value_types() {
        let types = vec!["string", "number", "bool", "json"];
        for vtype in types {
            let ev = IntegrationExtractValue {
                id: 0,
                integration_id: 0,
                project_id: 0,
                name: "test".to_string(),
                value_source: "body".to_string(),
                body_data_type: "json".to_string(),
                key: None,
                variable: None,
                value_name: "test".to_string(),
                value_type: vtype.to_string(),
            };
            let json = serde_json::to_string(&ev).unwrap();
            assert!(json.contains(&format!("\"value_type\":\"{}\"", vtype)));
        }
    }

    #[test]
    fn test_integration_extract_value_sources() {
        let sources = vec!["body", "header", "query", "path"];
        for source in sources {
            let ev = IntegrationExtractValue {
                id: 0,
                integration_id: 0,
                project_id: 0,
                name: "test".to_string(),
                value_source: source.to_string(),
                body_data_type: "json".to_string(),
                key: None,
                variable: None,
                value_name: "test".to_string(),
                value_type: "string".to_string(),
            };
            assert_eq!(ev.value_source, source);
        }
    }

    #[test]
    fn test_integration_matcher_with_all_fields_some() {
        let matcher = IntegrationMatcher {
            id: 42,
            integration_id: 100,
            project_id: 200,
            name: "full".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.data.id".to_string()),
            matcher_type: "regex".to_string(),
            matcher_value: "\\d+".to_string(),
            method: "POST".to_string(),
        };
        assert_eq!(matcher.id, 42);
        assert_eq!(matcher.key, Some("$.data.id".to_string()));
    }

    #[test]
    fn test_integration_extract_value_with_all_fields_some() {
        let ev = IntegrationExtractValue {
            id: 7,
            integration_id: 77,
            project_id: 777,
            name: "full".to_string(),
            value_source: "body".to_string(),
            body_data_type: "json".to_string(),
            key: Some("$.id".to_string()),
            variable: Some("ID".to_string()),
            value_name: "id".to_string(),
            value_type: "number".to_string(),
        };
        assert_eq!(ev.id, 7);
        assert_eq!(ev.variable, Some("ID".to_string()));
    }
}
