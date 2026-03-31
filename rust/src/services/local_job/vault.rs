//! LocalJob Vault - работа с Vault файлами
//!
//! Аналог services/tasks/local_job_vault.go из Go версии

use crate::error::Result;
use crate::services::local_job::LocalJob;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

impl LocalJob {
    /// Устанавливает файлы ключей Vault
    pub async fn install_vault_key_files(&mut self) -> Result<()> {
        self.vault_file_installations = HashMap::new();

        #[derive(serde::Deserialize)]
        struct VaultRef {
            vault_key_id: i32,
            #[serde(default)]
            r#type: String,
        }

        // Загружаем vault-ключи из шаблона (приоритет) или инвентаря
        let vault_refs: Vec<VaultRef> = if let Some(ref vaults_val) = self.template.vaults {
            match serde_json::from_value(vaults_val.clone()) {
                Ok(v) => v,
                Err(e) => {
                    self.log(&format!(
                        "Warning: failed to parse template vault refs: {}",
                        e
                    ));
                    return Ok(());
                }
            }
        } else if let Some(ref vaults_json) = self.inventory.vaults {
            if vaults_json.is_empty() {
                return Ok(());
            }
            match serde_json::from_str(vaults_json) {
                Ok(v) => v,
                Err(e) => {
                    self.log(&format!("Warning: failed to parse vault refs: {}", e));
                    return Ok(());
                }
            }
        } else {
            return Ok(());
        };

        if vault_refs.is_empty() {
            return Ok(());
        }

        let store = match self.store.as_ref() {
            Some(s) => s.clone(),
            None => {
                self.log("Warning: no store available for vault key loading");
                return Ok(());
            }
        };

        for (i, vref) in vault_refs.iter().enumerate() {
            use crate::db::store::AccessKeyManager;
            let key = match store
                .get_access_key(self.task.project_id, vref.vault_key_id)
                .await
            {
                Ok(k) => k,
                Err(e) => {
                    self.log(&format!(
                        "Warning: vault key {} not found: {}",
                        vref.vault_key_id, e
                    ));
                    continue;
                }
            };

            // Sanitize vault_name: only allow [a-zA-Z0-9_-] to prevent path traversal
            let raw_type = if vref.r#type.is_empty() {
                format!("vault_{}", i)
            } else {
                vref.r#type.clone()
            };
            let vault_name: String = raw_type
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if vault_name.is_empty() {
                self.log(&format!(
                    "Warning: vault type '{}' contains no valid chars, skipping",
                    raw_type
                ));
                continue;
            }

            // Получаем пароль из ключа
            let password = key.login_password_password.as_deref().unwrap_or("");

            if !password.is_empty() {
                let _vault_file = self
                    .create_vault_password_file(&vault_name, password)
                    .await?;
                self.log(&format!("Vault key installed: {}", vault_name));
                let installation = crate::services::ssh_agent::AccessKeyInstallation {
                    password: Some(password.to_string()),
                    ..Default::default()
                };
                self.vault_file_installations
                    .insert(vault_name, installation);
            }
        }

        Ok(())
    }

    /// Очищает файлы ключей Vault
    pub fn clear_vault_key_files(&mut self) {
        self.vault_file_installations.clear();
    }

    /// Создаёт временный файл для пароля Vault
    pub async fn create_vault_password_file(
        &self,
        vault_name: &str,
        password: &str,
    ) -> Result<PathBuf> {
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
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::services::task_logger::BasicLogger;
    use chrono::Utc;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn create_test_job() -> LocalJob {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            message: None,
            commit_hash: None,
            commit_message: None,
            version: None,
            project_id: 1,
            arguments: None,
            params: None,
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
