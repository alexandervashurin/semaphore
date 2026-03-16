//! LocalJob SSH - установка и очистка SSH ключей
//!
//! Аналог services/tasks/local_job_ssh.go из Go версии

use crate::error::Result;
use crate::services::local_job::LocalJob;
use crate::db_lib::{DbAccessKeyRole, AccessKeyInstallerTrait};

impl LocalJob {
    /// Устанавливает SSH ключи
    pub async fn install_ssh_keys(&mut self) -> Result<()> {
        // SSH ключ для инвентаря
        if let Some(key_id) = self.inventory.ssh_key_id {
            if let Some(store) = &self.store {
                match store.get_access_key(self.task.project_id, key_id).await {
                    Ok(ak) => {
                        let db_key = model_access_key_to_db(&ak);
                        match self.key_installer.install(&db_key, DbAccessKeyRole::AnsibleUser, self.logger.as_ref()) {
                            Ok(installation) => {
                                self.ssh_key_installation = Some(installation);
                                self.log(&format!("SSH key {} installed", ak.name));
                            }
                            Err(e) => {
                                self.log(&format!("SSH key install failed: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        self.log(&format!("Failed to load SSH key {}: {}", key_id, e));
                    }
                }
            } else {
                self.log(&format!("SSH key installation pending for key ID: {}", key_id));
            }
        }

        // Become ключ
        if let Some(key_id) = self.inventory.become_key_id {
            if let Some(store) = &self.store {
                match store.get_access_key(self.task.project_id, key_id).await {
                    Ok(ak) => {
                        let db_key = model_access_key_to_db(&ak);
                        match self.key_installer.install(&db_key, DbAccessKeyRole::AnsibleBecomeUser, self.logger.as_ref()) {
                            Ok(installation) => {
                                self.become_key_installation = Some(installation);
                                self.log(&format!("Become key {} installed", ak.name));
                            }
                            Err(e) => {
                                self.log(&format!("Become key install failed: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        self.log(&format!("Failed to load become key {}: {}", key_id, e));
                    }
                }
            } else {
                self.log(&format!("Become key installation pending for key ID: {}", key_id));
            }
        }

        Ok(())
    }

    /// Очищает SSH ключи
    pub fn clear_ssh_keys(&mut self) {
        self.ssh_key_installation = None;
        self.become_key_installation = None;
    }
}

/// Конвертирует AccessKey модель в DbAccessKey для установщика
pub fn model_access_key_to_db(ak: &crate::models::AccessKey) -> crate::db_lib::DbAccessKey {
    use crate::db_lib::{DbAccessKey, DbAccessKeyType, DbAccessKeyOwner, DbSshKey, DbLoginPassword};
    use crate::models::access_key::AccessKeyType;

    let key_type = match ak.r#type {
        AccessKeyType::SSH => DbAccessKeyType::Ssh,
        AccessKeyType::LoginPassword => DbAccessKeyType::LoginPassword,
        AccessKeyType::None | AccessKeyType::AccessKey => DbAccessKeyType::None,
    };

    let ssh_key = if key_type == DbAccessKeyType::Ssh {
        Some(DbSshKey {
            login: String::new(),
            passphrase: ak.ssh_passphrase.clone().unwrap_or_default(),
            private_key: ak.ssh_key.clone().unwrap_or_default(),
        })
    } else {
        None
    };

    let login_password = if key_type == DbAccessKeyType::LoginPassword {
        Some(DbLoginPassword {
            login: ak.login_password_login.clone().unwrap_or_default(),
            password: ak.login_password_password.clone().unwrap_or_default(),
        })
    } else {
        None
    };

    DbAccessKey {
        id: ak.id,
        name: ak.name.clone(),
        key_type,
        project_id: ak.project_id,
        secret: None,
        plain: None,
        string_value: None,
        login_password,
        ssh_key,
        override_secret: false,
        storage_id: None,
        environment_id: None,
        user_id: ak.user_id,
        empty: false,
        owner: DbAccessKeyOwner::Shared,
        source_storage_id: None,
        source_storage_key: None,
        source_storage_type: None,
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
    fn test_clear_ssh_keys() {
        let mut job = create_test_job();
        job.clear_ssh_keys();
        assert!(job.ssh_key_installation.is_none());
        assert!(job.become_key_installation.is_none());
    }
}
