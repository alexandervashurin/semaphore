//! KubernetesClusterManager — менеджер подключений к кластерам
//!
//! Фаза 1: один кластер (дефолтный), конфигурируется через переменные окружения.
//! Фаза 10: полный мульти-кластер UI с переключателем.
//!
//! Переменные окружения (Фаза 1):
//!   VELUM_K8S_KUBECONFIG   — путь к kubeconfig (по умолчанию: ~/.kube/config)
//!   VELUM_K8S_CONTEXT      — конкретный контекст (по умолчанию: current-context)
//!   VELUM_K8S_IN_CLUSTER   — "true" для in-cluster режима (Service Account)
//!   VELUM_K8S_CLUSTER_NAME — отображаемое имя кластера (по умолчанию: "default")

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::kubernetes::service::{KubernetesClusterService, ConnectionMode};
use crate::error::{Error, Result};

/// Метаданные подключения к кластеру
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConnectionMeta {
    /// Уникальный идентификатор (slug)
    pub id: String,
    /// Отображаемое название
    pub name: String,
    /// Способ подключения (для UI)
    pub auth_method: String,
    /// Контекст kubeconfig (если применимо)
    pub context: Option<String>,
}

/// Менеджер подключений к кластерам Kubernetes
///
/// Thread-safe: `Arc<KubernetesClusterManager>` хранится в AppState.
pub struct KubernetesClusterManager {
    /// Кэш проинициализированных сервисов
    services: RwLock<HashMap<String, Arc<KubernetesClusterService>>>,
    /// Метаданные подключений
    connections: Vec<ClusterConnectionMeta>,
}

