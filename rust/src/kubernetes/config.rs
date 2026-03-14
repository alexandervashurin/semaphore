//! Kubernetes Configuration - Конфигурация для Kubernetes интеграции

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Полная конфигурация Kubernetes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Путь к kubeconfig
    pub kubeconfig_path: Option<String>,
    /// Namespace по умолчанию
    pub default_namespace: String,
    /// Контекст для подключения
    pub context: Option<String>,
    /// Таймаут запросов
    pub timeout_secs: u64,
    /// Включить in-cluster config
    pub in_cluster: bool,
    /// Конфигурация для Job
    pub job_config: JobRunnerConfig,
    /// Конфигурация для Helm
    pub helm_config: HelmRunnerConfig,
}

impl Default for KubernetesConfig {
    fn default() -> Self {
        Self {
            kubeconfig_path: None,
            default_namespace: "default".to_string(),
            context: None,
            timeout_secs: 30,
            in_cluster: false,
            job_config: JobRunnerConfig::default(),
            helm_config: HelmRunnerConfig::default(),
        }
    }
}

/// Конфигурация для запуска Job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRunnerConfig {
    /// Image по умолчанию
    pub default_image: String,
    /// CPU limit
    pub cpu_limit: String,
    /// Memory limit
    pub memory_limit: String,
    /// CPU request
    pub cpu_request: String,
    /// Memory request
    pub memory_request: String,
    /// Service account
    pub service_account: Option<String>,
    /// TTL после завершения
    pub ttl_seconds: i32,
    /// Backoff limit
    pub backoff_limit: i32,
    /// Active deadline
    pub active_deadline: i64,
    /// Annotations для Job
    pub annotations: HashMap<String, String>,
    /// Labels для Job
    pub labels: HashMap<String, String>,
}

impl Default for JobRunnerConfig {
    fn default() -> Self {
        let mut annotations = HashMap::new();
        annotations.insert("app.kubernetes.io/managed-by".to_string(), "semaphore".to_string());
        
        let mut labels = HashMap::new();
        labels.insert("app.kubernetes.io/name".to_string(), "semaphore".to_string());
        
        Self {
            default_image: "alpine:latest".to_string(),
            cpu_limit: "1000m".to_string(),
            memory_limit: "1Gi".to_string(),
            cpu_request: "100m".to_string(),
            memory_request: "128Mi".to_string(),
            service_account: None,
            ttl_seconds: 3600,
            backoff_limit: 3,
            active_deadline: 7200,
            annotations,
            labels,
        }
    }
}

/// Конфигурация для Helm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRunnerConfig {
    /// Путь к helm бинарнику
    pub helm_path: String,
    /// Репозитории для добавления
    pub repositories: Vec<HelmRepository>,
    /// Charts по умолчанию
    pub default_charts: Vec<String>,
    /// Values по умолчанию
    pub default_values: HashMap<String, String>,
}

impl Default for HelmRunnerConfig {
    fn default() -> Self {
        Self {
            helm_path: "helm".to_string(),
            repositories: Vec::new(),
            default_charts: Vec::new(),
            default_values: HashMap::new(),
        }
    }
}

/// Helm репозиторий
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRepository {
    /// Имя репозитория
    pub name: String,
    /// URL репозитория
    pub url: String,
}

/// Конфигурация kubectl команды
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubectlCommand {
    /// Команда kubectl
    pub command: String,
    /// Аргументы
    pub args: Vec<String>,
    /// Namespace
    pub namespace: Option<String>,
    /// Ожидать завершения
    pub wait: bool,
    /// Таймаут
    pub timeout: Option<String>,
}

impl KubectlCommand {
    /// Создаёт новую kubectl команду
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            args: Vec::new(),
            namespace: None,
            wait: false,
            timeout: None,
        }
    }

    /// Добавляет аргумент
    pub fn with_arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    /// Устанавливает namespace
    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    /// Устанавливает wait flag
    pub fn with_wait(mut self, wait: bool) -> Self {
        self.wait = wait;
        self
    }

    /// Устанавливает timeout
    pub fn with_timeout(mut self, timeout: &str) -> Self {
        self.timeout = Some(timeout.to_string());
        self
    }

    /// Строит команду для выполнения
    pub fn build(&self) -> Vec<String> {
        let mut cmd = vec![self.command.clone()];
        
        if let Some(ns) = &self.namespace {
            cmd.push("-n".to_string());
            cmd.push(ns.clone());
        }
        
        if self.wait {
            cmd.push("--wait".to_string());
        }
        
        if let Some(timeout) = &self.timeout {
            cmd.push("--timeout".to_string());
            cmd.push(timeout.clone());
        }
        
        cmd.extend(self.args.clone());
        
        cmd
    }
}

/// Статус Kubernetes ресурса
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceStatus {
    /// Ресурс создаётся
    Pending,
    /// Ресурс активен
    Active,
    /// Ресурс завершён
    Completed,
    /// Ресурс провалился
    Failed,
    /// Неизвестный статус
    Unknown,
}

impl std::fmt::Display for ResourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceStatus::Pending => write!(f, "Pending"),
            ResourceStatus::Active => write!(f, "Active"),
            ResourceStatus::Completed => write!(f, "Completed"),
            ResourceStatus::Failed => write!(f, "Failed"),
            ResourceStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kubernetes_config_default() {
        let config = KubernetesConfig::default();
        assert_eq!(config.default_namespace, "default");
        assert_eq!(config.timeout_secs, 30);
        assert!(!config.in_cluster);
    }

    #[test]
    fn test_job_runner_config_default() {
        let config = JobRunnerConfig::default();
        assert_eq!(config.default_image, "alpine:latest");
        assert_eq!(config.cpu_limit, "1000m");
        assert_eq!(config.memory_limit, "1Gi");
        assert_eq!(config.ttl_seconds, 3600);
        assert_eq!(config.backoff_limit, 3);
    }

    #[test]
    fn test_kubectl_command_builder() {
        let cmd = KubectlCommand::new("kubectl")
            .with_arg("apply")
            .with_arg("-f")
            .with_arg("deployment.yaml")
            .with_namespace("production")
            .with_wait(true)
            .with_timeout("5m");
        
        let built = cmd.build();
        assert!(built.contains(&"kubectl".to_string()));
        assert!(built.contains(&"-n".to_string()));
        assert!(built.contains(&"production".to_string()));
        assert!(built.contains(&"--wait".to_string()));
        assert!(built.contains(&"--timeout".to_string()));
        assert!(built.contains(&"5m".to_string()));
    }

    #[test]
    fn test_resource_status_display() {
        assert_eq!(ResourceStatus::Pending.to_string(), "Pending");
        assert_eq!(ResourceStatus::Active.to_string(), "Active");
        assert_eq!(ResourceStatus::Completed.to_string(), "Completed");
    }
}
