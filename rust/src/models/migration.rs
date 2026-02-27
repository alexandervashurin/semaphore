//! Migration Model
//!
//! Модель миграции БД

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Миграция БД
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Migration {
    /// Уникальный идентификатор
    pub id: i32,

    /// Версия миграции
    pub version: i64,

    /// Название миграции
    pub name: String,

    /// Дата применения
    pub applied: DateTime<Utc>,
}

impl Migration {
    /// Создаёт новую миграцию
    pub fn new(version: i64, name: String) -> Self {
        Self {
            id: 0,
            version,
            name,
            applied: Utc::now(),
        }
    }
}
