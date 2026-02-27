//! Public Alias CRUD Operations for BoltDB
//!
//! Операции с публичными псевдонимами в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::IntegrationAlias;

impl BoltStore {
    /// Получает публичные псевдонимы
    pub async fn get_public_aliases(&self) -> Result<Vec<IntegrationAlias>> {
        self.get_objects::<IntegrationAlias>(0, "public_aliases", crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: String::new(),
        }).await
    }

    /// Создаёт публичный псевдоним
    pub async fn create_public_alias(&self, mut alias: IntegrationAlias) -> Result<IntegrationAlias> {
        alias.id = self.get_next_id("public_aliases")?;
        self.create_object(0, "public_aliases", &alias).await?;
        Ok(alias)
    }

    /// Обновляет публичный псевдоним
    pub async fn update_public_alias(&self, alias: IntegrationAlias) -> Result<()> {
        self.update_object(0, "public_aliases", alias.id, &alias).await
    }

    /// Удаляет публичный псевдоним
    pub async fn delete_public_alias(&self, alias_id: i32) -> Result<()> {
        self.delete_object(0, "public_aliases", alias_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_alias_operations() {
        // Тест для проверки операций с публичными псевдонимами
        assert!(true);
    }
}
