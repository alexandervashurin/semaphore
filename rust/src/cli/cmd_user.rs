//! CLI - User Commands
//!
//! Команды для управления пользователями

use clap::{Args, Subcommand};
use crate::cli::CliResult;
use crate::config::Config;
use crate::db::SqlStore;
use crate::db::store::UserManager;
use crate::models::User;
use chrono::Utc;
use std::sync::Arc;

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
    pub fn run(&self, config: Arc<Config>) -> CliResult<()> {
        match &self.command {
            UserCommands::Add(cmd) => cmd.run(config),
            UserCommands::List(cmd) => cmd.run(config),
            UserCommands::Delete(cmd) => cmd.run(config),
            UserCommands::Get(cmd) => cmd.run(config),
            UserCommands::Change(cmd) => cmd.run(config),
            UserCommands::Totp(cmd) => cmd.run(config),
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
    pub fn run(&self, config: Arc<Config>) -> CliResult<()> {
        println!("Adding user: {}", self.username);
        println!("  Name: {}", self.name);
        println!("  Email: {}", self.email);
        println!("  Admin: {}", self.admin);

        // Создаём runtime и выполняем async операцию
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        runtime.block_on(async {
            // Создаём хранилище
            let url = config.database_url()
                .map_err(|e| crate::error::Error::Other(e.to_string()))?;
            let store = SqlStore::new(&url).await?;

            // Создаём пользователя - create_user сам захэширует пароль
            let user = User {
                id: 0,
                created: Utc::now(),
                username: self.username.clone(),
                name: self.name.clone(),
                email: self.email.clone(),
                password: String::new(), // Будет заменён в create_user
                admin: self.admin,
                external: false,
                alert: false,
                pro: false,
                totp: None,
                email_otp: None,
            };

            store.create_user(user, &self.password).await?;

            println!("User {} successfully created", self.username);
            Ok::<(), crate::error::Error>(())
        })?;

        Ok(())
    }
}

/// Команда user list
#[derive(Debug, Args)]
pub struct UserListCommand {}

impl UserListCommand {
    pub fn run(&self, _config: Arc<Config>) -> CliResult<()> {
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
    pub fn run(&self, _config: Arc<Config>) -> CliResult<()> {
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
    pub fn run(&self, _config: Arc<Config>) -> CliResult<()> {
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
    pub fn run(&self, _config: Arc<Config>) -> CliResult<()> {
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
    pub fn run(&self, config: Arc<Config>) -> CliResult<()> {
        match &self.command {
            UserTotpCommands::Add(cmd) => cmd.run(config),
            UserTotpCommands::Delete(cmd) => cmd.run(config),
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
    pub fn run(&self, _config: Arc<Config>) -> CliResult<()> {
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
    pub fn run(&self, _config: Arc<Config>) -> CliResult<()> {
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
    use std::sync::Arc;

    #[test]
    fn test_user_list_command() {
        let cmd = UserListCommand {};
        assert!(cmd.run(Arc::new(Config::default())).is_ok());
    }
}
