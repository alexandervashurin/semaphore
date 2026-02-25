//! Модель инвентаря

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
