//! Inventory CRUD Operations
//!
//! Адаптер для декомпозированных модулей
//!
//! Новые модули: sqlite::inventory, postgres::inventory, mysql::inventory

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Inventory;

impl SqlDb {
    /// Получает инвентари проекта
    pub async fn get_inventories(&self, project_id: i32) -> Result<Vec<Inventory>> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::inventory::get_inventories(pool, project_id).await
    }

    /// Получает инвентарь по ID
    pub async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::inventory::get_inventory(pool, project_id, inventory_id).await
    }

    /// Создаёт инвентарь
    pub async fn create_inventory(&self, inventory: Inventory) -> Result<Inventory> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::inventory::create_inventory(pool, inventory).await
    }

    /// Обновляет инвентарь
    pub async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::inventory::update_inventory(pool, inventory).await
    }

    /// Удаляет инвентарь
    pub async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::inventory::delete_inventory(pool, project_id, inventory_id).await
    }
}
