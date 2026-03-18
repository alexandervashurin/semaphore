//! Модель представления (View)

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Представление - группировка шаблонов
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct View {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub project_id: i32,
    #[serde(alias = "name")]
    pub title: String,
    #[serde(default)]
    pub position: i32,
}

impl View {
    /// Получает имя представления (алиас на title)
    pub fn name(&self) -> &str {
        &self.title
    }
}
