//! InventoryManager - управление инвентарями
//!
//! Реализация трейта InventoryManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::{Inventory, InventoryType};
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

        Ok(rows
            .into_iter()
            .map(|row| Inventory {
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
            })
            .collect())
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

#[cfg(test)]
mod tests {
    use crate::models::{Inventory, InventoryType};

    #[test]
    fn test_inventory_type_display() {
        assert_eq!(InventoryType::Static.to_string(), "static");
        assert_eq!(InventoryType::StaticYaml.to_string(), "static_yaml");
        assert_eq!(InventoryType::StaticJson.to_string(), "static_json");
        assert_eq!(InventoryType::File.to_string(), "file");
    }

    #[test]
    fn test_inventory_type_from_str() {
        assert_eq!(
            "static".parse::<InventoryType>().unwrap(),
            InventoryType::Static
        );
        assert_eq!(
            "static_yaml".parse::<InventoryType>().unwrap(),
            InventoryType::StaticYaml
        );
        assert_eq!(
            "static_json".parse::<InventoryType>().unwrap(),
            InventoryType::StaticJson
        );
        assert_eq!(
            "file".parse::<InventoryType>().unwrap(),
            InventoryType::File
        );
        assert_eq!(
            "invalid".parse::<InventoryType>().unwrap(),
            InventoryType::Static
        );
    }

    #[test]
    fn test_inventory_type_serialize_all() {
        let types = [
            InventoryType::Static,
            InventoryType::StaticYaml,
            InventoryType::StaticJson,
            InventoryType::File,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            assert!(json.starts_with('"') && json.ends_with('"'));
        }
    }

    #[test]
    fn test_inventory_new() {
        let inv = Inventory::new(10, "test-inv".to_string(), InventoryType::Static);
        assert_eq!(inv.id, 0);
        assert_eq!(inv.project_id, 10);
        assert_eq!(inv.name, "test-inv");
        assert_eq!(inv.inventory_type, InventoryType::Static);
        assert_eq!(inv.ssh_login, "root");
        assert_eq!(inv.ssh_port, 22);
        assert!(inv.key_id.is_none());
    }

    #[test]
    fn test_inventory_default() {
        let inv = Inventory::default();
        assert_eq!(inv.id, 0);
        assert_eq!(inv.inventory_type, InventoryType::Static);
        assert_eq!(inv.ssh_login, "root");
        assert_eq!(inv.ssh_port, 22);
        assert!(inv.key_id.is_none());
        assert!(inv.extra_vars.is_none());
    }

    #[test]
    fn test_inventory_serialization_skip_nulls() {
        let inv = Inventory::default();
        let json = serde_json::to_string(&inv).unwrap();
        assert!(!json.contains("key_id"));
        assert!(!json.contains("secret_storage_id"));
        assert!(!json.contains("extra_vars"));
    }

    #[test]
    fn test_inventory_serialization_with_values() {
        let inv = Inventory {
            id: 1,
            project_id: 5,
            name: "production".to_string(),
            inventory_type: InventoryType::Static,
            inventory_data: "[servers]\nserver1\n".to_string(),
            key_id: Some(10),
            secret_storage_id: Some(2),
            ssh_login: "deploy".to_string(),
            ssh_port: 2222,
            extra_vars: Some(r#"{"env":"prod"}"#.to_string()),
            ssh_key_id: Some(3),
            become_key_id: Some(4),
            vaults: None,
            created: None,
            runner_tag: Some("linux".to_string()),
        };
        let json = serde_json::to_string(&inv).unwrap();
        assert!(json.contains("\"name\":\"production\""));
        assert!(json.contains("\"key_id\":10"));
        assert!(json.contains("\"runner_tag\":\"linux\""));
    }

    #[test]
    fn test_inventory_clone() {
        let inv = Inventory::new(10, "clone-test".to_string(), InventoryType::Static);
        let cloned = inv.clone();
        assert_eq!(cloned.name, inv.name);
        assert_eq!(cloned.project_id, inv.project_id);
    }

    #[test]
    fn test_inventory_with_file_type() {
        let inv = Inventory::new(1, "file-inv".to_string(), InventoryType::File);
        assert_eq!(inv.inventory_type, InventoryType::File);
        let json = serde_json::to_string(&inv).unwrap();
        assert!(json.contains("\"inventory_type\":\"file\""));
    }

    #[test]
    fn test_inventory_with_static_json() {
        let inv = Inventory::new(1, "json-inv".to_string(), InventoryType::StaticJson);
        assert_eq!(inv.inventory_type, InventoryType::StaticJson);
        assert_eq!(inv.ssh_login, "root");
    }

    #[test]
    fn test_inventory_with_extra_vars() {
        let inv = Inventory {
            id: 1,
            project_id: 1,
            name: "extra".to_string(),
            inventory_type: InventoryType::Static,
            inventory_data: "data".to_string(),
            key_id: None,
            secret_storage_id: None,
            ssh_login: "user".to_string(),
            ssh_port: 22,
            extra_vars: Some(r#"{"key":"value"}"#.to_string()),
            ssh_key_id: None,
            become_key_id: None,
            vaults: None,
            created: None,
            runner_tag: None,
        };
        assert!(inv.extra_vars.is_some());
    }

    #[test]
    fn test_inventory_with_runner_tag() {
        let inv = Inventory {
            id: 1,
            project_id: 1,
            name: "tagged".to_string(),
            inventory_type: InventoryType::Static,
            inventory_data: "".to_string(),
            key_id: None,
            secret_storage_id: None,
            ssh_login: "root".to_string(),
            ssh_port: 22,
            extra_vars: None,
            ssh_key_id: None,
            become_key_id: None,
            vaults: None,
            created: None,
            runner_tag: Some("docker".to_string()),
        };
        assert_eq!(inv.runner_tag, Some("docker".to_string()));
    }

    #[test]
    fn test_inventory_port_custom() {
        let inv = Inventory {
            id: 0,
            project_id: 0,
            name: "custom-port".to_string(),
            inventory_type: InventoryType::Static,
            inventory_data: "".to_string(),
            key_id: None,
            secret_storage_id: None,
            ssh_login: "admin".to_string(),
            ssh_port: 8022,
            extra_vars: None,
            ssh_key_id: None,
            become_key_id: None,
            vaults: None,
            created: None,
            runner_tag: None,
        };
        assert_eq!(inv.ssh_port, 8022);
    }

    #[test]
    fn test_inventory_type_roundtrip() {
        let variants = [
            InventoryType::Static,
            InventoryType::StaticYaml,
            InventoryType::StaticJson,
            InventoryType::File,
        ];
        for t in &variants {
            let json = serde_json::to_string(t).unwrap();
            let parsed: InventoryType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, *t);
        }
    }
}
