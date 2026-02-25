//! CmdGitClient - Git клиент через командную строку
//!
//! Полная замена Go db_lib/CmdGitClient.go
//! Использует системную команду `git` для выполнения операций

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::env;
use tokio::process::Command as TokioCommand;

use crate::error::{Error, Result};
use crate::services::ssh_agent::AccessKeyInstallation;
use crate::services::task_logger::TaskLogger;
use super::access_key_installer::{AccessKeyInstallerTrait, DbAccessKeyRole};

// ============================================================================
// Типы данных
// ============================================================================

/// Тип директории репозитория
#[derive(Debug, Clone, Copy)]
pub enum GitRepositoryDirType {
    /// Временная директория
    Tmp,
    /// Полная директория
    Full,
}

/// Git репозиторий (аналог Go GitRepository)
#[derive(Debug, Clone)]
pub struct GitRepository {
    /// Имя временной директории
    pub tmp_dir_name: Option<String>,
    /// ID шаблона
    pub template_id: i32,
    /// Репозиторий
    pub repository: DbRepository,
    /// Project ID
    pub project_id: i32,
}

/// Repository данные (упрощённая модель)
#[derive(Debug, Clone)]
pub struct DbRepository {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub git_url: String,
    pub git_branch: String,
    pub ssh_key_id: Option<i32>,
    pub git_path: Option<String>,
}

impl GitRepository {
    /// Создаёт новый GitRepository
    pub fn new(
        repository: DbRepository,
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

    /// Получает URL Git
    pub fn get_git_url(&self, with_auth: bool) -> String {
        if with_auth {
            // TODO: Добавить аутентификацию к URL
            self.repository.git_url.clone()
        } else {
            self.repository.git_url.clone()
        }
    }
}

// ============================================================================
// GitClient trait
// ============================================================================

/// Git клиент trait (аналог Go GitClient)
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

// ============================================================================
// CmdGitClient implementation
// ============================================================================

/// Command-line Git клиент (аналог Go CmdGitClient)
pub struct CmdGitClient {
    key_installer: Box<dyn AccessKeyInstallerTrait>,
}

impl CmdGitClient {
    /// Создаёт новый CmdGitClient
    pub fn new(key_installer: Box<dyn AccessKeyInstallerTrait>) -> Self {
        Self { key_installer }
    }

    /// Создаёт команду git
    fn make_cmd(
        &self,
        r: &GitRepository,
        target_dir: GitRepositoryDirType,
        installation: &AccessKeyInstallation,
        args: &[&str],
    ) -> Command {
        let mut cmd = Command::new("git");

        // Устанавливаем переменные окружения
        let mut env_vars = self.get_environment_vars();
        let git_env = installation.get_git_env();
        env_vars.extend(git_env);

        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        // Устанавливаем рабочую директорию
        match target_dir {
            GitRepositoryDirType::Tmp => {
                let dir = self.get_project_tmp_dir(r.project_id);
                cmd.current_dir(&dir);

                // Создаём директорию если не существует
                if !dir.exists() {
                    std::fs::create_dir_all(&dir).ok();
                }
            }
            GitRepositoryDirType::Full => {
                cmd.current_dir(r.get_full_path());
            }
        }

        cmd.args(args);
        cmd
    }

    /// Создаёт асинхронную команду git
    fn make_async_cmd(
        &self,
        r: &GitRepository,
        target_dir: GitRepositoryDirType,
        installation: &AccessKeyInstallation,
        args: &[&str],
    ) -> TokioCommand {
        let mut cmd = TokioCommand::new("git");

        // Устанавливаем переменные окружения
        let mut env_vars = self.get_environment_vars();
        let git_env = installation.get_git_env();
        env_vars.extend(git_env);

        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        // Устанавливаем рабочую директорию
        match target_dir {
            GitRepositoryDirType::Tmp => {
                let dir = self.get_project_tmp_dir(r.project_id);
                cmd.current_dir(&dir);

                // Создаём директорию если не существует
                if !dir.exists() {
                    std::fs::create_dir_all(&dir).ok();
                }
            }
            GitRepositoryDirType::Full => {
                cmd.current_dir(r.get_full_path());
            }
        }

        cmd.args(args);
        cmd
    }

    /// Получает переменные окружения
    fn get_environment_vars(&self) -> Vec<(String, String)> {
        vec![
            ("GIT_TERMINAL_PROMPT".to_string(), "0".to_string()),
        ]
    }

    /// Получает путь к временной директории проекта
    fn get_project_tmp_dir(&self, project_id: i32) -> PathBuf {
        PathBuf::from(format!("/tmp/semaphore/project_{}", project_id))
    }

