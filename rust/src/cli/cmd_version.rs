//! CLI - Version Command
//!
//! Команда для вывода версии

use crate::cli::CliResult;
use clap::Args;

/// Команда version
#[derive(Debug, Args)]
pub struct VersionCommand {}

impl VersionCommand {
    /// Выполняет команду
    pub fn run(&self) -> CliResult<()> {
        println!("Velum UI {}", env!("CARGO_PKG_VERSION"));
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
