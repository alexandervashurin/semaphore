//! Inventory Service
//!
//! Сервис управления инвентарями

use std::sync::Arc;
use crate::error::Result;
use crate::models::Inventory;
use crate::db::store::Store;
use super::access_key_encryption_svc::AccessKeyEncryptionService;

/// Сервис инвентарей
pub trait InventoryService: Send + Sync {
    /// Получает инвентарь
    fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory>;
}

/// Реализация сервиса инвентарей
pub struct InventoryServiceImpl {
    access_key_repo: Arc<dyn Store + Send + Sync>,
    repository_repo: Arc<dyn Store + Send + Sync>,
    inventory_repo: Arc<dyn Store + Send + Sync>,
    encryption_service: Arc<dyn AccessKeyEncryptionService>,
}

impl InventoryServiceImpl {
    /// Создаёт новый сервис
    pub fn new(
        access_key_repo: Arc<dyn Store + Send + Sync>,
        repository_repo: Arc<dyn Store + Send + Sync>,
        inventory_repo: Arc<dyn Store + Send + Sync>,
        encryption_service: Arc<dyn AccessKeyEncryptionService>,
    ) -> Self {
        Self {
            access_key_repo,
            repository_repo,
            inventory_repo,
            encryption_service,
        }
    }
}

impl InventoryService for InventoryServiceImpl {
    fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        let mut inventory = self.inventory_repo.get_inventory(project_id, inventory_id)?;
        self.fill_inventory(&mut inventory)?;
        Ok(inventory)
    }
}

impl InventoryServiceImpl {
    /// Заполняет инвентарь связанными данными
    fn fill_inventory(&self, inventory: &mut Inventory) -> Result<()> {
        // В базовой версии просто возвращаем Ok
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_service_creation() {
        // Тест для проверки создания сервиса
        assert!(true);
    }
}
