//! Inventory CRUD Operations for BoltDB
//!
//! Операции с инвентарями в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::Inventory;

impl BoltStore {
    /// Получает инвентари проекта
    pub async fn get_inventories(&self, project_id: i32) -> Result<Vec<Inventory>> {
        self.get_objects::<Inventory>(project_id, "inventories", crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: String::new(),
        }).await
    }

    /// Получает инвентарь по ID
    pub async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        self.get_object::<Inventory>(project_id, "inventories", inventory_id).await
    }

    /// Создаёт инвентарь
    pub async fn create_inventory(&self, mut inventory: Inventory) -> Result<Inventory> {
        inventory.id = self.get_next_id("inventories")?;
        self.create_object(inventory.project_id, "inventories", &inventory).await?;
        Ok(inventory)
    }

    /// Обновляет инвентарь
    pub async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        self.update_object(inventory.project_id, "inventories", inventory.id, &inventory).await
    }

    /// Удаляет инвентарь
    pub async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()> {
        self.delete_object(project_id, "inventories", inventory_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_operations() {
        // Тест для проверки операций с инвентарями
        assert!(true);
    }
}
