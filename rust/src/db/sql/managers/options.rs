//! OptionsManager - управление опциями

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashMap;

#[async_trait]
impl OptionsManager for SqlStore {
    async fn get_options(&self) -> Result<HashMap<String, String>> {
        let query = "SELECT key, value FROM option";
            let rows = sqlx::query(query)
                .fetch_all(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;

            Ok(rows.into_iter().map(|row| {
                let key: String = row.get("key");
                let value: String = row.get("value");
                (key, value)
            }).collect())
    }

    async fn get_option(&self, key: &str) -> Result<Option<String>> {
        let query = "SELECT value FROM option WHERE key = $1";
            let result = sqlx::query_scalar::<_, String>(query)
                .bind(key)
                .fetch_optional(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
            Ok(result)
    }

    async fn set_option(&self, key: &str, value: &str) -> Result<()> {
        let query = "INSERT INTO option (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value";
            sqlx::query(query)
                .bind(key)
                .bind(value)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_option(&self, key: &str) -> Result<()> {
        let query = "DELETE FROM option WHERE key = $1";
            sqlx::query(query)
                .bind(key)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }
}

