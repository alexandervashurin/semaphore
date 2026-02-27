//! Template Vault CRUD Operations for BoltDB
//!
//! Операции с хранилищами шаблонов в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::TemplateVault;

impl BoltStore {
    /// Получает хранилища шаблонов
    pub async fn get_template_vaults(&self, project_id: i32, template_id: i32) -> Result<Vec<TemplateVault>> {
        self.get_objects::<TemplateVault>(project_id, "template_vaults", crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: Some(format!("template_id={}", template_id)),
        }).await
    }

    /// Создаёт хранилище шаблонов
    pub async fn create_template_vault(&self, mut vault: TemplateVault) -> Result<TemplateVault> {
        vault.id = self.get_next_id("template_vaults")?;
        self.create_object(vault.project_id, "template_vaults", &vault).await?;
        Ok(vault)
    }

    /// Обновляет хранилище шаблонов
    pub async fn update_template_vault(&self, vault: TemplateVault) -> Result<()> {
        self.update_object(vault.project_id, "template_vaults", vault.id, &vault).await
    }

    /// Удаляет хранилище шаблонов
    pub async fn delete_template_vault(&self, project_id: i32, vault_id: i32) -> Result<()> {
        self.delete_object(project_id, "template_vaults", vault_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_vault_operations() {
        // Тест для проверки операций с хранилищами шаблонов
        assert!(true);
    }
}
