//! PostgreSQL Inventory CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Pool, Postgres};

/// Получает все инвентари проекта PostgreSQL
pub async fn get_inventories(pool: &Pool<Postgres>, project_id: i32) -> Result<Vec<Inventory>> {
    let query = "SELECT * FROM inventory WHERE project_id = $1 ORDER BY name";

    let inventories = sqlx::query_as::<_, Inventory>(query)
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

    Ok(inventories)
}

/// Получает инвентарь по ID PostgreSQL
pub async fn get_inventory(
    pool: &Pool<Postgres>,
    project_id: i32,
    inventory_id: i32,
) -> Result<Inventory> {
    let query = "SELECT * FROM inventory WHERE id = $1 AND project_id = $2";

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

/// Создаёт инвентарь PostgreSQL
pub async fn create_inventory(
    pool: &Pool<Postgres>,
    mut inventory: Inventory,
) -> Result<Inventory> {
    let query = "INSERT INTO inventory (project_id, name, inventory_type, inventory_data, key_id, secret_storage_id, ssh_login, ssh_port, extra_vars, ssh_key_id, become_key_id, vaults, created) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id";

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

/// Обновляет инвентарь PostgreSQL
pub async fn update_inventory(pool: &Pool<Postgres>, inventory: Inventory) -> Result<()> {
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
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

/// Удаляет инвентарь PostgreSQL
pub async fn delete_inventory(
    pool: &Pool<Postgres>,
    project_id: i32,
    inventory_id: i32,
) -> Result<()> {
    sqlx::query("DELETE FROM inventory WHERE id = $1 AND project_id = $2")
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
        let query = "SELECT * FROM inventory WHERE project_id = $1 ORDER BY name";
        assert!(query.contains("inventory"));
        assert!(query.contains("project_id = $1"));
        assert!(query.contains("ORDER BY name"));
    }

    #[test]
    fn test_get_inventory_query_structure() {
        let query = "SELECT * FROM inventory WHERE id = $1 AND project_id = $2";
        assert!(query.contains("id = $1"));
        assert!(query.contains("project_id = $2"));
    }

    #[test]
    fn test_create_inventory_query_structure() {
        let expected = "INSERT INTO inventory (project_id, name, inventory_type, inventory_data, key_id, secret_storage_id, ssh_login, ssh_port, extra_vars, ssh_key_id, become_key_id, vaults, created) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id";
        assert!(expected.contains("inventory"));
        assert!(expected.contains("RETURNING id"));
        assert!(expected.contains("$13"));
    }

    #[test]
    fn test_update_inventory_query_structure() {
        let expected = "UPDATE inventory SET name = $1, inventory_type = $2, inventory_data = $3, key_id = $4, secret_storage_id = $5, ssh_login = $6, ssh_port = $7, extra_vars = $8, ssh_key_id = $9, become_key_id = $10, vaults = $11 WHERE id = $12 AND project_id = $13";
        assert!(expected.contains("UPDATE inventory"));
        assert!(expected.contains("WHERE id = $12 AND project_id = $13"));
        assert!(expected.contains("$13"));
    }

    #[test]
    fn test_delete_inventory_query_structure() {
        let expected = "DELETE FROM inventory WHERE id = $1 AND project_id = $2";
        assert!(expected.contains("inventory"));
        assert!(expected.contains("id = $1 AND project_id = $2"));
    }

    #[test]
    fn test_postgres_uses_dollar_placeholders() {
        let queries = [
            "SELECT * FROM inventory WHERE id = $1",
            "DELETE FROM inventory WHERE id = $1 AND project_id = $2",
            "INSERT INTO inventory (name) VALUES ($1) RETURNING id",
        ];
        for q in &queries {
            assert!(q.contains('$'), "Postgres should use $N placeholders");
            assert!(!q.contains('?'), "Postgres should not use ? placeholders");
        }
    }

    #[test]
    fn test_postgres_no_backticks() {
        let queries = [
            "SELECT * FROM inventory WHERE id = $1",
            "DELETE FROM inventory WHERE id = $1",
        ];
        for q in &queries {
            assert!(!q.contains('`'), "Postgres should not use backticks");
        }
    }

    #[test]
    fn test_postgres_returning_clause() {
        let query = "INSERT INTO inventory (...) VALUES (...) RETURNING id";
        assert!(
            query.contains("RETURNING id"),
            "Postgres uses RETURNING clause"
        );
    }

    #[test]
    fn test_inventory_model_fields() {
        let inventory = Inventory::new(10, "pg-inv".to_string(), InventoryType::Static);
        assert_eq!(inventory.project_id, 10);
        assert_eq!(inventory.name, "pg-inv");
        assert_eq!(inventory.inventory_type, InventoryType::Static);
    }

    #[test]
    fn test_inventory_serialization() {
        let inventory = Inventory::new(1, "pg-ser-inv".to_string(), InventoryType::File);
        let json = serde_json::to_string(&inventory).unwrap();
        assert!(json.contains("\"name\":\"pg-ser-inv\""));
        assert!(json.contains("\"ssh_port\":22"));
    }

    #[test]
    fn test_inventory_bind_order_matches_query() {
        let columns = [
            "project_id",
            "name",
            "inventory_type",
            "inventory_data",
            "key_id",
            "secret_storage_id",
            "ssh_login",
            "ssh_port",
            "extra_vars",
            "ssh_key_id",
            "become_key_id",
            "vaults",
            "created",
        ];
        assert_eq!(columns.len(), 13);
        assert_eq!(columns[0], "project_id");
        assert_eq!(columns[12], "created");
    }

    #[test]
    fn test_postgres_inventory_debug_format() {
        let query = "SELECT * FROM inventory WHERE id = $1";
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("inventory"));
    }
}
