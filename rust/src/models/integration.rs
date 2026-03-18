//! Модель интеграции

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Интеграция - вебхук для внешних систем
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Integration {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub project_id: i32,
    pub name: String,
    pub template_id: i32,
    /// Метод аутентификации: "none", "hmac", "token"
    #[serde(default)]
    pub auth_method: String,
    /// Заголовок HTTP для проверки токена/подписи
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    /// ID ключа (secret) для HMAC/token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_secret_id: Option<i32>,
}

/// Извлекаемое значение интеграции
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationExtractValue {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub integration_id: i32,
    #[serde(default)]
    pub project_id: i32,
    pub name: String,
    pub value_source: String,
    pub body_data_type: String,
    pub key: Option<String>,
    pub variable: Option<String>,
    pub value_name: String,
    pub value_type: String,
}

/// Матчер интеграции
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationMatcher {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub integration_id: i32,
    #[serde(default)]
    pub project_id: i32,
    pub name: String,
    pub body_data_type: String,
    pub key: Option<String>,
    pub matcher_type: String,
    pub matcher_value: String,
    pub method: String,
}

/// Псевдоним интеграции
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationAlias {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub integration_id: i32,
    #[serde(default)]
    pub project_id: i32,
    pub alias: String,
}
