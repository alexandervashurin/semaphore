//! Terraform Inventory - операции с Terraform Inventory в SQL (PRO)
//!
//! Аналог pro/db/sql/terraform_inventory.go из Go версии

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{TerraformInventoryAlias, TerraformInventoryState, RetrieveQueryParams};
use sqlx::Row;

impl SqlDb {
    fn pg_pool_terraform(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Создаёт псевдоним для Terraform Inventory
    pub async fn create_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<TerraformInventoryAlias> {
        sqlx::query(
            "INSERT INTO terraform_inventory_alias (project_id, inventory_id, auth_key_id, alias) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (alias) DO UPDATE SET auth_key_id = EXCLUDED.auth_key_id"
        )
        .bind(alias.project_id)
        .bind(alias.inventory_id)
        .bind(alias.auth_key_id)
        .bind(&alias.alias)
        .execute(self.pg_pool_terraform()?)
        .await
        .map_err(Error::Database)?;

        Ok(alias)
    }

    /// Обновляет псевдоним
    pub async fn update_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<()> {
        sqlx::query(
            "UPDATE terraform_inventory_alias SET auth_key_id = $1 \
             WHERE alias = $2 AND project_id = $3 AND inventory_id = $4"
        )
        .bind(alias.auth_key_id)
        .bind(&alias.alias)
        .bind(alias.project_id)
        .bind(alias.inventory_id)
        .execute(self.pg_pool_terraform()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Получает псевдоним по алиасу
    pub async fn get_terraform_inventory_alias_by_alias(&self, alias: &str) -> Result<TerraformInventoryAlias> {
        let row = sqlx::query(
            "SELECT * FROM terraform_inventory_alias WHERE alias = $1"
        )
        .bind(alias)
        .fetch_one(self.pg_pool_terraform()?)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Terraform inventory alias not found".to_string()),
            _ => Error::Database(e),
        })?;

        Ok(TerraformInventoryAlias {
            project_id: row.get("project_id"),
            inventory_id: row.get("inventory_id"),
            auth_key_id: row.get("auth_key_id"),
            alias: row.get("alias"),
            task_id: None,
        })
    }

    /// Получает псевдоним по ID
    pub async fn get_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<TerraformInventoryAlias> {
        let row = sqlx::query(
            "SELECT * FROM terraform_inventory_alias WHERE alias = $1 AND project_id = $2 AND inventory_id = $3"
        )
        .bind(alias_id)
        .bind(project_id)
        .bind(inventory_id)
        .fetch_one(self.pg_pool_terraform()?)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Terraform inventory alias not found".to_string()),
            _ => Error::Database(e),
        })?;

        Ok(TerraformInventoryAlias {
            project_id: row.get("project_id"),
            inventory_id: row.get("inventory_id"),
            auth_key_id: row.get("auth_key_id"),
            alias: row.get("alias"),
            task_id: None,
        })
    }

    /// Получает все псевдонимы для инвентаря
    pub async fn get_terraform_inventory_aliases(&self, project_id: i32, inventory_id: i32) -> Result<Vec<TerraformInventoryAlias>> {
        let rows = sqlx::query(
            "SELECT * FROM terraform_inventory_alias WHERE project_id = $1 AND inventory_id = $2"
        )
        .bind(project_id)
        .bind(inventory_id)
        .fetch_all(self.pg_pool_terraform()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| TerraformInventoryAlias {
            project_id: row.get("project_id"),
            inventory_id: row.get("inventory_id"),
            auth_key_id: row.get("auth_key_id"),
            alias: row.get("alias"),
            task_id: None,
        }).collect())
    }

    /// Удаляет псевдоним
    pub async fn delete_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM terraform_inventory_alias WHERE alias = $1 AND project_id = $2 AND inventory_id = $3"
        )
        .bind(alias_id)
        .bind(project_id)
        .bind(inventory_id)
        .execute(self.pg_pool_terraform()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Получает состояния Terraform Inventory
    pub async fn get_terraform_inventory_states(&self, project_id: i32, inventory_id: i32, params: RetrieveQueryParams) -> Result<Vec<TerraformInventoryState>> {
        let limit = params.count.unwrap_or(100) as i64;
        let offset = params.offset as i64;

        let rows = sqlx::query(
            "SELECT * FROM terraform_inventory_state WHERE project_id = $1 AND inventory_id = $2 \
             ORDER BY created DESC LIMIT $3 OFFSET $4"
        )
        .bind(project_id)
        .bind(inventory_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pg_pool_terraform()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| TerraformInventoryState {
            id: row.get("id"),
            created: row.get("created"),
            task_id: row.try_get("task_id").ok().flatten(),
            project_id: row.get("project_id"),
            inventory_id: row.get("inventory_id"),
            state: row.try_get("state").ok().flatten(),
        }).collect())
    }

    /// Создаёт состояние Terraform Inventory
    pub async fn create_terraform_inventory_state(&self, mut state: TerraformInventoryState) -> Result<TerraformInventoryState> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO terraform_inventory_state (created, task_id, project_id, inventory_id, state) \
             VALUES ($1, $2, $3, $4, $5) RETURNING id"
        )
        .bind(state.created)
        .bind(state.task_id)
        .bind(state.project_id)
        .bind(state.inventory_id)
        .bind(&state.state)
        .fetch_one(self.pg_pool_terraform()?)
        .await
        .map_err(Error::Database)?;

        state.id = id;
        Ok(state)
    }

    /// Удаляет состояние
    pub async fn delete_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<()> {
        sqlx::query(
            "DELETE FROM terraform_inventory_state WHERE id = $1 AND project_id = $2 AND inventory_id = $3"
        )
        .bind(state_id)
        .bind(project_id)
        .bind(inventory_id)
        .execute(self.pg_pool_terraform()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Получает состояние по ID
    pub async fn get_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<TerraformInventoryState> {
        let row = sqlx::query(
            "SELECT * FROM terraform_inventory_state WHERE id = $1 AND project_id = $2 AND inventory_id = $3"
        )
        .bind(state_id)
        .bind(project_id)
        .bind(inventory_id)
        .fetch_one(self.pg_pool_terraform()?)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Terraform inventory state not found".to_string()),
            _ => Error::Database(e),
        })?;

        Ok(TerraformInventoryState {
            id: row.get("id"),
            created: row.get("created"),
            task_id: row.try_get("task_id").ok().flatten(),
            project_id: row.get("project_id"),
            inventory_id: row.get("inventory_id"),
            state: row.try_get("state").ok().flatten(),
        })
    }

    /// Получает количество состояний
    pub async fn get_terraform_state_count(&self) -> Result<i32> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM terraform_inventory_state"
        )
        .fetch_one(self.pg_pool_terraform()?)
        .await
        .map_err(Error::Database)?;

        Ok(count as i32)
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
