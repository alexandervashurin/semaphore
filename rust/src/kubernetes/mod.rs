//! Kubernetes Module - Интеграция с Kubernetes
//!
//! Этот модуль предоставляет:
//! - Запуск задач в Kubernetes Jobs
//! - Управление Pod'ами
//! - Поддержку Helm charts
//! - Kubectl команды

pub mod client;
pub mod config;
pub mod job;
pub mod helm;

pub use client::KubernetesClient;
pub use config::{KubernetesConfig, JobRunnerConfig, HelmRunnerConfig, HelmRepository};
pub use job::{KubernetesJob, JobConfig, JobStatus};
pub use helm::{HelmClient, HelmRelease, HelmChart};