impl KubernetesClusterManager {
    /// Создаёт менеджер из переменных окружения (Фаза 1: один дефолтный кластер)
    pub async fn from_env() -> Option<Arc<Self>> {
        let kubeconfig = std::env::var("VELUM_K8S_KUBECONFIG").ok();
        let context = std::env::var("VELUM_K8S_CONTEXT").ok();
        let in_cluster = std::env::var("VELUM_K8S_IN_CLUSTER")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let cluster_name = std::env::var("VELUM_K8S_CLUSTER_NAME")
            .unwrap_or_else(|_| "default".to_string());

        // Если ни один из параметров не задан — проверим автоматически
        let mode = if in_cluster {
            ConnectionMode::InCluster
        } else if kubeconfig.is_some() || context.is_some() {
            ConnectionMode::KubeConfig {
                path: kubeconfig.clone(),
                context: context.clone(),
            }
        } else {
            // Попытка автоматического обнаружения (in-cluster или ~/.kube/config)
            ConnectionMode::Infer
        };

        let auth_method = if in_cluster {
            "in-cluster"
        } else if kubeconfig.is_some() {
            "kubeconfig-file"
        } else {
            "kubeconfig-default"
        };

        let service = match KubernetesClusterService::connect(mode).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Kubernetes cluster not configured or unreachable: {e}");
                return None;
            }
        };

        let mut services = HashMap::new();
        services.insert("default".to_string(), Arc::new(service));

        let connections = vec![ClusterConnectionMeta {
            id: "default".to_string(),
            name: cluster_name,
            auth_method: auth_method.to_string(),
            context,
        }];

        tracing::info!("Kubernetes cluster manager initialized ({} cluster(s))", connections.len());

        Some(Arc::new(Self {
            services: RwLock::new(services),
            connections,
        }))
    }

    /// Возвращает список доступных кластеров
    pub fn list_clusters(&self) -> &[ClusterConnectionMeta] {
        &self.connections
    }

    /// Возвращает сервис для кластера по id
    ///
    /// Возвращает 404-like ошибку если cluster_id неизвестен.
    /// Изоляция: не раскрывает информацию о других кластерах.
    pub async fn get(&self, cluster_id: &str) -> Result<Arc<KubernetesClusterService>> {
        let services = self.services.read().await;
        services.get(cluster_id).cloned().ok_or_else(|| {
            Error::NotFound(format!("Kubernetes cluster '{}' not found", cluster_id))
        })
    }

    /// Возвращает сервис дефолтного кластера
    pub async fn default_cluster(&self) -> Result<Arc<KubernetesClusterService>> {
        self.get("default").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_connection_meta_default_fields() {
        let meta = ClusterConnectionMeta {
            id: "cluster-1".to_string(),
            name: "My Cluster".to_string(),
            auth_method: "kubeconfig".to_string(),
            context: None,
        };
        assert_eq!(meta.id, "cluster-1");
        assert_eq!(meta.name, "My Cluster");
        assert_eq!(meta.auth_method, "kubeconfig");
        assert!(meta.context.is_none());
    }

    #[test]
    fn test_cluster_connection_meta_with_context() {
        let meta = ClusterConnectionMeta {
            id: "prod".to_string(),
            name: "Production".to_string(),
            auth_method: "kubeconfig-file".to_string(),
            context: Some("prod-context".to_string()),
        };
        assert_eq!(meta.context, Some("prod-context".to_string()));
    }

    #[test]
    fn test_cluster_connection_meta_clone() {
        let meta = ClusterConnectionMeta {
            id: "test".to_string(),
            name: "Test".to_string(),
            auth_method: "test".to_string(),
            context: Some("ctx".to_string()),
        };
        let cloned = meta.clone();
        assert_eq!(cloned.id, meta.id);
        assert_eq!(cloned.name, meta.name);
    }

    #[test]
    fn test_cluster_connection_meta_debug() {
        let meta = ClusterConnectionMeta {
            id: "debug-test".to_string(),
            name: "Debug".to_string(),
            auth_method: "kubeconfig".to_string(),
            context: None,
        };
        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("ClusterConnectionMeta"));
        assert!(debug_str.contains("debug-test"));
    }

    #[test]
    fn test_cluster_connection_meta_serialize() {
        let meta = ClusterConnectionMeta {
            id: "cluster-1".to_string(),
            name: "My Cluster".to_string(),
            auth_method: "in-cluster".to_string(),
            context: None,
        };
        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("cluster-1"));
        assert!(json.contains("My Cluster"));
        assert!(json.contains("in-cluster"));
    }

    #[test]
    fn test_cluster_connection_meta_serialize_with_context() {
        let meta = ClusterConnectionMeta {
            id: "dev".to_string(),
            name: "Dev".to_string(),
            auth_method: "kubeconfig".to_string(),
            context: Some("dev-ctx".to_string()),
        };
        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("dev-ctx"));
    }

    #[test]
    fn test_cluster_connection_meta_deserialize() {
        let json = r#"{
            "id": "cluster-1",
            "name": "My Cluster",
            "auth_method": "kubeconfig",
            "context": null
        }"#;
        let meta: ClusterConnectionMeta = serde_json::from_str(json).unwrap();
        assert_eq!(meta.id, "cluster-1");
        assert_eq!(meta.name, "My Cluster");
        assert!(meta.context.is_none());
    }

    #[test]
    fn test_cluster_connection_meta_deserialize_with_context() {
        let json = r#"{
            "id": "prod",
            "name": "Production",
            "auth_method": "kubeconfig-file",
            "context": "prod-context"
        }"#;
        let meta: ClusterConnectionMeta = serde_json::from_str(json).unwrap();
        assert_eq!(meta.id, "prod");
        assert_eq!(meta.context, Some("prod-context".to_string()));
    }

    #[test]
    fn test_cluster_connection_meta_roundtrip() {
        let original = ClusterConnectionMeta {
            id: "roundtrip".to_string(),
            name: "Roundtrip Cluster".to_string(),
            auth_method: "service-account".to_string(),
            context: Some("rt-ctx".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: ClusterConnectionMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, original.id);
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.context, original.context);
    }

    #[test]
    fn test_connection_mode_variants() {
        // Verify ConnectionMode enum variants exist and are distinct
        let kubeconfig = crate::kubernetes::service::ConnectionMode::KubeConfig {
            path: Some("/path/to/kubeconfig".to_string()),
            context: Some("my-context".to_string()),
        };
        let in_cluster = crate::kubernetes::service::ConnectionMode::InCluster;
        let infer = crate::kubernetes::service::ConnectionMode::Infer;

        match kubeconfig {
            crate::kubernetes::service::ConnectionMode::KubeConfig { path, context } => {
                assert_eq!(path, Some("/path/to/kubeconfig".to_string()));
                assert_eq!(context, Some("my-context".to_string()));
            }
            _ => panic!("Expected KubeConfig variant"),
        }

        match in_cluster {
            crate::kubernetes::service::ConnectionMode::InCluster => {}
            _ => panic!("Expected InCluster variant"),
        }

        match infer {
            crate::kubernetes::service::ConnectionMode::Infer => {}
            _ => panic!("Expected Infer variant"),
        }
    }
}
