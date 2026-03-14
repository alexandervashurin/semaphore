//! Kubernetes Client - Клиент для подключения к Kubernetes API
//!
//! Использует kubectl command-line для взаимодействия с Kubernetes

use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{info, warn, error, debug};
use crate::error::{Error, Result};

/// Конфигурация Kubernetes клиента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Путь к kubeconfig файлу
    pub kubeconfig_path: Option<String>,
    /// Namespace по умолчанию
    pub default_namespace: String,
    /// Контекст для подключения
    pub context: Option<String>,
    /// Таймаут запросов (секунды)
    pub timeout_secs: u64,
    /// Включить in-cluster config
    pub in_cluster: bool,
}

impl Default for KubernetesConfig {
    fn default() -> Self {
        Self {
            kubeconfig_path: None,
            default_namespace: "default".to_string(),
            context: None,
            timeout_secs: 30,
            in_cluster: false,
        }
    }
}

/// Kubernetes клиент
pub struct KubernetesClient {
    default_namespace: String,
    kubeconfig: Option<String>,
    context: Option<String>,
}

impl KubernetesClient {
    /// Создаёт новый Kubernetes клиент
    pub fn new(config: KubernetesConfig) -> Result<Self> {
        Ok(Self {
            default_namespace: config.default_namespace,
            kubeconfig: config.kubeconfig_path,
            context: config.context,
        })
    }

    /// Создаёт kubectl команду
    fn kubectl(&self) -> Command {
        let mut cmd = Command::new("kubectl");
        
        if let Some(kubeconfig) = &self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        if let Some(context) = &self.context {
            cmd.arg("--context").arg(context);
        }
        
        cmd
    }

    /// Создаёт kubectl команду с namespace
    fn kubectl_ns(&self, namespace: Option<&str>) -> Command {
        let mut cmd = self.kubectl();
        let ns = namespace.unwrap_or(&self.default_namespace);
        cmd.arg("-n").arg(ns);
        cmd
    }

    /// Проверяет подключение к Kubernetes
    pub fn check_connection(&self) -> Result<bool> {
        let output = self.kubectl()
            .arg("cluster-info")
            .output()
            .map_err(|e| Error::Other(format!("Failed to run kubectl: {}", e)))?;
        
        if output.status.success() {
            info!("Connected to Kubernetes cluster");
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to connect to Kubernetes: {}", stderr);
            Ok(false)
        }
    }

    /// Получает список namespace'ов
    pub fn list_namespaces(&self) -> Result<Vec<String>> {
        let output = self.kubectl()
            .arg("get")
            .arg("namespaces")
            .arg("-o")
            .arg("jsonpath={.items[*].metadata.name}")
            .output()
            .map_err(|e| Error::Other(format!("Failed to list namespaces: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to list namespaces: {}", stderr)));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let namespaces: Vec<String> = stdout
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        
        Ok(namespaces)
    }

    /// Получает Pod по имени
    pub fn get_pod(&self, name: &str, namespace: Option<&str>) -> Result<String> {
        let output = self.kubectl_ns(namespace)
            .arg("get")
            .arg("pod")
            .arg(name)
            .arg("-o")
            .arg("json")
            .output()
            .map_err(|e| Error::Other(format!("Failed to get pod: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::NotFound(format!("Pod {} not found: {}", name, stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Получает Job по имени
    pub fn get_job(&self, name: &str, namespace: Option<&str>) -> Result<String> {
        let output = self.kubectl_ns(namespace)
            .arg("get")
            .arg("job")
            .arg(name)
            .arg("-o")
            .arg("json")
            .output()
            .map_err(|e| Error::Other(format!("Failed to get job: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::NotFound(format!("Job {} not found: {}", name, stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Создаёт Job из YAML
    pub fn create_job(&self, yaml: &str, namespace: Option<&str>) -> Result<()> {
        info!("Creating Kubernetes Job");
        
        let mut cmd = self.kubectl_ns(namespace);
        cmd.arg("apply").arg("-f").arg("-");
        cmd.stdin(std::process::Stdio::piped());
        
        let mut child = cmd.spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn kubectl: {}", e)))?;
        
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(yaml.as_bytes())
                .map_err(|e| Error::Other(format!("Failed to write YAML: {}", e)))?;
        }
        
        let output = child.wait_with_output()
            .map_err(|e| Error::Other(format!("Failed to wait for kubectl: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to create Job: {}", stderr)));
        }
        
        Ok(())
    }

    /// Удаляет Pod по имени
    pub fn delete_pod(&self, name: &str, namespace: Option<&str>) -> Result<()> {
        info!("Deleting Pod '{}'", name);
        
        let output = self.kubectl_ns(namespace)
            .arg("delete")
            .arg("pod")
            .arg(name)
            .output()
            .map_err(|e| Error::Other(format!("Failed to delete pod: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to delete Pod: {}", stderr)));
        }
        
        Ok(())
    }

    /// Получает логи Pod
    pub fn get_pod_logs(&self, name: &str, namespace: Option<&str>) -> Result<String> {
        let output = self.kubectl_ns(namespace)
            .arg("logs")
            .arg(name)
            .output()
            .map_err(|e| Error::Other(format!("Failed to get logs: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to get logs: {}", stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Получает статус Job
    pub fn get_job_status(&self, name: &str, namespace: Option<&str>) -> Result<String> {
        let output = self.kubectl_ns(namespace)
            .arg("get")
            .arg("job")
            .arg(name)
            .arg("-o")
            .arg("jsonpath={.status.conditions[*].type}")
            .output()
            .map_err(|e| Error::Other(format!("Failed to get job status: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to get job status: {}", stderr)));
        }
        
        let status = String::from_utf8_lossy(&output.stdout);
        if status.is_empty() {
            Ok("Pending".to_string())
        } else if status.contains("Complete") {
            Ok("Complete".to_string())
        } else if status.contains("Failed") {
            Ok("Failed".to_string())
        } else {
            Ok("Running".to_string())
        }
    }

    /// Ждёт завершения Job
    pub fn wait_for_job(&self, name: &str, namespace: Option<&str>, timeout_secs: u64) -> Result<String> {
        use std::time::{Duration, Instant};
        
        info!("Waiting for Job '{}' to complete (timeout: {}s)", name, timeout_secs);
        
        let start = Instant::now();
        
        while start.elapsed() < Duration::from_secs(timeout_secs) {
            let status = self.get_job_status(name, namespace)?;
            
            match status.as_str() {
                "Complete" => {
                    info!("Job '{}' completed", name);
                    return Ok("Complete".to_string());
                }
                "Failed" => {
                    warn!("Job '{}' failed", name);
                    return Ok("Failed".to_string());
                }
                _ => {
                    debug!("Job '{}' status: {}", name, status);
                    std::thread::sleep(Duration::from_secs(5));
                }
            }
        }
        
        warn!("Timeout waiting for Job '{}' to complete", name);
        Ok("Timeout".to_string())
    }

    /// Выполняет произвольную kubectl команду
    pub fn run_command(&self, args: &[&str]) -> Result<String> {
        let output = self.kubectl()
            .args(args)
            .output()
            .map_err(|e| Error::Other(format!("Failed to run kubectl: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("kubectl failed: {}", stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Получает namespace по умолчанию
    pub fn default_namespace(&self) -> &str {
        &self.default_namespace
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
        assert!(config.kubeconfig_path.is_none());
    }

    #[test]
    fn test_kubernetes_client_creation() {
        let config = KubernetesConfig::default();
        let client = KubernetesClient::new(config);
        assert!(client.is_ok());
    }
}
