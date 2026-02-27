//! CLI - Version Command
//!
//! Команда для вывода версии

use clap::Args;
use crate::cli::CliResult;

/// Команда version
#[derive(Debug, Args)]
pub struct VersionCommand {}

impl VersionCommand {
    /// Выполняет команду
    pub fn run(&self) -> CliResult<()> {
        println!("Semaphore UI {}", env!("CARGO_PKG_VERSION"));
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
    fn test_version_command() {
        let cmd = VersionCommand {};
        assert!(cmd.run().is_ok());
    }
}
