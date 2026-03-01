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

impl<DB: Database> Type<DB> for InventoryType {
    fn type_info() -> DB::TypeInfo {
        String::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        String::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for InventoryType {
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = String::decode(value)?;
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
{
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s: String = match self {
            InventoryType::Static => "static",
            InventoryType::StaticYaml => "static_yaml",
            InventoryType::StaticJson => "static_json",
            InventoryType::File => "file",
            InventoryType::TerraformInventory => "terraform_inventory",
        }.to_string();
        Encode::encode(s, buf)
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
    pub inventory: InventoryType,

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
            inventory: inventory_type,
            inventory_data: String::new(),
            key_id: 0,
            secret_storage_id: None,
            ssh_login: "root".to_string(),
            ssh_port: 22,
            extra_vars: None,
        }
    }
}
