//! Alias Model
//!
//! Модель псевдонима

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Псевдоним
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alias {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Псевдоним
    pub alias: String,

    /// ID владельца
    pub owner_id: i32,

    /// Тип владельца
    pub owner_type: String,

    /// Дата создания
    pub created: DateTime<Utc>,
}

impl Alias {
    /// Создаёт новый псевдоним
    pub fn new(project_id: i32, alias: String, owner_id: i32, owner_type: String) -> Self {
        Self {
            id: 0,
            project_id,
            alias,
            owner_id,
            owner_type,
            created: Utc::now(),
        }
    }
}
