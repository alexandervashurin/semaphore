//! Kubernetes Module - Интеграция с Kubernetes
//!
//! Этот модуль предоставляет:
//! - Запуск задач в Kubernetes Jobs
//! - Управление Pod'ами
//! - Поддержку Helm charts
//! - Kubectl команды

pub mod client;
pub mod config;
pub mod helm;
pub mod job;

pub use client::KubernetesClient;
pub use config::{HelmRepository, HelmRunnerConfig, JobRunnerConfig, KubernetesConfig};
pub use helm::{HelmChart, HelmClient, HelmRelease};
pub use job::{JobConfig, JobStatus, KubernetesJob};
