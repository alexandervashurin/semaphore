//! Модель события

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Событие системы
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: i32,
    pub project_id: Option<i32>,
    pub user_id: Option<i32>,
    pub object_id: Option<i32>,
    pub object_type: String,
    pub description: String,
    pub created: DateTime<Utc>,
}
