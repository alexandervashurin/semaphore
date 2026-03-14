//! Kubernetes Job Executor - Запуск задач в Kubernetes Jobs

use k8s_openapi::api::batch::v1::{Job, JobSpec};
use k8s_openapi::api::core::v1::{PodSpec, PodTemplateSpec, Container, EnvVar};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use crate::error::{Error, Result};
use crate::kubernetes::client::KubernetesClient;

/// Конфигурация Kubernetes Job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    /// Имя job
    pub name: String,
    /// Namespace для запуска
    pub namespace: Option<String>,
    /// Docker image
    pub image: String,
    /// Команда для выполнения
    pub command: Option<Vec<String>>,
    /// Аргументы команды
    pub args: Option<Vec<String>>,
    /// Переменные окружения
    pub env: Option<Vec<EnvVar>>,
    /// Limit CPU
    pub cpu_limit: Option<String>,
    /// Limit Memory
    pub memory_limit: Option<String>,
    /// Request CPU
    pub cpu_request: Option<String>,
    /// Request Memory
    pub memory_request: Option<String>,
    /// Service Account
    pub service_account: Option<String>,
    /// Restart Policy
    pub restart_policy: String,
    /// TTL после завершения (секунды)
    pub ttl_seconds: Option<i32>,
    /// Backoff limit
    pub backoff_limit: Option<i32>,
    /// Active deadline (секунды)
    pub active_deadline: Option<i64>,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            name: "semaphore-job".to_string(),
            namespace: None,
            image: "alpine:latest".to_string(),
            command: None,
            args: None,
            env: None,
            cpu_limit: Some("500m".to_string()),
            memory_limit: Some("512Mi".to_string()),
            cpu_request: Some("100m".to_string()),
            memory_request: Some("128Mi".to_string()),
            service_account: None,
            restart_policy: "Never".to_string(),
            ttl_seconds: Some(300),
            backoff_limit: Some(3),
            active_deadline: Some(3600),
        }
    }
}

/// Статус Kubernetes Job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Succeeded => write!(f, "Succeeded"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Kubernetes Job executor
pub struct KubernetesJob {
    config: JobConfig,
}

impl KubernetesJob {
    /// Создаёт новый Kubernetes Job
    pub fn new(config: JobConfig) -> Self {
        Self { config }
    }

    /// Генерирует YAML для Job
    pub fn generate_yaml(&self) -> String {
        let mut yaml = format!(
            r#"apiVersion: batch/v1
kind: Job
metadata:
  name: {}
  namespace: {}
  labels:
    app: semaphore
    job: {}
spec:
  ttlSecondsAfterFinished: {}
  backoffLimit: {}
  activeDeadlineSeconds: {}
  template:
    spec:
      restartPolicy: {}
"#,
            self.config.name,
            self.config.namespace.as_deref().unwrap_or("default"),
            self.config.name,
            self.config.ttl_seconds.unwrap_or(300),
            self.config.backoff_limit.unwrap_or(3),
            self.config.active_deadline.unwrap_or(3600),
            self.config.restart_policy
        );

        if let Some(sa) = &self.config.service_account {
            yaml.push_str(&format!("      serviceAccountName: {}\n", sa));
        }

        yaml.push_str("      containers:\n");
        yaml.push_str(&format!("      - name: {}\n", self.config.name));
        yaml.push_str(&format!("        image: {}\n", self.config.image));

        if let Some(command) = &self.config.command {
            yaml.push_str("        command:\n");
            for cmd in command {
                yaml.push_str(&format!("        - {}\n", cmd));
            }
        }

        if let Some(args) = &self.config.args {
            yaml.push_str("        args:\n");
            for arg in args {
                yaml.push_str(&format!("        - {}\n", arg));
            }
        }

        // Resources
        yaml.push_str("        resources:\n");
        yaml.push_str("          limits:\n");
        if let Some(cpu) = &self.config.cpu_limit {
            yaml.push_str(&format!("            cpu: {}\n", cpu));
        }
        if let Some(memory) = &self.config.memory_limit {
            yaml.push_str(&format!("            memory: {}\n", memory));
        }
        yaml.push_str("          requests:\n");
        if let Some(cpu) = &self.config.cpu_request {
            yaml.push_str(&format!("            cpu: {}\n", cpu));
        }
        if let Some(memory) = &self.config.memory_request {
            yaml.push_str(&format!("            memory: {}\n", memory));
        }

        yaml
    }

