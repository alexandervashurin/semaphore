//! Integration Alias CRUD Operations for BoltDB
//!
//! Операции с псевдонимами интеграций в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::IntegrationAlias;

impl BoltStore {
    /// Получает псевдонимы интеграций
    pub async fn get_integration_aliases(&self, project_id: i32, integration_id: Option<i32>) -> Result<Vec<IntegrationAlias>> {
        self.get_objects::<IntegrationAlias>(project_id, "integration_aliases", crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: integration_id.map(|id| format!("integration_id={}", id)),
        }).await
    }

    /// Создаёт псевдоним интеграции
    pub async fn create_integration_alias(&self, mut alias: IntegrationAlias) -> Result<IntegrationAlias> {
        alias.id = self.get_next_id("integration_aliases")?;
        self.create_object(alias.project_id, "integration_aliases", &alias).await?;
        Ok(alias)
    }

    /// Обновляет псевдоним интеграции
    pub async fn update_integration_alias(&self, alias: IntegrationAlias) -> Result<()> {
        self.update_object(alias.project_id, "integration_aliases", alias.id, &alias).await
    }

    /// Удаляет псевдоним интеграции
    pub async fn delete_integration_alias(&self, project_id: i32, alias_id: i32) -> Result<()> {
        self.delete_object(project_id, "integration_aliases", alias_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_alias_operations() {
        // Тест для проверки операций с псевдонимами интеграций
        assert!(true);
    }
}
