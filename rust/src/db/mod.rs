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
    InventoryManager, MigrationManager, OptionsManager, OrganizationManager, ProjectStore,
    RepositoryManager, RunnerManager, ScheduleManager, SessionManager, Store, TaskManager,
    TemplateManager, TokenManager, UserManager, ViewManager,
};

pub use sql::SqlStore;

pub mod mock;
pub use mock::MockStore;
