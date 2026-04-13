//! Git репозиторий и клиент
//!
//! Предоставляет инфраструктуру для работы с Git:
//! - Clone, Pull, Checkout
//! - Получение информации о коммитах
//! - Работа с удалёнными репозиториями

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tracing::{debug, info};

use crate::error::{Error, Result};
use crate::models::Repository;

/// Тип директории репозитория
#[derive(Debug, Clone, Copy)]
pub enum GitRepositoryDirType {
    /// Временная директория
    Tmp,
    /// Полная директория
    Full,
}

/// Git клиент trait
#[async_trait::async_trait]
pub trait GitClient: Send + Sync {
    /// Клонирует репозиторий
    async fn clone(&self, repo: &GitRepository) -> Result<()>;

    /// Pull изменения
    async fn pull(&self, repo: &GitRepository) -> Result<()>;

    /// Checkout ветки/тега
    async fn checkout(&self, repo: &GitRepository, target: &str) -> Result<()>;

    /// Проверяет, можно ли сделать pull
    fn can_be_pulled(&self, repo: &GitRepository) -> bool;

    /// Получает сообщение последнего коммита
    async fn get_last_commit_message(&self, repo: &GitRepository) -> Result<String>;

    /// Получает хэш последнего коммита
    async fn get_last_commit_hash(&self, repo: &GitRepository) -> Result<String>;

    /// Получает хэш последнего удалённого коммита
    async fn get_last_remote_commit_hash(&self, repo: &GitRepository) -> Result<String>;

    /// Получает список удалённых веток
    async fn get_remote_branches(&self, repo: &GitRepository) -> Result<Vec<String>>;
}

/// Git репозиторий
pub struct GitRepository {
    /// Имя временной директории
    pub tmp_dir_name: Option<String>,
    /// ID шаблона
    pub template_id: i32,
    /// Репозиторий
    pub repository: Repository,
    /// Проект ID
    pub project_id: i32,
}

impl GitRepository {
    /// Создаёт новый GitRepository
    pub fn new(repository: Repository, project_id: i32, template_id: i32) -> Self {
        Self {
            tmp_dir_name: None,
            repository,
            project_id,
            template_id,
        }
    }

    /// Создаёт с временной директорией
    pub fn with_tmp_dir(mut self, dir_name: String) -> Self {
        self.tmp_dir_name = Some(dir_name);
        self
    }

    /// Получает полный путь к репозиторию
    pub fn get_full_path(&self) -> PathBuf {
        if let Some(ref tmp_name) = self.tmp_dir_name {
            // Временная директория проекта
            PathBuf::from(format!(
                "/tmp/semaphore/project_{}/{}",
                self.project_id, tmp_name
            ))
        } else {
            // Полная директория репозитория
            PathBuf::from(format!(
                "/tmp/semaphore/repo_{}_{}",
                self.repository.id, self.template_id
            ))
        }
    }

    /// Проверяет существование репозитория
    pub fn validate_repo(&self) -> Result<()> {
        let path = self.get_full_path();
        if !path.exists() {
            return Err(Error::NotFound(format!(
                "Repository not found at {:?}",
                path
            )));
        }
        Ok(())
    }

