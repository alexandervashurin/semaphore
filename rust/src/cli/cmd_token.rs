//! CLI - Token Command
//!
//! Команда для управления API токенами

use clap::{Args, Subcommand};
use crate::cli::CliResult;

/// Команда token
#[derive(Debug, Args)]
pub struct TokenCommand {
    #[command(subcommand)]
    pub command: TokenCommands,
}

#[derive(Debug, Subcommand)]
pub enum TokenCommands {
    /// Создать токен
    Create(TokenCreateCommand),
    /// Список токенов
    List(TokenListCommand),
    /// Удалить токен
    Delete(TokenDeleteCommand),
}

impl TokenCommand {
    /// Выполняет команду
    pub fn run(&self) -> CliResult<()> {
        match &self.command {
            TokenCommands::Create(cmd) => cmd.run(),
            TokenCommands::List(cmd) => cmd.run(),
            TokenCommands::Delete(cmd) => cmd.run(),
        }
    }
}

/// Команда token create
#[derive(Debug, Args)]
pub struct TokenCreateCommand {
    /// ID пользователя
    #[arg(long)]
    pub user_id: i32,
}

impl TokenCreateCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Creating API token for user {}...", self.user_id);
        // В реальной реализации нужно создать токен
        Ok(())
    }
}

/// Команда token list
#[derive(Debug, Args)]
pub struct TokenListCommand {
    /// ID пользователя
    #[arg(long)]
    pub user_id: i32,
}

impl TokenListCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Listing API tokens for user {}...", self.user_id);
        // В реальной реализации нужно получить список токенов
        Ok(())
    }
}

/// Команда token delete
#[derive(Debug, Args)]
pub struct TokenDeleteCommand {
    /// ID токена
    #[arg(long)]
    pub token_id: String,
}

impl TokenDeleteCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Deleting API token {}...", self.token_id);
        // В реальной реализации нужно удалить токен
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_create_command() {
        let cmd = TokenCreateCommand { user_id: 1 };
        assert!(cmd.run().is_ok());
    }

    #[test]
    fn test_token_list_command() {
        let cmd = TokenListCommand { user_id: 1 };
        assert!(cmd.run().is_ok());
    }
}
