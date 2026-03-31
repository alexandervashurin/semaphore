//! LocalJob модуль
//!
//! Реализация локального выполнения задач
//! Аналог services/tasks/LocalJob.go из Go версии

pub mod args;
pub mod cli;
pub mod environment;
pub mod repository;
pub mod run;
pub mod ssh;
pub mod types;
pub mod vault;

pub use types::LocalJob;
