//! KubernetesClusterService — сервис для работы с Kubernetes API через kube-rs
//!
//! Это основной слой взаимодействия с apiserver.
//! Существующий [client.rs](client.rs) с kubectl subprocess остаётся для задач Job/Helm.

use kube::{Client, Config, config::KubeConfigOptions};
use k8s_openapi::api::core::v1::{Namespace, Pod};
use kube::api::{Api, ListParams, LogParams, DeleteParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

    // ─── Pods ────────────────────────────────────────────────────────────────

    /// Список Pod'ов в namespace с пагинацией и опциональным фильтром по labelSelector
    pub async fn list_pods(
        &self,
        namespace: &str,
        limit: Option<u32>,
        continue_token: Option<String>,
        label_selector: Option<String>,
        field_selector: Option<String>,
    ) -> Result<PodList> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let limit = limit.unwrap_or(100).min(500);
        let mut lp = ListParams::default().limit(limit);
        if let Some(ref cont) = continue_token {
            lp = lp.continue_token(cont.as_str());
        }
        if let Some(ref ls) = label_selector {
            lp = lp.labels(ls.as_str());
        }
        if let Some(ref fs) = field_selector {
            lp = lp.fields(fs.as_str());
        }

        let pod_list = api.list(&lp).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("403") || msg.contains("Forbidden") {
                Error::Other(format!("FORBIDDEN: {msg}"))
            } else {
                Error::Other(format!("Failed to list pods: {msg}"))
            }
        })?;

        let cont = pod_list.metadata.continue_.filter(|s| !s.is_empty());

        let items = pod_list.items.into_iter().map(pod_to_info).collect();
        Ok(PodList { items, continue_token: cont })
    }

    /// Получить детальную информацию о Pod'е
    pub async fn get_pod(&self, namespace: &str, name: &str) -> Result<PodInfo> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let pod = api.get(name).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("404") || msg.contains("not found") {
                Error::NotFound(format!("Pod {name} not found in namespace {namespace}"))
            } else {
                Error::Other(format!("Failed to get pod {name}: {msg}"))
            }
        })?;
        Ok(pod_to_info(pod))
    }

    /// Удалить Pod
    pub async fn delete_pod(
        &self,
        namespace: &str,
        name: &str,
        grace_period_seconds: Option<i64>,
    ) -> Result<()> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let mut dp = DeleteParams::default();
        if let Some(grace) = grace_period_seconds {
            dp.grace_period_seconds = Some(grace as u32);
        }
        api.delete(name, &dp).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("403") || msg.contains("Forbidden") {
                Error::Other(format!("FORBIDDEN: {msg}"))
            } else if msg.contains("404") || msg.contains("not found") {
                Error::NotFound(format!("Pod {name} not found"))
            } else {
                Error::Other(format!("Failed to delete pod {name}: {msg}"))
            }
        })?;
        Ok(())
    }

    /// Логи Pod'а (статический snapshot, не streaming)
    pub async fn pod_logs(
        &self,
        namespace: &str,
        name: &str,
        container: Option<String>,
        tail_lines: Option<i64>,
        since_seconds: Option<i64>,
        previous: bool,
    ) -> Result<String> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let lp = LogParams {
            container,
            tail_lines,
            since_seconds,
            previous,
            ..Default::default()
        };
        api.logs(name, &lp).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("403") || msg.contains("Forbidden") {
                Error::Other(format!("FORBIDDEN: {msg}"))
            } else if msg.contains("404") || msg.contains("not found") {
                Error::NotFound(format!("Pod {name} not found"))
            } else {
                Error::Other(format!("Failed to get pod logs: {msg}"))
            }
        })
    }
}

// ─── Pod DTO helpers ─────────────────────────────────────────────────────────

/// Краткая информация о Pod'е для списка
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    /// Причина (CrashLoopBackOff, OOMKilled, …)
    pub reason: Option<String>,
    pub node_name: Option<String>,
    pub pod_ip: Option<String>,
    pub host_ip: Option<String>,
    /// Список контейнеров с ready/restart_count
    pub containers: Vec<ContainerStatus>,
    pub labels: BTreeMap<String, String>,
    pub created_at: Option<String>,
    pub ready_count: u32,
    pub total_count: u32,
}

/// Статус контейнера в Pod'е
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatus {
    pub name: String,
    pub image: String,
    pub ready: bool,
    pub restart_count: i32,
    pub state: String,
    pub reason: Option<String>,
}

/// Список Pod'ов с пагинацией
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodList {
    pub items: Vec<PodInfo>,
    pub continue_token: Option<String>,
}

fn pod_to_info(pod: Pod) -> PodInfo {
    let meta = &pod.metadata;
    let name = meta.name.clone().unwrap_or_default();
    let namespace = meta.namespace.clone().unwrap_or_default();
    let labels = meta.labels.clone().unwrap_or_default();
    let created_at = meta.creation_timestamp.as_ref()
        .map(|t| t.0.to_rfc3339());

    let spec = pod.spec.as_ref();
    let node_name = spec.and_then(|s| s.node_name.clone());

    let status = pod.status.as_ref();
    let phase = status
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let reason = status.and_then(|s| s.reason.clone());
    let pod_ip = status.and_then(|s| s.pod_ip.clone());
    let host_ip = status.and_then(|s| s.host_ip.clone());

    // Container statuses
    let cs_list = status.and_then(|s| s.container_statuses.clone()).unwrap_or_default();

    // Spec containers for image info fallback
    let spec_containers: Vec<_> = spec
        .map(|s| s.containers.clone())
        .unwrap_or_default();

    let containers: Vec<ContainerStatus> = spec_containers.iter().map(|c| {
        let cs = cs_list.iter().find(|cs| cs.name == c.name);
        let (ready, restart_count, state_str, state_reason) = cs.map(|cs| {
            let ready = cs.ready;
            let rc = cs.restart_count;
            let (st, r) = if let Some(ref s) = cs.state {
                if s.running.is_some() {
                    ("running".to_string(), None)
                } else if let Some(ref w) = s.waiting {
                    ("waiting".to_string(), w.reason.clone())
                } else if let Some(ref t) = s.terminated {
                    ("terminated".to_string(), t.reason.clone())
                } else {
                    ("unknown".to_string(), None)
                }
            } else {
                ("unknown".to_string(), None)
            };
            (ready, rc, st, r)
        }).unwrap_or((false, 0, "unknown".to_string(), None));

        ContainerStatus {
            name: c.name.clone(),
            image: c.image.clone().unwrap_or_default(),
            ready,
            restart_count,
            state: state_str,
            reason: state_reason,
        }
    }).collect();

    let ready_count = containers.iter().filter(|c| c.ready).count() as u32;
    let total_count = containers.len() as u32;

    PodInfo {
        name,
        namespace,
        phase,
        reason,
        node_name,
        pod_ip,
        host_ip,
        containers,
        labels,
        created_at,
        ready_count,
        total_count,
    }
}
