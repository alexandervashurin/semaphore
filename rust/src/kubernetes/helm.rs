//! Helm Client - Интеграция с Helm для управления chart'ами

use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{info, warn, error, debug};
use crate::error::{Error, Result};

/// Helm chart информация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmChart {
    /// Имя chart
    pub name: String,
    /// Версия chart
    pub version: Option<String>,
    /// Репозиторий chart
    pub repo: Option<String>,
    /// Путь к локальному chart
    pub path: Option<String>,
}

/// Helm release информация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRelease {
    /// Имя release
    pub name: String,
    /// Chart имя
    pub chart: String,
    /// Версия release
    pub revision: i32,
    /// Namespace
    pub namespace: String,
    /// Статус
    pub status: String,
}

/// Helm клиент
pub struct HelmClient {
    /// Путь к helm бинарнику
    helm_path: String,
    /// Namespace по умолчанию
    default_namespace: String,
    /// Kubeconfig путь
    kubeconfig: Option<String>,
}

impl HelmClient {
    /// Создаёт новый Helm клиент
    pub fn new() -> Self {
        Self {
            helm_path: "helm".to_string(),
            default_namespace: "default".to_string(),
            kubeconfig: None,
        }
    }

    /// Устанавливает путь к helm бинарнику
    pub fn with_helm_path(mut self, path: String) -> Self {
        self.helm_path = path;
        self
    }

