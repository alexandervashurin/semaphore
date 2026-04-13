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
    fn decode(
        value: <DB as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<'r, DB>>::decode(value)?;
        Ok(s.parse().unwrap_or(SecretStorageType::Local))
    }
}

/// Хранилище секретов
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecretStorage {
    /// Уникальный идентификатор
    #[serde(default)]
    pub id: i32,

    /// ID проекта
    #[serde(default)]
    pub project_id: i32,

    /// Название хранилища
    pub name: String,

    /// Тип хранилища
    pub r#type: SecretStorageType,

    /// Параметры (JSON)
    #[serde(default)]
    pub params: String,

    /// Только для чтения
    #[serde(default)]
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
    pub fn new(
        project_id: i32,
        name: String,
        storage_type: SecretStorageType,
        params: String,
    ) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_storage_type_display() {
        assert_eq!(SecretStorageType::Local.to_string(), "local");
        assert_eq!(SecretStorageType::Vault.to_string(), "vault");
        assert_eq!(SecretStorageType::Dvls.to_string(), "dvls");
    }

    #[test]
    fn test_secret_storage_type_from_str() {
        assert_eq!("vault".parse::<SecretStorageType>().unwrap(), SecretStorageType::Vault);
        assert_eq!("dvls".parse::<SecretStorageType>().unwrap(), SecretStorageType::Dvls);
        assert_eq!("unknown".parse::<SecretStorageType>().unwrap(), SecretStorageType::Local);
    }

    #[test]
    fn test_secret_storage_type_serialization() {
        assert_eq!(serde_json::to_string(&SecretStorageType::Local).unwrap(), "\"local\"");
    }

    #[test]
    fn test_secret_storage_new() {
        let storage = SecretStorage::new(
            10,
            "My Vault".to_string(),
            SecretStorageType::Vault,
            r#"{"url":"https://vault.example.com"}"#.to_string(),
        );
        assert_eq!(storage.id, 0);
        assert_eq!(storage.project_id, 10);
        assert_eq!(storage.r#type, SecretStorageType::Vault);
        assert!(!storage.read_only);
        assert!(storage.secret.is_none());
    }

    #[test]
    fn test_secret_storage_serialization() {
        let storage = SecretStorage {
            id: 1,
            project_id: 5,
            name: "Production Vault".to_string(),
            r#type: SecretStorageType::Vault,
            params: r#"{"url":"https://vault.prod.com"}"#.to_string(),
            read_only: true,
            source_storage_type: Some("vault".to_string()),
            secret: None,
        };
        let json = serde_json::to_string(&storage).unwrap();
        assert!(json.contains("\"name\":\"Production Vault\""));
        assert!(json.contains("\"type\":\"vault\""));
        assert!(json.contains("\"read_only\":true"));
    }

    #[test]
    fn test_secret_storage_skip_nulls() {
        let storage = SecretStorage::new(
            1,
            "Local".to_string(),
            SecretStorageType::Local,
            "{}".to_string(),
        );
        let json = serde_json::to_string(&storage).unwrap();
        assert!(!json.contains("\"source_storage_type\":"));
        assert!(!json.contains("\"secret\":"));
    }

    #[test]
    fn test_secret_storage_clone() {
        let storage = SecretStorage::new(
            5,
            "Test Vault".to_string(),
            SecretStorageType::Vault,
            "{}".to_string(),
        );
        let cloned = storage.clone();
        assert_eq!(cloned.name, storage.name);
        assert_eq!(cloned.r#type, storage.r#type);
    }

    #[test]
    fn test_secret_storage_type_clone() {
        let t = SecretStorageType::Dvls;
        let cloned = t.clone();
        assert_eq!(cloned, t);
    }

    #[test]
    fn test_secret_storage_type_deserialization() {
        assert_eq!("local".parse::<SecretStorageType>().unwrap(), SecretStorageType::Local);
        assert_eq!("vault".parse::<SecretStorageType>().unwrap(), SecretStorageType::Vault);
        assert_eq!("dvls".parse::<SecretStorageType>().unwrap(), SecretStorageType::Dvls);
        assert_eq!("invalid".parse::<SecretStorageType>().unwrap(), SecretStorageType::Local);
    }

    #[test]
    fn test_secret_storage_with_source() {
        let storage = SecretStorage {
            id: 1,
            project_id: 5,
            name: "External Vault".to_string(),
            r#type: SecretStorageType::Vault,
            params: r#"{"url":"https://ext.vault.com"}"#.to_string(),
            read_only: false,
            source_storage_type: Some("vault".to_string()),
            secret: Some("token123".to_string()),
        };
        let json = serde_json::to_string(&storage).unwrap();
        assert!(json.contains("\"source_storage_type\":\"vault\""));
        assert!(json.contains("\"secret\":\"token123\""));
    }

    #[test]
    fn test_secret_storage_type_all_variants_display() {
        assert_eq!(SecretStorageType::Local.to_string(), "local");
        assert_eq!(SecretStorageType::Vault.to_string(), "vault");
        assert_eq!(SecretStorageType::Dvls.to_string(), "dvls");
    }

    #[test]
    fn test_secret_storage_type_invalid_fallback() {
        let result = "invalid_type".parse::<SecretStorageType>();
        assert_eq!(result.unwrap(), SecretStorageType::Local);
    }

    #[test]
    fn test_secret_storage_unicode_name() {
        let storage = SecretStorage::new(
            1,
            "Хранилище".to_string(),
            SecretStorageType::Local,
            "{}".to_string(),
        );
        let json = serde_json::to_string(&storage).unwrap();
        let restored: SecretStorage = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "Хранилище");
    }

    #[test]
    fn test_secret_storage_clone_independence() {
        let mut storage = SecretStorage::new(1, "Original".to_string(), SecretStorageType::Local, "{}".to_string());
        let cloned = storage.clone();
        storage.name = "Modified".to_string();
        assert_eq!(cloned.name, "Original");
    }

    #[test]
    fn test_secret_storage_read_only_true() {
        let mut storage = SecretStorage::new(1, "RO Vault".to_string(), SecretStorageType::Vault, "{}".to_string());
        storage.read_only = true;
        let json = serde_json::to_string(&storage).unwrap();
        assert!(json.contains("\"read_only\":true"));
    }

    #[test]
    fn test_secret_storage_params_special_chars() {
        let storage = SecretStorage::new(
            1,
            "Test".to_string(),
            SecretStorageType::Local,
            r#"{"key":"value with <special> & \"chars\""}"#.to_string(),
        );
        let json = serde_json::to_string(&storage).unwrap();
        let restored: SecretStorage = serde_json::from_str(&json).unwrap();
        assert!(restored.params.contains("<special>"));
    }
}
