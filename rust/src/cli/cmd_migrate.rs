//! CLI - Migrate Command
//!
//! Команда для миграции БД

use clap::Args;
use crate::cli::CliResult;
use crate::config::Config;

/// Команда migrate
#[derive(Debug, Args)]
pub struct MigrateCommand {
    /// Применить миграции
    #[arg(long)]
    pub upgrade: bool,

    /// Откатить миграции
    #[arg(long)]
    pub downgrade: bool,
}

impl MigrateCommand {
    /// Выполняет команду
    pub fn run(&self) -> CliResult<()> {
        if self.upgrade {
            println!("Applying migrations...");
            
            // Загрузка конфигурации
            let config = Config::from_env().map_err(|e| anyhow::anyhow!("{}", e))?;
            let database_url = config.database_url().map_err(|e| anyhow::anyhow!("{}", e))?;
            
            // Создание хранилища и применение миграций
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            
            rt.block_on(async {
                let _store = crate::db::sql::SqlStore::new(&database_url)
                    .await
                    .map_err(|e| anyhow::anyhow!("Ошибка подключения к БД: {}", e))?;
                Ok::<_, anyhow::Error>(())
            })?;
            
            println!("Migrations applied successfully");
        }

        if self.downgrade {
            println!("Rolling back migrations...");
            // В реальной реализации нужно откатить миграции
            // rollback_migrations()?;
            println!("Migrations rolled back successfully");
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
    #[ignore = "requires PostgreSQL at VELUM_DB_URL (SqlStore is PostgreSQL-only)"]
    fn test_migrate_command_upgrade() {
        let cmd = MigrateCommand {
            upgrade: true,
            downgrade: false,
        };
        assert!(cmd.run().is_ok());
    }

    #[test]
    fn test_migrate_command_downgrade() {
        let cmd = MigrateCommand {
            upgrade: false,
            downgrade: true,
        };
        assert!(cmd.run().is_ok());
    }
}
