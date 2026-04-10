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
    pub async fn get_template_vaults(
        &self,
        project_id: i32,
        template_id: i32,
    ) -> Result<Vec<TemplateVault>> {
        let rows =
            sqlx::query("SELECT * FROM template_vault WHERE template_id = $1 AND project_id = $2")
                .bind(template_id)
                .bind(project_id)
                .fetch_all(self.pg_pool_template_vault()?)
                .await
                .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| TemplateVault {
                id: row.get("id"),
                template_id: row.get("template_id"),
                project_id: row.get("project_id"),
                vault_id: row.get("vault_id"),
                vault_key_id: row.try_get("vault_key_id").ok().unwrap_or(0),
                name: row.get("name"),
            })
            .collect())
    }

    /// Создаёт TemplateVault
    pub async fn create_template_vault(&self, mut vault: TemplateVault) -> Result<TemplateVault> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO template_vault (template_id, project_id, vault_id, vault_key_id, name) \
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
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
             WHERE id = $4 AND template_id = $5 AND project_id = $6",
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
    pub async fn delete_template_vault(
        &self,
        project_id: i32,
        template_id: i32,
        vault_id: i32,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM template_vault WHERE id = $1 AND template_id = $2 AND project_id = $3",
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
    pub async fn update_template_vaults(
        &self,
        project_id: i32,
        template_id: i32,
        vaults: Vec<TemplateVault>,
    ) -> Result<()> {
        sqlx::query("DELETE FROM template_vault WHERE template_id = $1 AND project_id = $2")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_vault_struct_fields() {
        let vault = TemplateVault {
            id: 1,
            template_id: 10,
            project_id: 5,
            vault_id: 3,
            vault_key_id: 2,
            name: "Production Vault".to_string(),
        };
        assert_eq!(vault.id, 1);
        assert_eq!(vault.template_id, 10);
        assert_eq!(vault.vault_id, 3);
        assert_eq!(vault.name, "Production Vault");
    }

    #[test]
    fn test_template_vault_clone() {
        let vault = TemplateVault {
            id: 5,
            template_id: 20,
            project_id: 15,
            vault_id: 8,
            vault_key_id: 4,
            name: "Dev Vault".to_string(),
        };
        let cloned = vault.clone();
        assert_eq!(cloned.id, vault.id);
        assert_eq!(cloned.name, vault.name);
        assert_eq!(cloned.vault_key_id, vault.vault_key_id);
    }

    #[test]
    fn test_template_vault_serialization() {
        let vault = TemplateVault {
            id: 1,
            template_id: 10,
            project_id: 5,
            vault_id: 3,
            vault_key_id: 2,
            name: "Test Vault".to_string(),
        };
        let json = serde_json::to_string(&vault).unwrap();
        assert!(json.contains("\"name\":\"Test Vault\""));
        assert!(json.contains("\"vault_id\":3"));
        assert!(json.contains("\"template_id\":10"));
    }

    #[test]
    fn test_template_vault_deserialization() {
        let json = r#"{"id":10,"template_id":50,"project_id":25,"vault_id":15,"vault_key_id":7,"name":"Staging Vault"}"#;
        let vault: TemplateVault = serde_json::from_str(json).unwrap();
        assert_eq!(vault.id, 10);
        assert_eq!(vault.name, "Staging Vault");
        assert_eq!(vault.vault_key_id, 7);
    }

    #[test]
    fn test_template_vault_equality() {
        let vault1 = TemplateVault {
            id: 1, template_id: 10, project_id: 5, vault_id: 3, vault_key_id: 2,
            name: "A".to_string(),
        };
        let vault2 = TemplateVault {
            id: 1, template_id: 10, project_id: 5, vault_id: 3, vault_key_id: 2,
            name: "A".to_string(),
        };
        let vault3 = TemplateVault {
            id: 2, template_id: 10, project_id: 5, vault_id: 3, vault_key_id: 2,
            name: "A".to_string(),
        };
        assert_eq!(vault1.id, vault2.id);
        assert_ne!(vault1.id, vault3.id);
    }

    #[test]
    fn test_template_vault_all_fields_set() {
        let vault = TemplateVault {
            id: 100,
            template_id: 200,
            project_id: 300,
            vault_id: 400,
            vault_key_id: 500,
            name: "Full Vault".to_string(),
        };
        assert_eq!(vault.id, 100);
        assert_eq!(vault.template_id, 200);
        assert_eq!(vault.project_id, 300);
        assert_eq!(vault.vault_id, 400);
        assert_eq!(vault.vault_key_id, 500);
        assert_eq!(vault.name, "Full Vault");
    }

    #[test]
    fn test_template_vault_name_variants() {
        let names = ["vault", "Vault 1", "Production-Vault", "vault_name_with_underscores"];
        for name in &names {
            let vault = TemplateVault {
                id: 1, template_id: 1, project_id: 1, vault_id: 1, vault_key_id: 1,
                name: name.to_string(),
            };
            let json = serde_json::to_string(&vault).unwrap();
            assert!(json.contains(name));
        }
    }

    #[test]
    fn test_template_vault_zero_values() {
        let vault = TemplateVault {
            id: 0,
            template_id: 0,
            project_id: 0,
            vault_id: 0,
            vault_key_id: 0,
            name: String::new(),
        };
        assert_eq!(vault.id, 0);
        assert_eq!(vault.vault_key_id, 0);
        assert!(vault.name.is_empty());
    }

    #[test]
    fn test_template_vault_vec_serialization() {
        let vaults = vec![
            TemplateVault { id: 1, template_id: 10, project_id: 5, vault_id: 1, vault_key_id: 1, name: "V1".to_string() },
            TemplateVault { id: 2, template_id: 10, project_id: 5, vault_id: 2, vault_key_id: 2, name: "V2".to_string() },
        ];
        let json = serde_json::to_string(&vaults).unwrap();
        assert!(json.contains("\"V1\""));
        assert!(json.contains("\"V2\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"id\":2"));
    }

    #[test]
    fn test_template_vault_debug() {
        let vault = TemplateVault {
            id: 1, template_id: 10, project_id: 5, vault_id: 3, vault_key_id: 2,
            name: "Debug Vault".to_string(),
        };
        let debug_str = format!("{:?}", vault);
        assert!(debug_str.contains("Debug Vault"));
        assert!(debug_str.contains("TemplateVault"));
    }
}
