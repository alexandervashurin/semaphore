//! CLI - Server Command
//!
//! Команда для запуска сервера

use clap::Parser;
use std::sync::Arc;
use crate::cli::CliResult;
use crate::config::Config;
use crate::db::SqlStore;
use crate::api;

/// Команда server
#[derive(Debug, Parser)]
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
        println!("Starting Velum UI server...");
        println!("Listening on {}:{}", self.host, self.port);

        // Создаём хранилище и запускаем сервер в одном runtime
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        runtime.block_on(async {
            // Создаём хранилище
            let store: Arc<dyn crate::db::Store + Send + Sync> =
                Arc::from(Self::create_store_async(&config).await?);

            // Сид admin-пользователя при первом запуске
            Self::seed_admin_if_empty(store.as_ref()).await;

            // Запускаем планировщик задач
            let scheduler = crate::services::scheduler::SchedulePool::new(store.clone());
            if let Err(e) = scheduler.start().await {
                eprintln!("Warning: scheduler failed to start: {e}");
            } else {
                println!("Task scheduler started");
            }

            // Запускаем сервис автобэкапа (если включён через env)
            let backup_enabled = std::env::var("SEMAPHORE_AUTO_BACKUP_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);
            if backup_enabled {
                let backup_config = crate::services::auto_backup::AutoBackupConfig {
                    enabled: true,
                    interval_hours: std::env::var("SEMAPHORE_AUTO_BACKUP_INTERVAL_HOURS")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(24),
                    backup_path: std::env::var("SEMAPHORE_AUTO_BACKUP_PATH")
                        .unwrap_or_else(|_| "./backups".to_string()),
                    max_backups: std::env::var("SEMAPHORE_AUTO_BACKUP_MAX")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(7),
                    compress: true,
                };
                let backup_svc = crate::services::auto_backup::AutoBackupService::new(
                    backup_config,
                    store.clone(),
                );
                backup_svc.start().await;
                println!("Auto backup service started");
            }

            // Создаём приложение
            let app = api::create_app(store.clone());

            // Запускаем сервер с graceful shutdown
            let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port))
                .await
                .map_err(|e| crate::error::Error::Other(e.to_string()))?;
            println!("Server started at http://{}:{}/", self.host, self.port);
            
            // Graceful shutdown с обработкой сигналов (кросс-платформенно)
            let shutdown_future = async {
                #[cfg(unix)]
                {
                    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                        .expect("Failed to create SIGTERM signal handler");
                    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
                        .expect("Failed to create SIGINT signal handler");
                    tokio::select! {
                        _ = sigterm.recv() => println!("\nReceived SIGTERM, shutting down gracefully..."),
                        _ = sigint.recv() => println!("\nReceived SIGINT, shutting down gracefully..."),
                    }
                }
                #[cfg(not(unix))]
                {
                    tokio::signal::ctrl_c()
                        .await
                        .expect("Failed to listen for ctrl+c");
                    println!("\nReceived Ctrl+C, shutting down gracefully...");
                }
            };

            // Запуск сервера с graceful shutdown
            let server_future = axum::serve(listener, app);
            
            tokio::select! {
                _ = server_future => {
                    println!("Server stopped");
                }
                _ = shutdown_future => {
                    // Graceful shutdown
                    println!("Stopping task scheduler...");
                    if let Err(e) = scheduler.stop().await {
                        eprintln!("Warning: scheduler stop error: {}", e);
                    }
                    
                    println!("Stopping backup service...");
                    if backup_enabled {
                        // Остановка backup сервиса
                    }
                    
                    println!("Closing database connections...");
                    // Закрытие соединений с БД будет выполнено автоматически при дропе store
                    
                    println!("Graceful shutdown completed");
                }
            }
            
            Ok::<(), crate::error::Error>(())
        })?;

        Ok(())
    }

    /// Создаёт admin-пользователя из env-переменных если БД пустая
    async fn seed_admin_if_empty(store: &dyn crate::db::Store) {
        use crate::db::store::RetrieveQueryParams;
        use crate::models::User;
        use bcrypt::hash;

        let admin_login = std::env::var("SEMAPHORE_ADMIN").unwrap_or_else(|_| "admin".to_string());
        let admin_password = std::env::var("SEMAPHORE_ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
        let admin_email = std::env::var("SEMAPHORE_ADMIN_EMAIL").unwrap_or_else(|_| "admin@localhost".to_string());
        let admin_name = std::env::var("SEMAPHORE_ADMIN_NAME").unwrap_or_else(|_| admin_login.clone());

        let existing = store.get_users(RetrieveQueryParams { count: Some(1), offset: 0, sort_by: None, sort_inverted: false, filter: None }).await;
        match existing {
            Ok(users) if !users.is_empty() => return,
            Err(e) => {
                eprintln!("seed_admin: failed to query users: {e}");
                return;
            }
            _ => {}
        }

        let password_hash = match hash(&admin_password, 12) {
            Ok(h) => h,
            Err(e) => { eprintln!("seed_admin: bcrypt error: {e}"); return; }
        };

        let user = User {
            id: 0,
            created: chrono::Utc::now(),
            username: admin_login.clone(),
            name: admin_name,
            email: admin_email,
            password: password_hash,
            admin: true,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };

        match store.create_user(user, &admin_password).await {
            Ok(u) => println!("Admin user '{}' created (first-run seed)", u.username),
            Err(e) => eprintln!("seed_admin: failed to create user: {e}"),
        }
    }

    /// Создаёт хранилище (async версия)
    async fn create_store_async(config: &Config) -> Result<Box<dyn crate::db::Store + Send + Sync>, crate::error::Error> {
        match config.database.dialect.clone().unwrap_or(crate::config::DbDialect::SQLite) {
            crate::config::DbDialect::SQLite |
            crate::config::DbDialect::MySQL |
            crate::config::DbDialect::Postgres => {
                let url = config.database_url()
                    .map_err(|e| crate::error::Error::Other(e.to_string()))?;
                let store = SqlStore::new(&url).await?;
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
