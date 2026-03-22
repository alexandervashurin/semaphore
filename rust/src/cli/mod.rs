//! Интерфейс командной строки (CLI)
//!
//! Предоставляет команды для управления Velum:
//! - server - запуск веб-сервера
//! - runner - запуск раннера задач
//! - migrate - миграции базы данных
//! - user - управление пользователями
//! - project - управление проектами

use anyhow::Result;

/// Тип результата для CLI команд
pub type CliResult<T> = Result<T>;

#[cfg(test)]
mod tests;

pub mod cmd_migrate;
pub mod cmd_project;
pub mod cmd_runner;
pub mod cmd_server;
pub mod cmd_setup;
pub mod cmd_token;
pub mod cmd_user;
pub mod cmd_vault;
pub mod cmd_version;

use clap::{Parser, Subcommand};
use std::sync::Arc;
use crate::config::{Config, DbDialect};
use crate::db::SqlStore;

pub use cmd_migrate::MigrateCommand;
pub use cmd_project::ProjectCommand;
pub use cmd_runner::RunnerCommand;
pub use cmd_server::ServerCommand;
pub use cmd_setup::SetupCommand;
pub use cmd_token::TokenCommand;
pub use cmd_user::UserCommand;
pub use cmd_vault::VaultCommand;
pub use cmd_version::VersionCommand;

/// Velum UI - современный веб-интерфейс для управления DevOps-инструментами
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Путь к файлу конфигурации
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Тип базы данных (bolt, sqlite, mysql, postgres)
    #[arg(long, global = true)]
    db_dialect: Option<String>,

    /// Путь к базе данных
    #[arg(long, global = true)]
    db_path: Option<String>,

    /// Хост базы данных
    #[arg(long, global = true)]
    db_host: Option<String>,

    /// Порт базы данных
    #[arg(long, global = true)]
    db_port: Option<u16>,

    /// Имя пользователя базы данных
    #[arg(long, global = true)]
    db_user: Option<String>,

    /// Пароль базы данных
    #[arg(long, global = true)]
    db_password: Option<String>,

    /// Имя базы данных
    #[arg(long, global = true)]
    db_name: Option<String>,

    /// HTTP порт
    #[arg(long, global = true)]
    http_port: Option<u16>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Запуск веб-сервера
    Server(ServerCommand),

    /// Запуск раннера задач
    Runner(RunnerCommand),

    /// Применение миграций базы данных
    Migrate(MigrateCommand),

    /// Управление пользователями
    User(UserCommand),

    /// Управление проектами
    Project(ProjectCommand),

    /// Настройка Velum (интерактивный мастер)
    Setup(SetupCommand),

    /// Управление API токенами
    Token(TokenCommand),

    /// Управление хранилищами секретов
    Vault(VaultCommand),

    /// Версия приложения
    Version(VersionCommand),
}

#[derive(Parser, Debug)]
struct RunnerArgs {
    /// Токен раннера
    #[arg(long)]
    token: Option<String>,

    /// URL сервера
    #[arg(long)]
    server_url: Option<String>,
}

#[derive(Parser, Debug)]
struct MigrateArgs {
    /// Применить миграции
    #[arg(long)]
    upgrade: bool,

    /// Откатить миграции
    #[arg(long)]
    downgrade: bool,
}

#[derive(Parser, Debug)]
struct UserArgs {
    #[command(subcommand)]
    command: UserCommands,
}

#[derive(Subcommand, Debug)]
enum UserCommands {
    /// Добавить нового пользователя
    Add(UserAddArgs),

    /// Изменить пользователя
    Change(UserChangeArgs),

    /// Удалить пользователя
    Delete(UserDeleteArgs),

    /// Получить информацию о пользователе
    Get(UserGetArgs),

    /// Список пользователей
    List,

    /// Управление TOTP
    Totp(UserTotpArgs),
}

#[derive(Parser, Debug)]
struct UserAddArgs {
    /// Имя пользователя
    #[arg(short, long)]
    username: String,

    /// Полное имя
    #[arg(short, long)]
    name: String,

    /// Электронная почта
    #[arg(short, long)]
    email: String,

    /// Пароль
    #[arg(short = 'P', long)]
    password: String,

    /// Сделать администратором
    #[arg(long)]
    admin: bool,
}

#[derive(Parser, Debug)]
struct UserChangeArgs {
    /// ID пользователя
    #[arg(long)]
    id: i32,

    /// Новое имя пользователя
    #[arg(long)]
    username: Option<String>,

    /// Новое полное имя
    #[arg(long)]
    name: Option<String>,

