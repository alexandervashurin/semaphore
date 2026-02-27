//! Inventory CRUD Operations
//!
//! Операции с инвентарями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Inventory;

impl SqlDb {
    /// Получает инвентари проекта
    pub async fn get_inventories(&self, project_id: i32) -> Result<Vec<Inventory>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let inventories = sqlx::query_as::<_, Inventory>(
                    "SELECT * FROM inventory WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(inventories)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает инвентарь по ID
    pub async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let inventory = sqlx::query_as::<_, Inventory>(
                    "SELECT * FROM inventory WHERE id = ? AND project_id = ?"
                )
                .bind(inventory_id)
                .bind(project_id)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                inventory.ok_or(Error::NotFound("Inventory not found".to_string()))
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Создаёт инвентарь
    pub async fn create_inventory(&self, mut inventory: Inventory) -> Result<Inventory> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO inventory (project_id, name, type, inventory_data, ssh_key_id)
                     VALUES (?, ?, ?, ?, ?)"
                )
                .bind(inventory.project_id)
                .bind(&inventory.name)
                .bind(&inventory.inventory_type)
                .bind(&inventory.inventory_data)
                .bind(inventory.ssh_key_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                inventory.id = result.last_insert_rowid() as i32;
                Ok(inventory)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Обновляет инвентарь
    pub async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE inventory SET name = ?, type = ?, inventory_data = ?, ssh_key_id = ?
                     WHERE id = ? AND project_id = ?"
                )
                .bind(&inventory.name)
                .bind(&inventory.inventory_type)
                .bind(&inventory.inventory_data)
                .bind(inventory.ssh_key_id)
                .bind(inventory.id)
                .bind(inventory.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Удаляет инвентарь
    pub async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM inventory WHERE id = ? AND project_id = ?")
                    .bind(inventory_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
}
