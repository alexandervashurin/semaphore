//! CLI - Project Commands
//!
//! Команды для управления проектами

use clap::{Args, Subcommand};
use crate::cli::CliResult;

/// Команда project
#[derive(Debug, Args)]
pub struct ProjectCommand {
    #[command(subcommand)]
    pub command: ProjectCommands,
}

#[derive(Debug, Subcommand)]
pub enum ProjectCommands {
    /// Экспорт проекта
    Export(ProjectExportCommand),
    /// Импорт проекта
    Import(ProjectImportCommand),
}

impl ProjectCommand {
    /// Выполняет команду
    pub fn run(&self) -> CliResult<()> {
        match &self.command {
            ProjectCommands::Export(cmd) => cmd.run(),
            ProjectCommands::Import(cmd) => cmd.run(),
        }
    }
}

/// Команда project export
#[derive(Debug, Args)]
pub struct ProjectExportCommand {
    /// ID проекта
    #[arg(long)]
    pub id: i32,

    /// Путь к файлу экспорта
    #[arg(short, long)]
    pub file: String,
}

impl ProjectExportCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Exporting project {} to {}...", self.id, self.file);
        // В реальной реализации нужно экспортировать проект
        Ok(())
    }
}

/// Команда project import
#[derive(Debug, Args)]
pub struct ProjectImportCommand {
    /// Путь к файлу импорта
    #[arg(short, long)]
    pub file: String,
}

impl ProjectImportCommand {
    pub fn run(&self) -> CliResult<()> {
        println!("Importing project from {}...", self.file);
        // В реальной реализации нужно импортировать проект
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
    fn test_project_export_command() {
        let cmd = ProjectExportCommand {
            id: 1,
            file: "backup.json".to_string(),
        };
        assert!(cmd.run().is_ok());
    }

    #[test]
    fn test_project_import_command() {
        let cmd = ProjectImportCommand {
            file: "backup.json".to_string(),
        };
        assert!(cmd.run().is_ok());
    }
}
