//! LocalJob Vault - работа с Vault файлами
//!
//! Аналог services/tasks/local_job_vault.go из Go версии

use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use crate::error::Result;
use crate::services::local_job::LocalJob;
use crate::db_lib::DbAccessKeyRole;

impl LocalJob {
    /// Устанавливает файлы ключей Vault
    pub async fn install_vault_key_files(&mut self) -> Result<()> {
        self.vault_file_installations = HashMap::new();

        for vault in &self.inventory.vaults {
            if let Some(vault_key_id) = vault.vault_key_id {
                // TODO: Загрузить ключ из БД
                // let key = self.store.get_access_key(vault_key_id).await?;
                // let installation = self.key_installer.install(
                //     &key,
                //     DbAccessKeyRole::AnsiblePasswordVault,
                //     &self.logger
                // ).await?;
                // self.vault_file_installations.insert(vault.name.clone(), installation);
                
                self.log(&format!("Vault key installation pending for vault '{}' (key ID: {})", 
                    vault.name, vault_key_id));
            }
        }

        Ok(())
    }

    /// Очищает файлы ключей Vault
    pub fn clear_vault_key_files(&mut self) {
        self.vault_file_installations.clear();
    }

    /// Создаёт временный файл для пароля Vault
    pub async fn create_vault_password_file(&self, vault_name: &str, password: &str) -> Result<PathBuf> {
        let tmp_dir = &self.tmp_dir;
        let vault_password_file = tmp_dir.join(format!("vault_{}_password", vault_name));

        fs::create_dir_all(tmp_dir).await?;
        fs::write(&vault_password_file, password).await?;
        
        // Устанавливаем права 0600
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&vault_password_file).await?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&vault_password_file, perms).await?;
        }

        Ok(vault_password_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::sync::Arc;
    use crate::services::task_logger::BasicLogger;
    use crate::db_lib::AccessKeyInstallerImpl;
    use std::path::PathBuf;

    fn create_test_job() -> LocalJob {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::models::TaskStatus::Waiting,
            message: String::new(),
            commit_hash: None,
            commit_message: None,
            version: None,
            project_id: 1,
            arguments: None,
            params: String::new(),
            ..Default::default()
        };

        LocalJob::new(
            task,
            crate::models::Template::default(),
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        )
    }

    #[test]
    fn test_clear_vault_key_files() {
        let mut job = create_test_job();
        job.clear_vault_key_files();
        assert!(job.vault_file_installations.is_empty());
    }
}
