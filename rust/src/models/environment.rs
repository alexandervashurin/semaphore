//! Модель окружения (Environment)

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, decode::Decode, encode::Encode, database::Database};

/// Тип секрета окружения
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentSecretType {
    /// Переменная окружения
    Env,
    /// Секретная переменная
    Var,
}

impl<DB: Database> Type<DB> for EnvironmentSecretType {
    fn type_info() -> DB::TypeInfo {
        String::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        String::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for EnvironmentSecretType {
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = String::decode(value)?;
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
{
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s: String = match self {
            EnvironmentSecretType::Env => "env",
            EnvironmentSecretType::Var => "var",
        }.to_string();
        Encode::encode(s, buf)
    }
}

/// Секрет окружения
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EnvironmentSecret {
    pub id: i32,
    pub environment_id: i32,
    pub secret_id: i32,
    pub secret_type: EnvironmentSecretType,
}

/// Окружение - переменные окружения для задач
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
        }
    }

    /// Парсит JSON с переменными окружения
    pub fn parse_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.json)
    }
}
