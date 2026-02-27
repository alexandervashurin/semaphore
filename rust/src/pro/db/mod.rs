//! PRO DB Module
//!
//! PRO DB модуль для Semaphore

pub mod factory;

pub use factory::{new_terraform_store, new_ansible_task_repository};
