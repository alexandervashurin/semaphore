//! Template Vault - операции с TemplateVault
//!
//! Аналог db/sql/template.go из Go версии (часть 2: TemplateVault)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает все vaults для шаблона
    pub async fn get_template_vaults(&self, project_id: i32, template_id: i32) -> Result<Vec<TemplateVault>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let vaults = sqlx::query_as::<_, TemplateVault>(
                    "SELECT * FROM template_vault WHERE template_id = ? AND project_id = ?"
                )
                .bind(template_id)
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(vaults)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт TemplateVault
    pub async fn create_template_vault(&self, mut vault: TemplateVault) -> Result<TemplateVault> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO template_vault (template_id, project_id, vault_id, vault_key_id, name) 
                     VALUES (?, ?, ?, ?, ?)"
                )
                .bind(vault.template_id)
                .bind(vault.project_id)
                .bind(vault.vault_id)
                .bind(vault.vault_key_id)
                .bind(&vault.name)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                vault.id = result.last_insert_rowid() as i32;
                Ok(vault)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Обновляет TemplateVault
    pub async fn update_template_vault(&self, vault: TemplateVault) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE template_vault SET vault_id = ?, vault_key_id = ?, name = ? 
                     WHERE id = ? AND template_id = ? AND project_id = ?"
                )
                .bind(vault.vault_id)
                .bind(vault.vault_key_id)
                .bind(&vault.name)
                .bind(vault.id)
                .bind(vault.template_id)
                .bind(vault.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Удаляет TemplateVault
    pub async fn delete_template_vault(&self, project_id: i32, template_id: i32, vault_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "DELETE FROM template_vault WHERE id = ? AND template_id = ? AND project_id = ?"
                )
                .bind(vault_id)
                .bind(template_id)
                .bind(project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Обновляет все vaults для шаблона
    pub async fn update_template_vaults(&self, project_id: i32, template_id: i32, vaults: Vec<TemplateVault>) -> Result<()> {
        // Сначала удаляем старые vaults
        sqlx::query(
            "DELETE FROM template_vault WHERE template_id = ? AND project_id = ?"
        )
        .bind(template_id)
        .bind(project_id)
        .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
        .await
        .map_err(|e| Error::Database(e))?;
        
        // Создаём новые vaults
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
    use std::env;

    async fn create_test_db() -> SqlDb {
        let temp_db = env::temp_dir().join("test_template_vault.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();
        
        // Создаём таблицу template_vault
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS template_vault (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                template_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                vault_id INTEGER NOT NULL,
                vault_key_id INTEGER,
                name TEXT NOT NULL
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();
        
        db
    }

    #[tokio::test]
    async fn test_create_and_get_template_vault() {
        let db = create_test_db().await;
        
        let vault = TemplateVault {
            id: 0,
            template_id: 1,
            project_id: 1,
            vault_id: 1,
            vault_key_id: None,
            name: "Test Vault".to_string(),
        };
        
        let created = db.create_template_vault(vault.clone()).await.unwrap();
        assert!(created.id > 0);
        
        let vaults = db.get_template_vaults(1, 1).await.unwrap();
        assert!(vaults.len() >= 1);
        assert_eq!(vaults[0].name, "Test Vault");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_template_vault() {
        let db = create_test_db().await;
        
        let vault = TemplateVault {
            id: 0,
            template_id: 1,
            project_id: 1,
            vault_id: 1,
            vault_key_id: None,
            name: "Test Vault".to_string(),
        };
        
        let created = db.create_template_vault(vault).await.unwrap();
        
        let mut updated = created.clone();
        updated.name = "Updated Vault".to_string();
        
        db.update_template_vault(updated).await.unwrap();
        
        let vaults = db.get_template_vaults(1, 1).await.unwrap();
        assert_eq!(vaults[0].name, "Updated Vault");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_template_vault() {
        let db = create_test_db().await;
        
        let vault = TemplateVault {
            id: 0,
            template_id: 1,
            project_id: 1,
            vault_id: 1,
            vault_key_id: None,
            name: "Test Vault".to_string(),
        };
        
        let created = db.create_template_vault(vault).await.unwrap();
        
        db.delete_template_vault(1, 1, created.id).await.unwrap();
        
        let vaults = db.get_template_vaults(1, 1).await.unwrap();
        assert!(vaults.is_empty());
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_template_vaults() {
        let db = create_test_db().await;
        
        // Создаём несколько vaults
        let vaults = vec![
            TemplateVault {
                id: 0,
                template_id: 1,
                project_id: 1,
                vault_id: 1,
                vault_key_id: None,
                name: "Vault 1".to_string(),
            },
            TemplateVault {
                id: 0,
                template_id: 1,
                project_id: 1,
                vault_id: 2,
                vault_key_id: None,
                name: "Vault 2".to_string(),
            },
        ];
        
        db.update_template_vaults(1, 1, vaults).await.unwrap();
        
        let result = db.get_template_vaults(1, 1).await.unwrap();
        assert_eq!(result.len(), 2);
        
        // Cleanup
        let _ = db.close().await;
    }
}