    /// Выполняет команду
    fn run(
        &self,
        r: &GitRepository,
        target_dir: GitRepositoryDirType,
        args: &[&str],
        logger: &dyn TaskLogger,
    ) -> Result<()> {
        // Устанавливаем SSH ключ
        let ssh_key = r.repository.ssh_key_id.ok_or_else(|| {
            Error::Validation("SSH key ID is required".to_string())
        })?;

        // TODO: Получить ключ из БД и установить
        // Пока используем заглушку
        let installation = AccessKeyInstallation::new();

        let mut cmd = self.make_cmd(r, target_dir, &installation, args);
        logger.log_cmd(&cmd);

        let output = cmd.output().map_err(|e| {
            Error::Other(format!("Git command failed: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Git failed: {}", stderr)));
        }

        Ok(())
    }

    /// Выполняет команду и возвращает вывод
    fn output(
        &self,
        r: &GitRepository,
        target_dir: GitRepositoryDirType,
        args: &[&str],
        logger: &dyn TaskLogger,
    ) -> Result<String> {
        // Устанавливаем SSH ключ
        let ssh_key = r.repository.ssh_key_id.ok_or_else(|| {
            Error::Validation("SSH key ID is required".to_string())
        })?;

        // TODO: Получить ключ из БД и установить
        let installation = AccessKeyInstallation::new();

        let mut cmd = self.make_cmd(r, target_dir, &installation, args);
        logger.log_cmd(&cmd);

        let output = cmd.output().map_err(|e| {
            Error::Other(format!("Git command failed: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Git failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    /// Асинхронно выполняет команду и возвращает вывод
    async fn async_output(
        &self,
        r: &GitRepository,
        target_dir: GitRepositoryDirType,
        args: &[&str],
    ) -> Result<String> {
        let installation = AccessKeyInstallation::new();

        let mut cmd = self.make_async_cmd(r, target_dir, &installation, args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Git command failed: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Git failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }
}

#[async_trait::async_trait]
impl GitClient for CmdGitClient {
    async fn clone(&self, repo: &GitRepository) -> Result<()> {
        // TODO: Интеграция с logger
        println!("Cloning repository {}", repo.repository.git_url);

        let dir_name = repo.tmp_dir_name.clone()
            .unwrap_or_else(|| repo.repository.get_dir_name(repo.template_id));

        // Временная заглушка - будет интегрирована с AccessKeyInstaller
        let installation = AccessKeyInstallation::new();

        let mut cmd = self.make_async_cmd(
            repo,
            GitRepositoryDirType::Tmp,
            &installation,
            &[
                "clone",
                "--recursive",
                "--branch",
                &repo.repository.git_branch,
                &repo.get_git_url(false),
                &dir_name,
            ],
        );

        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Git clone failed: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Git clone failed: {}", stderr)));
        }

        Ok(())
    }

    async fn pull(&self, repo: &GitRepository) -> Result<()> {
        let installation = AccessKeyInstallation::new();

        // Pull changes
        let mut cmd = self.make_async_cmd(
            repo,
            GitRepositoryDirType::Full,
            &installation,
            &["pull", "origin", &repo.repository.git_branch],
        );

        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Git pull failed: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Git pull failed: {}", stderr)));
        }

        // Update submodules
        let mut cmd = self.make_async_cmd(
            repo,
            GitRepositoryDirType::Full,
            &installation,
            &["submodule", "update", "--init", "--recursive"],
        );

        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Git submodule update failed: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Git submodule update failed: {}", stderr)));
        }

        Ok(())
    }

    async fn checkout(&self, repo: &GitRepository, target: &str) -> Result<()> {
        let installation = AccessKeyInstallation::new();

        let mut cmd = self.make_async_cmd(
            repo,
            GitRepositoryDirType::Full,
            &installation,
            &["checkout", target],
        );

        let output = cmd.output().await.map_err(|e| {
            Error::Other(format!("Git checkout failed: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Git checkout failed: {}", stderr)));
        }

        Ok(())
    }

    fn can_be_pulled(&self, repo: &GitRepository) -> bool {
        // Синхронная версия для can_be_pulled
        let installation = AccessKeyInstallation::new();

        // Fetch
        let mut cmd = self.make_cmd(
            repo,
            GitRepositoryDirType::Full,
            &installation,
            &["fetch"],
        );

        if let Ok(output) = cmd.output() {
            if !output.status.success() {
                return false;
            }
        } else {
            return false;
        }

        // Check if ancestor
        let mut cmd = self.make_cmd(
            repo,
            GitRepositoryDirType::Full,
            &installation,
            &[
                "merge-base",
                "--is-ancestor",
                "HEAD",
                &format!("origin/{}", repo.repository.git_branch),
            ],
        );

        if let Ok(output) = cmd.output() {
            output.status.success()
        } else {
            false
        }
    }

    async fn get_last_commit_message(&self, repo: &GitRepository) -> Result<String> {
        let msg = self.async_output(
            repo,
            GitRepositoryDirType::Full,
            &["show-branch", "--no-name", "HEAD"],
        ).await?;

        // Ограничиваем длину сообщения
        let msg = if msg.len() > 100 {
            msg[..100].to_string()
        } else {
            msg
        };

        Ok(msg)
    }

    async fn get_last_commit_hash(&self, repo: &GitRepository) -> Result<String> {
        self.async_output(
            repo,
            GitRepositoryDirType::Full,
            &["rev-parse", "HEAD"],
        ).await
    }

    async fn get_last_remote_commit_hash(&self, repo: &GitRepository) -> Result<String> {
        let out = self.async_output(
            repo,
            GitRepositoryDirType::Tmp,
            &[
                "ls-remote",
                &repo.get_git_url(false),
                &repo.repository.git_branch,
            ],
        ).await?;

        // Парсим вывод: "hash\trefs/heads/branch"
        if let Some(tab_pos) = out.find('\t') {
            Ok(out[..tab_pos].to_string())
        } else {
            Err(Error::Other("Can't retrieve remote commit hash".to_string()))
        }
    }

    async fn get_remote_branches(&self, repo: &GitRepository) -> Result<Vec<String>> {
        let out = self.async_output(
            repo,
            GitRepositoryDirType::Tmp,
            &["ls-remote", "--heads", &repo.get_git_url(false)],
        ).await?;

        if out.is_empty() {
            return Ok(vec![]);
        }

        let branches: Vec<String> = out
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() < 2 {
                    return None;
                }

                let ref_path = parts[1];
                if let Some(idx) = ref_path.rfind('/') {
                    Some(ref_path[idx + 1..].to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(branches)
    }
}

// ============================================================================
// Extension traits для DbRepository
// ============================================================================

impl DbRepository {
    /// Получает имя директории репозитория
    pub fn get_dir_name(&self, template_id: i32) -> String {
        format!("repo_{}_{}", self.id, template_id)
    }

    /// Получает полный путь к репозиторию
    pub fn get_full_path(&self, template_id: i32) -> PathBuf {
        PathBuf::from(format!("/tmp/semaphore/repo_{}_{}", self.id, template_id))
    }
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_lib::access_key_installer::AccessKeyInstallerImpl;

    #[test]
    fn test_git_repository_creation() {
        let repo = DbRepository {
            id: 1,
            project_id: 1,
            name: "Test Repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_branch: "main".to_string(),
            ssh_key_id: Some(1),
            git_path: None,
        };

        let git_repo = GitRepository::new(repo, 1, 1);

        assert_eq!(git_repo.project_id, 1);
        assert_eq!(git_repo.template_id, 1);
    }

    #[test]
    fn test_git_repository_with_tmp_dir() {
        let repo = DbRepository {
            id: 1,
            project_id: 1,
            name: "Test Repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_branch: "main".to_string(),
            ssh_key_id: Some(1),
            git_path: None,
        };

        let git_repo = GitRepository::new(repo, 1, 1)
            .with_tmp_dir("test_tmp".to_string());

        assert!(git_repo.tmp_dir_name.is_some());
        assert_eq!(git_repo.tmp_dir_name.unwrap(), "test_tmp");
    }

    #[test]
    fn test_git_repository_full_path() {
        let repo = DbRepository {
            id: 1,
            project_id: 1,
            name: "Test Repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_branch: "main".to_string(),
            ssh_key_id: Some(1),
            git_path: None,
        };

        let git_repo = GitRepository::new(repo, 1, 1);
        let path = git_repo.get_full_path();

        assert!(path.display().to_string().contains("repo_1_1"));
    }

    #[test]
    fn test_cmd_git_client_creation() {
        let installer = Box::new(AccessKeyInstallerImpl::new());
        let client = CmdGitClient::new(installer);

        // Проверяем, что клиент создан
        let _ = client;
    }

    #[test]
    fn test_get_environment_vars() {
        let installer = Box::new(AccessKeyInstallerImpl::new());
        let client = CmdGitClient::new(installer);

        let vars = client.get_environment_vars();

        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].0, "GIT_TERMINAL_PROMPT");
        assert_eq!(vars[0].1, "0");
    }

    #[test]
    fn test_db_repository_get_dir_name() {
        let repo = DbRepository {
            id: 5,
            project_id: 1,
            name: "Test".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_branch: "main".to_string(),
            ssh_key_id: None,
            git_path: None,
        };

        let dir_name = repo.get_dir_name(10);
        assert_eq!(dir_name, "repo_5_10");
    }
}
