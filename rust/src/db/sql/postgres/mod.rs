//! PostgreSQL Implementation
//!
//! Реализация операций с БД для PostgreSQL

pub mod user;
pub mod template;
pub mod project;
pub mod inventory;
pub mod repository;
pub mod environment;

// Re-export для удобства
pub use user::*;
pub use template::*;
pub use project::*;
pub use inventory::*;
pub use repository::*;
pub use environment::*;
