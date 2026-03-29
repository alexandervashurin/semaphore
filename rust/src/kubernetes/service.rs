//! KubernetesClusterService — сервис для работы с Kubernetes API через kube-rs
//!
//! Это основной слой взаимодействия с apiserver.
//! Существующий [client.rs](client.rs) с kubectl subprocess остаётся для задач Job/Helm.

use kube::{Client, Config, config::KubeConfigOptions};
use k8s_openapi::api::core::v1::Namespace;
use kube::api::{Api, ListParams};
use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};

/// Информация о версии кластера Kubernetes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterVersionInfo {
    pub major: String,
    pub minor: String,
    pub git_version: String,
    pub platform: String,
}

/// Краткое состояние кластера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub reachable: bool,
    pub version: Option<ClusterVersionInfo>,
    /// Краткий human-readable статус: "ok" | "unreachable" | "unauthorized"
    pub status: String,
    pub message: Option<String>,
}

/// Namespace в ответе API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceInfo {
    pub name: String,
    /// "Active" | "Terminating"
    pub phase: String,
    pub labels: std::collections::BTreeMap<String, String>,
}

/// Список namespace'ов с поддержкой pagination (continue token)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceList {
    pub items: Vec<NamespaceInfo>,
    /// Токен для следующей страницы (если есть)
    pub continue_token: Option<String>,
}

/// Способ загрузки конфигурации подключения
#[derive(Debug, Clone)]
pub enum ConnectionMode {
    /// Из kubeconfig-файла, опциональный контекст
    KubeConfig { path: Option<String>, context: Option<String> },
    /// Из переменных окружения KUBERNETES_SERVICE_HOST (in-cluster / SA)
    InCluster,
    /// Автоматический выбор: сначала in-cluster, затем kubeconfig
    Infer,
}

/// Сервис для взаимодействия с Kubernetes apiserver
///
/// Создаётся один раз при старте и хранится в AppState (Arc).
/// Не хранит mutable state — безопасен для Clone + Send + Sync.
#[derive(Clone)]
pub struct KubernetesClusterService {
    client: Client,
}

impl KubernetesClusterService {
    /// Создаёт сервис по заданному режиму подключения
    pub async fn connect(mode: ConnectionMode) -> Result<Self> {
        let config = match mode {
            ConnectionMode::Infer => {
                Config::infer().await.map_err(|e| {
                    Error::Other(format!("Kubernetes config inference failed: {e}"))
                })?
            }
            ConnectionMode::InCluster => {
                Config::incluster().map_err(|e| {
                    Error::Other(format!("Kubernetes in-cluster config failed: {e}"))
                })?
            }
            ConnectionMode::KubeConfig { path, context } => {
                // Если путь задан — выставляем KUBECONFIG env временно
                let options = KubeConfigOptions {
                    context: context.clone(),
                    cluster: None,
                    user: None,
                };
                if let Some(ref p) = path {
                    // Загрузка из конкретного файла
                    let kube_config = kube::config::Kubeconfig::read_from(p).map_err(|e| {
                        Error::Other(format!("Failed to read kubeconfig from {p}: {e}"))
                    })?;
                    Config::from_custom_kubeconfig(kube_config, &options)
                        .await
                        .map_err(|e| {
                            Error::Other(format!("Failed to build kube Config: {e}"))
                        })?
                } else {
                    // Из KUBECONFIG env / ~/.kube/config
                    Config::from_kubeconfig(&options).await.map_err(|e| {
                        Error::Other(format!("Failed to load kubeconfig: {e}"))
                    })?
                }
            }
        };

        let client = Client::try_from(config)
            .map_err(|e| Error::Other(format!("Failed to create kube Client: {e}")))?;

        Ok(Self { client })
    }

    /// Проверяет доступность apiserver и возвращает версию кластера
    pub async fn cluster_info(&self) -> ClusterInfo {
        match self.client.apiserver_version().await {
            Ok(v) => ClusterInfo {
                reachable: true,
                status: "ok".to_string(),
                message: None,
                version: Some(ClusterVersionInfo {
                    major: v.major,
                    minor: v.minor,
                    git_version: v.git_version,
                    platform: v.platform,
                }),
            },
            Err(e) => {
                let msg = e.to_string();
                let status = if msg.contains("401") || msg.contains("403") || msg.contains("Unauthorized") {
                    "unauthorized"
                } else {
                    "unreachable"
                };
                ClusterInfo {
                    reachable: false,
                    status: status.to_string(),
                    message: Some(msg),
                    version: None,
                }
            }
        }
    }

    /// Возвращает список namespace'ов с пагинацией
    ///
    /// * `limit` — макс. кол-во элементов (по умолчанию 100)
    /// * `continue_token` — токен из предыдущей страницы
    pub async fn list_namespaces(
        &self,
        limit: Option<u32>,
        continue_token: Option<String>,
    ) -> Result<NamespaceList> {
        let api: Api<Namespace> = Api::all(self.client.clone());

        let limit = limit.unwrap_or(100).min(500);
        let mut lp = ListParams::default().limit(limit);
        if let Some(ref cont) = continue_token {
            lp = lp.continue_token(cont.as_str());
        }

        let ns_list = api.list(&lp).await.map_err(|e| {
            // Прокидываем 403 как ошибку авторизации
            let msg = e.to_string();
            if msg.contains("403") || msg.contains("Forbidden") {
                Error::Other(format!("FORBIDDEN: {msg}"))
            } else {
                Error::Other(format!("Failed to list namespaces: {msg}"))
            }
        })?;

        let cont = ns_list.metadata.continue_.filter(|s| !s.is_empty());

        let items = ns_list.items.into_iter().map(|ns| {
            let name = ns.metadata.name.unwrap_or_default();
            let phase = ns.status
                .and_then(|s| s.phase)
                .unwrap_or_else(|| "Unknown".to_string());
            let labels = ns.metadata.labels.unwrap_or_default();
            NamespaceInfo { name, phase, labels }
        }).collect();

        Ok(NamespaceList { items, continue_token: cont })
    }
}
