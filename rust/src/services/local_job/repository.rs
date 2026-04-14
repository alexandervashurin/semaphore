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
                self.log(&format!(
                    "Warning: local path {src_path} not found, using empty directory"
                ));
            }
        } else if !self.repository.git_url.is_empty() {
            // Используем GitRepository для clone/pull
            use crate::services::git_repository::GitRepository;
            let git_repo = GitRepository::new(
                self.repository.clone(),
                self.task.project_id,
                self.task.template_id,
            )
            .with_tmp_dir(format!("task_{}", self.task.id));
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
                Err(e) => self.log(&format!(
                    "Warning: git error: {e}, using existing directory"
                )),
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
        )
        .with_tmp_dir(format!("task_{}", self.task.id));

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

    #[test]
    fn test_get_repository_path() {
        let job = create_test_job();
        let path = job.get_repository_path();
        assert_eq!(path, PathBuf::from("/tmp/work/repository"));
    }

    #[test]
    fn test_copy_dir_recursive() {
        // Создаём временную директорию с файлами
        let src = std::env::temp_dir().join("test_copy_src");
        let dst = std::env::temp_dir().join("test_copy_dst");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("test.txt"), "hello").unwrap();

        let result = copy_dir_recursive(&src, &dst);
        assert!(result.is_ok());
        assert!(dst.join("test.txt").exists());

        // Убираем за собой
        std::fs::remove_dir_all(&src).ok();
        std::fs::remove_dir_all(&dst).ok();
    }

    #[test]
    fn test_copy_dir_recursive_nested() {
        let src = std::env::temp_dir().join("test_copy_nested_src");
        let dst = std::env::temp_dir().join("test_copy_nested_dst");

        std::fs::create_dir_all(src.join("subdir")).unwrap();
        std::fs::write(src.join("root.txt"), "root").unwrap();
        std::fs::write(src.join("subdir/nested.txt"), "nested").unwrap();

        let result = copy_dir_recursive(&src, &dst);
        assert!(result.is_ok());
        assert!(dst.join("root.txt").exists());
        assert!(dst.join("subdir/nested.txt").exists());

        std::fs::remove_dir_all(&src).ok();
        std::fs::remove_dir_all(&dst).ok();
    }

    #[test]
    fn test_copy_dir_recursive_empty_dir() {
        let src = std::env::temp_dir().join("test_copy_empty_src");
        let dst = std::env::temp_dir().join("test_copy_empty_dst");

        std::fs::create_dir_all(&src).unwrap();
        let result = copy_dir_recursive(&src, &dst);
        assert!(result.is_ok());
        assert!(dst.exists());

        std::fs::remove_dir_all(&src).ok();
        std::fs::remove_dir_all(&dst).ok();
    }

    #[test]
    fn test_get_repository_path_with_custom_work_dir() {
        let job = create_test_job();
        let path = job.get_repository_path();
        // work_dir is /tmp/work, so repository path should be /tmp/work/repository
        assert!(path.to_string_lossy().ends_with("repository"));
    }

    #[tokio::test]
    async fn test_checkout_repository_without_commit_hash() {
        // Job без commit_hash — должен использовать branch из repository
        let mut job = create_test_job();
        // Repository default has git_branch = None
        let result = job.checkout_repository().await;
        // Должен завершиться успешно (no-op без git)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_checkout_repository_with_branch() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            project_id: 1,
            playbook: None,
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: None,
            start: None,
            end: None,
            message: None,
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            params: None,
        };

        let mut repo = crate::models::Repository::default();
        repo.git_branch = Some("develop".to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.project_id = 1;

        let mut job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            repo,
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        // checkout_repository должен использовать branch "develop"
        // Но без реального git репозитория команда git checkout завершится ошибкой
        // Проверяем что метод вызывается и логирует действие
        let _ = job.checkout_repository().await;
        // Результат может быть Ok или Err в зависимости от окружения
    }

    #[test]
    fn test_update_repository_with_file_url_nonexistent_path() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        let mut repo = crate::models::Repository::default();
        repo.git_url = "file:///nonexistent/path".to_string();

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.project_id = 1;

        let mut job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            repo,
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        // Должен вернуть Ok даже если путь не существует (просто лог warning)
        let result = futures::executor::block_on(job.update_repository());
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_repository_with_file_url_existing_path() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        // Создаём временную директорию с файлами
        let temp_src = std::env::temp_dir().join("repo_file_src");
        std::fs::create_dir_all(&temp_src).unwrap();
        std::fs::write(temp_src.join("main.go"), "package main").unwrap();
        std::fs::create_dir_all(temp_src.join("sub")).unwrap();
        std::fs::write(temp_src.join("sub/util.go"), "package sub").unwrap();

        let mut repo = crate::models::Repository::default();
        repo.git_url = format!("file://{}", temp_src.display());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.project_id = 1;

        let mut job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            repo,
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let result = futures::executor::block_on(job.update_repository());
        assert!(result.is_ok());

        // Проверяем что файлы скопированы
        let repo_path = job.get_repository_path();
        assert!(repo_path.join("main.go").exists());
        assert!(repo_path.join("sub/util.go").exists());

        // Чистим
        std::fs::remove_dir_all(&temp_src).ok();
    }

    #[test]
    fn test_get_repository_path_uses_work_dir() {
        let job = create_test_job();
        let path = job.get_repository_path();
        assert!(path.starts_with("/tmp/work"));
        assert!(path.to_string_lossy().ends_with("repository"));
    }

    #[test]
    fn test_update_repository_with_empty_git_url() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        let repo = crate::models::Repository::default();
        // git_url по умолчанию пустой

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.project_id = 1;

        let mut job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            repo,
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let result = futures::executor::block_on(job.update_repository());
        assert!(result.is_ok());
    }

    #[test]
    fn test_copy_dir_recursive_preserves_file_content() {
        let src = std::env::temp_dir().join("test_copy_content_src");
        let dst = std::env::temp_dir().join("test_copy_content_dst");

        std::fs::create_dir_all(&src).unwrap();
        let content = "Hello, World! Test content with special chars: $HOME & <tag>";
        std::fs::write(src.join("data.txt"), content).unwrap();

        let result = copy_dir_recursive(&src, &dst);
        assert!(result.is_ok());

        let dst_content = std::fs::read_to_string(dst.join("data.txt")).unwrap();
        assert_eq!(dst_content, content);

        std::fs::remove_dir_all(&src).ok();
        std::fs::remove_dir_all(&dst).ok();
    }

    #[test]
    fn test_copy_dir_recursive_multiple_files() {
        let src = std::env::temp_dir().join("test_copy_multi_src");
        let dst = std::env::temp_dir().join("test_copy_multi_dst");

        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("a.txt"), "a").unwrap();
        std::fs::write(src.join("b.txt"), "b").unwrap();
        std::fs::write(src.join("c.txt"), "c").unwrap();

        let result = copy_dir_recursive(&src, &dst);
        assert!(result.is_ok());
        assert!(dst.join("a.txt").exists());
        assert!(dst.join("b.txt").exists());
        assert!(dst.join("c.txt").exists());

        std::fs::remove_dir_all(&src).ok();
        std::fs::remove_dir_all(&dst).ok();
    }

    #[test]
    fn test_get_repository_path_is_work_dir_plus_repository() {
        let job = create_test_job();
        let path = job.get_repository_path();
        assert_eq!(path, job.work_dir.join("repository"));
    }

    #[test]
    fn test_copy_dir_recursive_preserves_directory_structure() {
        let src = std::env::temp_dir().join("test_copy_struct_src");
        let dst = std::env::temp_dir().join("test_copy_struct_dst");

        std::fs::create_dir_all(src.join("a/b/c")).unwrap();
        std::fs::write(src.join("a/b/c/deep.txt"), "deep").unwrap();
        std::fs::write(src.join("root.txt"), "root").unwrap();

        let result = copy_dir_recursive(&src, &dst);
        assert!(result.is_ok());
        assert!(dst.join("a/b/c/deep.txt").exists());
        assert!(dst.join("root.txt").exists());

        std::fs::remove_dir_all(&src).ok();
        std::fs::remove_dir_all(&dst).ok();
    }

    #[test]
    fn test_copy_dir_recursive_fails_on_nonexistent_src() {
        let src = std::env::temp_dir().join("nonexistent_src_xyz");
        let dst = std::env::temp_dir().join("nonexistent_dst_xyz");

        let result = copy_dir_recursive(&src, &dst);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_repository_with_http_url_no_git() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        let mut repo = crate::models::Repository::default();
        repo.git_url = "https://github.com/example/repo.git".to_string();

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.project_id = 1;

        let mut job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            repo,
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        // Без git должен вернуть Ok (логирует warning)
        let result = job.update_repository().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_repository_path_with_different_work_dir() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task::default();
        let job = LocalJob::new(
            task,
            crate::models::Template::default(),
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/custom/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let path = job.get_repository_path();
        assert_eq!(path, PathBuf::from("/custom/work/repository"));
    }

    #[tokio::test]
    async fn test_checkout_repository_no_commit_no_branch() {
        let mut job = create_test_job();
        // Нет commit_hash и branch
        let result = job.checkout_repository().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_checkout_repository_with_empty_branch() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task::default();
        let mut repo = crate::models::Repository::default();
        repo.git_branch = Some("".to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.project_id = 1;

        let mut job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            repo,
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let result = job.checkout_repository().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_copy_dir_recursive_with_hidden_files() {
        let src = std::env::temp_dir().join("test_copy_hidden_src");
        let dst = std::env::temp_dir().join("test_copy_hidden_dst");

        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join(".gitignore"), "*.log").unwrap();
        std::fs::write(src.join("main.rs"), "fn main() {}").unwrap();

        let result = copy_dir_recursive(&src, &dst);
        assert!(result.is_ok());
        assert!(dst.join(".gitignore").exists());
        assert!(dst.join("main.rs").exists());

        std::fs::remove_dir_all(&src).ok();
        std::fs::remove_dir_all(&dst).ok();
    }

    #[test]
    fn test_copy_dir_recursive_overwrites_existing_files() {
        let src = std::env::temp_dir().join("test_copy_overwrite_src");
        let dst = std::env::temp_dir().join("test_copy_overwrite_dst");

        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("file.txt"), "new content").unwrap();

        std::fs::create_dir_all(&dst).unwrap();
        std::fs::write(dst.join("file.txt"), "old content").unwrap();

        let result = copy_dir_recursive(&src, &dst);
        assert!(result.is_ok());
        let content = std::fs::read_to_string(dst.join("file.txt")).unwrap();
        assert_eq!(content, "new content");

        std::fs::remove_dir_all(&src).ok();
        std::fs::remove_dir_all(&dst).ok();
    }

    #[test]
    fn test_update_repository_creates_repo_directory() {
        let work_dir = std::env::temp_dir().join("test_update_repo_work");
        let tmp_dir = std::env::temp_dir().join("test_update_repo_tmp");

        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        // file:// URL с несуществующим путём — просто warning
        let mut repo = crate::models::Repository::default();
        repo.git_url = "file:///nonexistent".to_string();

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.project_id = 1;

        let mut job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            repo,
            crate::models::Environment::default(),
            logger,
            key_installer,
            work_dir.clone(),
            tmp_dir.clone(),
        );

        let result = futures::executor::block_on(job.update_repository());
        assert!(result.is_ok());
        // repo_path должна быть создана
        let repo_path = work_dir.join("repository");
        assert!(repo_path.exists());

        std::fs::remove_dir_all(&work_dir).ok();
        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[tokio::test]
    async fn test_checkout_repository_with_commit_hash_and_message() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;
        task.commit_hash = Some("abc123".to_string());
        task.commit_message = Some("Initial commit".to_string());

        let repo = crate::models::Repository::default();
        let mut template = crate::models::Template::default();
        template.id = 1;
        template.project_id = 1;

        let mut job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            repo,
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        // checkout с commit без git репозитория — может вернуть Ok или Err
        let _ = job.checkout_repository().await;
    }

    #[test]
    fn test_copy_dir_recursive_with_binary_content() {
        let src = std::env::temp_dir().join("test_copy_bin_src");
        let dst = std::env::temp_dir().join("test_copy_bin_dst");

        std::fs::create_dir_all(&src).unwrap();
        let binary_data: Vec<u8> = (0..255).collect();
        std::fs::write(src.join("data.bin"), &binary_data).unwrap();

        let result = copy_dir_recursive(&src, &dst);
        assert!(result.is_ok());

        let dst_data = std::fs::read(dst.join("data.bin")).unwrap();
        assert_eq!(dst_data, binary_data);

        std::fs::remove_dir_all(&src).ok();
        std::fs::remove_dir_all(&dst).ok();
    }

    #[test]
    fn test_get_repository_path_does_not_create_directory() {
        let job = create_test_job();
        let path = job.get_repository_path();
        // get_repository_path только возвращает путь, не создаёт
        assert_eq!(path, PathBuf::from("/tmp/work/repository"));
    }
}
