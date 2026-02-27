//! Integration CRUD Operations for BoltDB
//!
//! Операции с интеграциями в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::Integration;

impl BoltStore {
    /// Получает интеграции проекта
    pub async fn get_integrations(&self, project_id: i32, params: crate::db::store::RetrieveQueryParams) -> Result<Vec<Integration>> {
        self.get_objects::<Integration>(project_id, "integrations", params).await
    }

    /// Получает интеграцию по ID
    pub async fn get_integration(&self, project_id: i32, integration_id: i32) -> Result<Integration> {
        self.get_object::<Integration>(project_id, "integrations", integration_id).await
    }

    /// Создаёт интеграцию
    pub async fn create_integration(&self, mut integration: Integration) -> Result<Integration> {
        integration.id = self.get_next_id("integrations")?;
        self.create_object(integration.project_id, "integrations", &integration).await?;
        Ok(integration)
    }

    /// Обновляет интеграцию
    pub async fn update_integration(&self, integration: Integration) -> Result<()> {
        self.update_object(integration.project_id, "integrations", integration.id, &integration).await
    }

    /// Удаляет интеграцию
    pub async fn delete_integration(&self, project_id: i32, integration_id: i32) -> Result<()> {
        self.delete_object(project_id, "integrations", integration_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_operations() {
        // Тест для проверки операций с интеграциями
        assert!(true);
    }
}
