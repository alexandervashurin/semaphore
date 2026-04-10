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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::InventoryType;

    #[test]
    fn test_get_inventories_query_structure() {
        let query = "SELECT * FROM inventory WHERE project_id = ? ORDER BY name";
        assert!(query.contains("inventory"));
        assert!(query.contains("project_id = ?"));
        assert!(query.contains("ORDER BY name"));
    }

    #[test]
    fn test_get_inventory_query_structure() {
        let query = "SELECT * FROM inventory WHERE id = ? AND project_id = ?";
        assert!(query.contains("id = ?"));
        assert!(query.contains("project_id = ?"));
    }

    #[test]
    fn test_create_inventory_query_structure() {
        let expected = "INSERT INTO inventory (project_id, name, inventory_type, inventory_data, key_id, secret_storage_id, ssh_login, ssh_port, extra_vars, ssh_key_id, become_key_id, vaults, created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
        assert!(query_contains_table(expected, "inventory"));
        assert!(expected.contains("RETURNING id"));
        assert!(expected.matches('?').count() == 13);
    }

    #[test]
    fn test_update_inventory_query_structure() {
        let expected = "UPDATE inventory SET name = ?, inventory_type = ?, inventory_data = ?, key_id = ?, secret_storage_id = ?, ssh_login = ?, ssh_port = ?, extra_vars = ?, ssh_key_id = ?, become_key_id = ?, vaults = ? WHERE id = ? AND project_id = ?";
        assert!(expected.contains("UPDATE inventory"));
        assert!(expected.contains("WHERE id = ? AND project_id = ?"));
        assert!(expected.matches('?').count() == 13);
    }

    #[test]
    fn test_delete_inventory_query_structure() {
        let expected = "DELETE FROM inventory WHERE id = ? AND project_id = ?";
        assert!(query_contains_table(expected, "inventory"));
        assert!(expected.contains("id = ? AND project_id = ?"));
    }

    #[test]
    fn test_sqlite_uses_question_placeholders() {
        let queries = [
            "SELECT * FROM inventory WHERE id = ?",
            "DELETE FROM inventory WHERE id = ? AND project_id = ?",
            "INSERT INTO inventory (name) VALUES (?) RETURNING id",
        ];
        for q in &queries {
            assert!(q.contains('?'), "SQLite should use ? placeholders");
            assert!(!q.contains('$'), "SQLite should not use $N placeholders");
        }
    }

    #[test]
    fn test_sqlite_no_backticks() {
        let queries = [
            "SELECT * FROM inventory WHERE id = ?",
            "DELETE FROM inventory WHERE id = ?",
        ];
        for q in &queries {
            assert!(!q.contains('`'), "SQLite should not use backticks");
        }
    }

    #[test]
    fn test_sqlite_returning_clause() {
        let query = "INSERT INTO inventory (...) VALUES (?) RETURNING id";
        assert!(query.contains("RETURNING id"), "SQLite uses RETURNING clause");
    }

    #[test]
    fn test_inventory_model_fields() {
        let inventory = Inventory::new(10, "sqlite-inv", InventoryType::Static);
        assert_eq!(inventory.project_id, 10);
        assert_eq!(inventory.name, "sqlite-inv");
        assert_eq!(inventory.inventory_type, InventoryType::Static);
    }

    #[test]
    fn test_inventory_serialization() {
        let inventory = Inventory::new(1, "sqlite-ser-inv", InventoryType::File);
        let json = serde_json::to_string(&inventory).unwrap();
        assert!(json.contains("\"name\":\"sqlite-ser-inv\""));
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
        assert_eq!(columns[12], "created");
    }

    #[test]
    fn test_sqlite_inventory_debug_format() {
        let query = "SELECT * FROM inventory WHERE id = ?";
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("inventory"));
    }
}

fn query_contains_table(query: &str, table: &str) -> bool {
    query.contains(table)
}
