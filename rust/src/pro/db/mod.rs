//! PRO DB Module
//!
//! PRO DB модуль для Velum

pub mod factory;

pub use factory::{new_ansible_task_repository, new_terraform_store};
