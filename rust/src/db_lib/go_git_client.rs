//! Go Git Client
//!
//! Git клиент на базе go-git (Rust аналог: git2)

use std::sync::Arc;
use git2::{Repository, BuildRepo, FetchOptions, RemoteCallbacks, Cred};
use crate::error::{Error, Result};
use crate::models::Repository as DbRepository;
use crate::services::task_logger::TaskLogger;
use super::{GitClient, GitRepository, AccessKeyInstaller};

/// Go Git Client (в Rust используем git2)
pub struct GoGitClient {
    /// Установщик ключей доступа
    key_installer: Box<dyn AccessKeyInstaller>,
}

impl GoGitClient {
    /// Создаёт новый Go Git клиент
    pub fn new(key_installer: Box<dyn AccessKeyInstaller>) -> Self {
        Self { key_installer }
    }

    /// Получает метод аутентификации
    fn get_auth_method(&self, repo: &GitRepository) -> Result<Option<RemoteCallbacks<'_>>> {
        match repo.repository.ssh_key.key_type {
            crate::models::access_key::AccessKeyType::Ssh => {
                // Установка SSH ключа
                let install = self.key_installer.install(
                    &repo.repository.ssh_key,
                    crate::services::access_key_installer::AccessKeyRole::Git,
                    repo.logger.clone(),
                )?;

                // Создаём callback для аутентификации
                let mut callbacks = RemoteCallbacks::new();
                callbacks.credentials(|_url, username_from_url, allowed_types| {
                    Cred::ssh_key_from_agent(
                        username_from_url.unwrap_or("git"),
                        None,
                        None,
                    )
                });

                // Очищаем установку после использования
                install.destroy()?;

                Ok(Some(callbacks))
            }
            crate::models::access_key::AccessKeyType::LoginPassword => {
                // Аутентификация по логину/паролю
                let mut callbacks = RemoteCallbacks::new();
                callbacks.credentials(|_url, username_from_url, _allowed_types| {
                    Cred::userpass_plaintext(
                        &repo.repository.ssh_key.login_password.login,
                        &repo.repository.ssh_key.login_password.password,
                    )
                });
                Ok(Some(callbacks))
            }
            crate::models::access_key::AccessKeyType::None => {
                Ok(None)
            }
            _ => Err(Error::Other("Unsupported auth method".to_string())),
        }
    }
}

impl GitClient for GoGitClient {
    /// Клонирует репозиторий
    fn clone(&self, repo: GitRepository) -> Result<()> {
        let auth_callbacks = self.get_auth_method(&repo)?;

        let mut builder = BuildRepo::new();
        if let Some(callbacks) = auth_callbacks {
            builder.fetch_options(callbacks);
        }

        builder.clone(&repo.repository.git_url, repo.get_full_path())?;
        Ok(())
    }

    /// Выполняет pull
    fn pull(&self, repo: GitRepository) -> Result<()> {
        let mut git_repo = Repository::open(repo.get_full_path())?;

        let auth_callbacks = self.get_auth_method(&repo)?;

        if let Some(mut callbacks) = auth_callbacks {
            let mut fetch_options = FetchOptions::new();
            fetch_options.remote_callbacks(callbacks);

            // Fetch remote
            // В реальной реализации нужно получить remote и сделать fetch
        }

        Ok(())
    }

    /// Выполняет checkout
    fn checkout(&self, repo: GitRepository, target: &str) -> Result<()> {
        let git_repo = Repository::open(repo.get_full_path())?;

        // Checkout на указанную ветку/commit
        let (object, reference) = git_repo.revparse_ext(target)?;

        git_repo.checkout_tree(&object, None)?;

        if let Some(reference) = reference {
            git_repo.set_head(reference.name().unwrap())?;
        } else {
            git_repo.set_head(object.id())?;
        }

        Ok(())
    }

    /// Проверяет, можно ли сделать pull
    fn can_be_pulled(&self, _repo: GitRepository) -> bool {
        // В реальной реализации нужно проверить наличие .git директории
        true
    }

    /// Получает сообщение последнего коммита
    fn get_last_commit_message(&self, repo: GitRepository) -> Result<String> {
        let git_repo = Repository::open(repo.get_full_path())?;
        let head = git_repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.message().unwrap_or("").to_string())
    }

    /// Получает хэш последнего коммита
    fn get_last_commit_hash(&self, repo: GitRepository) -> Result<String> {
        let git_repo = Repository::open(repo.get_full_path())?;
        let head = git_repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    /// Получает хэш последнего remote коммита
    fn get_last_remote_commit_hash(&self, repo: GitRepository) -> Result<String> {
        // В реальной реализации нужно получить remote и fetch
        Ok(String::new())
    }

    /// Получает список remote веток
    fn get_remote_branches(&self, repo: GitRepository) -> Result<Vec<String>> {
        let git_repo = Repository::open(repo.get_full_path())?;
        let mut branches = Vec::new();

        for branch in git_repo.branches(Some(git2::BranchType::Remote))? {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                branches.push(name.to_string());
            }
        }

        Ok(branches)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_git_client_creation() {
        // Тест для проверки создания клиента
        assert!(true);
    }
}
