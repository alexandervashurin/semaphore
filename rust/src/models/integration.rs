//! Модель интеграции

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Интеграция - вебхук для внешних систем
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Integration {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub template_id: i32,
}

/// Извлекаемое значение интеграции
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationExtractValue {
    pub id: i32,
    pub integration_id: i32,
    pub name: String,
    pub value_source: String,
    pub body_data_type: String,
    pub key: Option<String>,
    pub variable: Option<String>,
}

/// Матчер интеграции
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationMatcher {
    pub id: i32,
    pub integration_id: i32,
    pub name: String,
    pub body_data_type: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub method: String,
}

/// Псевдоним интеграции
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationAlias {
    pub id: i32,
    pub integration_id: i32,
    pub alias: String,
}
