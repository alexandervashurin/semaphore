//! SQLite Inventory CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Sqlite, Pool};

/// Получает все инвентари проекта SQLite
pub async fn get_inventories(pool: &Pool<Sqlite>, project_id: i32) -> Result<Vec<Inventory>> {
    let query = "SELECT * FROM inventory WHERE project_id = ? ORDER BY name";
    
    let inventories = sqlx::query_as::<_, Inventory>(query)
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

    Ok(inventories)
}

/// Получает инвентарь по ID SQLite
pub async fn get_inventory(pool: &Pool<Sqlite>, project_id: i32, inventory_id: i32) -> Result<Inventory> {
    let query = "SELECT * FROM inventory WHERE id = ? AND project_id = ?";
    
    let inventory = sqlx::query_as::<_, Inventory>(query)
        .bind(inventory_id)
        .bind(project_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Inventory not found".to_string()),
            _ => Error::Database(e),
        })?;

    Ok(inventory)
}

/// Создаёт инвентарь SQLite
pub async fn create_inventory(pool: &Pool<Sqlite>, mut inventory: Inventory) -> Result<Inventory> {
    let query = "INSERT INTO inventory (project_id, name, inventory_type, inventory_data, key_id, secret_storage_id, ssh_login, ssh_port, extra_vars, ssh_key_id, become_key_id, vaults, created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
    
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
        .bind(inventory.created)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

    inventory.id = id;
    Ok(inventory)
}

/// Обновляет инвентарь SQLite
pub async fn update_inventory(pool: &Pool<Sqlite>, inventory: Inventory) -> Result<()> {
    let query = "UPDATE inventory SET name = ?, inventory_type = ?, inventory_data = ?, key_id = ?, secret_storage_id = ?, ssh_login = ?, ssh_port = ?, extra_vars = ?, ssh_key_id = ?, become_key_id = ?, vaults = ? WHERE id = ? AND project_id = ?";
    
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
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

/// Удаляет инвентарь SQLite
pub async fn delete_inventory(pool: &Pool<Sqlite>, project_id: i32, inventory_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM inventory WHERE id = ? AND project_id = ?")
        .bind(inventory_id)
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}