    /// Новая электронная почта
    #[arg(long)]
    email: Option<String>,

    /// Новый пароль
    #[arg(long)]
    password: Option<String>,
}

#[derive(Parser, Debug)]
struct UserDeleteArgs {
    /// ID пользователя
    #[arg(long)]
    id: Option<i32>,

    /// Имя пользователя
    #[arg(long)]
    username: Option<String>,
}

#[derive(Parser, Debug)]
struct UserGetArgs {
    /// ID пользователя
    #[arg(long)]
    id: Option<i32>,

    /// Имя пользователя
    #[arg(long)]
    username: Option<String>,
}

#[derive(Parser, Debug)]
struct UserTotpArgs {
    #[command(subcommand)]
    command: TotpCommands,
}

#[derive(Subcommand, Debug)]
enum TotpCommands {
    /// Добавить TOTP
    Add(TotpAddArgs),

    /// Удалить TOTP
    Delete(TotpDeleteArgs),
}

#[derive(Parser, Debug)]
struct TotpAddArgs {
    /// ID пользователя
    #[arg(long)]
    user_id: i32,
}

#[derive(Parser, Debug)]
struct TotpDeleteArgs {
    /// ID пользователя
    #[arg(long)]
    user_id: i32,

    /// ID TOTP
    #[arg(long)]
    totp_id: i32,
}

#[derive(Parser, Debug)]
struct ProjectArgs {
    #[command(subcommand)]
    command: ProjectCommands,
}

#[derive(Subcommand, Debug)]
enum ProjectCommands {
    /// Экспорт проекта
    Export(ProjectExportArgs),

    /// Импорт проекта
    Import(ProjectImportArgs),
}

#[derive(Parser, Debug)]
struct ProjectExportArgs {
    /// ID проекта
    #[arg(long)]
    id: i32,

    /// Путь к файлу экспорта
    #[arg(short, long)]
    file: String,
}

#[derive(Parser, Debug)]
struct ProjectImportArgs {
    /// Путь к файлу импорта
    #[arg(short, long)]
    file: String,
}

#[derive(Parser, Debug)]
struct SetupArgs {
    /// Пропустить интерактивный режим
    #[arg(long)]
    non_interactive: bool,
}

impl Cli {
    /// Выполняет команду CLI
    pub fn run(self) -> anyhow::Result<()> {
        // Загрузка .env файла из текущего каталога и из родительских директорий
        let _ = dotenvy::dotenv();
        let _ = dotenvy::from_path("../.env");
        let _ = dotenvy::from_path("../../.env");

        // Инициализация логирования
        crate::init_logging();

        // Загрузка конфигурации
        let mut config = Config::from_env()?;

        // Переопределение из аргументов командной строки
        if let Some(db_dialect) = self.db_dialect {
            config.database.dialect = Some(match db_dialect.as_str() {
                "sqlite" => DbDialect::SQLite,
                "mysql" => DbDialect::MySQL,
                "postgres" => DbDialect::Postgres,
                _ => DbDialect::SQLite,
            });
        }

        if let Some(db_path) = self.db_path {
            config.database.path = Some(db_path);
        }

        // http_port is handled via tcp_address

        match self.command {
            Commands::Server(cmd) => cmd.run(Arc::new(config)),
            Commands::Runner(cmd) => cmd.run(),
            Commands::Migrate(cmd) => cmd.run(),
            Commands::User(cmd) => cmd.run(Arc::new(config.clone())),
            Commands::Project(cmd) => cmd.run(),
            Commands::Setup(cmd) => cmd.run(),
            Commands::Token(cmd) => cmd.run(),
            Commands::Vault(cmd) => cmd.run(),
            Commands::Version(cmd) => cmd.run(),
        }
    }
}

/// Команда: запуск раннера
#[allow(unused_variables)]
fn cmd_runner(args: RunnerArgs, config: Config) -> anyhow::Result<()> {
    tracing::info!("Запуск раннера Velum...");
    tracing::warn!("Запуск раннера через CLI пока не поддерживается. Используйте 'semaphore server' вместо этого.");
    Ok(())
}

