//! CLI - Runner Command
//!
//! Команда для запуска раннера

use clap::Args;
use crate::cli::CliResult;

/// Команда runner
#[derive(Debug, Args)]
pub struct RunnerCommand {
    /// Токен раннера
    #[arg(long)]
    pub token: Option<String>,

    /// URL сервера
    #[arg(long)]
    pub server_url: Option<String>,
}

impl RunnerCommand {
    /// Выполняет команду
    pub fn run(&self) -> CliResult<()> {
        println!("Semaphore UI Runner");
        println!("Use 'help' for more information");

        // В реальной реализации нужно запустить runner
        // run_runner(&self.token, &self.server_url)?;

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
    fn test_runner_command_creation() {
        let cmd = RunnerCommand {
            token: None,
            server_url: None,
        };
        assert!(cmd.run().is_ok());
    }
}
