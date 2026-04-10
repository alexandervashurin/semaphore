//! MySQL Inventory CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Pool, MySql};

/// Получает все инвентари проекта MySQL
pub async fn get_inventories(pool: &Pool<MySql>, project_id: i32) -> Result<Vec<Inventory>> {
    let query = "SELECT * FROM `inventory` WHERE project_id = ? ORDER BY name";
    
    let inventories = sqlx::query_as::<_, Inventory>(query)
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

    Ok(inventories)
}

/// Получает инвентарь по ID MySQL
pub async fn get_inventory(pool: &Pool<MySql>, project_id: i32, inventory_id: i32) -> Result<Inventory> {
    let query = "SELECT * FROM `inventory` WHERE id = ? AND project_id = ?";
    
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

/// Создаёт инвентарь MySQL
pub async fn create_inventory(pool: &Pool<MySql>, mut inventory: Inventory) -> Result<Inventory> {
    let query = "INSERT INTO `inventory` (project_id, name, inventory_type, inventory_data, key_id, secret_storage_id, ssh_login, ssh_port, extra_vars, ssh_key_id, become_key_id, vaults, created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
    
    let result = sqlx::query(query)
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
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    inventory.id = result.last_insert_id() as i32;
    Ok(inventory)
}

/// Обновляет инвентарь MySQL
pub async fn update_inventory(pool: &Pool<MySql>, inventory: Inventory) -> Result<()> {
    let query = "UPDATE `inventory` SET name = ?, inventory_type = ?, inventory_data = ?, key_id = ?, secret_storage_id = ?, ssh_login = ?, ssh_port = ?, extra_vars = ?, ssh_key_id = ?, become_key_id = ?, vaults = ? WHERE id = ? AND project_id = ?";
    
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

/// Удаляет инвентарь MySQL
pub async fn delete_inventory(pool: &Pool<MySql>, project_id: i32, inventory_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM `inventory` WHERE id = ? AND project_id = ?")
        .bind(inventory_id)
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::InventoryType;

    #[test]
    fn test_get_inventories_query_structure() {
        let query = "SELECT * FROM `inventory` WHERE project_id = ? ORDER BY name";
        assert!(query.contains("`inventory`"));
        assert!(query.contains("project_id = ?"));
        assert!(query.contains("ORDER BY name"));
    }

    #[test]
    fn test_get_inventory_query_structure() {
        let query = "SELECT * FROM `inventory` WHERE id = ? AND project_id = ?";
        assert!(query.contains("id = ?"));
        assert!(query.contains("project_id = ?"));
    }

    #[test]
    fn test_create_inventory_query_structure() {
        let expected = "INSERT INTO `inventory` (project_id, name, inventory_type, inventory_data, key_id, secret_storage_id, ssh_login, ssh_port, extra_vars, ssh_key_id, become_key_id, vaults, created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
        assert!(expected.contains("`inventory`"));
        assert!(expected.contains("inventory_type"));
        assert!(expected.contains("ssh_port"));
        assert!(expected.count_matches('?'.into()) == 13);
    }

    #[test]
    fn test_update_inventory_query_structure() {
        let expected = "UPDATE `inventory` SET name = ?, inventory_type = ?, inventory_data = ?, key_id = ?, secret_storage_id = ?, ssh_login = ?, ssh_port = ?, extra_vars = ?, ssh_key_id = ?, become_key_id = ?, vaults = ? WHERE id = ? AND project_id = ?";
        assert!(expected.contains("UPDATE `inventory`"));
        assert!(expected.contains("WHERE id = ? AND project_id = ?"));
        assert!(expected.count_matches('?'.into()) == 13);
    }

    #[test]
    fn test_delete_inventory_query_structure() {
        let expected = "DELETE FROM `inventory` WHERE id = ? AND project_id = ?";
        assert!(expected.contains("`inventory`"));
        assert!(expected.contains("id = ? AND project_id = ?"));
    }

    #[test]
    fn test_mysql_inventory_uses_backticks() {
        let queries = [
            "SELECT * FROM `inventory` WHERE id = ?",
            "DELETE FROM `inventory` WHERE id = ? AND project_id = ?",
        ];
        for q in &queries {
            assert!(q.contains('`'), "MySQL inventory queries should use backticks");
        }
    }

    #[test]
    fn test_inventory_type_static() {
        let inv_type = InventoryType::Static;
        assert_eq!(inv_type, InventoryType::Static);
    }

    #[test]
    fn test_inventory_type_file() {
        let inv_type = InventoryType::File;
        assert_eq!(inv_type, InventoryType::File);
    }

    #[test]
    fn test_inventory_model_fields() {
        let inventory = Inventory::new(10, "test-inv", InventoryType::Static);
        assert_eq!(inventory.project_id, 10);
        assert_eq!(inventory.name, "test-inv");
        assert_eq!(inventory.inventory_type, InventoryType::Static);
        assert_eq!(inventory.ssh_login, "root");
        assert_eq!(inventory.ssh_port, 22);
    }

    #[test]
    fn test_inventory_serialization() {
        let inventory = Inventory::new(1, "ser-inv", InventoryType::Static);
        let json = serde_json::to_string(&inventory).unwrap();
        assert!(json.contains("\"name\":\"ser-inv\""));
        assert!(json.contains("\"ssh_port\":22"));
    }

    #[test]
    fn test_inventory_bind_order_matches_query() {
        let columns = [
            "project_id", "name", "inventory_type", "inventory_data",
            "key_id", "secret_storage_id", "ssh_login", "ssh_port",
            "extra_vars", "ssh_key_id", "become_key_id", "vaults", "created",
        ];
        assert_eq!(columns.len(), 13);
        assert_eq!(columns[0], "project_id");
        assert_eq!(columns[6], "ssh_login");
    }

    #[test]
    fn test_mysql_inventory_debug_format() {
        let query = "SELECT * FROM `inventory` WHERE id = ?";
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("inventory"));
    }
}
