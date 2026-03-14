//! ConnectionManager - управление подключением к БД

use crate::db::sql::SqlStore;
use crate::db::sql::SqlDialect;
use crate::db::store::*;
use crate::error::{Error, Result};
use async_trait::async_trait;

#[async_trait]
impl ConnectionManager for SqlStore {
    async fn connect(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?.close().await,
            SqlDialect::PostgreSQL => self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?.close().await,
            SqlDialect::MySQL => self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?.close().await,
        }
        Ok(())
    }

    fn is_permanent(&self) -> bool {
        true
    }
}

