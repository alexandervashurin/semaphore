//! Модель инвентаря

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{database::Database, decode::Decode, encode::Encode, FromRow, Type};

/// Тип инвентаря
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum InventoryType {
    #[default]
    Static,
    StaticYaml,
    StaticJson,
    File,
    TerraformInventory,
    TerraformWorkspace,
    TofuWorkspace,
}

impl std::fmt::Display for InventoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InventoryType::Static => write!(f, "static"),
            InventoryType::StaticYaml => write!(f, "static_yaml"),
            InventoryType::StaticJson => write!(f, "static_json"),
            InventoryType::File => write!(f, "file"),
            InventoryType::TerraformInventory => write!(f, "terraform_inventory"),
            InventoryType::TerraformWorkspace => write!(f, "terraform_workspace"),
            InventoryType::TofuWorkspace => write!(f, "tofu_workspace"),
        }
    }
}

impl std::str::FromStr for InventoryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "static" => Ok(InventoryType::Static),
            "static_yaml" => Ok(InventoryType::StaticYaml),
            "static_json" => Ok(InventoryType::StaticJson),
            "file" => Ok(InventoryType::File),
            "terraform_inventory" => Ok(InventoryType::TerraformInventory),
            "terraform_workspace" => Ok(InventoryType::TerraformWorkspace),
            "tofu_workspace" => Ok(InventoryType::TofuWorkspace),
            _ => Ok(InventoryType::Static),
        }
    }
}

impl<DB: Database> Type<DB> for InventoryType
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for InventoryType
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "static" => InventoryType::Static,
            "static_yaml" => InventoryType::StaticYaml,
            "static_json" => InventoryType::StaticJson,
            "file" => InventoryType::File,
            "terraform_inventory" => InventoryType::TerraformInventory,
            _ => InventoryType::Static,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for InventoryType
where
    DB: 'q,
    String: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            InventoryType::Static => "static",
            InventoryType::StaticYaml => "static_yaml",
            InventoryType::StaticJson => "static_json",
            InventoryType::File => "file",
            InventoryType::TerraformInventory => "terraform_inventory",
            InventoryType::TerraformWorkspace => "terraform_workspace",
            InventoryType::TofuWorkspace => "tofu_workspace",
        }
        .to_string();
        <String as Encode<'q, DB>>::encode(s, buf)
    }
}

/// Инвентарь - коллекция целевых хостов
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Inventory {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Название инвентаря
    pub name: String,

    /// Тип инвентаря
    pub inventory_type: InventoryType,

    /// Содержимое инвентаря (для static)
    pub inventory_data: String,

    /// ID ключа доступа
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<i32>,

    /// ID хранилища секретов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_storage_id: Option<i32>,

    /// SSH-пользователь
    pub ssh_login: String,

    /// SSH-порт
    pub ssh_port: i32,

    /// Дополнительные параметры
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_vars: Option<String>,

    /// ID SSH ключа
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_id: Option<i32>,

    /// ID ключа become
    #[serde(skip_serializing_if = "Option::is_none")]
    pub become_key_id: Option<i32>,

    /// Хранилища секретов
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaults: Option<String>,

    /// Дата создания
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<Utc>>,

    /// Runner tag для фильтрации раннеров
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner_tag: Option<String>,
}

impl Inventory {
    /// Создаёт новый инвентарь
    pub fn new(project_id: i32, name: String, inventory_type: InventoryType) -> Self {
        Self {
            id: 0,
            project_id,
            name,
            inventory_type,
            inventory_data: String::new(),
            key_id: None,
            secret_storage_id: None,
            ssh_login: "root".to_string(),
            ssh_port: 22,
            extra_vars: None,
            ssh_key_id: None,
            become_key_id: None,
            vaults: None,
            created: None,
            runner_tag: None,
        }
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            id: 0,
            project_id: 0,
            name: String::new(),
            inventory_type: InventoryType::Static,
            inventory_data: String::new(),
            key_id: None,
            secret_storage_id: None,
            ssh_login: "root".to_string(),
            ssh_port: 22,
            extra_vars: None,
            ssh_key_id: None,
            become_key_id: None,
            vaults: None,
            created: None,
            runner_tag: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_type_display() {
        assert_eq!(InventoryType::Static.to_string(), "static");
        assert_eq!(InventoryType::StaticYaml.to_string(), "static_yaml");
        assert_eq!(InventoryType::File.to_string(), "file");
        assert_eq!(InventoryType::TerraformInventory.to_string(), "terraform_inventory");
        assert_eq!(InventoryType::TofuWorkspace.to_string(), "tofu_workspace");
    }

    #[test]
    fn test_inventory_type_from_str() {
        assert_eq!("static".parse::<InventoryType>().unwrap(), InventoryType::Static);
        assert_eq!("file".parse::<InventoryType>().unwrap(), InventoryType::File);
        assert_eq!("unknown".parse::<InventoryType>().unwrap(), InventoryType::Static);
    }

    #[test]
    fn test_inventory_type_serialization() {
        let json = serde_json::to_string(&InventoryType::StaticYaml).unwrap();
        assert_eq!(json, "\"static_yaml\"");
    }

    #[test]
    fn test_inventory_new() {
        let inv = Inventory::new(10, "my-inventory".to_string(), InventoryType::Static);
        assert_eq!(inv.id, 0);
        assert_eq!(inv.project_id, 10);
        assert_eq!(inv.name, "my-inventory");
        assert_eq!(inv.inventory_type, InventoryType::Static);
        assert_eq!(inv.ssh_login, "root");
        assert_eq!(inv.ssh_port, 22);
        assert!(inv.key_id.is_none());
    }

    #[test]
    fn test_inventory_default() {
        let inv = Inventory::default();
        assert_eq!(inv.id, 0);
        assert!(inv.name.is_empty());
        assert_eq!(inv.inventory_type, InventoryType::Static);
        assert_eq!(inv.ssh_login, "root");
        assert_eq!(inv.ssh_port, 22);
    }

    #[test]
    fn test_inventory_serialization_skip_nulls() {
        let inv = Inventory::default();
        let json = serde_json::to_string(&inv).unwrap();
        assert!(!json.contains("key_id"));
        assert!(!json.contains("secret_storage_id"));
        assert!(!json.contains("extra_vars"));
        assert!(json.contains("\"ssh_port\":22"));
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
            created: Some(Utc::now()),
            runner_tag: Some("linux".to_string()),
        };
        let json = serde_json::to_string(&inv).unwrap();
        assert!(json.contains("\"name\":\"production\""));
        assert!(json.contains("\"key_id\":10"));
        assert!(json.contains("\"extra_vars\":\"{\\\"env\\\":\\\"prod\\\"}\""));
        assert!(json.contains("\"runner_tag\":\"linux\""));
    }

