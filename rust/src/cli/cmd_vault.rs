//! CLI - Vault Commands
//!
//! Команды для управления хранилищами секретов

use clap::{Args, Subcommand};
use crate::cli::CliResult;

/// Команда vault
#[derive(Debug, Args)]
pub struct VaultCommand {
    #[command(subcommand)]
    pub command: VaultCommands,
}

#[derive(Debug, Subcommand)]
pub enum VaultCommands {
    /// Переконфигурировать хранилище
    Rekey(VaultRekeyCommand),
}

impl VaultCommand {
    /// Выполняет команду
    pub fn run(&self) -> CliResult<()> {
        match &self.command {
            VaultCommands::Rekey(cmd) => cmd.run(),
        }
    }
}

/// Команда vault rekey
#[derive(Debug, Args)]
pub struct VaultRekeyCommand {
    /// Старый ключ
    #[arg(long)]
    pub old_key: Option<String>,

    /// Новый ключ
    #[arg(long)]
    pub new_key: Option<String>,
}

impl VaultRekeyCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Rekeying vault...");
        // В реальной реализации нужно переконфигурировать хранилище
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
    fn test_vault_rekey_command() {
        let cmd = VaultRekeyCommand {
            old_key: None,
            new_key: None,
        };
        assert!(cmd.run().is_ok());
    }
}