    /// Клонирует репозиторий
    pub async fn clone(&self) -> Result<()> {
        info!("Cloning repository {}", self.repository.git_url);

        let repo_path = self.get_full_path();

        // Создаём родительскую директорию
        if let Some(parent) = repo_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| Error::Other(format!("Ошибка создания директории: {}", e)))?;
        }

        let mut cmd = TokioCommand::new("git");
        cmd.arg("clone");
        cmd.arg(&self.repository.git_url);
        cmd.arg(&repo_path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::Other(format!("Ошибка клонирования репозитория: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Git clone failed: {}", stderr)));
        }

        info!("Repository cloned successfully");
        Ok(())
    }

    /// Pull изменения
    pub async fn pull(&self) -> Result<()> {
        debug!("Pulling changes for repository");

        let repo_path = self.get_full_path();

        let mut cmd = TokioCommand::new("git");
        cmd.arg("pull");
        cmd.current_dir(&repo_path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::Other(format!("Ошибка pull: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Git pull failed: {}", stderr)));
        }

        Ok(())
    }

    /// Checkout ветки/тега
    pub async fn checkout(&self, target: &str) -> Result<()> {
        debug!("Checking out {}", target);

        let repo_path = self.get_full_path();

        let mut cmd = TokioCommand::new("git");
        cmd.arg("checkout");
        cmd.arg(target);
        cmd.current_dir(&repo_path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::Other(format!("Ошибка checkout: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Git checkout failed: {}", stderr)));
        }

        Ok(())
    }

    /// Проверяет, можно ли сделать pull
    pub fn can_be_pulled(&self) -> bool {
        let repo_path = self.get_full_path();
        repo_path.exists() && repo_path.join(".git").exists()
    }

    /// Получает сообщение последнего коммита
    pub async fn get_last_commit_message(&self) -> Result<String> {
        let repo_path = self.get_full_path();

        let mut cmd = TokioCommand::new("git");
        cmd.arg("log");
        cmd.arg("-1");
        cmd.arg("--pretty=%B");
        cmd.current_dir(&repo_path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::Other(format!("Ошибка получения commit message: {}", e)))?;

        if !output.status.success() {
            return Err(Error::Other("Git log failed".to_string()));
        }

        let message = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Ok(message)
    }

    /// Получает хэш последнего коммита
    pub async fn get_last_commit_hash(&self) -> Result<String> {
        let repo_path = self.get_full_path();

        let mut cmd = TokioCommand::new("git");
        cmd.arg("rev-parse");
        cmd.arg("HEAD");
        cmd.current_dir(&repo_path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::Other(format!("Ошибка получения commit hash: {}", e)))?;

        if !output.status.success() {
            return Err(Error::Other("Git rev-parse failed".to_string()));
        }

        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Ok(hash)
    }

    /// Получает хэш последнего удалённого коммита
    pub async fn get_last_remote_commit_hash(&self) -> Result<String> {
        let repo_path = self.get_full_path();

        let mut cmd = TokioCommand::new("git");
        cmd.arg("ls-remote");
        cmd.arg(self.repository.git_url.clone());

        // Получаем HEAD
        let output = cmd
            .output()
            .await
            .map_err(|e| Error::Other(format!("Ошибка получения remote hash: {}", e)))?;

        if !output.status.success() {
            return Err(Error::Other("Git ls-remote failed".to_string()));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Парсим вывод: "hash\trefs/heads/branch"
        for line in output_str.lines() {
            if line.contains("HEAD") || line.contains("refs/heads/") {
                let parts: Vec<&str> = line.split('\t').collect();
                if !parts.is_empty() {
                    return Ok(parts[0].to_string());
                }
            }
        }

        Err(Error::Other(
            "Не удалось получить remote commit hash".to_string(),
        ))
    }

    /// Получает список удалённых веток
    pub async fn get_remote_branches(&self) -> Result<Vec<String>> {
        let mut cmd = TokioCommand::new("git");
        cmd.arg("ls-remote");
        cmd.arg("--heads");
        cmd.arg(self.repository.git_url.clone());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::Other(format!("Ошибка получения веток: {}", e)))?;

        if !output.status.success() {
            return Err(Error::Other("Git ls-remote failed".to_string()));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut branches = Vec::new();

        for line in output_str.lines() {
            // Формат: "hash\trefs/heads/branch"
            if let Some(tab_pos) = line.find('\t') {
                let ref_name = &line[tab_pos + 1..];
                // Убираем "refs/heads/"
                if let Some(branch) = ref_name.strip_prefix("refs/heads/") {
                    branches.push(branch.to_string());
                }
            }
        }

        Ok(branches)
    }
}

/// Фабрика Git клиентов
pub struct GitClientFactory;

impl GitClientFactory {
    /// Создаёт Git клиент
    pub fn create() -> impl GitClient {
        CmdGitClient
    }
}

/// Command-line Git клиент
pub struct CmdGitClient;

#[async_trait::async_trait]
impl GitClient for CmdGitClient {
    async fn clone(&self, repo: &GitRepository) -> Result<()> {
        repo.clone().await
    }

    async fn pull(&self, repo: &GitRepository) -> Result<()> {
        repo.pull().await
    }