    /// Устанавливает namespace по умолчанию
    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.default_namespace = namespace;
        self
    }

    /// Устанавливает kubeconfig
    pub fn with_kubeconfig(mut self, kubeconfig: String) -> Self {
        self.kubeconfig = Some(kubeconfig);
        self
    }

    /// Проверяет наличие helm
    pub fn check_helm(&self) -> Result<()> {
        let output = Command::new(&self.helm_path)
            .arg("version")
            .output()
            .map_err(|e| Error::Other(format!("Failed to run helm: {}", e)))?;
        
        if !output.status.success() {
            return Err(Error::Other("Helm not found or not working".to_string()));
        }
        
        let version = String::from_utf8_lossy(&output.stdout);
        info!("Helm version: {}", version.trim());
        
        Ok(())
    }

    /// Добавляет Helm репозиторий
    pub fn add_repo(&self, name: &str, url: &str) -> Result<()> {
        info!("Adding Helm repository '{}' ({})", name, url);
        
        let mut cmd = Command::new(&self.helm_path);
        cmd.arg("repo").arg("add").arg(name).arg(url);
        
        if let Some(kubeconfig) = &self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        let output = cmd.output()
            .map_err(|e| Error::Other(format!("Failed to add repo: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to add repo: {}", stderr)));
        }
        
        // Обновляем репозитории
        self.update_repo()?;
        
        Ok(())
    }

    /// Обновляет Helm репозитории
    pub fn update_repo(&self) -> Result<()> {
        debug!("Updating Helm repositories");
        
        let mut cmd = Command::new(&self.helm_path);
        cmd.arg("repo").arg("update");
        
        if let Some(kubeconfig) = &self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        let output = cmd.output()
            .map_err(|e| Error::Other(format!("Failed to update repos: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to update repos: {}", stderr)));
        }
        
        Ok(())
    }

    /// Устанавливает Helm chart
    pub fn install(
        &self,
        release_name: &str,
        chart: &HelmChart,
        namespace: Option<&str>,
        values: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<HelmRelease> {
        let ns = namespace.unwrap_or(&self.default_namespace);
        
        info!("Installing Helm chart '{}' as release '{}' in namespace '{}'", 
              chart.name, release_name, ns);
        
        let mut cmd = Command::new(&self.helm_path);
        cmd.arg("install")
            .arg(release_name)
            .arg("-n")
            .arg(ns);
        
        // Определяем источник chart
        let chart_ref = if let Some(path) = &chart.path {
            path.clone()
        } else if let Some(repo) = &chart.repo {
            format!("{}/{}", repo, chart.name)
        } else {
            chart.name.clone()
        };
        
        cmd.arg(&chart_ref);
        
        // Добавляем version если указан
        if let Some(version) = &chart.version {
            cmd.arg("--version").arg(version);
        }
        
        // Добавляем values
        if let Some(values_map) = values {
            for (key, value) in values_map {
                cmd.arg("--set").arg(format!("{}={}", key, value));
            }
        }
        
        if let Some(kubeconfig) = &self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        let output = cmd.output()
            .map_err(|e| Error::Other(format!("Failed to install chart: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to install chart: {}", stderr)));
        }
        
        Ok(HelmRelease {
            name: release_name.to_string(),
            chart: chart.name.clone(),
            revision: 1,
            namespace: ns.to_string(),
            status: "deployed".to_string(),
        })
    }

    /// Обновляет Helm release
    pub fn upgrade(
        &self,
        release_name: &str,
        chart: &HelmChart,
        namespace: Option<&str>,
        values: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<HelmRelease> {
        let ns = namespace.unwrap_or(&self.default_namespace);
        
        info!("Upgrading Helm release '{}' in namespace '{}'", release_name, ns);
        
        let mut cmd = Command::new(&self.helm_path);
        cmd.arg("upgrade")
            .arg(release_name)
            .arg("-n")
            .arg(ns);
        
        // Определяем источник chart
        let chart_ref = if let Some(path) = &chart.path {
            path.clone()
        } else if let Some(repo) = &chart.repo {
            format!("{}/{}", repo, chart.name)
        } else {
            chart.name.clone()
        };
        
        cmd.arg(&chart_ref);
        
        // Добавляем version если указан
        if let Some(version) = &chart.version {
            cmd.arg("--version").arg(version);
        }
        
        // Добавляем values
        if let Some(values_map) = values {
            for (key, value) in values_map {
                cmd.arg("--set").arg(format!("{}={}", key, value));
            }
        }
        
        if let Some(kubeconfig) = &self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        let output = cmd.output()
            .map_err(|e| Error::Other(format!("Failed to upgrade release: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to upgrade release: {}", stderr)));
        }
        
        Ok(HelmRelease {
            name: release_name.to_string(),
            chart: chart.name.clone(),
            revision: 2,
            namespace: ns.to_string(),
            status: "deployed".to_string(),
        })
    }

    /// Получает список release'ов
    pub fn list_releases(&self, namespace: Option<&str>) -> Result<Vec<HelmRelease>> {
        let ns = namespace.unwrap_or(&self.default_namespace);
        
        let mut cmd = Command::new(&self.helm_path);
        cmd.arg("list")
            .arg("-n")
            .arg(ns)
            .arg("-o")
            .arg("json");
        
        if let Some(kubeconfig) = &self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        let output = cmd.output()
            .map_err(|e| Error::Other(format!("Failed to list releases: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to list releases: {}", stderr)));
        }
        
        let releases: Vec<HelmRelease> = serde_json::from_slice(&output.stdout)
            .map_err(|e| Error::Other(format!("Failed to parse releases: {}", e)))?;
        
        Ok(releases)
    }

    /// Получает статус release
    pub fn get_release_status(&self, release_name: &str, namespace: Option<&str>) -> Result<String> {
        let ns = namespace.unwrap_or(&self.default_namespace);
        
        let mut cmd = Command::new(&self.helm_path);
        cmd.arg("status")
            .arg(release_name)
            .arg("-n")
            .arg(ns)
            .arg("-o")
            .arg("json");
        
        if let Some(kubeconfig) = &self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        let output = cmd.output()
            .map_err(|e| Error::Other(format!("Failed to get release status: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to get release status: {}", stderr)));
        }
        
        // Парсим JSON и получаем статус
        let status: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| Error::Other(format!("Failed to parse status: {}", e)))?;
        
        let status_str = status["info"]["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        
        Ok(status_str)
    }

    /// Удаляет Helm release
    pub fn uninstall(&self, release_name: &str, namespace: Option<&str>) -> Result<()> {
        let ns = namespace.unwrap_or(&self.default_namespace);
        
        info!("Uninstalling Helm release '{}' from namespace '{}'", release_name, ns);
        
        let mut cmd = Command::new(&self.helm_path);
        cmd.arg("uninstall")
            .arg(release_name)
            .arg("-n")
            .arg(ns);
        
        if let Some(kubeconfig) = &self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        let output = cmd.output()
            .map_err(|e| Error::Other(format!("Failed to uninstall release: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to uninstall release: {}", stderr)));
        }
        
        Ok(())
    }

    /// Получает историю release
    pub fn get_history(&self, release_name: &str, namespace: Option<&str>) -> Result<Vec<HelmRelease>> {
        let ns = namespace.unwrap_or(&self.default_namespace);
        
        let mut cmd = Command::new(&self.helm_path);
        cmd.arg("history")
            .arg(release_name)
            .arg("-n")
            .arg(ns)
            .arg("-o")
            .arg("json");
        
        if let Some(kubeconfig) = &self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        let output = cmd.output()
            .map_err(|e| Error::Other(format!("Failed to get history: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to get history: {}", stderr)));
        }
        
        let history: Vec<HelmRelease> = serde_json::from_slice(&output.stdout)
            .map_err(|e| Error::Other(format!("Failed to parse history: {}", e)))?;
        
        Ok(history)
    }

    /// Rollback release к revision
    pub fn rollback(&self, release_name: &str, revision: i32, namespace: Option<&str>) -> Result<()> {
        let ns = namespace.unwrap_or(&self.default_namespace);
        
        info!("Rolling back release '{}' to revision {} in namespace '{}'", 
              release_name, revision, ns);
        
        let mut cmd = Command::new(&self.helm_path);
        cmd.arg("rollback")
            .arg(release_name)
            .arg(revision.to_string())
            .arg("-n")
            .arg(ns);
        
        if let Some(kubeconfig) = &self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        let output = cmd.output()
            .map_err(|e| Error::Other(format!("Failed to rollback release: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Failed to rollback release: {}", stderr)));
        }
        
        Ok(())
    }
}

impl Default for HelmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helm_client_creation() {
        let client = HelmClient::new();
        assert_eq!(client.helm_path, "helm");
        assert_eq!(client.default_namespace, "default");
        assert!(client.kubeconfig.is_none());
    }

    #[test]
    fn test_helm_client_builder() {
        let client = HelmClient::new()
            .with_helm_path("/usr/local/bin/helm".to_string())
            .with_namespace("production".to_string())
            .with_kubeconfig("/home/user/.kube/config".to_string());
        
        assert_eq!(client.helm_path, "/usr/local/bin/helm");
        assert_eq!(client.default_namespace, "production");
        assert!(client.kubeconfig.is_some());
    }

    #[test]
    fn test_helm_chart() {
        let chart = HelmChart {
            name: "nginx".to_string(),
            version: Some("1.0.0".to_string()),
            repo: Some("bitnami".to_string()),
            path: None,
        };
        
        assert_eq!(chart.name, "nginx");
        assert_eq!(chart.version, Some("1.0.0".to_string()));
        assert_eq!(chart.repo, Some("bitnami".to_string()));
    }
}
