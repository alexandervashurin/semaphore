//! CLI - Setup Command
//!
//! Команда для настройки Velum

use crate::cli::CliResult;
use clap::Args;

/// Команда setup
#[derive(Debug, Args)]
pub struct SetupCommand {
    /// Пропустить интерактивный режим
    #[arg(long)]
    pub non_interactive: bool,
}

impl SetupCommand {
    /// Выполняет команду
    pub fn run(&self) -> CliResult<()> {
        println!("Velum UI Setup Wizard");
        println!("========================");

        if self.non_interactive {
            println!("Running in non-interactive mode...");
            // В реальной реализации нужно настроить без интерактивного режима
        } else {
            println!("Running in interactive mode...");
            // В реальной реализации нужно запустить интерактивный мастер
        }

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
    fn test_setup_command() {
        let cmd = SetupCommand {
            non_interactive: true,
        };
        assert!(cmd.run().is_ok());
    }
}
