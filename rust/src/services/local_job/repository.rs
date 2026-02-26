//! LocalJob Repository - работа с Git репозиторием
//!
//! Аналог services/tasks/local_job_repository.go из Go версии

use crate::error::Result;
use crate::services::local_job::LocalJob;

impl LocalJob {
    /// Обновляет репозиторий
    pub async fn update_repository(&mut self) -> Result<()> {
        self.log(&format!("Updating repository: {}", self.repository.git_url));

        // TODO: Использовать GitRepository для clone/pull
        // let git_repo = GitRepository::new(
        //     &self.repository,
        //     &self.tmp_dir,
        //     self.logger.clone(),
        //     self.key_installer.clone(),
        // )?;
        // git_repo.clone_or_pull().await?;

        self.log("Repository update completed (pending implementation)");
        Ok(())
    }

    /// Переключает репозиторий на нужный коммит/ветку
    pub async fn checkout_repository(&mut self) -> Result<()> {
        // TODO: Использовать GitRepository для checkout
        // let git_repo = GitRepository::new(...)?;
        
        if let Some(ref commit_hash) = self.task.commit_hash {
            self.log(&format!("Checking out commit: {}", commit_hash));
            // git_repo.checkout(commit_hash).await?;
            
            self.set_commit(commit_hash, &self.task.commit_message.clone().unwrap_or_default());
        } else if !self.repository.git_branch.is_empty() {
            self.log(&format!("Checking out branch: {}", self.repository.git_branch));
            // git_repo.checkout(&self.repository.git_branch).await?;
        }

        self.log("Repository checkout completed (pending implementation)");
        Ok(())
    }

    /// Получает полный путь к репозиторию
    pub fn get_repository_path(&self) -> std::path::PathBuf {
        self.tmp_dir.join("repository")
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
    fn test_update_repository() {
        // Просто проверяем, что метод вызывается без паники
        let mut job = create_test_job();
        let result = futures::executor::block_on(job.update_repository());
        assert!(result.is_ok()); // Пока всегда Ok

    }

    #[test]
    fn test_checkout_repository() {
        // Просто проверяем, что метод вызывается без паники
        let mut job = create_test_job();
        let result = futures::executor::block_on(job.checkout_repository());
        assert!(result.is_ok()); // Пока всегда Ok
    }
}