/// Команда: миграции
fn cmd_migrate(args: MigrateArgs, config: Config) -> anyhow::Result<()> {
    tracing::info!("Применение миграций...");

    let database_url = config.database_url().map_err(|e| anyhow::anyhow!("{}", e))?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            if args.upgrade {
                tracing::info!("Применение миграций...");
                // Создаём SqlStore и применяем миграции
                let store = crate::db::sql::SqlStore::new(&database_url).await
                    .map_err(|e| anyhow::anyhow!("Ошибка подключения к БД: {}", e))?;
                
                // Миграции применяются автоматически при создании SqlStore
                tracing::info!("Миграции успешно применены");
            }

            if args.downgrade {
                tracing::info!("Откат миграций...");
                tracing::warn!("Откат миграций пока не поддерживается");
            }

            Ok::<_, anyhow::Error>(())
        })?;

    Ok(())
}

/// Команда: пользователи
fn cmd_user(args: UserArgs, config: Config) -> anyhow::Result<()> {
    match args.command {
        UserCommands::Add(add_args) => cmd_user_add(add_args, config),
        UserCommands::List => cmd_user_list(config),
        _ => {
            tracing::warn!("Команда ещё не реализована");
            Ok(())
        }
    }
}

/// Команда: добавить пользователя
fn cmd_user_add(args: UserAddArgs, config: Config) -> anyhow::Result<()> {
    use crate::models::User;
    use bcrypt::hash;

    let store = create_store(&config).map_err(|e| anyhow::anyhow!(e))?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            // Хеширование пароля
            let password_hash = hash(&args.password, 12)?;

            let user = User {
                id: 0,
                created: chrono::Utc::now(),
                username: args.username,
                name: args.name,
                email: args.email,
                password: password_hash,
                admin: args.admin,
                external: false,
                alert: false,
                pro: false,
                totp: None,
                email_otp: None,
            };

            let created_user = store.create_user(user, &args.password).await?;
            tracing::info!("Пользователь создан: ID = {}", created_user.id);

            Ok::<_, anyhow::Error>(())
        })?;

    Ok(())
}

/// Команда: список пользователей
fn cmd_user_list(config: Config) -> anyhow::Result<()> {
    use crate::db::store::RetrieveQueryParams;

    let store = create_store(&config).map_err(|e| anyhow::anyhow!(e))?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let users = store.get_users(RetrieveQueryParams::default()).await?;

            println!("{:<6} {:<20} {:<30} Name", "ID", "Username", "Email");
            println!("{}", "-".repeat(70));
            for user in users {
                println!("{:<6} {:<20} {:<30} {}", user.id, user.username, user.email, user.name);
            }

            Ok::<_, anyhow::Error>(())
        })?;

    Ok(())
}

/// Команда: проекты
#[allow(unused_variables)]
fn cmd_project(args: ProjectArgs, config: Config) -> anyhow::Result<()> {
    match args.command {
        ProjectCommands::Export(_) | ProjectCommands::Import(_) => {
            tracing::warn!("Команда ещё не реализована");
            Ok(())
        }
    }
}

/// Команда: настройка
fn cmd_setup(_args: SetupArgs, _config: Config) -> anyhow::Result<()> {
    tracing::info!("Мастер настройки Velum...");
    
    println!("\n=== Мастер настройки Velum ===\n");
    println!("Создайте файл конфигурации вручную или используйте переменные окружения:");
    println!();
    println!("  SEMAPHORE_DB_DIALECT=sqlite    # или postgres, mysql");
    println!("  SEMAPHORE_DB_PATH=/path/to/db  # для SQLite");
    println!("  SEMAPHORE_DB_HOST=localhost    # для PostgreSQL/MySQL");
    println!("  SEMAPHORE_DB_PORT=5432         # для PostgreSQL");
    println!("  SEMAPHORE_DB_USER=semaphore");
    println!("  SEMAPHORE_DB_PASS=secret");
    println!("  SEMAPHORE_DB_NAME=semaphore");
    println!();
    println!("Затем выполните:");
    println!("  semaphore migrate --upgrade");
    println!("  semaphore user add --username admin --name Admin --email admin@example.com --password admin123 --admin");
    println!();

    Ok(())
}

/// Команда: версия
fn cmd_version() -> anyhow::Result<()> {
    println!("Velum UI {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

/// Создаёт хранилище на основе конфигурации
fn create_store(config: &Config) -> anyhow::Result<Box<dyn crate::db::Store + Send + Sync>> {
    let database_url = config.database_url().map_err(|e| anyhow::anyhow!("{}", e))?;

    let store: Box<dyn crate::db::Store + Send + Sync> = match config.database.dialect.clone().unwrap_or(DbDialect::SQLite) {
        DbDialect::SQLite | DbDialect::MySQL | DbDialect::Postgres => {
            Box::new(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?
                    .block_on(SqlStore::new(&database_url))?
            )
        }
    };

    Ok(store)
}
