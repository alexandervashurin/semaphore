//! Inventory CRUD Operations
//!
//! Адаптер для декомпозированных модулей
//!
//! Новые модули: sqlite::inventory, postgres::inventory, mysql::inventory

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{Inventory, InventoryType};

impl SqlDb {
    /// Получает инвентари проекта
    pub async fn get_inventories(&self, project_id: i32) -> Result<Vec<Inventory>> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::inventory::get_inventories(pool, project_id).await
    }

    /// Получает инвентарь по ID
    pub async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::inventory::get_inventory(pool, project_id, inventory_id).await
    }

    /// Создаёт инвентарь
    pub async fn create_inventory(&self, inventory: Inventory) -> Result<Inventory> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::inventory::create_inventory(pool, inventory).await
    }

    /// Обновляет инвентарь
    pub async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::inventory::update_inventory(pool, inventory).await
    }

    /// Удаляет инвентарь
    pub async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::inventory::delete_inventory(pool, project_id, inventory_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_new() {
        let inv = Inventory::new(10, "my-inventory".to_string(), InventoryType::Static);
        assert_eq!(inv.id, 0);
        assert_eq!(inv.project_id, 10);
        assert_eq!(inv.name, "my-inventory");
        assert_eq!(inv.ssh_login, "root");
        assert_eq!(inv.ssh_port, 22);
    }

    #[test]
    fn test_inventory_default() {
        let inv = Inventory::default();
        assert_eq!(inv.id, 0);
        assert!(inv.name.is_empty());
        assert_eq!(inv.inventory_type, InventoryType::Static);
        assert_eq!(inv.ssh_login, "root");
    }

    #[test]
    fn test_inventory_type_display() {
        assert_eq!(InventoryType::Static.to_string(), "static");
        assert_eq!(InventoryType::StaticYaml.to_string(), "static_yaml");
        assert_eq!(InventoryType::File.to_string(), "file");
    }

    #[test]
    fn test_inventory_type_serialization() {
        let json = serde_json::to_string(&InventoryType::StaticJson).unwrap();
        assert_eq!(json, "\"static_json\"");
    }

    #[test]
    fn test_inventory_type_deserialization() {
        let t: InventoryType = serde_json::from_str("\"file\"").unwrap();
        assert_eq!(t, InventoryType::File);
    }

    #[test]
    fn test_inventory_serialization() {
        let inv = Inventory::new(1, "prod".to_string(), InventoryType::Static);
        let json = serde_json::to_string(&inv).unwrap();
        assert!(json.contains("\"name\":\"prod\""));
        assert!(json.contains("\"ssh_port\":22"));
    }

    #[test]
    fn test_inventory_skip_nulls() {
        let inv = Inventory::default();
        let json = serde_json::to_string(&inv).unwrap();
        assert!(!json.contains("key_id"));
        assert!(!json.contains("secret_storage_id"));
        assert!(!json.contains("extra_vars"));
    }

    #[test]
    fn test_inventory_clone() {
        let inv = Inventory::new(5, "clone".to_string(), InventoryType::Static);
        let cloned = inv.clone();
        assert_eq!(cloned.name, inv.name);
        assert_eq!(cloned.inventory_type, inv.inventory_type);
    }

    #[test]
    fn test_inventory_with_ssh_settings() {
        let inv = Inventory {
            id: 1,
            project_id: 1,
            name: "ssh-inv".to_string(),
            inventory_type: InventoryType::Static,
            inventory_data: String::new(),
            key_id: None,
            secret_storage_id: None,
            ssh_login: "deploy".to_string(),
            ssh_port: 2222,
            extra_vars: None,
            ssh_key_id: None,
            become_key_id: None,
            vaults: None,
            created: None,
            runner_tag: None,
        };
        assert_eq!(inv.ssh_login, "deploy");
        assert_eq!(inv.ssh_port, 2222);
    }

    #[test]
    fn test_inventory_type_equality() {
        assert_eq!(InventoryType::Static, InventoryType::Static);
        assert_ne!(InventoryType::Static, InventoryType::File);
    }

    #[test]
    fn test_inventory_debug_format() {
        let inv = Inventory::new(1, "debug".to_string(), InventoryType::Static);
        let debug_str = format!("{:?}", inv);
        assert!(debug_str.contains("Inventory"));
        assert!(debug_str.contains("debug"));
    }

    #[test]
    fn test_inventory_type_from_str() {
        assert_eq!("static".parse::<InventoryType>().unwrap(), InventoryType::Static);
        assert_eq!("unknown".parse::<InventoryType>().unwrap(), InventoryType::Static);
    }
}
