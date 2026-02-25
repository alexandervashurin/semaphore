//! Интерфейс командной строки (CLI)
//!
//! Предоставляет команды для управления Semaphore:
//! - server - запуск веб-сервера
//! - runner - запуск раннера задач
//! - migrate - миграции базы данных
//! - user - управление пользователями
//! - project - управление проектами

#[cfg(test)]
mod tests;

use clap::{Parser, Subcommand};
use crate::config::{Config, DbDialect};
use crate::db::{SqlStore, BoltStore};

/// Semaphore UI - современный веб-интерфейс для управления DevOps-инструментами
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
    Server(ServerArgs),

    /// Запуск раннера задач
    Runner(RunnerArgs),

    /// Применение миграций базы данных
    Migrate(MigrateArgs),

    /// Управление пользователями
    User(UserArgs),

    /// Управление проектами
    Project(ProjectArgs),

    /// Настройка Semaphore (интерактивный мастер)
    Setup(SetupArgs),

    /// Версия приложения
    Version,
}

#[derive(Parser, Debug, Clone)]
struct ServerArgs {
    /// Хост для прослушивания
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Порт HTTP
    #[arg(short = 'p', long, default_value = "3000")]
    port: u16,
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
        // Инициализация логирования
        crate::init_logging();

        // Загрузка конфигурации
        let mut config = Config::from_env()?;

        // Переопределение из аргументов командной строки
        if let Some(db_dialect) = self.db_dialect {
            config.db_dialect = match db_dialect.as_str() {
                "bolt" => DbDialect::Bolt,
                "sqlite" => DbDialect::SQLite,
                "mysql" => DbDialect::MySQL,
                "postgres" => DbDialect::PostgreSQL,
                _ => DbDialect::Bolt,
            };
        }

        if let Some(db_path) = self.db_path {
            config.db_path = Some(db_path);
        }

        if let Some(http_port) = self.http_port {
            config.http_port = http_port;
        }

        match self.command {
            Commands::Server(args) => cmd_server(args, config),
            Commands::Runner(args) => cmd_runner(args, config),
            Commands::Migrate(args) => cmd_migrate(args, config),
            Commands::User(args) => cmd_user(args, config),
            Commands::Project(args) => cmd_project(args, config),
            Commands::Setup(args) => cmd_setup(args, config),
            Commands::Version => cmd_version(),
        }
    }
}

/// Команда: запуск сервера
fn cmd_server(args: ServerArgs, config: Config) -> anyhow::Result<()> {
    use crate::api::create_app;
    use std::net::SocketAddr;

    tracing::info!("Запуск сервера Semaphore...");

    // Создание хранилища
    let store = create_store(&config).map_err(|e| anyhow::anyhow!(e))?;

    // Создание приложения Axum
    let app = create_app(store);

    // Адрес для прослушивания
    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .expect("Неверный адрес");

    tracing::info!("Сервер слушает на {}", addr);

    // Запуск сервера
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
            Ok::<_, anyhow::Error>(())
        })?;

    Ok(())
}

/// Команда: запуск раннера
#[allow(unused_variables)]
fn cmd_runner(args: RunnerArgs, config: Config) -> anyhow::Result<()> {
    tracing::info!("Запуск раннера Semaphore...");

    // TODO: Реализовать запуск раннера

    Ok(())
}

/// Команда: миграции
fn cmd_migrate(args: MigrateArgs, config: Config) -> anyhow::Result<()> {
    tracing::info!("Применение миграций...");

    let _store = create_store(&config)?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            if args.upgrade {
                tracing::info!("Применение миграций...");
                // TODO: Применить миграции
            }

            if args.downgrade {
                tracing::info!("Откат миграций...");
                // TODO: Откатить миграции
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

            println!("{:<6} {:<20} {:<30} {}", "ID", "Username", "Email", "Name");
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
#[allow(unused_variables)]
fn cmd_setup(args: SetupArgs, config: Config) -> anyhow::Result<()> {
    tracing::info!("Мастер настройки Semaphore...");

    // TODO: Реализовать интерактивный мастер настройки

    Ok(())
}

/// Команда: версия
fn cmd_version() -> anyhow::Result<()> {
    println!("Semaphore UI {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

/// Создаёт хранилище на основе конфигурации
fn create_store(config: &Config) -> anyhow::Result<Box<dyn crate::db::Store + Send + Sync>> {
    let database_url = config.database_url().map_err(|e| anyhow::anyhow!("{}", e))?;

    let store: Box<dyn crate::db::Store + Send + Sync> = match config.db_dialect {
        DbDialect::Bolt => {
            let path = config.db_path.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Путь к базе данных не указан"))?;
            Box::new(BoltStore::new(path).map_err(|e| anyhow::anyhow!("{}", e))?)
        }
        DbDialect::SQLite | DbDialect::MySQL | DbDialect::PostgreSQL => {
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
