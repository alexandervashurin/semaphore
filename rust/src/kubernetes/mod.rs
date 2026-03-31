//! Kubernetes Module - Интеграция с Kubernetes
//!
//! Этот модуль предоставляет:
//! - Запуск задач в Kubernetes Jobs (kubectl subprocess — задачи)
//! - Управление Pod'ами / Job / Helm (kubectl subprocess)
//! - **[service]** — KubernetesClusterService (kube-rs) — для UI API
//! - **[cluster_manager]** — мульти-кластер подключения для UI

pub mod client;
pub mod config;
pub mod helm;
pub mod job;

// Новые модули UI (kube-rs)
pub mod service;
pub mod cluster_manager;

pub use client::KubernetesClient;
pub use config::{HelmRepository, HelmRunnerConfig, JobRunnerConfig, KubernetesConfig};
pub use helm::{HelmChart, HelmClient, HelmRelease};
pub use job::{JobConfig, JobStatus, KubernetesJob};
