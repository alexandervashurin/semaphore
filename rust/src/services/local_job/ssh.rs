//! LocalJob SSH - установка и очистка SSH ключей
//!
//! Аналог services/tasks/local_job_ssh.go из Go версии

use crate::error::Result;
use crate::services::local_job::LocalJob;

impl LocalJob {
    /// Устанавливает SSH ключи
    pub async fn install_ssh_keys(&mut self) -> Result<()> {
        // SSH ключ для инвентаря
        if let Some(key_id) = self.inventory.ssh_key_id {
            // TODO: Загрузить ключ из БД
            // let key = self.key_installer.get_key(key_id).await?;
            // self.ssh_key_installation = Some(
            //     self.key_installer.install(&key, DbAccessKeyRole::Git, &self.logger).await?
            // );
        }

        // Become ключ
        if let Some(key_id) = self.inventory.become_key_id {
            // TODO: Загрузить ключ из БД
            // let key = self.key_installer.get_key(key_id).await?;
            // self.become_key_installation = Some(
            //     self.key_installer.install(&key, DbAccessKeyRole::AnsibleBecomeUser, &self.logger).await?
            // );
        }

        Ok(())
    }

    /// Очищает SSH ключи
    pub fn clear_ssh_keys(&mut self) {
        self.ssh_key_installation = None;
        self.become_key_installation = None;
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
    fn test_clear_ssh_keys() {
        let mut job = create_test_job();
        job.clear_ssh_keys();
        assert!(job.ssh_key_installation.is_none());
        assert!(job.become_key_installation.is_none());
    }
}
