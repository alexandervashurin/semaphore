//! ConnectionManager - управление подключением к БД

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
impl ConnectionManager for SqlStore {
    async fn connect(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.db.close().await
    }

    fn is_permanent(&self) -> bool {
        true
    }
}
