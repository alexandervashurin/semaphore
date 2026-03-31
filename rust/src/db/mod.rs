//! Слой доступа к данным
//!
//! Этот модуль предоставляет абстракции для работы с различными базами данных:
//! - SQLite
//! - MySQL
//! - PostgreSQL

pub mod sql;
pub mod store;

// Ре-экспорт основных типов
pub use store::{
    AccessKeyManager, ConnectionManager, EnvironmentManager, EventManager, IntegrationManager,
    InventoryManager, MigrationManager, OptionsManager, ProjectStore, RepositoryManager,
    RunnerManager, ScheduleManager, SessionManager, Store, TaskManager, TemplateManager,
    TokenManager, UserManager, ViewManager,
};

pub use sql::SqlStore;

#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub use mock::MockStore;
