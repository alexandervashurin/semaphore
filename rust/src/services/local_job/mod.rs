//! LocalJob модуль
//!
//! Реализация локального выполнения задач
//! Аналог services/tasks/LocalJob.go из Go версии

pub mod types;
pub mod environment;
pub mod cli;
pub mod args;

pub use types::LocalJob;
