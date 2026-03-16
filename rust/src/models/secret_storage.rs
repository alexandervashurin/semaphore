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

impl std::fmt::Display for SecretStorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretStorageType::Local => write!(f, "local"),
            SecretStorageType::Vault => write!(f, "vault"),
            SecretStorageType::Dvls => write!(f, "dvls"),
        }
    }
}

impl std::str::FromStr for SecretStorageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(SecretStorageType::Local),
            "vault" => Ok(SecretStorageType::Vault),
            "dvls" => Ok(SecretStorageType::Dvls),
            _ => Ok(SecretStorageType::Local),
        }
    }
}

impl<DB: sqlx::Database> sqlx::Type<DB> for SecretStorageType
where
    String: sqlx::Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as sqlx::Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as sqlx::Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for SecretStorageType
where
    String: sqlx::Decode<'r, DB>,
{
    fn decode(value: <DB as sqlx::Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<'r, DB>>::decode(value)?;
        Ok(s.parse().unwrap_or(SecretStorageType::Local))
    }
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

    /// Тип источника (для ключей доступа к внешнему хранилищу)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_storage_type: Option<String>,

    /// Секрет/токен доступа к хранилищу (маскируется при сериализации)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
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
            source_storage_type: None,
            secret: None,
        }
    }
}
