//! Kubernetes Module - Интеграция с Kubernetes
//!
//! Этот модуль предоставляет:
//! - Запуск задач в Kubernetes Jobs (kubectl subprocess — задачи)
//! - Управление Pod'ами / Job / Helm (kubectl subprocess)
//! - **[service]** — KubernetesClusterService (kube-rs) — для UI API
//! - **[cluster_manager]** — мульти-кластер подключения для UI

pub mod client;
pub mod config;
pub mod job;
pub mod helm;

// Новые модули UI (kube-rs)
pub mod service;
pub mod cluster_manager;

pub use client::KubernetesClient;
pub use config::{KubernetesConfig, JobRunnerConfig, HelmRunnerConfig, HelmRepository};
pub use job::{KubernetesJob, JobConfig, JobStatus};
pub use helm::{HelmClient, HelmRelease, HelmChart};
pub use service::KubernetesClusterService;
pub use cluster_manager::KubernetesClusterManager;