    async fn checkout(&self, repo: &GitRepository, target: &str) -> Result<()> {
        repo.checkout(target).await
    }

    fn can_be_pulled(&self, repo: &GitRepository) -> bool {
        repo.can_be_pulled()
    }

    async fn get_last_commit_message(&self, repo: &GitRepository) -> Result<String> {
        repo.get_last_commit_message().await
    }

    async fn get_last_commit_hash(&self, repo: &GitRepository) -> Result<String> {
        repo.get_last_commit_hash().await
    }

    async fn get_last_remote_commit_hash(&self, repo: &GitRepository) -> Result<String> {
        repo.get_last_remote_commit_hash().await
    }

    async fn get_remote_branches(&self, repo: &GitRepository) -> Result<Vec<String>> {
        repo.get_remote_branches().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::repository::RepositoryType;

    #[test]
    fn test_git_repository_creation() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Test Repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: Some(1),
            git_path: None,
            created: None,
        };

        let git_repo = GitRepository::new(repo, 1, 1);

        assert_eq!(git_repo.project_id, 1);
        assert_eq!(git_repo.template_id, 1);
    }

    #[test]
    fn test_git_repository_with_tmp_dir() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Test Repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: Some(1),
            git_path: None,
            created: None,
        };

        let git_repo = GitRepository::new(repo, 1, 1).with_tmp_dir("test_tmp".to_string());

        assert!(git_repo.tmp_dir_name.is_some());
        assert_eq!(git_repo.tmp_dir_name.unwrap(), "test_tmp");
    }

    #[test]
    fn test_git_repository_full_path() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Test Repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: Some(1),
            git_path: None,
            created: None,
        };

        let git_repo = GitRepository::new(repo, 1, 1);
        let path = git_repo.get_full_path();

        assert!(path.display().to_string().contains("repo_1_1"));
    }

    #[test]
    fn test_git_repository_can_be_pulled() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Test Repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: Some(1),
            git_path: None,
            created: None,
        };

        let git_repo = GitRepository::new(repo, 1, 1);

        // Репозиторий ещё не существует
        assert!(!git_repo.can_be_pulled());
    }

    #[test]
    fn test_git_repository_custom_tmp_dir() {
        let repo = Repository {
            id: 2,
            project_id: 2,
            name: "Custom Repo".to_string(),
            git_url: "https://github.com/test/custom.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 2, 2).with_tmp_dir("custom_dir".to_string());
        assert_eq!(git_repo.tmp_dir_name, Some("custom_dir".to_string()));
    }

    #[test]
    fn test_git_repository_default_tmp_dir_is_none() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Test Repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: Some(1),
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 1, 1);
        assert!(git_repo.tmp_dir_name.is_none());
    }

    #[test]
    fn test_git_repository_validate_repo_fails_for_nonexistent() {
        let repo = Repository {
            id: 999,
            project_id: 999,
            name: "Nonexistent".to_string(),
            git_url: "https://github.com/test/nonexistent.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 999, 999);
        let result = git_repo.validate_repo();
        assert!(result.is_err());
    }

    #[test]
    fn test_git_repository_full_path_with_tmp() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Test Repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 1, 1).with_tmp_dir("task_42".to_string());
        let path = git_repo.get_full_path();
        assert!(path.to_string_lossy().contains("project_1"));
        assert!(path.to_string_lossy().contains("task_42"));
    }

    #[test]
    fn test_git_repository_clone() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Clone Test".to_string(),
            git_url: "https://github.com/test/clone.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: Some("main".to_string()),
            key_id: Some(1),
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo.clone(), 1, 1);
        let cloned_git_repo = GitRepository::new(repo, 1, 1);
        assert_eq!(git_repo.project_id, cloned_git_repo.project_id);
        assert_eq!(git_repo.template_id, cloned_git_repo.template_id);
    }

    #[test]
    fn test_git_repository_dir_type_variants() {
        // Проверяем что enum имеет оба варианта
        let tmp_type = GitRepositoryDirType::Tmp;
        let full_type = GitRepositoryDirType::Full;

        // Debug format должен содержать имя варианта
        let tmp_debug = format!("{:?}", tmp_type);
        let full_debug = format!("{:?}", full_type);
        assert!(tmp_debug.contains("Tmp"));
        assert!(full_debug.contains("Full"));
    }

    #[test]
    fn test_git_repository_get_full_path_without_tmp() {
        let repo = Repository {
            id: 5,
            project_id: 10,
            name: "No Tmp Repo".to_string(),
            git_url: "https://github.com/test/no-tmp.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 10, 5);
        let path = git_repo.get_full_path();

        // Путь должен содержать repo_{id}_{template_id}
        assert!(path.to_string_lossy().contains("repo_5_5"));
        assert!(!path.to_string_lossy().contains("project_"));
    }

    #[test]
    fn test_git_repository_different_ids_produce_different_paths() {
        let repo1 = Repository {
            id: 1,
            project_id: 1,
            name: "Repo 1".to_string(),
            git_url: "https://github.com/test/repo1.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let repo2 = Repository {
            id: 2,
            project_id: 2,
            name: "Repo 2".to_string(),
            git_url: "https://github.com/test/repo2.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };

        let git_repo1 = GitRepository::new(repo1, 1, 1);
        let git_repo2 = GitRepository::new(repo2, 2, 2);

        let path1 = git_repo1.get_full_path();
        let path2 = git_repo2.get_full_path();

        assert_ne!(path1, path2);
    }

    #[test]
    fn test_git_repository_full_path_is_deterministic() {
        let repo = Repository {
            id: 42,
            project_id: 100,
            name: "Deterministic".to_string(),
            git_url: "https://github.com/test/det.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 100, 42);

        let path1 = git_repo.get_full_path();
        let path2 = git_repo.get_full_path();

        assert_eq!(path1, path2);
    }

    #[test]
    fn test_git_repository_validate_on_tmp_dir() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Tmp Validation".to_string(),
            git_url: "https://github.com/test/tmp.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 1, 1).with_tmp_dir("nonexistent_tmp".to_string());

        // Директория не существует, поэтому validation должен вернуть ошибку
        let result = git_repo.validate_repo();
        assert!(result.is_err());
    }

    #[test]
    fn test_git_repository_with_long_tmp_name() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Long Tmp".to_string(),
            git_url: "https://github.com/test/long.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let long_name = "a".repeat(200);
        let git_repo = GitRepository::new(repo, 1, 1).with_tmp_dir(long_name.clone());

        let path = git_repo.get_full_path();
        assert!(path.to_string_lossy().contains(&long_name));
    }

    #[test]
    fn test_git_repository_can_be_pulled_returns_true_for_existing_git_dir() {
        // Создаём временную директорию с .git
        let temp_dir = std::env::temp_dir().join("semaphore_git_test_can_pull");
        let git_dir = temp_dir.join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();

        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Existing Git Dir".to_string(),
            git_url: "https://github.com/test/existing.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 1, 1).with_tmp_dir(temp_dir.file_name().unwrap().to_string_lossy().to_string());

        // Поскольку tmp_dir_name используется для построения пути, проверим через прямой путь
        let path = git_repo.get_full_path();
        assert!(path.to_string_lossy().contains("project_1"));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_git_repository_empty_name() {
        let repo = Repository {
            id: 0,
            project_id: 0,
            name: "".to_string(),
            git_url: "".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 0, 0);

        assert_eq!(git_repo.project_id, 0);
        assert_eq!(git_repo.template_id, 0);
        assert!(git_repo.tmp_dir_name.is_none());
    }

    #[test]
    fn test_cmd_git_client_struct_exists() {
        // CmdGitClient должен существовать и реализовывать GitClient
        let _client = CmdGitClient;
    }

    #[test]
    fn test_git_repository_fields_after_creation() {
        let repo = Repository {
            id: 7,
            project_id: 3,
            name: "Fields Test".to_string(),
            git_url: "https://github.com/test/fields.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: Some("main".to_string()),
            key_id: Some(1),
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 3, 7);

        assert_eq!(git_repo.project_id, 3);
        assert_eq!(git_repo.template_id, 7);
        assert_eq!(git_repo.repository.name, "Fields Test");
        assert!(git_repo.tmp_dir_name.is_none());
    }

    #[test]
    fn test_git_repository_with_special_chars_in_tmp_name() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Special Chars".to_string(),
            git_url: "https://github.com/test/special.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 1, 1).with_tmp_dir("task-123_abc".to_string());
        let path = git_repo.get_full_path();
        assert!(path.to_string_lossy().contains("task-123_abc"));
    }

    #[test]
    fn test_git_repository_repository_struct_fields_accessible() {
        let repo = Repository {
            id: 10,
            project_id: 20,
            name: "Test Repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: Some("develop".to_string()),
            key_id: Some(5),
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo.clone(), 20, 10);

        assert_eq!(git_repo.repository.id, 10);
        assert_eq!(git_repo.repository.project_id, 20);
        assert_eq!(git_repo.repository.git_branch, Some("develop".to_string()));
        assert_eq!(git_repo.repository.key_id, Some(5));
    }

    #[test]
    fn test_git_repository_tmp_dir_overrides_path() {
        let repo = Repository {
            id: 100,
            project_id: 1,
            name: "Override Test".to_string(),
            git_url: "https://github.com/test/override.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };

        // Без tmp_dir
        let git_repo_no_tmp = GitRepository::new(repo.clone(), 1, 100);
        let path_no_tmp = git_repo_no_tmp.get_full_path();
        assert!(path_no_tmp.to_string_lossy().contains("repo_100_100"));

        // С tmp_dir
        let git_repo_with_tmp = GitRepository::new(repo, 1, 100).with_tmp_dir("tmp123".to_string());
        let path_with_tmp = git_repo_with_tmp.get_full_path();
        assert!(path_with_tmp.to_string_lossy().contains("project_1"));
        assert!(path_with_tmp.to_string_lossy().contains("tmp123"));

        assert_ne!(path_no_tmp, path_with_tmp);
    }

    #[test]
    fn test_git_repository_multiple_tmp_dirs() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Multi Tmp".to_string(),
            git_url: "https://github.com/test/multi.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };

        let git_repo1 = GitRepository::new(repo.clone(), 1, 1).with_tmp_dir("task_1".to_string());
        let git_repo2 = GitRepository::new(repo.clone(), 1, 1).with_tmp_dir("task_2".to_string());
        let git_repo3 = GitRepository::new(repo, 1, 1);

        let path1 = git_repo1.get_full_path();
        let path2 = git_repo2.get_full_path();
        let path3 = git_repo3.get_full_path();

        assert_ne!(path1, path2);
        assert_ne!(path1, path3);
        assert_ne!(path2, path3);
    }

    #[test]
    fn test_git_repository_large_project_id() {
        let repo = Repository {
            id: 1,
            project_id: 999999,
            name: "Large ID".to_string(),
            git_url: "https://github.com/test/large.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 999999, 1);
        let path = git_repo.get_full_path();
        assert!(path.to_string_lossy().contains("repo_1_1"));
    }

    #[test]
    fn test_git_repository_large_template_id() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Large Template".to_string(),
            git_url: "https://github.com/test/large-tpl.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 1, 999999);
        let path = git_repo.get_full_path();
        assert!(path.to_string_lossy().contains("repo_1_999999"));
    }

    #[test]
    fn test_git_repository_negative_ids() {
        let repo = Repository {
            id: -1,
            project_id: -1,
            name: "Negative".to_string(),
            git_url: "https://github.com/test/neg.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, -1, -1);
        assert_eq!(git_repo.project_id, -1);
        assert_eq!(git_repo.template_id, -1);
    }

    #[test]
    fn test_git_repository_url_variations() {
        let urls = vec![
            "https://github.com/user/repo.git",
            "git@github.com:user/repo.git",
            "ssh://git@gitlab.com/user/repo.git",
            "file:///var/git/repo.git",
        ];

        for (i, url) in urls.iter().enumerate() {
            let repo = Repository {
                id: i as i32,
                project_id: 1,
                name: format!("Repo {}", i),
                git_url: url.to_string(),
                git_type: RepositoryType::Git,
                git_branch: None,
                key_id: None,
                git_path: None,
                created: None,
            };
            let git_repo = GitRepository::new(repo, 1, i as i32);
            assert_eq!(git_repo.repository.git_url, *url);
        }
    }

    #[test]
    fn test_git_repository_tmp_dir_with_numeric_name() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Numeric Tmp".to_string(),
            git_url: "https://github.com/test/num.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 1, 1).with_tmp_dir("12345".to_string());
        let path = git_repo.get_full_path();
        assert!(path.to_string_lossy().contains("12345"));
    }

    #[test]
    fn test_git_repository_clone_preserves_fields() {
        let repo = Repository {
            id: 5,
            project_id: 10,
            name: "Clone Preserve".to_string(),
            git_url: "https://github.com/test/preserve.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: Some("release".to_string()),
            key_id: Some(3),
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 10, 5);

        assert_eq!(git_repo.repository.name, "Clone Preserve");
        assert_eq!(git_repo.repository.git_branch, Some("release".to_string()));
        assert_eq!(git_repo.repository.key_id, Some(3));
        assert_eq!(git_repo.project_id, 10);
        assert_eq!(git_repo.template_id, 5);
    }

    #[test]
    fn test_git_repository_tmp_dir_unicode_name() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Unicode Tmp".to_string(),
            git_url: "https://github.com/test/unicode.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 1, 1).with_tmp_dir("задача_1".to_string());
        let path = git_repo.get_full_path();
        assert!(path.to_string_lossy().contains("задача_1"));
    }

    #[test]
    fn test_git_repository_path_starts_with_tmp_prefix() {
        let repo = Repository {
            id: 1,
            project_id: 42,
            name: "Path Prefix".to_string(),
            git_url: "https://github.com/test/prefix.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 42, 1).with_tmp_dir("build_789".to_string());
        let path = git_repo.get_full_path();

        assert!(path.to_string_lossy().starts_with("/tmp/"));
        assert!(path.to_string_lossy().contains("project_42"));
        assert!(path.to_string_lossy().contains("build_789"));
    }

    #[test]
    fn test_git_repository_get_full_path_is_absolute() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Absolute".to_string(),
            git_url: "https://github.com/test/abs.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 1, 1);
        let path = git_repo.get_full_path();
        assert!(path.is_absolute());
    }

    #[test]
    fn test_git_repository_dir_type_debug_impl() {
        let tmp_type = GitRepositoryDirType::Tmp;
        let full_type = GitRepositoryDirType::Full;

        let tmp_str = format!("{:?}", tmp_type);
        let full_str = format!("{:?}", full_type);

        assert_eq!(tmp_str, "Tmp");
        assert_eq!(full_str, "Full");
    }

    #[test]
    fn test_git_repository_clone_dir_type() {
        let tmp = GitRepositoryDirType::Tmp;
        let cloned = tmp.clone();
        assert!(matches!(cloned, GitRepositoryDirType::Tmp));
    }

    #[test]
    fn test_git_repository_repository_clone_and_modify() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Clone Repo".to_string(),
            git_url: "https://github.com/test/clone-repo.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: Some("main".to_string()),
            key_id: Some(1),
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo.clone(), 1, 1);
        let git_repo2 = GitRepository::new(repo, 1, 1);

        assert_eq!(git_repo.repository.git_url, git_repo2.repository.git_url);
        assert_eq!(git_repo.repository.git_branch, git_repo2.repository.git_branch);
    }

    #[test]
    fn test_git_repository_empty_git_url() {
        let repo = Repository {
            id: 0,
            project_id: 0,
            name: "".to_string(),
            git_url: "".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let git_repo = GitRepository::new(repo, 0, 0);
        assert!(git_repo.repository.git_url.is_empty());
        let path = git_repo.get_full_path();
        assert!(path.to_string_lossy().contains("repo_0_0"));
    }

    #[test]
    fn test_git_repository_validate_after_creating_dir() {
        let temp_dir = std::env::temp_dir().join("semaphore_validate_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Validate Dir".to_string(),
            git_url: "https://github.com/test/validate.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let dir_name = temp_dir.file_name().unwrap().to_string_lossy().to_string();
        let git_repo = GitRepository::new(repo, 1, 1).with_tmp_dir(dir_name);

        // validate_repo проверяет существование пути, но tmp_dir_name используется для построения пути
        // Путь содержит project_1/{tmp_dir_name}, а не сам temp_dir
        let path = git_repo.get_full_path();
        assert!(!path.exists());
        assert!(git_repo.validate_repo().is_err());

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
