//! IntegrationMatcherManager + IntegrationExtractValueManager

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::{IntegrationMatcher, IntegrationExtractValue};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl IntegrationMatcherManager for SqlStore {
    async fn get_integration_matchers(&self, project_id: i32, integration_id: i32) -> Result<Vec<IntegrationMatcher>> {
        let rows = sqlx::query("SELECT * FROM integration_matcher WHERE integration_id = $1 AND project_id = $2")
                .bind(integration_id).bind(project_id)
                .fetch_all(self.get_postgres_pool()?)
                .await.map_err(Error::Database)?;
            Ok(rows.into_iter().map(|row| IntegrationMatcher {
                id: row.get("id"), integration_id: row.get("integration_id"),
                project_id: row.get("project_id"), name: row.get("name"),
                body_data_type: row.get("body_data_type"), key: row.get("key"),
                matcher_type: row.get("matcher_type"), matcher_value: row.get("matcher_value"),
                method: row.get("method"),
            }).collect())
    }

    async fn create_integration_matcher(&self, mut m: IntegrationMatcher) -> Result<IntegrationMatcher> {
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

    async fn delete_integration_matcher(&self, project_id: i32, _integration_id: i32, matcher_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM integration_matcher WHERE id=$1 AND project_id=$2")
                .bind(matcher_id).bind(project_id)
                .execute(self.get_postgres_pool()?)
                .await.map_err(Error::Database)?;
            Ok(())
    }
}

#[async_trait]
impl IntegrationExtractValueManager for SqlStore {
    async fn get_integration_extract_values(&self, project_id: i32, integration_id: i32) -> Result<Vec<IntegrationExtractValue>> {
        let rows = sqlx::query("SELECT * FROM integration_extract_value WHERE integration_id=$1 AND project_id=$2")
                .bind(integration_id).bind(project_id)
                .fetch_all(self.get_postgres_pool()?)
                .await.map_err(Error::Database)?;
            Ok(rows.into_iter().map(|row| IntegrationExtractValue {
                id: row.get("id"), integration_id: row.get("integration_id"),
                project_id: row.get("project_id"), name: row.get("name"),
                value_source: row.get("value_source"), body_data_type: row.get("body_data_type"),
                key: row.get("key"), variable: row.get("variable"),
                value_name: row.get("value_name"), value_type: row.get("value_type"),
            }).collect())
    }

    async fn create_integration_extract_value(&self, mut v: IntegrationExtractValue) -> Result<IntegrationExtractValue> {
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

    async fn delete_integration_extract_value(&self, project_id: i32, _integration_id: i32, value_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM integration_extract_value WHERE id=$1 AND project_id=$2")
                .bind(value_id).bind(project_id)
                .execute(self.get_postgres_pool()?)
                .await.map_err(Error::Database)?;
            Ok(())
    }
}