    /// Запускает Job в Kubernetes
    pub fn run(&self, client: &KubernetesClient) -> Result<String> {
        let yaml = self.generate_yaml();
        let namespace = self.config.namespace.as_deref().or(Some(client.default_namespace()));
        
        info!("Creating Kubernetes Job '{}' in namespace '{:?}'", 
              self.config.name, namespace);
        
        client.create_job(&yaml, namespace)?;
        
        info!("Job '{}' created successfully", self.config.name);
        
        Ok(self.config.name.clone())
    }

    /// Получает статус Job
    pub fn get_status(&self, client: &KubernetesClient) -> Result<JobStatus> {
        let namespace = self.config.namespace.as_deref().or(Some(client.default_namespace()));
        let status_str = client.get_job_status(&self.config.name, namespace)?;
        
        let status = match status_str.as_str() {
            "Complete" => JobStatus::Succeeded,
            "Failed" => JobStatus::Failed,
            "Running" => JobStatus::Running,
            _ => JobStatus::Pending,
        };
        
        Ok(status)
    }

    /// Ждёт завершения Job
    pub fn wait_for_completion(
        &self,
        client: &KubernetesClient,
        timeout_secs: u64,
    ) -> Result<JobStatus> {
        use std::time::{Duration, Instant};
        
        let namespace = self.config.namespace.as_deref().or(Some(client.default_namespace()));
        
        info!("Waiting for Job '{}' to complete (timeout: {}s)", self.config.name, timeout_secs);
        
        let start = Instant::now();
        
        while start.elapsed() < Duration::from_secs(timeout_secs) {
            let status_str = client.get_job_status(&self.config.name, namespace)?;
            
            match status_str.as_str() {
                "Complete" => {
                    info!("Job '{}' completed", self.config.name);
                    return Ok(JobStatus::Succeeded);
                }
                "Failed" => {
                    warn!("Job '{}' failed", self.config.name);
                    return Ok(JobStatus::Failed);
                }
                _ => {
                    debug!("Job '{}' status: {}", self.config.name, status_str);
                    std::thread::sleep(Duration::from_secs(5));
                }
            }
        }
        
        warn!("Timeout waiting for Job '{}' to complete", self.config.name);
        Ok(JobStatus::Unknown)
    }

    /// Удаляет Job
    pub fn delete(&self, client: &KubernetesClient) -> Result<()> {
        let namespace = self.config.namespace.as_deref().or(Some(client.default_namespace()));
        
        info!("Deleting Job '{}'", self.config.name);
        
        let _ = client.run_command(&[
            "delete",
            "job",
            &self.config.name,
            "-n",
            namespace.unwrap_or("default"),
            "--ignore-not-found",
        ]);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_config_default() {
        let config = JobConfig::default();
        assert_eq!(config.name, "semaphore-job");
        assert_eq!(config.image, "alpine:latest");
        assert_eq!(config.restart_policy, "Never");
        assert_eq!(config.ttl_seconds, Some(300));
        assert_eq!(config.backoff_limit, Some(3));
    }

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Pending.to_string(), "Pending");
        assert_eq!(JobStatus::Running.to_string(), "Running");
        assert_eq!(JobStatus::Succeeded.to_string(), "Succeeded");
        assert_eq!(JobStatus::Failed.to_string(), "Failed");
    }

    #[test]
    fn test_job_yaml_generation() {
        let config = JobConfig {
            name: "test-job".to_string(),
            namespace: Some("test-ns".to_string()),
            image: "nginx:latest".to_string(),
            ..Default::default()
        };
        
        let job = KubernetesJob::new(config);
        let yaml = job.generate_yaml();
        
        assert!(yaml.contains("name: test-job"));
        assert!(yaml.contains("namespace: test-ns"));
        assert!(yaml.contains("image: nginx:latest"));
        assert!(yaml.contains("kind: Job"));
    }
}
