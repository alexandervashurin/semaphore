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
}

/// Тег раннера
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RunnerTag {
    pub id: i32,
    pub runner_id: i32,
    pub tag: String,
}
