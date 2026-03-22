//! InventoryManager - управление инвентарями
//!
//! Реализация трейта InventoryManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::models::{Inventory, InventoryType};
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl InventoryManager for SqlStore {
    async fn get_inventories(&self, project_id: i32) -> Result<Vec<Inventory>> {
        let query = "SELECT * FROM inventory WHERE project_id = $1 ORDER BY name";
            let rows = sqlx::query(query)
                .bind(project_id)
                .fetch_all(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;

            Ok(rows.into_iter().map(|row| Inventory {
                id: row.get("id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                inventory_type: row.get("inventory_type"),
                inventory_data: row.get("inventory_data"),
                key_id: row.try_get("key_id").ok().flatten(),
                secret_storage_id: row.try_get("secret_storage_id").ok().flatten(),
                ssh_login: row.get("ssh_login"),
                ssh_port: row.get("ssh_port"),
                extra_vars: row.try_get("extra_vars").ok().flatten(),
                ssh_key_id: row.try_get("ssh_key_id").ok().flatten(),
                become_key_id: row.try_get("become_key_id").ok().flatten(),
                vaults: row.try_get("vaults").ok().flatten(),
                created: row.get("created"),
                runner_tag: row.try_get("runner_tag").ok().flatten(),
            }).collect())
    }

    async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        let query = "SELECT * FROM inventory WHERE id = $1 AND project_id = $2";
            let row = sqlx::query(query)
                .bind(inventory_id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Инвентарь не найден".to_string()),
                    _ => Error::Database(e),
                })?;

            Ok(Inventory {
                id: row.get("id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                inventory_type: row.get("inventory_type"),
                inventory_data: row.get("inventory_data"),
                key_id: row.get("key_id"),
                secret_storage_id: row.get("secret_storage_id"),
                ssh_login: row.get("ssh_login"),
                ssh_port: row.get("ssh_port"),
                extra_vars: row.get("extra_vars"),
                ssh_key_id: row.get("ssh_key_id"),
                become_key_id: row.get("become_key_id"),
                vaults: row.get("vaults"),
                created: row.get("created"),
                runner_tag: row.try_get("runner_tag").ok().flatten(),
            })
    }

    async fn create_inventory(&self, mut inventory: Inventory) -> Result<Inventory> {
        let query = "INSERT INTO inventory (project_id, name, inventory_type, inventory_data, key_id, secret_storage_id, ssh_login, ssh_port, extra_vars, ssh_key_id, become_key_id, vaults) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id";
            let id: i32 = sqlx::query_scalar(query)
                .bind(inventory.project_id)
                .bind(&inventory.name)
                .bind(&inventory.inventory_type)
                .bind(&inventory.inventory_data)
                .bind(inventory.key_id)
                .bind(inventory.secret_storage_id)
                .bind(&inventory.ssh_login)
                .bind(inventory.ssh_port)
                .bind(&inventory.extra_vars)
                .bind(inventory.ssh_key_id)
                .bind(inventory.become_key_id)
                .bind(&inventory.vaults)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;

            inventory.id = id;
            Ok(inventory)
    }

    async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        let query = "UPDATE inventory SET name = $1, inventory_type = $2, inventory_data = $3, key_id = $4, secret_storage_id = $5, ssh_login = $6, ssh_port = $7, extra_vars = $8, ssh_key_id = $9, become_key_id = $10, vaults = $11 WHERE id = $12 AND project_id = $13";
            sqlx::query(query)
                .bind(&inventory.name)
                .bind(&inventory.inventory_type)
                .bind(&inventory.inventory_data)
                .bind(inventory.key_id)
                .bind(inventory.secret_storage_id)
                .bind(&inventory.ssh_login)
                .bind(inventory.ssh_port)
                .bind(&inventory.extra_vars)
                .bind(inventory.ssh_key_id)
                .bind(inventory.become_key_id)
                .bind(&inventory.vaults)
                .bind(inventory.id)
                .bind(inventory.project_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()> {
        let query = "DELETE FROM inventory WHERE id = $1 AND project_id = $2";
            sqlx::query(query)
                .bind(inventory_id)
                .bind(project_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }
}

