//! Модель представления (View)

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Представление - группировка шаблонов
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct View {
    pub id: i32,
    pub project_id: i32,
    pub title: String,
    pub position: i32,
}
