//! Модель SecretStorage - хранилище секретов

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Тип хранилища секретов
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SecretStorageType {
    Local,
    Vault,
    Dvls,
}

/// Хранилище секретов
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecretStorage {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Название хранилища
    pub name: String,

    /// Тип хранилища
    pub r#type: SecretStorageType,

    /// Параметры (JSON)
    pub params: String,

    /// Только для чтения
    pub read_only: bool,
}

impl SecretStorage {
    /// Создаёт новое хранилище
    pub fn new(project_id: i32, name: String, storage_type: SecretStorageType, params: String) -> Self {
        Self {
            id: 0,
            project_id,
            name,
            r#type: storage_type,
            params,
            read_only: false,
        }
    }
}
