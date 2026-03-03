//! CLI - Server Command
//!
//! Команда для запуска сервера

use clap::Args;
use std::sync::Arc;
use crate::cli::CliResult;
use crate::config::Config;
use crate::db::SqlStore;
use crate::api;

/// Команда server
#[derive(Debug, Args)]
pub struct ServerCommand {
    /// Хост для прослушивания
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// Порт HTTP
    #[arg(short = 'p', long, default_value = "3000")]
    pub port: u16,
}

impl ServerCommand {
    /// Выполняет команду
    pub fn run(&self, config: Arc<Config>) -> CliResult<()> {
        println!("Starting Semaphore UI server...");
        println!("Listening on {}:{}", self.host, self.port);

        // Создаём хранилище
        let store = Self::create_store(&config)?;

        // Создаём приложение
        let app = api::create_app(store);

        // Запускаем сервер
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        runtime.block_on(async {
            let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port))
                .await?;
            println!("Server started at http://{}:{}/", self.host, self.port);
            axum::serve(listener, app).await?;
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        })?;

        Ok(())
    }

    /// Создаёт хранилище
    fn create_store(config: &Config) -> CliResult<Box<dyn crate::db::Store + Send + Sync>> {
        match config.database.dialect.clone().unwrap_or(crate::config::DbDialect::SQLite) {
            crate::config::DbDialect::SQLite |
            crate::config::DbDialect::MySQL |
            crate::config::DbDialect::Postgres => {
                let url = config.database_url()?;
                let store = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?
                    .block_on(SqlStore::new(&url))?;
                Ok(Box::new(store))
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_command_creation() {
        let cmd = ServerCommand {
            host: "0.0.0.0".to_string(),
            port: 3000,
        };
        assert_eq!(cmd.host, "0.0.0.0");
        assert_eq!(cmd.port, 3000);
    }
}
