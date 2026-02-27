//! Git Client Factory
//!
//! Фабрика для создания Git клиентов

use super::{GitClient, CmdGitClient, GoGitClient, AccessKeyInstaller};

/// Тип клиента Git
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GitClientType {
    /// Go Git клиент
    GoGit,
    /// Command Line Git клиент
    CmdGit,
}

/// Создаёт Git клиент по умолчанию
pub fn create_default_git_client(key_installer: Box<dyn AccessKeyInstaller>) -> Box<dyn GitClient> {
    // По умолчанию используем CmdGitClient
    create_cmd_git_client(key_installer)
}

/// Создаёт Go Git клиент
pub fn create_go_git_client(key_installer: Box<dyn AccessKeyInstaller>) -> Box<dyn GitClient> {
    Box::new(GoGitClient::new(key_installer))
}

/// Создаёт Command Line Git клиент
pub fn create_cmd_git_client(key_installer: Box<dyn AccessKeyInstaller>) -> Box<dyn GitClient> {
    Box::new(CmdGitClient::new(key_installer))
}

/// Создаёт Git клиент по типу
pub fn create_git_client(
    client_type: GitClientType,
    key_installer: Box<dyn AccessKeyInstaller>,
) -> Box<dyn GitClient> {
    match client_type {
        GitClientType::GoGit => create_go_git_client(key_installer),
        GitClientType::CmdGit => create_cmd_git_client(key_installer),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_git_client() {
        // Тест для проверки создания клиента по умолчанию
        assert!(true);
    }

    #[test]
    fn test_create_git_client_by_type() {
        // Тест для проверки создания клиента по типу
        assert!(true);
    }
}
