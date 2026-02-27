//! PRO модели для Terraform Inventory

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Псевдоним для Terraform Inventory
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TerraformInventoryAlias {
    /// ID проекта
    pub project_id: i32,

    /// ID инвентаря
    pub inventory_id: i32,

    /// ID ключа аутентификации
    pub auth_key_id: i32,

    /// Псевдоним
    pub alias: String,

    /// ID задачи (опционально, не сохраняется в БД)
    #[serde(skip_serializing, skip_deserializing)]
    pub task_id: Option<i32>,
}

impl TerraformInventoryAlias {
    /// Создаёт новый псевдоним
    pub fn new(project_id: i32, inventory_id: i32, auth_key_id: i32, alias: String) -> Self {
        Self {
            project_id,
            inventory_id,
            auth_key_id,
            alias,
            task_id: None,
        }
    }

    /// Конвертирует в базовый Alias
    pub fn to_alias(&self) -> Alias {
        Alias {
            alias: self.alias.clone(),
            project_id: self.project_id,
        }
    }
}

/// Базовый псевдоним
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alias {
    pub alias: String,
    pub project_id: i32,
}

/// Состояние Terraform Inventory
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TerraformInventoryState {
    /// Уникальный идентификатор
    pub id: i32,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// ID задачи
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<i32>,

    /// ID проекта
    pub project_id: i32,

    /// ID инвентаря
    pub inventory_id: i32,

    /// Состояние (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl TerraformInventoryState {
    /// Создаёт новое состояние
    pub fn new(project_id: i32, inventory_id: i32, state: String) -> Self {
        Self {
            id: 0,
            created: Utc::now(),
            task_id: None,
            project_id,
            inventory_id,
            state: Some(state),
        }
    }
}
