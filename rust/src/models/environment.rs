//! Модель окружения (Environment)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{database::Database, decode::Decode, encode::Encode, FromRow, Type};

/// Тип секрета окружения
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentSecretType {
    /// Переменная окружения
    Env,
    /// Секретная переменная
    Var,
}

impl<DB: Database> Type<DB> for EnvironmentSecretType
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

impl<'r, DB: Database> Decode<'r, DB> for EnvironmentSecretType
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "env" => EnvironmentSecretType::Env,
            "var" => EnvironmentSecretType::Var,
            _ => EnvironmentSecretType::Env,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for EnvironmentSecretType
where
    DB: 'q,
    String: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            EnvironmentSecretType::Env => "env",
            EnvironmentSecretType::Var => "var",
        }
        .to_string();
        <String as Encode<'q, DB>>::encode(s, buf)
    }
}

/// Секрет окружения (DB row)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EnvironmentSecret {
    pub id: i32,
    pub environment_id: i32,
    pub secret_id: i32,
    pub secret_type: EnvironmentSecretType,
}

/// Значение секрета окружения (хранится в JSON строке `environment.secrets`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSecretValue {
    pub name: String,
    pub secret: String,
    pub secret_type: EnvironmentSecretType,
}

/// Окружение - переменные окружения для задач
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct Environment {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Название окружения
    pub name: String,

    /// JSON с переменными окружения
    pub json: String,

    /// ID хранилища секретов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_storage_id: Option<i32>,

    /// Префикс ключей в хранилище секретов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_storage_key_prefix: Option<String>,

    /// Секреты окружения
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<String>,

    /// Дата создания
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<Utc>>,
}

impl Environment {
    /// Создаёт новое окружение
    pub fn new(project_id: i32, name: String, json: String) -> Self {
        Self {
            id: 0,
            project_id,
            name,
            json,
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        }
    }

    /// Парсит JSON с переменными окружения
    pub fn parse_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_secret_type_display() {
        assert_eq!(
            serde_json::to_string(&EnvironmentSecretType::Env).unwrap(),
            "\"env\""
        );
        assert_eq!(
            serde_json::to_string(&EnvironmentSecretType::Var).unwrap(),
            "\"var\""
        );
    }

    #[test]
    fn test_environment_secret_serialization() {
        let secret = EnvironmentSecret {
            id: 1,
            environment_id: 10,
            secret_id: 5,
            secret_type: EnvironmentSecretType::Env,
        };
        let json = serde_json::to_string(&secret).unwrap();
        assert!(json.contains("\"environment_id\":10"));
        assert!(json.contains("\"secret_id\":5"));
    }

    #[test]
    fn test_environment_secret_value_serialization() {
        let value = EnvironmentSecretValue {
            name: "DB_PASSWORD".to_string(),
            secret: "encrypted_value".to_string(),
            secret_type: EnvironmentSecretType::Var,
        };
        let json = serde_json::to_string(&value).unwrap();
        assert!(json.contains("\"name\":\"DB_PASSWORD\""));
        assert!(json.contains("\"secret_type\":\"var\""));
    }

    #[test]
    fn test_environment_new() {
        let env = Environment::new(10, "production".to_string(), r#"{"KEY":"value"}"#.to_string());
        assert_eq!(env.id, 0);
        assert_eq!(env.project_id, 10);
        assert_eq!(env.name, "production");
        assert!(env.secret_storage_id.is_none());
    }

    #[test]
    fn test_environment_parse_json() {
        let env = Environment {
            id: 1,
            project_id: 10,
            name: "test".to_string(),
            json: r#"{"KEY1":"val1","KEY2":"val2"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };
        let parsed = env.parse_json().unwrap();
        assert_eq!(parsed["KEY1"], "val1");
        assert_eq!(parsed["KEY2"], "val2");
    }

    #[test]
    fn test_environment_parse_invalid_json() {
        let env = Environment {
            id: 1,
            project_id: 10,
            name: "test".to_string(),
            json: "not valid json".to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };
        assert!(env.parse_json().is_err());
    }

    #[test]
    fn test_environment_default() {
        let env = Environment::default();
        assert_eq!(env.id, 0);
        assert!(env.name.is_empty());
        assert!(env.json.is_empty());
    }

    #[test]
    fn test_environment_serialization_skip_nulls() {
        let env = Environment::default();
        let json = serde_json::to_string(&env).unwrap();
        assert!(!json.contains("secret_storage_id"));
        assert!(!json.contains("secret_storage_key_prefix"));
        assert!(!json.contains("secrets"));
    }
}
