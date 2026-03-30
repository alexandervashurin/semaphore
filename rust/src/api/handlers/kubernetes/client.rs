//! Kubernetes Client на основе kube-rs
//!
//! Этот модуль предоставляет клиент для подключения к Kubernetes API
//! с использованием библиотеки kube-rs

use kube::{
    api::{Api, ListParams, ResourceExt},
    client::Client,
    config::{KubeConfigOptions, Kubeconfig},
    Config, Resource,
};
use k8s_openapi::api::core::v1::Namespace;
use tracing::{info, warn, error, debug};
use std::sync::Arc;
use crate::error::{Error, Result};

use super::types::KubeResourceMeta;

/// Конфигурация подключения к Kubernetes
#[derive(Debug, Clone)]
pub struct KubeConfig {
    /// Путь к kubeconfig файлу
    pub kubeconfig_path: Option<String>,
    /// Контекст для подключения
    pub context: Option<String>,
    /// Namespace по умолчанию
    pub default_namespace: String,
    /// Таймаут запросов (секунды)
    pub timeout_secs: u64,
}

impl Default for KubeConfig {
    fn default() -> Self {
        Self {
            kubeconfig_path: None,
            context: None,
            default_namespace: "default".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Kubernetes клиент
/// 
/// Обёртка над kube::Client для упрощения работы с Kubernetes API
pub struct KubeClient {
    client: Client,
    default_namespace: String,
    config: KubeConfig,
}

impl KubeClient {
    /// Создаёт новый Kubernetes клиент
    ///
    /// # Пример
    /// ```ignore
    /// // Doctest requires a running Kubernetes cluster
    /// // Example usage:
    /// // let config = KubeConfig::default();
    /// // let client = KubeClient::new(config).await.unwrap();
    /// ```
    pub async fn new(config: KubeConfig) -> Result<Self> {
        info!("Creating Kubernetes client");

        let client_config = if let Some(kubeconfig_path) = &config.kubeconfig_path {
            // Загрузка из файла
            debug!("Loading kubeconfig from: {}", kubeconfig_path);
            let kubeconfig = Kubeconfig::read_from(kubeconfig_path)
                .map_err(|e| Error::Kubernetes(format!("Failed to read kubeconfig: {}", e)))?;

            let config_opts = KubeConfigOptions {
                context: config.context.clone(),
                ..Default::default()
            };

            Config::from_custom_kubeconfig(kubeconfig, &config_opts)
                .await
                .map_err(|e| Error::Kubernetes(format!("Failed to load kubeconfig: {}", e)))?
        } else {
            // In-cluster config или ~/.kube/config
            debug!("Using default kubeconfig inference");
            Config::infer()
                .await
                .map_err(|e| Error::Kubernetes(format!("Failed to infer config: {}", e)))?
        };

        let client = Client::try_from(client_config)
            .map_err(|e| Error::Kubernetes(format!("Failed to create client: {}", e)))?;

        info!("Kubernetes client created successfully");

        Ok(Self {
            client,
            default_namespace: config.default_namespace.clone(),
            config,
        })
    }

    /// Проверяет подключение к кластеру
    /// 
    /// # Возвращает
    /// * `Ok(true)` - подключение успешно
    /// * `Err(Error)` - ошибка подключения
    pub async fn check_connection(&self) -> Result<bool> {
        debug!("Checking Kubernetes connection");
        
        let api: Api<Namespace> = Api::all(self.client.clone());

        match api.list(&ListParams::default().limit(1)).await {
            Ok(_) => {
                info!("Successfully connected to Kubernetes cluster");
                Ok(true)
            }
            Err(e) => {
                error!("Failed to connect to Kubernetes: {}", e);
                Err(Error::Kubernetes(e.to_string()))
            }
        }
    }

    /// Получает API для работы с ресурсами в namespace
    /// 
    /// # Типы параметров
    /// * `T: Resource` - тип Kubernetes ресурса
    /// * `namespace` - опциональный namespace, если None используется default
    pub fn api<T: Resource<Scope = kube::core::NamespaceResourceScope>>(&self, namespace: Option<&str>) -> Api<T>
    where
        T::DynamicType: Default,
    {
        let ns = namespace.unwrap_or(&self.default_namespace);
        Api::namespaced(self.client.clone(), ns)
    }

    /// Получает API для cluster-scoped ресурсов
    /// 
    /// # Типы параметров
    /// * `T: Resource` - тип Kubernetes ресурса (должен быть cluster-scoped)
    pub fn api_all<T: Resource>(&self) -> Api<T>
    where
        T::DynamicType: Default,
    {
        Api::all(self.client.clone())
    }

    /// Namespace по умолчанию
    pub fn default_namespace(&self) -> &str {
        &self.default_namespace
    }

    /// Возвращает низкоуровневый kube::Client
    pub fn raw(&self) -> &Client {
        &self.client
    }

    /// Получает конфигурацию клиента
    pub fn config(&self) -> &KubeConfig {
        &self.config
    }
}

/// Сервис для управления Kubernetes кластером
/// 
/// Высокоуровневая обёртка над KubeClient для бизнес-логики
pub struct KubernetesClusterService {
    client: Arc<KubeClient>,
}

impl KubernetesClusterService {
    /// Создаёт новый сервис управления кластером
    pub fn new(client: Arc<KubeClient>) -> Self {
        Self { client }
    }

    /// Получает список всех namespace'ов
    pub async fn list_namespaces(&self) -> Result<Vec<serde_json::Value>> {
        let api: Api<Namespace> = self.client.api_all();
        
        let namespaces = api
            .list(&ListParams::default())
            .await
            .map_err(|e| Error::Kubernetes(e.to_string()))?;

        let result = namespaces
            .items
            .iter()
            .map(|ns| {
                let meta = &ns.metadata;
                let status = &ns.status;

                serde_json::json!({
                    "name": meta.name.clone().unwrap_or_else(|| "unknown".to_string()),
                    "uid": meta.uid.clone().unwrap_or_else(|| "unknown".to_string()),
                    "status": status.as_ref().and_then(|s| s.phase.clone()).unwrap_or_else(|| "Unknown".to_string()),
                    "created_at": meta.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339()).unwrap_or_else(|| "unknown".to_string()),
                    "labels": meta.labels.clone().unwrap_or_default(),
                    "annotations": meta.annotations.clone().unwrap_or_default(),
                })
            })
            .collect();

        Ok(result)
    }

    /// Получает детальную информацию о namespace
    pub async fn get_namespace(&self, name: &str) -> Result<serde_json::Value> {
        let api: Api<Namespace> = self.client.api_all();
        
        let ns = api
            .get(name)
            .await
            .map_err(|e| Error::Kubernetes(e.to_string()))?;

        Ok(serde_json::json!(ns))
    }

    /// Создаёт новый namespace
    pub async fn create_namespace(&self, name: &str, labels: Option<std::collections::BTreeMap<String, String>>) -> Result<serde_json::Value> {
        let api: Api<Namespace> = self.client.api_all();

        let ns = Namespace {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(name.to_string()),
                labels,
                ..Default::default()
            },
            ..Default::default()
        };

        let created = api
            .create(&kube::api::PostParams::default(), &ns)
            .await
            .map_err(|e| Error::Kubernetes(e.to_string()))?;

        Ok(serde_json::json!(created))
    }

    /// Удаляет namespace
    pub async fn delete_namespace(&self, name: &str) -> Result<()> {
        let api: Api<Namespace> = self.client.api_all();
        
        api.delete(name, &kube::api::DeleteParams::default())
            .await
            .map_err(|e| Error::Kubernetes(e.to_string()))?;

        Ok(())
    }

    /// Получает информацию о кластере
    pub async fn get_cluster_info(&self) -> Result<serde_json::Value> {
        let version = self.client
            .raw()
            .apiserver_version()
            .await
            .map_err(|e| Error::Kubernetes(e.to_string()))?;

        Ok(serde_json::json!({
            "kubernetes_version": version.git_version,
            "platform": version.platform,
            "git_version": version.git_version,
            "git_commit": version.git_commit,
            "build_date": version.build_date,
            "go_version": version.go_version,
            "compiler": version.compiler,
        }))
    }

    /// Получает список узлов кластера
    pub async fn list_nodes(&self) -> Result<Vec<serde_json::Value>> {
        use k8s_openapi::api::core::v1::Node;
        
        let api: Api<Node> = self.client.api_all();
        
        let nodes = api
            .list(&ListParams::default())
            .await
            .map_err(|e| Error::Kubernetes(e.to_string()))?;

        let mut summaries = Vec::new();

        for node in nodes.items {
            let status = node.status.as_ref();
            let meta = &node.metadata;

            // Определяем роли
            let mut roles = Vec::new();
            if let Some(labels) = &meta.labels {
                for (key, value) in labels {
                    if key.starts_with("node-role.kubernetes.io/") && value.is_empty() {
                        roles.push(key.split('/').next_back().unwrap_or("unknown").to_string());
                    }
                }
            }
            if roles.is_empty() {
                roles.push("<none>".to_string());
            }

            // Определяем статус
            let node_status = status
                .and_then(|s| s.conditions.as_ref())
                .as_ref()
                .and_then(|conditions| {
                    conditions
                        .iter()
                        .find(|c| c.type_ == "Ready")
                        .map(|c| if c.status == "True" { "Ready" } else { "NotReady" }.to_string())
                })
                .unwrap_or_else(|| "Unknown".to_string());

            summaries.push(serde_json::json!({
                "name": meta.name.clone().unwrap_or_else(|| "unknown".to_string()),
                "status": node_status,
                "roles": roles,
                "version": status.as_ref().and_then(|s| s.node_info.as_ref().map(|ni| ni.kubelet_version.clone())).unwrap_or_else(|| "unknown".to_string()),
                "internal_ip": status.and_then(|s| s.addresses.as_ref()).and_then(|addrs| {
                    addrs.iter().find(|a| a.type_ == "InternalIP").map(|a| a.address.clone())
                }).unwrap_or_else(|| "unknown".to_string()),
                "os_image": status.as_ref().and_then(|s| s.node_info.as_ref().map(|ni| ni.os_image.clone())).unwrap_or_else(|| "unknown".to_string()),
                "kernel_version": status.as_ref().and_then(|s| s.node_info.as_ref().map(|ni| ni.kernel_version.clone())).unwrap_or_else(|| "unknown".to_string()),
                "container_runtime": status
                    .and_then(|s| s.node_info.as_ref().map(|ni| ni.container_runtime_version.clone()))
                    .unwrap_or_else(|| "unknown".to_string()),
            }));
        }

        Ok(summaries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kube_config_default() {
        let config = KubeConfig::default();
        assert_eq!(config.default_namespace, "default");
        assert_eq!(config.timeout_secs, 30);
        assert!(config.kubeconfig_path.is_none());
        assert!(config.context.is_none());
    }

    #[tokio::test]
    #[ignore] // Требуется реальный кластер для тестирования
    async fn test_kube_client_creation() {
        let config = KubeConfig::default();
        let result = KubeClient::new(config).await;
        assert!(result.is_ok());
    }
}
