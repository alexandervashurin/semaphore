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
