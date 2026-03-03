//! Слой доступа к данным
//!
//! Этот модуль предоставляет абстракции для работы с различными базами данных:
//! - SQLite
//! - MySQL
//! - PostgreSQL

pub mod store;
pub mod sql;

// Ре-экспорт основных типов
pub use store::{
    Store,
    ConnectionManager,
    MigrationManager,
    OptionsManager,
    UserManager,
    ProjectStore,
    TemplateManager,
    InventoryManager,
    RepositoryManager,
    EnvironmentManager,
    AccessKeyManager,
    TaskManager,
    ScheduleManager,
    SessionManager,
    TokenManager,
    EventManager,
    RunnerManager,
    ViewManager,
    IntegrationManager,
};

pub use sql::SqlStore;

#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub use mock::MockStore;