    #[test]
    fn test_inventory_type_display_all_variants() {
        assert_eq!(InventoryType::Static.to_string(), "static");
        assert_eq!(InventoryType::StaticYaml.to_string(), "static_yaml");
        assert_eq!(InventoryType::StaticJson.to_string(), "static_json");
        assert_eq!(InventoryType::File.to_string(), "file");
        assert_eq!(InventoryType::TerraformInventory.to_string(), "terraform_inventory");
        assert_eq!(InventoryType::TerraformWorkspace.to_string(), "terraform_workspace");
        assert_eq!(InventoryType::TofuWorkspace.to_string(), "tofu_workspace");
    }

    #[test]
    fn test_inventory_type_from_str_all_variants() {
        assert_eq!("static_yaml".parse::<InventoryType>().unwrap(), InventoryType::StaticYaml);
        assert_eq!("static_json".parse::<InventoryType>().unwrap(), InventoryType::StaticJson);
        assert_eq!("terraform_workspace".parse::<InventoryType>().unwrap(), InventoryType::TerraformWorkspace);
        assert_eq!("tofu_workspace".parse::<InventoryType>().unwrap(), InventoryType::TofuWorkspace);
    }

    #[test]
    fn test_inventory_type_serialize_all_variants() {
        let types = [
            InventoryType::Static,
            InventoryType::StaticYaml,
            InventoryType::StaticJson,
            InventoryType::File,
            InventoryType::TerraformInventory,
            InventoryType::TerraformWorkspace,
            InventoryType::TofuWorkspace,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            assert!(json.starts_with('"') && json.ends_with('"'));
        }
    }

    #[test]
    fn test_inventory_clone() {
        let inv = Inventory::new(10, "clone-test".to_string(), InventoryType::Static);
        let cloned = inv.clone();
        assert_eq!(cloned.name, inv.name);
        assert_eq!(cloned.project_id, inv.project_id);
        assert_eq!(cloned.inventory_type, inv.inventory_type);
    }

    #[test]
    fn test_inventory_with_all_fields() {
        let inv = Inventory {
            id: 1,
            project_id: 1,
            name: "full".to_string(),
            inventory_type: InventoryType::StaticJson,
            inventory_data: "{}".to_string(),
            key_id: Some(1),
            secret_storage_id: Some(1),
            ssh_login: "user".to_string(),
            ssh_port: 22,
            extra_vars: None,
            ssh_key_id: Some(2),
            become_key_id: Some(3),
            vaults: None,
            created: Some(Utc::now()),
            runner_tag: None,
        };
        assert_eq!(inv.inventory_type, InventoryType::StaticJson);
        assert_eq!(inv.ssh_port, 22);
        assert!(inv.extra_vars.is_none());
    }

    #[test]
    fn test_inventory_with_terraform_workspace() {
        let inv = Inventory::new(1, "tf-workspace".to_string(), InventoryType::TerraformWorkspace);
        assert_eq!(inv.inventory_type, InventoryType::TerraformWorkspace);
    }

    #[test]
    fn test_inventory_with_tofu_workspace() {
        let inv = Inventory::new(1, "tofu-workspace".to_string(), InventoryType::TofuWorkspace);
        assert_eq!(inv.inventory_type, InventoryType::TofuWorkspace);
    }

    #[test]
    fn test_inventory_clone_independence() {
        let mut inv = Inventory::new(1, "original".to_string(), InventoryType::Static);
        let cloned = inv.clone();
        inv.name = "modified".to_string();
        assert_eq!(cloned.name, "original");
    }
}
