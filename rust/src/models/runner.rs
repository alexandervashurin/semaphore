//! Модель раннера

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Раннер - исполнитель задач
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Runner {
    pub id: i32,
    pub project_id: Option<i32>,
    pub token: String,
    pub name: String,
    pub active: bool,
    pub last_active: Option<DateTime<Utc>>,
    
    /// Webhook URL для уведомлений
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    
    /// Максимальное количество параллельных задач
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_parallel_tasks: Option<i32>,
    
    /// Тег раннера
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    
    /// Время запроса очистки
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleaning_requested: Option<DateTime<Utc>>,
    
    /// Время последнего обращения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touched: Option<DateTime<Utc>>,
    
    /// Дата создания
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
}
