//! PostgreSQL Implementation
//!
//! Реализация операций с БД для PostgreSQL

pub mod environment;
pub mod inventory;
pub mod project;
pub mod repository;
pub mod template;
pub mod user;

// Re-export для удобства
pub use environment::*;
pub use inventory::*;
pub use project::*;
pub use repository::*;
pub use template::*;
pub use user::*;
