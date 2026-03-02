//! Модель инвентаря

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, decode::Decode, encode::Encode, database::Database};

/// Тип инвентаря
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InventoryType {
    Static,
    StaticYaml,
    StaticJson,
    File,
    TerraformInventory,
}

impl std::fmt::Display for InventoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InventoryType::Static => write!(f, "static"),
            InventoryType::StaticYaml => write!(f, "static_yaml"),
            InventoryType::StaticJson => write!(f, "static_json"),
            InventoryType::File => write!(f, "file"),
            InventoryType::TerraformInventory => write!(f, "terraform_inventory"),
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
            _ => Ok(InventoryType::Static),
        }
    }
}

impl<DB: Database> Type<DB> for InventoryType {
    fn type_info() -> DB::TypeInfo {
        <&str as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <&str as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for InventoryType
where
    &'r str: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as Decode<'r, DB>>::decode(value)?;
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
    for<'a> &'a str: Encode<'q, DB>,
{
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            InventoryType::Static => "static",
            InventoryType::StaticYaml => "static_yaml",
            InventoryType::StaticJson => "static_json",
            InventoryType::File => "file",
            InventoryType::TerraformInventory => "terraform_inventory",
        };
        <&str as Encode<'q, DB>>::encode(&s, buf)
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
    pub key_id: i32,

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
            key_id: 0,
            secret_storage_id: None,
            ssh_login: "root".to_string(),
            ssh_port: 22,
            extra_vars: None,
            ssh_key_id: None,
            become_key_id: None,
            vaults: None,
        }
    }
}
