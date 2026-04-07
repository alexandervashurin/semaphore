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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terraform_inventory_alias_new() {
        let alias = TerraformInventoryAlias::new(
            10, 5, 3, "prod-inventory".to_string(),
        );
        assert_eq!(alias.project_id, 10);
        assert_eq!(alias.inventory_id, 5);
        assert_eq!(alias.alias, "prod-inventory");
        assert!(alias.task_id.is_none());
    }

    #[test]
    fn test_terraform_inventory_alias_to_alias() {
        let tf_alias = TerraformInventoryAlias::new(
            10, 5, 3, "test-alias".to_string(),
        );
        let base = tf_alias.to_alias();
        assert_eq!(base.alias, "test-alias");
        assert_eq!(base.project_id, 10);
    }

    #[test]
    fn test_terraform_inventory_alias_serialization() {
        let alias = TerraformInventoryAlias::new(
            10, 5, 3, "prod".to_string(),
        );
        let json = serde_json::to_string(&alias).unwrap();
        assert!(json.contains("\"alias\":\"prod\""));
        assert!(json.contains("\"project_id\":10"));
    }

    #[test]
    fn test_terraform_inventory_state_new() {
        let state = TerraformInventoryState::new(10, 5, "{\"resources\":[]}".to_string());
        assert_eq!(state.id, 0);
        assert_eq!(state.project_id, 10);
        assert_eq!(state.inventory_id, 5);
        assert_eq!(state.state, Some("{\"resources\":[]}".to_string()));
    }

    #[test]
    fn test_terraform_inventory_state_serialization() {
        let state = TerraformInventoryState {
            id: 1,
            created: Utc::now(),
            task_id: Some(100),
            project_id: 10,
            inventory_id: 5,
            state: Some("{\"outputs\":{}}".to_string()),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"project_id\":10"));
        assert!(json.contains("\"task_id\":100"));
    }

    #[test]
    fn test_alias_struct() {
        let alias = Alias {
            alias: "my-alias".to_string(),
            project_id: 42,
        };
        assert_eq!(alias.alias, "my-alias");
        assert_eq!(alias.project_id, 42);
    }

    #[test]
    fn test_terraform_inventory_state_no_task_id() {
        let state = TerraformInventoryState::new(10, 5, "{}".to_string());
        assert!(state.task_id.is_none());
    }
}
