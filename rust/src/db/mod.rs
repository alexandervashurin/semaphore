//! Слой доступа к данным
//!
//! Этот модуль предоставляет абстракции для работы с различными базами данных:
//! - BoltDB (ключ-значение, через sled)
//! - SQLite
//! - MySQL
//! - PostgreSQL

pub mod store;
pub mod sql;
pub mod bolt;

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
pub use bolt::BoltStore;

#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub use mock::MockStore;
