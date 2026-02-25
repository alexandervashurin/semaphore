//! Модель роли

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Роль - набор разрешений
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}
