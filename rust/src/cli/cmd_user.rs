//! CLI - User Commands
//!
//! Команды для управления пользователями

use clap::{Args, Subcommand};
use crate::cli::CliResult;

/// Команда user
#[derive(Debug, Args)]
pub struct UserCommand {
    #[command(subcommand)]
    pub command: UserCommands,
}

#[derive(Debug, Subcommand)]
pub enum UserCommands {
    /// Добавить пользователя
    Add(UserAddCommand),
    /// Список пользователей
    List(UserListCommand),
    /// Удалить пользователя
    Delete(UserDeleteCommand),
    /// Получить пользователя
    Get(UserGetCommand),
    /// Изменить пользователя
    Change(UserChangeCommand),
    /// Управление TOTP
    Totp(UserTotpCommand),
}

impl UserCommand {
    /// Выполняет команду
    pub fn run(&self) -> CliResult<()> {
        match &self.command {
            UserCommands::Add(cmd) => cmd.run(),
            UserCommands::List(cmd) => cmd.run(),
            UserCommands::Delete(cmd) => cmd.run(),
            UserCommands::Get(cmd) => cmd.run(),
            UserCommands::Change(cmd) => cmd.run(),
            UserCommands::Totp(cmd) => cmd.run(),
        }
    }
}

/// Команда user add
#[derive(Debug, Args)]
pub struct UserAddCommand {
    /// Имя пользователя
    #[arg(short, long)]
    pub username: String,

    /// Полное имя
    #[arg(short, long)]
    pub name: String,

    /// Email
    #[arg(short, long)]
    pub email: String,

    /// Пароль
    #[arg(short = 'P', long)]
    pub password: String,

    /// Сделать администратором
    #[arg(long)]
    pub admin: bool,
}

impl UserAddCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Adding user: {}", self.username);
        println!("  Name: {}", self.name);
        println!("  Email: {}", self.email);
        println!("  Admin: {}", self.admin);
        // В реальной реализации нужно добавить пользователя в БД
        Ok(())
    }
}

/// Команда user list
#[derive(Debug, Args)]
pub struct UserListCommand {}

impl UserListCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Listing users...");
        // В реальной реализации нужно получить список пользователей из БД
        Ok(())
    }
}

/// Команда user delete
#[derive(Debug, Args)]
pub struct UserDeleteCommand {
    /// ID пользователя
    #[arg(long)]
    pub id: Option<i32>,

    /// Имя пользователя
    #[arg(long)]
    pub username: Option<String>,
}

impl UserDeleteCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Deleting user...");
        // В реальной реализации нужно удалить пользователя из БД
        Ok(())
    }
}

/// Команда user get
#[derive(Debug, Args)]
pub struct UserGetCommand {
    /// ID пользователя
    #[arg(long)]
    pub id: Option<i32>,

    /// Имя пользователя
    #[arg(long)]
    pub username: Option<String>,
}

impl UserGetCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Getting user...");
        // В реальной реализации нужно получить пользователя из БД
        Ok(())
    }
}

/// Команда user change
#[derive(Debug, Args)]
pub struct UserChangeCommand {
    /// ID пользователя
    #[arg(long)]
    pub id: i32,

    /// Новое имя пользователя
    #[arg(long)]
    pub username: Option<String>,

    /// Новое полное имя
    #[arg(long)]
    pub name: Option<String>,

    /// Новый email
    #[arg(long)]
    pub email: Option<String>,

    /// Новый пароль
    #[arg(long)]
    pub password: Option<String>,
}

impl UserChangeCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Changing user {}...", self.id);
        // В реальной реализации нужно изменить пользователя в БД
        Ok(())
    }
}

/// Команда user totp
#[derive(Debug, Args)]
pub struct UserTotpCommand {
    #[command(subcommand)]
    pub command: UserTotpCommands,
}

#[derive(Debug, Subcommand)]
pub enum UserTotpCommands {
    /// Добавить TOTP
    Add(UserTotpAddCommand),
    /// Удалить TOTP
    Delete(UserTotpDeleteCommand),
}

impl UserTotpCommand {
    pub fn run(&self) -> CliResult<()> {
        match &self.command {
            UserTotpCommands::Add(cmd) => cmd.run(),
            UserTotpCommands::Delete(cmd) => cmd.run(),
        }
    }
}

/// Команда user totp add
#[derive(Debug, Args)]
pub struct UserTotpAddCommand {
    /// ID пользователя
    #[arg(long)]
    pub user_id: i32,
}

impl UserTotpAddCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Adding TOTP for user {}...", self.user_id);
        // В реальной реализации нужно добавить TOTP
        Ok(())
    }
}

/// Команда user totp delete
#[derive(Debug, Args)]
pub struct UserTotpDeleteCommand {
    /// ID пользователя
    #[arg(long)]
    pub user_id: i32,

    /// ID TOTP
    #[arg(long)]
    pub totp_id: i32,
}

impl UserTotpDeleteCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Deleting TOTP for user {}...", self.user_id);
        // В реальной реализации нужно удалить TOTP
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
    fn test_user_add_command() {
        let cmd = UserAddCommand {
            username: "test".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            admin: false,
        };
        assert!(cmd.run().is_ok());
    }

    #[test]
    fn test_user_list_command() {
        let cmd = UserListCommand {};
        assert!(cmd.run().is_ok());
    }
}
