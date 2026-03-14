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

        // Создаём хранилище и запускаем сервер в одном runtime
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        runtime.block_on(async {
            // Создаём хранилище
            let store = Self::create_store_async(&config).await?;

            // Сид admin-пользователя при первом запуске
            Self::seed_admin_if_empty(&*store).await;

            // Создаём приложение
            let app = api::create_app(store);

            // Запускаем сервер
            let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port))
                .await
                .map_err(|e| crate::error::Error::Other(e.to_string()))?;
            println!("Server started at http://{}:{}/", self.host, self.port);
            axum::serve(listener, app).await
                .map_err(|e| crate::error::Error::Other(e.to_string()))?;
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
