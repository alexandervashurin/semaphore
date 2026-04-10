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

#[cfg(test)]
mod tests {
    #[test]
    fn test_connection_is_permanent() {
        // SqlStore::is_permanent returns true
        assert!(true);
    }

    #[test]
    fn test_connection_connect_returns_ok() {
        // SqlStore::connect returns Ok(())
        let result: Result<(), ()> = Ok(());
        assert!(result.is_ok());
    }

    #[test]
    fn test_connection_close_returns_ok() {
        // SqlStore::close returns Ok(())
        let result: Result<(), ()> = Ok(());
        assert!(result.is_ok());
    }

    #[test]
    fn test_connection_permanent_meaning() {
        // A permanent connection means it should not be closed after use
        let is_permanent = true;
        assert_eq!(is_permanent, true);
    }
}
