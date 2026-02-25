//! Git репозиторий и клиент
//!
//! Предоставляет инфраструктуру для работы с Git:
//! - Clone, Pull, Checkout
//! - Получение информации о коммитах
//! - Работа с удалёнными репозиториями

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tracing::{info, debug};

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
    pub fn new(
        repository: Repository,
        project_id: i32,
        template_id: i32,
    ) -> Self {
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
            PathBuf::from(format!("/tmp/semaphore/project_{}/{}", self.project_id, tmp_name))
        } else {
            // Полная директория репозитория
            PathBuf::from(format!(
                "/tmp/semaphore/repo_{}_{}",
                self.repository.id,
                self.template_id
            ))
        }
    }

    /// Проверяет существование репозитория
    pub fn validate_repo(&self) -> Result<()> {
        let path = self.get_full_path();
        if !path.exists() {
            return Err(Error::NotFound(format!("Repository not found at {:?}", path)));
        }
        Ok(())
    }

    /// Клонирует репозиторий
    pub async fn clone(&self) -> Result<()> {
        info!("Cloning repository {}", self.repository.git_url);
        
        let repo_path = self.get_full_path();
        
        // Создаём родительскую директорию
        if let Some(parent) = repo_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                Error::Other(format!("Ошибка создания директории: {}", e))
            })?;
        }

        let mut cmd = TokioCommand::new("git");
        cmd.arg("clone");
        cmd.arg(&self.repository.git_url);
        cmd.arg(&repo_path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Ошибка клонирования репозитория: {}", e))
        })?;

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

        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Ошибка pull: {}", e))
        })?;

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

        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Ошибка checkout: {}", e))
        })?;

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

        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Ошибка получения commit message: {}", e))
        })?;

        if !output.status.success() {
            return Err(Error::Other("Git log failed".to_string()));
        }

        let message = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

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

        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Ошибка получения commit hash: {}", e))
        })?;

        if !output.status.success() {
            return Err(Error::Other("Git rev-parse failed".to_string()));
        }

        let hash = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        Ok(hash)
    }

    /// Получает хэш последнего удалённого коммита
    pub async fn get_last_remote_commit_hash(&self) -> Result<String> {
        let repo_path = self.get_full_path();
        
        let mut cmd = TokioCommand::new("git");
        cmd.arg("ls-remote");
        cmd.arg(self.repository.git_url.clone());
        
        // Получаем HEAD
        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Ошибка получения remote hash: {}", e))
        })?;

        if !output.status.success() {
            return Err(Error::Other("Git ls-remote failed".to_string()));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Парсим вывод: "hash\trefs/heads/branch"
        for line in output_str.lines() {
            if line.contains("HEAD") || line.contains("refs/heads/") {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 1 {
                    return Ok(parts[0].to_string());
                }
            }
        }

        Err(Error::Other("Не удалось получить remote commit hash".to_string()))
    }

    /// Получает список удалённых веток
    pub async fn get_remote_branches(&self) -> Result<Vec<String>> {
        let mut cmd = TokioCommand::new("git");
        cmd.arg("ls-remote");
        cmd.arg("--heads");
        cmd.arg(self.repository.git_url.clone());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Ошибка получения веток: {}", e))
        })?;

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
            key_id: 1,
            git_path: None,
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
            key_id: 1,
            git_path: None,
        };

        let git_repo = GitRepository::new(repo, 1, 1)
            .with_tmp_dir("test_tmp".to_string());

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
            key_id: 1,
            git_path: None,
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
            key_id: 1,
            git_path: None,
        };

        let git_repo = GitRepository::new(repo, 1, 1);

        // Репозиторий ещё не существует
        assert!(!git_repo.can_be_pulled());
    }
}
