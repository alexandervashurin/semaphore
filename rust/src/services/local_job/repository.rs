//! LocalJob Repository - работа с Git репозиторием
//!
//! Аналог services/tasks/local_job_repository.go из Go версии

use crate::error::Result;
use crate::services::local_job::LocalJob;

impl LocalJob {
    /// Обновляет репозиторий
    pub async fn update_repository(&mut self) -> Result<()> {
        self.log(&format!("Updating repository: {}", self.repository.git_url));

        let repo_path = self.get_repository_path();
        std::fs::create_dir_all(&repo_path)?;

        if self.repository.git_url.starts_with("file://") {
            // Для локальных репозиториев — копируем файлы напрямую
            let src_path = self.repository.git_url.trim_start_matches("file://");
            let src = std::path::Path::new(src_path);
            if src.is_dir() {
                if let Err(e) = copy_dir_recursive(src, &repo_path) {
                    self.log(&format!("Warning: could not copy local repo: {e}"));
                } else {
                    self.log(&format!("Copied local repository from {src_path}"));
                }
            } else {
                self.log(&format!("Warning: local path {src_path} not found, using empty directory"));
            }
        } else if !self.repository.git_url.is_empty() {
            // Используем GitRepository для clone/pull
            use crate::services::git_repository::GitRepository;
            let git_repo = GitRepository::new(
                self.repository.clone(),
                self.task.project_id,
                self.task.template_id,
            ).with_tmp_dir(format!("task_{}", self.task.id));
            let full_path = git_repo.get_full_path();
            let result = if full_path.exists() && full_path.join(".git").exists() {
                git_repo.pull().await
            } else {
                git_repo.clone().await
            };
            match result {
                Ok(()) => {
                    self.log("Repository cloned/updated");
                    // Копируем в repo_path
                    if let Err(e) = copy_dir_recursive(&full_path, &repo_path) {
                        self.log(&format!("Warning: could not copy repo: {e}"));
                    }
                }
                Err(e) => self.log(&format!("Warning: git error: {e}, using existing directory")),
            }
        }

        self.log("Repository update completed");
        Ok(())
    }

    /// Переключает репозиторий на нужный коммит/ветку
    pub async fn checkout_repository(&mut self) -> Result<()> {
        use crate::services::git_repository::GitRepository;

        let git_repo = GitRepository::new(
            self.repository.clone(),
            self.task.project_id,
            self.task.template_id,
        ).with_tmp_dir(format!("task_{}", self.task.id));

        if let Some(commit_hash) = self.task.commit_hash.clone() {
            self.log(&format!("Checking out commit: {}", commit_hash));
            git_repo.checkout(&commit_hash).await?;
            let msg = self.task.commit_message.clone().unwrap_or_default();
            self.set_commit(&commit_hash, &msg);
        } else if let Some(branch) = self.repository.git_branch.clone() {
            if !branch.is_empty() {
                self.log(&format!("Checking out branch: {}", branch));
                git_repo.checkout(&branch).await?;
            }
        }

        self.log("Repository checkout completed");
        Ok(())
    }

    /// Получает полный путь к репозиторию
    pub fn get_repository_path(&self) -> std::path::PathBuf {
        self.work_dir.join("repository")
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ftype = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ftype.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
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
    fn test_update_repository() {
        // Просто проверяем, что метод вызывается без паники
        let mut job = create_test_job();
        let result = futures::executor::block_on(job.update_repository());
        assert!(result.is_ok()); // Пока всегда Ok

    }

    #[tokio::test]
    async fn test_checkout_repository() {
        let mut job = create_test_job();
        let result = job.checkout_repository().await;
        assert!(result.is_ok());
    }
}
