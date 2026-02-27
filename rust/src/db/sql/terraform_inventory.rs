//! Terraform Inventory - операции с Terraform Inventory в SQL (PRO)
//!
//! Аналог pro/db/sql/terraform_inventory.go из Go версии

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{TerraformInventoryAlias, TerraformInventoryState, RetrieveQueryParams};

impl SqlDb {
    /// Создаёт псевдоним для Terraform Inventory
    pub async fn create_terraform_inventory_alias(&self, mut alias: TerraformInventoryAlias) -> Result<TerraformInventoryAlias> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO project__terraform_inventory_alias (project_id, inventory_id, auth_key_id, alias)
                     VALUES (?, ?, ?, ?)"
                )
                .bind(alias.project_id)
                .bind(alias.inventory_id)
                .bind(alias.auth_key_id)
                .bind(&alias.alias)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                // В SQLite last_insert_rowid() для составных ключей не нужен
                // alias.id = result.last_insert_rowid() as i32;
                
                Ok(alias)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Обновляет псевдоним
    pub async fn update_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE project__terraform_inventory_alias 
                     SET auth_key_id = ? 
                     WHERE project_id = ? AND inventory_id = ? AND alias = ?"
                )
                .bind(alias.auth_key_id)
                .bind(alias.project_id)
                .bind(alias.inventory_id)
                .bind(&alias.alias)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает псевдоним по алиасу
    pub async fn get_terraform_inventory_alias_by_alias(&self, alias: &str) -> Result<TerraformInventoryAlias> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query_as::<_, TerraformInventoryAlias>(
                    "SELECT * FROM project__terraform_inventory_alias WHERE alias = ?"
                )
                .bind(alias)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                result.ok_or(Error::NotFound("Terraform inventory alias not found".to_string()))
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает псевдоним по ID
    pub async fn get_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<TerraformInventoryAlias> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query_as::<_, TerraformInventoryAlias>(
                    "SELECT * FROM project__terraform_inventory_alias 
                     WHERE project_id = ? AND inventory_id = ? AND alias = ?"
                )
                .bind(project_id)
                .bind(inventory_id)
                .bind(alias_id)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                result.ok_or(Error::NotFound("Terraform inventory alias not found".to_string()))
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает все псевдонимы для инвентаря
    pub async fn get_terraform_inventory_aliases(&self, project_id: i32, inventory_id: i32) -> Result<Vec<TerraformInventoryAlias>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let aliases = sqlx::query_as::<_, TerraformInventoryAlias>(
                    "SELECT * FROM project__terraform_inventory_alias 
                     WHERE project_id = ? AND inventory_id = ?
                     ORDER BY alias"
                )
                .bind(project_id)
                .bind(inventory_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(aliases)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Удаляет псевдоним
    pub async fn delete_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "DELETE FROM project__terraform_inventory_alias 
                     WHERE project_id = ? AND inventory_id = ? AND alias = ?"
                )
                .bind(project_id)
                .bind(inventory_id)
                .bind(alias_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает состояния Terraform Inventory
    pub async fn get_terraform_inventory_states(&self, project_id: i32, inventory_id: i32, params: RetrieveQueryParams) -> Result<Vec<TerraformInventoryState>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let limit = params.count.unwrap_or(100) as i64;
                let offset = params.offset as i64;

                let states = sqlx::query_as::<_, TerraformInventoryState>(
                    "SELECT * FROM project__terraform_inventory_state 
                     WHERE project_id = ? AND inventory_id = ?
                     ORDER BY created DESC
                     LIMIT ? OFFSET ?"
                )
                .bind(project_id)
                .bind(inventory_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(states)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Создаёт состояние Terraform Inventory
    pub async fn create_terraform_inventory_state(&self, mut state: TerraformInventoryState) -> Result<TerraformInventoryState> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO project__terraform_inventory_state (created, task_id, project_id, inventory_id, state)
                     VALUES (?, ?, ?, ?, ?)"
                )
                .bind(state.created)
                .bind(state.task_id)
                .bind(state.project_id)
                .bind(state.inventory_id)
                .bind(&state.state)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                state.id = result.last_insert_rowid() as i32;
                Ok(state)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Удаляет состояние
    pub async fn delete_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "DELETE FROM project__terraform_inventory_state 
                     WHERE id = ? AND project_id = ? AND inventory_id = ?"
                )
                .bind(state_id)
                .bind(project_id)
                .bind(inventory_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает состояние по ID
    pub async fn get_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<TerraformInventoryState> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query_as::<_, TerraformInventoryState>(
                    "SELECT * FROM project__terraform_inventory_state 
                     WHERE id = ? AND project_id = ? AND inventory_id = ?"
                )
                .bind(state_id)
                .bind(project_id)
                .bind(inventory_id)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                result.ok_or(Error::NotFound("Terraform inventory state not found".to_string()))
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает количество состояний
    pub async fn get_terraform_state_count(&self) -> Result<i32> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM project__terraform_inventory_state")
                    .fetch_one(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(result)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terraform_inventory_alias_creation() {
        let alias = TerraformInventoryAlias::new(1, 2, 3, "test-alias".to_string());
        assert_eq!(alias.project_id, 1);
        assert_eq!(alias.inventory_id, 2);
        assert_eq!(alias.auth_key_id, 3);
        assert_eq!(alias.alias, "test-alias");
    }

    #[test]
    fn test_terraform_inventory_state_creation() {
        let state = TerraformInventoryState::new(1, 2, "{\"resources\": []}".to_string());
        assert_eq!(state.project_id, 1);
        assert_eq!(state.inventory_id, 2);
        assert!(state.state.is_some());
    }
}
