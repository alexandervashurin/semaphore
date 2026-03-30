//! Template Vault - операции с TemplateVault
//!
//! Аналог db/sql/template.go из Go версии (часть 2: TemplateVault)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_template_vault(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает все vaults для шаблона
    pub async fn get_template_vaults(&self, project_id: i32, template_id: i32) -> Result<Vec<TemplateVault>> {
        let rows = sqlx::query(
            "SELECT * FROM template_vault WHERE template_id = $1 AND project_id = $2"
        )
        .bind(template_id)
        .bind(project_id)
        .fetch_all(self.pg_pool_template_vault()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| TemplateVault {
            id: row.get("id"),
            template_id: row.get("template_id"),
            project_id: row.get("project_id"),
            vault_id: row.get("vault_id"),
            vault_key_id: row.try_get("vault_key_id").ok().unwrap_or(0),
            name: row.get("name"),
        }).collect())
    }

    /// Создаёт TemplateVault
    pub async fn create_template_vault(&self, mut vault: TemplateVault) -> Result<TemplateVault> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO template_vault (template_id, project_id, vault_id, vault_key_id, name) \
             VALUES ($1, $2, $3, $4, $5) RETURNING id"
        )
        .bind(vault.template_id)
        .bind(vault.project_id)
        .bind(vault.vault_id)
        .bind(vault.vault_key_id)
        .bind(&vault.name)
        .fetch_one(self.pg_pool_template_vault()?)
        .await
        .map_err(Error::Database)?;

        vault.id = id;
        Ok(vault)
    }

    /// Обновляет TemplateVault
    pub async fn update_template_vault(&self, vault: TemplateVault) -> Result<()> {
        sqlx::query(
            "UPDATE template_vault SET vault_id = $1, vault_key_id = $2, name = $3 \
             WHERE id = $4 AND template_id = $5 AND project_id = $6"
        )
        .bind(vault.vault_id)
        .bind(vault.vault_key_id)
        .bind(&vault.name)
        .bind(vault.id)
        .bind(vault.template_id)
        .bind(vault.project_id)
        .execute(self.pg_pool_template_vault()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет TemplateVault
    pub async fn delete_template_vault(&self, project_id: i32, template_id: i32, vault_id: i32) -> Result<()> {
        sqlx::query(
            "DELETE FROM template_vault WHERE id = $1 AND template_id = $2 AND project_id = $3"
        )
        .bind(vault_id)
        .bind(template_id)
        .bind(project_id)
        .execute(self.pg_pool_template_vault()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Обновляет все vaults для шаблона
    pub async fn update_template_vaults(&self, project_id: i32, template_id: i32, vaults: Vec<TemplateVault>) -> Result<()> {
        sqlx::query(
            "DELETE FROM template_vault WHERE template_id = $1 AND project_id = $2"
        )
        .bind(template_id)
        .bind(project_id)
        .execute(self.pg_pool_template_vault()?)
        .await
        .map_err(Error::Database)?;

        for mut vault in vaults {
            vault.template_id = template_id;
            vault.project_id = project_id;
            self.create_template_vault(vault).await?;
        }

        Ok(())
    }
}
