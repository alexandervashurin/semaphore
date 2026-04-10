//! KubernetesClusterService — сервис для работы с Kubernetes API через kube-rs
//!
//! Это основной слой взаимодействия с apiserver.
//! Существующий [client.rs](client.rs) с kubectl subprocess остаётся для задач Job/Helm.

use kube::{Client, Config, config::KubeConfigOptions};
use k8s_openapi::api::core::v1::{Namespace, Pod, Event, Service, ConfigMap, Secret};
use k8s_openapi::api::apps::v1::{Deployment, DaemonSet, StatefulSet, ReplicaSet};
use k8s_openapi::api::networking::v1::Ingress;
use kube::api::{Api, ListParams, LogParams, DeleteParams, Patch, PatchParams, PostParams};
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

    // ─── Deployments ─────────────────────────────────────────────────────────

    /// Список Deployment'ов в namespace
    pub async fn list_deployments(
        &self,
        namespace: &str,
        limit: Option<u32>,
        continue_token: Option<String>,
    ) -> Result<DeploymentList> {
        let api: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        let limit = limit.unwrap_or(100).min(500);
        let mut lp = ListParams::default().limit(limit);
        if let Some(ref cont) = continue_token {
            lp = lp.continue_token(cont.as_str());
        }
        let list = api.list(&lp).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("403") || msg.contains("Forbidden") {
                Error::Other(format!("FORBIDDEN: {msg}"))
            } else {
                Error::Other(format!("Failed to list deployments: {msg}"))
            }
        })?;
        let cont = list.metadata.continue_.filter(|s| !s.is_empty());
        let items = list.items.into_iter().map(deployment_to_info).collect();
        Ok(DeploymentList { items, continue_token: cont })
    }

    /// Детальная информация о Deployment'е
    pub async fn get_deployment(&self, namespace: &str, name: &str) -> Result<DeploymentInfo> {
        let api: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        let d = api.get(name).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("404") || msg.contains("not found") {
                Error::NotFound(format!("Deployment {name} not found"))
            } else {
                Error::Other(format!("Failed to get deployment: {msg}"))
            }
        })?;
        Ok(deployment_to_info(d))
    }

    /// Масштабировать Deployment (patch replicas)
    pub async fn scale_deployment(
        &self,
        namespace: &str,
        name: &str,
        replicas: i32,
    ) -> Result<()> {
        let api: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        let patch = serde_json::json!({
            "spec": { "replicas": replicas }
        });
        api.patch(name, &PatchParams::apply("velum").force(), &Patch::Merge(&patch))
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("403") || msg.contains("Forbidden") {
                    Error::Other(format!("FORBIDDEN: {msg}"))
                } else {
                    Error::Other(format!("Failed to scale deployment: {msg}"))
                }
            })?;
        Ok(())
    }

    /// Restart Deployment (patch annotation)
    pub async fn restart_deployment(&self, namespace: &str, name: &str) -> Result<()> {
        let api: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        let now = chrono::Utc::now().to_rfc3339();
        let patch = serde_json::json!({
            "spec": {
                "template": {
                    "metadata": {
                        "annotations": {
                            "kubectl.kubernetes.io/restartedAt": now
                        }
                    }
                }
            }
        });
        api.patch(name, &PatchParams::apply("velum").force(), &Patch::Merge(&patch))
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("403") || msg.contains("Forbidden") {
                    Error::Other(format!("FORBIDDEN: {msg}"))
                } else {
                    Error::Other(format!("Failed to restart deployment: {msg}"))
                }
            })?;
        Ok(())
    }

    // ─── DaemonSets ───────────────────────────────────────────────────────────

    pub async fn list_daemonsets(
        &self,
        namespace: &str,
        limit: Option<u32>,
        continue_token: Option<String>,
    ) -> Result<DaemonSetList> {
        let api: Api<DaemonSet> = Api::namespaced(self.client.clone(), namespace);
        let limit = limit.unwrap_or(100).min(500);
        let mut lp = ListParams::default().limit(limit);
        if let Some(ref cont) = continue_token {
            lp = lp.continue_token(cont.as_str());
        }
        let list = api.list(&lp).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("403") || msg.contains("Forbidden") {
                Error::Other(format!("FORBIDDEN: {msg}"))
            } else {
                Error::Other(format!("Failed to list daemonsets: {msg}"))
            }
        })?;
        let cont = list.metadata.continue_.filter(|s| !s.is_empty());
        let items = list.items.into_iter().map(daemonset_to_info).collect();
        Ok(DaemonSetList { items, continue_token: cont })
    }

    pub async fn get_daemonset(&self, namespace: &str, name: &str) -> Result<DaemonSetInfo> {
        let api: Api<DaemonSet> = Api::namespaced(self.client.clone(), namespace);
        let d = api.get(name).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("404") || msg.contains("not found") {
                Error::NotFound(format!("DaemonSet {name} not found"))
            } else {
                Error::Other(format!("Failed to get daemonset: {msg}"))
            }
        })?;
        Ok(daemonset_to_info(d))
    }

    pub async fn restart_daemonset(&self, namespace: &str, name: &str) -> Result<()> {
        let api: Api<DaemonSet> = Api::namespaced(self.client.clone(), namespace);
        let now = chrono::Utc::now().to_rfc3339();
        let patch = serde_json::json!({
            "spec": { "template": { "metadata": { "annotations": {
                "kubectl.kubernetes.io/restartedAt": now
            }}}}
        });
        api.patch(name, &PatchParams::apply("velum").force(), &Patch::Merge(&patch))
            .await.map_err(|e| {
                let msg = e.to_string();
                if msg.contains("403") || msg.contains("Forbidden") {
                    Error::Other(format!("FORBIDDEN: {msg}"))
                } else {
                    Error::Other(format!("Failed to restart daemonset: {msg}"))
                }
            })?;
        Ok(())
    }

    // ─── StatefulSets ─────────────────────────────────────────────────────────

    pub async fn list_statefulsets(
        &self,
        namespace: &str,
        limit: Option<u32>,
        continue_token: Option<String>,
    ) -> Result<StatefulSetList> {
        let api: Api<StatefulSet> = Api::namespaced(self.client.clone(), namespace);
        let limit = limit.unwrap_or(100).min(500);
        let mut lp = ListParams::default().limit(limit);
        if let Some(ref cont) = continue_token {
            lp = lp.continue_token(cont.as_str());
        }
        let list = api.list(&lp).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("403") || msg.contains("Forbidden") {
                Error::Other(format!("FORBIDDEN: {msg}"))
            } else {
                Error::Other(format!("Failed to list statefulsets: {msg}"))
            }
        })?;
        let cont = list.metadata.continue_.filter(|s| !s.is_empty());
        let items = list.items.into_iter().map(statefulset_to_info).collect();
        Ok(StatefulSetList { items, continue_token: cont })
    }

    pub async fn get_statefulset(&self, namespace: &str, name: &str) -> Result<StatefulSetInfo> {
        let api: Api<StatefulSet> = Api::namespaced(self.client.clone(), namespace);
        let s = api.get(name).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("404") || msg.contains("not found") {
                Error::NotFound(format!("StatefulSet {name} not found"))
            } else {
                Error::Other(format!("Failed to get statefulset: {msg}"))
            }
        })?;
        Ok(statefulset_to_info(s))
    }

    pub async fn scale_statefulset(&self, namespace: &str, name: &str, replicas: i32) -> Result<()> {
        let api: Api<StatefulSet> = Api::namespaced(self.client.clone(), namespace);
        let patch = serde_json::json!({ "spec": { "replicas": replicas } });
        api.patch(name, &PatchParams::apply("velum").force(), &Patch::Merge(&patch))
            .await.map_err(|e| {
                let msg = e.to_string();
                if msg.contains("403") || msg.contains("Forbidden") {
                    Error::Other(format!("FORBIDDEN: {msg}"))
                } else {
                    Error::Other(format!("Failed to scale statefulset: {msg}"))
                }
            })?;
        Ok(())
    }

    // ─── ReplicaSets ──────────────────────────────────────────────────────────

    pub async fn list_replicasets(
        &self,
        namespace: &str,
        limit: Option<u32>,
        continue_token: Option<String>,
    ) -> Result<ReplicaSetList> {
        let api: Api<ReplicaSet> = Api::namespaced(self.client.clone(), namespace);
        let limit = limit.unwrap_or(100).min(500);
        let mut lp = ListParams::default().limit(limit);
        if let Some(ref cont) = continue_token {
            lp = lp.continue_token(cont.as_str());
        }
        let list = api.list(&lp).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("403") || msg.contains("Forbidden") {
                Error::Other(format!("FORBIDDEN: {msg}"))
            } else {
                Error::Other(format!("Failed to list replicasets: {msg}"))
            }
        })?;
        let cont = list.metadata.continue_.filter(|s| !s.is_empty());
        let items = list.items.into_iter().map(replicaset_to_info).collect();
        Ok(ReplicaSetList { items, continue_token: cont })
    }

    // ─── Events ───────────────────────────────────────────────────────────────

    /// Список событий в namespace с опциональным фильтром по involvedObject
    pub async fn list_events(
        &self,
        namespace: &str,
        involved_object_name: Option<String>,
        involved_object_kind: Option<String>,
        event_type: Option<String>,   // Normal | Warning
        limit: Option<u32>,
    ) -> Result<EventList> {
        let api: Api<Event> = Api::namespaced(self.client.clone(), namespace);
        let limit = limit.unwrap_or(100).min(500);
        let mut field_selectors: Vec<String> = vec![];
        if let Some(ref n) = involved_object_name {
            field_selectors.push(format!("involvedObject.name={n}"));
        }
        if let Some(ref k) = involved_object_kind {
            field_selectors.push(format!("involvedObject.kind={k}"));
        }
        if let Some(ref t) = event_type {
            field_selectors.push(format!("type={t}"));
        }
        let mut lp = ListParams::default().limit(limit);
        if !field_selectors.is_empty() {
            lp = lp.fields(&field_selectors.join(","));
        }
        let list = api.list(&lp).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("403") || msg.contains("Forbidden") {
                Error::Other(format!("FORBIDDEN: {msg}"))
            } else {
                Error::Other(format!("Failed to list events: {msg}"))
            }
        })?;
        let items = list.items.into_iter().map(|e| {
            let meta = &e.metadata;
            EventInfo {
                name: meta.name.clone().unwrap_or_default(),
                namespace: meta.namespace.clone().unwrap_or_default(),
                type_: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
                reason: e.reason.clone().unwrap_or_default(),
                message: e.message.clone().unwrap_or_default(),
                involved_object_name: e.involved_object.name.clone().unwrap_or_default(),
                involved_object_kind: e.involved_object.kind.clone().unwrap_or_default(),
                count: e.count.unwrap_or(1),
                first_time: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
                last_time: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            }
        }).collect();
        Ok(EventList { items })
    }

    // ─── Services ─────────────────────────────────────────────────────────────

    pub async fn list_services(&self, namespace: &str, limit: Option<u32>, continue_token: Option<String>) -> Result<ServiceList> {
        let api: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        let limit = limit.unwrap_or(100).min(500);
        let mut lp = ListParams::default().limit(limit);
        if let Some(ref c) = continue_token { lp = lp.continue_token(c.as_str()); }
        let list = api.list(&lp).await.map_err(|e| map_err("list services", e))?;
        let cont = list.metadata.continue_.filter(|s| !s.is_empty());
        Ok(ServiceList { items: list.items.into_iter().map(service_to_info).collect(), continue_token: cont })
    }

    pub async fn get_service(&self, namespace: &str, name: &str) -> Result<ServiceInfo> {
        let api: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        api.get(name).await.map(service_to_info).map_err(|e| map_err_named("Service", name, e))
    }

    pub async fn delete_service(&self, namespace: &str, name: &str) -> Result<()> {
        let api: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::default()).await.map_err(|e| map_err_named("Service", name, e))?;
        Ok(())
    }

    // ─── Ingress ──────────────────────────────────────────────────────────────

    pub async fn list_ingresses(&self, namespace: &str, limit: Option<u32>, continue_token: Option<String>) -> Result<IngressList> {
        let api: Api<Ingress> = Api::namespaced(self.client.clone(), namespace);
        let limit = limit.unwrap_or(100).min(500);
        let mut lp = ListParams::default().limit(limit);
        if let Some(ref c) = continue_token { lp = lp.continue_token(c.as_str()); }
        let list = api.list(&lp).await.map_err(|e| map_err("list ingresses", e))?;
        let cont = list.metadata.continue_.filter(|s| !s.is_empty());
        Ok(IngressList { items: list.items.into_iter().map(ingress_to_info).collect(), continue_token: cont })
    }

    pub async fn get_ingress(&self, namespace: &str, name: &str) -> Result<IngressInfo> {
        let api: Api<Ingress> = Api::namespaced(self.client.clone(), namespace);
        api.get(name).await.map(ingress_to_info).map_err(|e| map_err_named("Ingress", name, e))
    }

    pub async fn delete_ingress(&self, namespace: &str, name: &str) -> Result<()> {
        let api: Api<Ingress> = Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::default()).await.map_err(|e| map_err_named("Ingress", name, e))?;
        Ok(())
    }

    // ─── ConfigMaps ───────────────────────────────────────────────────────────

    pub async fn list_configmaps(&self, namespace: &str, limit: Option<u32>, continue_token: Option<String>) -> Result<ConfigMapList> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), namespace);
        let limit = limit.unwrap_or(100).min(500);
        let mut lp = ListParams::default().limit(limit);
        if let Some(ref c) = continue_token { lp = lp.continue_token(c.as_str()); }
        let list = api.list(&lp).await.map_err(|e| map_err("list configmaps", e))?;
        let cont = list.metadata.continue_.filter(|s| !s.is_empty());
        Ok(ConfigMapList { items: list.items.into_iter().map(configmap_to_info).collect(), continue_token: cont })
    }

    pub async fn get_configmap(&self, namespace: &str, name: &str) -> Result<ConfigMapInfo> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), namespace);
        api.get(name).await.map(configmap_to_info).map_err(|e| map_err_named("ConfigMap", name, e))
    }

    pub async fn delete_configmap(&self, namespace: &str, name: &str) -> Result<()> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::default()).await.map_err(|e| map_err_named("ConfigMap", name, e))?;
        Ok(())
    }

    pub async fn create_configmap(&self, namespace: &str, cm: ConfigMap) -> Result<ConfigMapInfo> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), namespace);
        api.create(&PostParams::default(), &cm).await.map(configmap_to_info).map_err(|e| map_err("create configmap", e))
    }

    pub async fn update_configmap(&self, namespace: &str, name: &str, data: BTreeMap<String, String>) -> Result<ConfigMapInfo> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), namespace);
        let patch = serde_json::json!({ "data": data });
        api.patch(name, &PatchParams::apply("velum").force(), &Patch::Merge(&patch))
            .await.map(configmap_to_info).map_err(|e| map_err("update configmap", e))
    }

    // ─── Secrets ──────────────────────────────────────────────────────────────

    pub async fn list_secrets(&self, namespace: &str, limit: Option<u32>, continue_token: Option<String>) -> Result<SecretList> {
        let api: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        let limit = limit.unwrap_or(100).min(500);
        let mut lp = ListParams::default().limit(limit);
        if let Some(ref c) = continue_token { lp = lp.continue_token(c.as_str()); }
        let list = api.list(&lp).await.map_err(|e| map_err("list secrets", e))?;
        let cont = list.metadata.continue_.filter(|s| !s.is_empty());
        Ok(SecretList { items: list.items.into_iter().map(secret_to_info).collect(), continue_token: cont })
    }

    pub async fn get_secret(&self, namespace: &str, name: &str) -> Result<SecretInfo> {
        let api: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        api.get(name).await.map(secret_to_info).map_err(|e| map_err_named("Secret", name, e))
    }

    pub async fn delete_secret(&self, namespace: &str, name: &str) -> Result<()> {
        let api: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::default()).await.map_err(|e| map_err_named("Secret", name, e))?;
        Ok(())
    }

    /// Получить Secret; при reveal=true декодирует base64-значения
    pub async fn get_secret_raw(&self, namespace: &str, name: &str, reveal: bool) -> Result<SecretInfo> {
        let api: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        let s = api.get(name).await.map_err(|e| map_err_named("Secret", name, e))?;
        if reveal {
            Ok(secret_to_info_revealed(s))
        } else {
            Ok(secret_to_info(s))
        }
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

// ─── Deployment DTO helpers ───────────────────────────────────────────────────

/// Информация о Deployment'е
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub name: String,
    pub namespace: String,
    pub replicas_desired: i32,
    pub replicas_ready: i32,
    pub replicas_available: i32,
    pub replicas_updated: i32,
    pub labels: BTreeMap<String, String>,
    pub selector: BTreeMap<String, String>,
    pub images: Vec<String>,
    pub created_at: Option<String>,
    pub conditions: Vec<DeploymentCondition>,
}

/// Условие состояния Deployment'а
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentCondition {
    pub type_: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
}

/// Список Deployment'ов с пагинацией
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentList {
    pub items: Vec<DeploymentInfo>,
    pub continue_token: Option<String>,
}

fn deployment_to_info(d: Deployment) -> DeploymentInfo {
    let meta = &d.metadata;
    let name = meta.name.clone().unwrap_or_default();
    let namespace = meta.namespace.clone().unwrap_or_default();
    let labels = meta.labels.clone().unwrap_or_default();
    let created_at = meta.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339());

    let spec = d.spec.as_ref();
    let replicas_desired = spec.and_then(|s| s.replicas).unwrap_or(1);
    let selector = spec
        .and_then(|s| s.selector.match_labels.clone())
        .unwrap_or_default();

    let images: Vec<String> = spec
        .map(|s| {
            s.template.spec.as_ref().map(|ps| {
                ps.containers.iter()
                    .filter_map(|c| c.image.clone())
                    .collect()
            }).unwrap_or_default()
        })
        .unwrap_or_default();

    let status = d.status.as_ref();
    let replicas_ready = status.and_then(|s| s.ready_replicas).unwrap_or(0);
    let replicas_available = status.and_then(|s| s.available_replicas).unwrap_or(0);
    let replicas_updated = status.and_then(|s| s.updated_replicas).unwrap_or(0);

    let conditions = status
        .and_then(|s| s.conditions.clone())
        .unwrap_or_default()
        .into_iter()
        .map(|c| DeploymentCondition {
            type_: c.type_,
            status: c.status,
            reason: c.reason,
            message: c.message,
        })
        .collect();

    DeploymentInfo {
        name,
        namespace,
        replicas_desired,
        replicas_ready,
        replicas_available,
        replicas_updated,
        labels,
        selector,
        images,
        created_at,
        conditions,
    }
}

// ─── DaemonSet DTOs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSetInfo {
    pub name: String,
    pub namespace: String,
    pub desired: i32,
    pub current: i32,
    pub ready: i32,
    pub updated: i32,
    pub available: i32,
    pub images: Vec<String>,
    pub labels: BTreeMap<String, String>,
    pub node_selector: BTreeMap<String, String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSetList {
    pub items: Vec<DaemonSetInfo>,
    pub continue_token: Option<String>,
}

fn daemonset_to_info(d: DaemonSet) -> DaemonSetInfo {
    let meta = &d.metadata;
    let name = meta.name.clone().unwrap_or_default();
    let namespace = meta.namespace.clone().unwrap_or_default();
    let labels = meta.labels.clone().unwrap_or_default();
    let created_at = meta.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339());

    let spec = d.spec.as_ref();
    let images: Vec<String> = spec.map(|s| {
        s.template.spec.as_ref().map(|ps| {
            ps.containers.iter().filter_map(|c| c.image.clone()).collect()
        }).unwrap_or_default()
    }).unwrap_or_default();
    let node_selector = spec.and_then(|s| s.template.spec.as_ref())
        .and_then(|ps| ps.node_selector.clone())
        .unwrap_or_default();

    let status = d.status.as_ref();
    DaemonSetInfo {
        name, namespace, labels, images, node_selector, created_at,
        desired:   status.map(|s| s.desired_number_scheduled).unwrap_or(0),
        current:   status.map(|s| s.current_number_scheduled).unwrap_or(0),
        ready:     status.map(|s| s.number_ready).unwrap_or(0),
        updated:   status.and_then(|s| s.updated_number_scheduled).unwrap_or(0),
        available: status.and_then(|s| s.number_available).unwrap_or(0),
    }
}

// ─── StatefulSet DTOs ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatefulSetInfo {
    pub name: String,
    pub namespace: String,
    pub replicas_desired: i32,
    pub replicas_ready: i32,
    pub replicas_current: i32,
    pub images: Vec<String>,
    pub labels: BTreeMap<String, String>,
    pub service_name: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatefulSetList {
    pub items: Vec<StatefulSetInfo>,
    pub continue_token: Option<String>,
}

fn statefulset_to_info(s: StatefulSet) -> StatefulSetInfo {
    let meta = &s.metadata;
    let name = meta.name.clone().unwrap_or_default();
    let namespace = meta.namespace.clone().unwrap_or_default();
    let labels = meta.labels.clone().unwrap_or_default();
    let created_at = meta.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339());

    let spec = s.spec.as_ref();
    let replicas_desired = spec.and_then(|sp| sp.replicas).unwrap_or(1);
    let service_name = spec.map(|sp| sp.service_name.clone()).unwrap_or_default();
    let images: Vec<String> = spec.map(|sp| {
        sp.template.spec.as_ref().map(|ps| {
            ps.containers.iter().filter_map(|c| c.image.clone()).collect()
        }).unwrap_or_default()
    }).unwrap_or_default();

    let status = s.status.as_ref();
    StatefulSetInfo {
        name, namespace, labels, images, service_name, created_at,
        replicas_desired,
        replicas_ready:   status.and_then(|st| st.ready_replicas).unwrap_or(0),
        replicas_current: status.and_then(|st| st.current_replicas).unwrap_or(0),
    }
}

// ─── ReplicaSet DTOs ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaSetInfo {
    pub name: String,
    pub namespace: String,
    pub replicas_desired: i32,
    pub replicas_ready: i32,
    pub owner_deployment: Option<String>,
    pub images: Vec<String>,
    pub labels: BTreeMap<String, String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaSetList {
    pub items: Vec<ReplicaSetInfo>,
    pub continue_token: Option<String>,
}

fn replicaset_to_info(rs: ReplicaSet) -> ReplicaSetInfo {
    let meta = &rs.metadata;
    let name = meta.name.clone().unwrap_or_default();
    let namespace = meta.namespace.clone().unwrap_or_default();
    let labels = meta.labels.clone().unwrap_or_default();
    let created_at = meta.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339());

    // Найти владельца Deployment через ownerReferences
    let owner_deployment = meta.owner_references.as_ref()
        .and_then(|refs| refs.iter().find(|r| r.kind == "Deployment"))
        .map(|r| r.name.clone());

    let spec = rs.spec.as_ref();
    let replicas_desired = spec.and_then(|s| s.replicas).unwrap_or(0);
    let images: Vec<String> = spec.and_then(|s| s.template.as_ref()).map(|t| {
        t.spec.as_ref().map(|ps| {
            ps.containers.iter().filter_map(|c| c.image.clone()).collect()
        }).unwrap_or_default()
    }).unwrap_or_default();

    let status = rs.status.as_ref();
    ReplicaSetInfo {
        name, namespace, labels, images, owner_deployment, created_at,
        replicas_desired,
        replicas_ready: status.and_then(|s| s.ready_replicas).unwrap_or(0),
    }
}

// ─── Event DTOs ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub name: String,
    pub namespace: String,
    pub type_: String,
    pub reason: String,
    pub message: String,
    pub involved_object_name: String,
    pub involved_object_kind: String,
    pub count: i32,
    pub first_time: Option<String>,
    pub last_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventList {
    pub items: Vec<EventInfo>,
}

// ─── Service DTOs ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePortInfo {
    pub name: Option<String>,
    pub protocol: String,
    pub port: i32,
    pub target_port: String,
    pub node_port: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub namespace: String,
    pub type_: String,
    pub cluster_ip: Option<String>,
    pub external_ips: Vec<String>,
    pub load_balancer_ip: Option<String>,
    pub ports: Vec<ServicePortInfo>,
    pub selector: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,
    pub headless: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceList {
    pub items: Vec<ServiceInfo>,
    pub continue_token: Option<String>,
}

fn service_to_info(svc: Service) -> ServiceInfo {
    let meta = &svc.metadata;
    let name = meta.name.clone().unwrap_or_default();
    let namespace = meta.namespace.clone().unwrap_or_default();
    let labels = meta.labels.clone().unwrap_or_default();
    let created_at = meta.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339());

    let spec = svc.spec.as_ref();
    let type_ = spec.and_then(|s| s.type_.clone()).unwrap_or_else(|| "ClusterIP".to_string());
    let cluster_ip = spec.and_then(|s| s.cluster_ip.clone()).filter(|ip| !ip.is_empty());
    let headless = cluster_ip.as_deref() == Some("None");
    let selector = spec.and_then(|s| s.selector.clone()).unwrap_or_default();
    let external_ips = spec.and_then(|s| s.external_ips.clone()).unwrap_or_default();
    let load_balancer_ip = spec.and_then(|s| s.load_balancer_ip.clone());

    let ports = spec.and_then(|s| s.ports.clone()).unwrap_or_default().into_iter().map(|p| {
        let target_port = p.target_port.map(|tp| match tp {
            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(n) => n.to_string(),
            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(s) => s,
        }).unwrap_or_default();
        ServicePortInfo {
            name: p.name,
            protocol: p.protocol.unwrap_or_else(|| "TCP".to_string()),
            port: p.port,
            target_port,
            node_port: p.node_port,
        }
    }).collect();

    ServiceInfo { name, namespace, type_, cluster_ip, external_ips, load_balancer_ip, ports, selector, labels, headless, created_at }
}

// ─── Ingress DTOs ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressRule {
    pub host: Option<String>,
    pub paths: Vec<IngressPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressPath {
    pub path: String,
    pub path_type: String,
    pub backend_service: String,
    pub backend_port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressTls {
    pub hosts: Vec<String>,
    pub secret_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressInfo {
    pub name: String,
    pub namespace: String,
    pub ingress_class: Option<String>,
    pub rules: Vec<IngressRule>,
    pub tls: Vec<IngressTls>,
    pub labels: BTreeMap<String, String>,
    pub annotations: BTreeMap<String, String>,
    pub load_balancer_ips: Vec<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressList {
    pub items: Vec<IngressInfo>,
    pub continue_token: Option<String>,
}

fn ingress_to_info(ing: Ingress) -> IngressInfo {
    let meta = &ing.metadata;
    let name = meta.name.clone().unwrap_or_default();
    let namespace = meta.namespace.clone().unwrap_or_default();
    let labels = meta.labels.clone().unwrap_or_default();
    let annotations = meta.annotations.clone().unwrap_or_default();
    let created_at = meta.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339());

    let spec = ing.spec.as_ref();
    let ingress_class = spec.and_then(|s| s.ingress_class_name.clone());

    let rules = spec.and_then(|s| s.rules.clone()).unwrap_or_default().into_iter().map(|r| {
        let host = r.host.clone();
        let paths = r.http.as_ref().map(|h| h.paths.iter().map(|p| {
            let backend_service = p.backend.service.as_ref()
                .map(|s| s.name.clone()).unwrap_or_default();
            let backend_port = p.backend.service.as_ref()
                .and_then(|s| s.port.as_ref())
                .map(|port| {
                    if let Some(n) = port.number { n.to_string() }
                    else if let Some(ref nm) = port.name { nm.clone() }
                    else { String::new() }
                }).unwrap_or_default();
            IngressPath {
                path: p.path.clone().unwrap_or_else(|| "/".to_string()),
                path_type: p.path_type.clone(),
                backend_service,
                backend_port,
            }
        }).collect()).unwrap_or_default();
        IngressRule { host, paths }
    }).collect();

    let tls = spec.and_then(|s| s.tls.clone()).unwrap_or_default().into_iter().map(|t| IngressTls {
        hosts: t.hosts.unwrap_or_default(),
        secret_name: t.secret_name,
    }).collect();

    let load_balancer_ips = ing.status.as_ref()
        .and_then(|s| s.load_balancer.as_ref())
        .and_then(|lb| lb.ingress.as_ref())
        .map(|ingresses| ingresses.iter()
            .filter_map(|i| i.ip.clone().or_else(|| i.hostname.clone()))
            .collect())
        .unwrap_or_default();

    IngressInfo { name, namespace, ingress_class, rules, tls, labels, annotations, load_balancer_ips, created_at }
}

// ─── ConfigMap DTOs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapInfo {
    pub name: String,
    pub namespace: String,
    pub data: BTreeMap<String, String>,
    pub binary_data_keys: Vec<String>,
    pub labels: BTreeMap<String, String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapList {
    pub items: Vec<ConfigMapInfo>,
    pub continue_token: Option<String>,
}

fn configmap_to_info(cm: ConfigMap) -> ConfigMapInfo {
    let meta = &cm.metadata;
    ConfigMapInfo {
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        labels: meta.labels.clone().unwrap_or_default(),
        created_at: meta.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        data: cm.data.unwrap_or_default(),
        binary_data_keys: cm.binary_data.map(|b| b.keys().cloned().collect()).unwrap_or_default(),
    }
}

// ─── Secret DTOs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretInfo {
    pub name: String,
    pub namespace: String,
    pub type_: String,
    /// Только ключи — значения не возвращаются по умолчанию (безопасность)
    pub data_keys: Vec<String>,
    /// base64-декодированные значения (только при явном запросе reveal=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_revealed: Option<BTreeMap<String, String>>,
    pub labels: BTreeMap<String, String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretList {
    pub items: Vec<SecretInfo>,
    pub continue_token: Option<String>,
}

fn secret_to_info(s: Secret) -> SecretInfo {
    let meta = &s.metadata;
    let data_keys = s.data.as_ref()
        .map(|d| d.keys().cloned().collect())
        .unwrap_or_default();
    SecretInfo {
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        type_: s.type_.clone().unwrap_or_else(|| "Opaque".to_string()),
        labels: meta.labels.clone().unwrap_or_default(),
        created_at: meta.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        data_keys,
        data_revealed: None,
    }
}

/// Секрет с раскрытыми значениями (явный вызов из handler при reveal=true)
pub fn secret_to_info_revealed(s: Secret) -> SecretInfo {
    let mut info = secret_to_info(s.clone());
    let revealed: BTreeMap<String, String> = s.data.unwrap_or_default()
        .into_iter()
        .map(|(k, v)| {
            let decoded = String::from_utf8(v.0).unwrap_or_else(|_| "<binary>".to_string());
            (k, decoded)
        })
        .collect();
    info.data_revealed = Some(revealed);
    info
}

// ─── Error helpers ────────────────────────────────────────────────────────────

fn map_err(op: &str, e: kube::Error) -> Error {
    let msg = e.to_string();
    if msg.contains("403") || msg.contains("Forbidden") {
        Error::Other(format!("FORBIDDEN: {msg}"))
    } else {
        Error::Other(format!("Failed to {op}: {msg}"))
    }
}

fn map_err_named(kind: &str, name: &str, e: kube::Error) -> Error {
    let msg = e.to_string();
    if msg.contains("404") || msg.contains("not found") {
        Error::NotFound(format!("{kind} {name} not found"))
    } else if msg.contains("403") || msg.contains("Forbidden") {
        Error::Other(format!("FORBIDDEN: {msg}"))
    } else {
        Error::Other(format!("Failed to get {kind} {name}: {msg}"))
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // =========================================================================
    // 1. Тесты для моделей и типов
    // =========================================================================

    #[test]
    fn test_cluster_version_info_serialize() {
        let info = ClusterVersionInfo {
            major: "1".to_string(),
            minor: "28".to_string(),
            git_version: "v1.28.4".to_string(),
            platform: "linux/amd64".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"major\":\"1\""));
        assert!(json.contains("\"minor\":\"28\""));
        assert!(json.contains("\"git_version\":\"v1.28.4\""));
        assert!(json.contains("\"platform\":\"linux/amd64\""));
    }

    #[test]
    fn test_cluster_info_reachable() {
        let info = ClusterInfo {
            reachable: true,
            status: "ok".to_string(),
            message: None,
            version: Some(ClusterVersionInfo {
                major: "1".into(),
                minor: "28".into(),
                git_version: "v1.28.0".into(),
                platform: "linux/amd64".into(),
            }),
        };
        assert!(info.reachable);
        assert_eq!(info.status, "ok");
        assert!(info.version.is_some());
    }

    #[test]
    fn test_cluster_info_unreachable() {
        let info = ClusterInfo {
            reachable: false,
            status: "unreachable".to_string(),
            message: Some("connection refused".to_string()),
            version: None,
        };
        assert!(!info.reachable);
        assert!(info.message.is_some());
        assert!(info.version.is_none());
    }

    #[test]
    fn test_namespace_info_from_api() {
        let mut labels = BTreeMap::new();
        labels.insert("env".to_string(), "prod".to_string());
        let ns = NamespaceInfo {
            name: "default".to_string(),
            phase: "Active".to_string(),
            labels,
        };
        assert_eq!(ns.name, "default");
        assert_eq!(ns.phase, "Active");
        assert_eq!(ns.labels.get("env"), Some(&"prod".to_string()));
    }

    #[test]
    fn test_namespace_list_pagination() {
        let list = NamespaceList {
            items: vec![
                NamespaceInfo { name: "ns1".into(), phase: "Active".into(), labels: BTreeMap::new() },
                NamespaceInfo { name: "ns2".into(), phase: "Terminating".into(), labels: BTreeMap::new() },
            ],
            continue_token: Some("token123".into()),
        };
        assert_eq!(list.items.len(), 2);
        assert!(list.continue_token.is_some());
    }

    #[test]
    fn test_namespace_list_no_more_pages() {
        let list = NamespaceList {
            items: vec![],
            continue_token: None,
        };
        assert!(list.items.is_empty());
        assert!(list.continue_token.is_none());
    }

    #[test]
    fn test_pod_info_defaults() {
        let pod = PodInfo {
            name: "test-pod".into(),
            namespace: "default".into(),
            phase: "Running".into(),
            reason: None,
            node_name: None,
            pod_ip: None,
            host_ip: None,
            containers: vec![],
            labels: BTreeMap::new(),
            created_at: None,
            ready_count: 0,
            total_count: 0,
        };
        assert_eq!(pod.name, "test-pod");
        assert!(pod.containers.is_empty());
        assert_eq!(pod.total_count, 0);
    }

    #[test]
    fn test_container_status_ready() {
        let cs = ContainerStatus {
            name: "app".into(),
            image: "nginx:latest".into(),
            ready: true,
            restart_count: 0,
            state: "running".into(),
            reason: None,
        };
        assert!(cs.ready);
        assert_eq!(cs.restart_count, 0);
        assert_eq!(cs.state, "running");
    }

    #[test]
    fn test_container_status_crash_loop() {
        let cs = ContainerStatus {
            name: "app".into(),
            image: "myapp:v1".into(),
            ready: false,
            restart_count: 5,
            state: "waiting".into(),
            reason: Some("CrashLoopBackOff".into()),
        };
        assert!(!cs.ready);
        assert_eq!(cs.restart_count, 5);
        assert_eq!(cs.reason, Some("CrashLoopBackOff".into()));
    }

    #[test]
    fn test_pod_list_with_items() {
        let list = PodList {
            items: vec![
                PodInfo {
                    name: "pod1".into(), namespace: "default".into(),
                    phase: "Running".into(), reason: None,
                    node_name: Some("node1".into()),
                    pod_ip: Some("10.0.0.1".into()),
                    host_ip: Some("192.168.1.1".into()),
                    containers: vec![], labels: BTreeMap::new(),
                    created_at: None, ready_count: 1, total_count: 1,
                },
            ],
            continue_token: None,
        };
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].name, "pod1");
        assert_eq!(list.items[0].phase, "Running");
    }

    // =========================================================================
    // 2. Тесты для Deployment моделей
    // =========================================================================

    #[test]
    fn test_deployment_info_serialize() {
        let dep = DeploymentInfo {
            name: "web-app".into(),
            namespace: "default".into(),
            replicas_desired: 3,
            replicas_ready: 2,
            replicas_available: 2,
            replicas_updated: 1,
            labels: BTreeMap::new(),
            selector: BTreeMap::new(),
            images: vec!["nginx:1.25".into()],
            created_at: None,
            conditions: vec![],
        };
        let json = serde_json::to_string(&dep).unwrap();
        assert!(json.contains("\"name\":\"web-app\""));
        assert!(json.contains("\"replicas_desired\":3"));
        assert!(json.contains("\"replicas_ready\":2"));
    }

    #[test]
    fn test_deployment_condition() {
        let cond = DeploymentCondition {
            type_: "Available".into(),
            status: "True".into(),
            reason: Some("MinimumReplicasAvailable".into()),
            message: Some("Deployment has minimum availability".into()),
        };
        assert_eq!(cond.type_, "Available");
        assert_eq!(cond.status, "True");
    }

    #[test]
    fn test_deployment_list() {
        let list = DeploymentList {
            items: vec![
                DeploymentInfo {
                    name: "dep1".into(), namespace: "default".into(),
                    replicas_desired: 1, replicas_ready: 1, replicas_available: 1,
                    replicas_updated: 1, labels: BTreeMap::new(), selector: BTreeMap::new(),
                    images: vec![], created_at: None, conditions: vec![],
                },
            ],
            continue_token: Some("abc".into()),
        };
        assert_eq!(list.items.len(), 1);
        assert!(list.continue_token.is_some());
    }

    // =========================================================================
    // 3. Тесты для DaemonSet моделей
    // =========================================================================

    #[test]
    fn test_daemonset_info() {
        let ds = DaemonSetInfo {
            name: "fluentd".into(),
            namespace: "kube-system".into(),
            desired: 3,
            current: 3,
            ready: 2,
            updated: 3,
            available: 2,
            images: vec!["fluentd:v1".into()],
            labels: BTreeMap::new(),
            node_selector: BTreeMap::new(),
            created_at: None,
        };
        assert_eq!(ds.desired, 3);
        assert_eq!(ds.ready, 2);
        assert_eq!(ds.images.len(), 1);
    }

    #[test]
    fn test_daemonset_list_empty() {
        let list = DaemonSetList {
            items: vec![],
            continue_token: None,
        };
        assert!(list.items.is_empty());
    }

    // =========================================================================
    // 4. Тесты для StatefulSet моделей
    // =========================================================================

    #[test]
    fn test_statefulset_info() {
        let ss = StatefulSetInfo {
            name: "postgres".into(),
            namespace: "db".into(),
            replicas_desired: 3,
            replicas_ready: 3,
            replicas_current: 3,
            images: vec!["postgres:15".into()],
            labels: BTreeMap::new(),
            service_name: "postgres-headless".into(),
            created_at: None,
        };
        assert_eq!(ss.replicas_desired, 3);
        assert_eq!(ss.service_name, "postgres-headless");
        assert_eq!(ss.images.len(), 1);
    }

    #[test]
    fn test_statefulset_list() {
        let list = StatefulSetList {
            items: vec![
                StatefulSetInfo {
                    name: "ss1".into(), namespace: "default".into(),
                    replicas_desired: 1, replicas_ready: 0, replicas_current: 1,
                    images: vec![], labels: BTreeMap::new(), service_name: "svc".into(),
                    created_at: None,
                },
            ],
            continue_token: None,
        };
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].replicas_ready, 0);
    }

    // =========================================================================
    // 5. Тесты для ReplicaSet моделей
    // =========================================================================

    #[test]
    fn test_replicaset_info_with_owner() {
        let rs = ReplicaSetInfo {
            name: "web-app-abc12".into(),
            namespace: "default".into(),
            replicas_desired: 3,
            replicas_ready: 3,
            owner_deployment: Some("web-app".into()),
            images: vec!["nginx:latest".into()],
            labels: BTreeMap::new(),
            created_at: None,
        };
        assert_eq!(rs.owner_deployment, Some("web-app".into()));
        assert_eq!(rs.replicas_ready, 3);
    }

    #[test]
    fn test_replicaset_info_no_owner() {
        let rs = ReplicaSetInfo {
            name: "orphan-rs".into(),
            namespace: "default".into(),
            replicas_desired: 0,
            replicas_ready: 0,
            owner_deployment: None,
            images: vec![],
            labels: BTreeMap::new(),
            created_at: None,
        };
        assert!(rs.owner_deployment.is_none());
    }

    #[test]
    fn test_replicaset_list() {
        let list = ReplicaSetList {
            items: vec![],
            continue_token: Some("token".into()),
        };
        assert!(list.items.is_empty());
        assert!(list.continue_token.is_some());
    }

    // =========================================================================
    // 6. Тесты для Event моделей
    // =========================================================================

    #[test]
    fn test_event_info_normal() {
        let ev = EventInfo {
            name: "event1".into(),
            namespace: "default".into(),
            type_: "Warning".into(),
            reason: "FailedScheduling".into(),
            message: "No nodes available".into(),
            involved_object_name: "pod-x".into(),
            involved_object_kind: "Pod".into(),
            count: 5,
            first_time: Some("2024-01-01T00:00:00Z".into()),
            last_time: Some("2024-01-01T01:00:00Z".into()),
        };
        assert_eq!(ev.type_, "Warning");
        assert_eq!(ev.count, 5);
        assert!(ev.first_time.is_some());
    }

    #[test]
    fn test_event_list() {
        let list = EventList {
            items: vec![
                EventInfo {
                    name: "ev1".into(), namespace: "default".into(),
                    type_: "Normal".into(), reason: "Started".into(),
                    message: "Started container".into(),
                    involved_object_name: "pod1".into(),
                    involved_object_kind: "Pod".into(),
                    count: 1, first_time: None, last_time: None,
                },
            ],
        };
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].type_, "Normal");
    }

    // =========================================================================
    // 7. Тесты для Service моделей
    // =========================================================================

    #[test]
    fn test_service_port_info() {
        let port = ServicePortInfo {
            name: Some("http".into()),
            protocol: "TCP".into(),
            port: 80,
            target_port: "8080".into(),
            node_port: Some(30080),
        };
        assert_eq!(port.port, 80);
        assert_eq!(port.target_port, "8080");
        assert_eq!(port.node_port, Some(30080));
    }

    #[test]
    fn test_service_port_no_name() {
        let port = ServicePortInfo {
            name: None,
            protocol: "UDP".into(),
            port: 53,
            target_port: "53".into(),
            node_port: None,
        };
        assert!(port.name.is_none());
        assert_eq!(port.protocol, "UDP");
    }

    #[test]
    fn test_service_info_cluster_ip() {
        let svc = ServiceInfo {
            name: "my-svc".into(),
            namespace: "default".into(),
            type_: "ClusterIP".into(),
            cluster_ip: Some("10.96.0.1".into()),
            external_ips: vec![],
            load_balancer_ip: None,
            ports: vec![],
            selector: BTreeMap::new(),
            labels: BTreeMap::new(),
            headless: false,
            created_at: None,
        };
        assert!(!svc.headless);
        assert_eq!(svc.cluster_ip, Some("10.96.0.1".into()));
    }

    #[test]
    fn test_service_info_headless() {
        let svc = ServiceInfo {
            name: "headless-svc".into(),
            namespace: "default".into(),
            type_: "ClusterIP".into(),
            cluster_ip: Some("None".into()),
            external_ips: vec![],
            load_balancer_ip: None,
            ports: vec![],
            selector: BTreeMap::new(),
            labels: BTreeMap::new(),
            headless: true,
            created_at: None,
        };
        assert!(svc.headless);
    }

    #[test]
    fn test_service_list() {
        let list = ServiceList {
            items: vec![],
            continue_token: None,
        };
        assert!(list.items.is_empty());
    }

    // =========================================================================
    // 8. Тесты для Ingress моделей
    // =========================================================================

    #[test]
    fn test_ingress_rule_with_paths() {
        let rule = IngressRule {
            host: Some("example.com".into()),
            paths: vec![
                IngressPath {
                    path: "/api".into(),
                    path_type: "Prefix".into(),
                    backend_service: "api-svc".into(),
                    backend_port: "8080".into(),
                },
            ],
        };
        assert_eq!(rule.host, Some("example.com".into()));
        assert_eq!(rule.paths.len(), 1);
        assert_eq!(rule.paths[0].backend_service, "api-svc");
    }

    #[test]
    fn test_ingress_rule_no_host() {
        let rule = IngressRule {
            host: None,
            paths: vec![],
        };
        assert!(rule.host.is_none());
        assert!(rule.paths.is_empty());
    }

    #[test]
    fn test_ingress_tls() {
        let tls = IngressTls {
            hosts: vec!["example.com".into(), "www.example.com".into()],
            secret_name: Some("tls-secret".into()),
        };
        assert_eq!(tls.hosts.len(), 2);
        assert_eq!(tls.secret_name, Some("tls-secret".into()));
    }

    #[test]
    fn test_ingress_tls_no_secret() {
        let tls = IngressTls {
            hosts: vec!["example.com".into()],
            secret_name: None,
        };
        assert!(tls.secret_name.is_none());
    }

    #[test]
    fn test_ingress_info() {
        let ing = IngressInfo {
            name: "web-ingress".into(),
            namespace: "default".into(),
            ingress_class: Some("nginx".into()),
            rules: vec![],
            tls: vec![],
            labels: BTreeMap::new(),
            annotations: BTreeMap::new(),
            load_balancer_ips: vec!["1.2.3.4".into()],
            created_at: None,
        };
        assert_eq!(ing.ingress_class, Some("nginx".into()));
        assert_eq!(ing.load_balancer_ips.len(), 1);
    }

    #[test]
    fn test_ingress_list() {
        let list = IngressList {
            items: vec![],
            continue_token: Some("next".into()),
        };
        assert!(list.items.is_empty());
        assert!(list.continue_token.is_some());
    }

    // =========================================================================
    // 9. Тесты для ConfigMap моделей
    // =========================================================================

    #[test]
    fn test_configmap_info_with_data() {
        let mut data = BTreeMap::new();
        data.insert("key1".into(), "value1".into());
        let cm = ConfigMapInfo {
            name: "app-config".into(),
            namespace: "default".into(),
            data,
            binary_data_keys: vec!["binary_key".into()],
            labels: BTreeMap::new(),
            created_at: None,
        };
        assert_eq!(cm.data.get("key1"), Some(&"value1".into()));
        assert_eq!(cm.binary_data_keys.len(), 1);
    }

    #[test]
    fn test_configmap_info_empty() {
        let cm = ConfigMapInfo {
            name: "empty-cm".into(),
            namespace: "default".into(),
            data: BTreeMap::new(),
            binary_data_keys: vec![],
            labels: BTreeMap::new(),
            created_at: None,
        };
        assert!(cm.data.is_empty());
        assert!(cm.binary_data_keys.is_empty());
    }

    #[test]
    fn test_configmap_list() {
        let list = ConfigMapList {
            items: vec![],
            continue_token: None,
        };
        assert!(list.items.is_empty());
    }

    // =========================================================================
    // 10. Тесты для Secret моделей
    // =========================================================================

    #[test]
    fn test_secret_info_keys_only() {
        let secret = SecretInfo {
            name: "my-secret".into(),
            namespace: "default".into(),
            type_: "Opaque".into(),
            data_keys: vec!["password".into(), "api-key".into()],
            data_revealed: None,
            labels: BTreeMap::new(),
            created_at: None,
        };
        assert_eq!(secret.data_keys.len(), 2);
        assert!(secret.data_revealed.is_none());
    }

    #[test]
    fn test_secret_info_revealed() {
        let mut revealed = BTreeMap::new();
        revealed.insert("password".into(), "secret123".into());
        let secret = SecretInfo {
            name: "my-secret".into(),
            namespace: "default".into(),
            type_: "Opaque".into(),
            data_keys: vec!["password".into()],
            data_revealed: Some(revealed),
            labels: BTreeMap::new(),
            created_at: None,
        };
        assert!(secret.data_revealed.is_some());
        assert_eq!(secret.data_revealed.as_ref().unwrap().get("password"), Some(&"secret123".into()));
    }

    #[test]
    fn test_secret_list() {
        let list = SecretList {
            items: vec![],
            continue_token: None,
        };
        assert!(list.items.is_empty());
    }

    // =========================================================================
    // 11. Тесты для ConnectionMode enum
    // =========================================================================

    #[test]
    fn test_connection_mode_kubeconfig_with_path() {
        let mode = ConnectionMode::KubeConfig {
            path: Some("/etc/kubernetes/config".into()),
            context: Some("prod".into()),
        };
        match mode {
            ConnectionMode::KubeConfig { path, context } => {
                assert_eq!(path, Some("/etc/kubernetes/config".into()));
                assert_eq!(context, Some("prod".into()));
            }
            _ => panic!("Expected KubeConfig variant"),
        }
    }

    #[test]
    fn test_connection_mode_kubeconfig_defaults() {
        let mode = ConnectionMode::KubeConfig {
            path: None,
            context: None,
        };
        match mode {
            ConnectionMode::KubeConfig { path, context } => {
                assert!(path.is_none());
                assert!(context.is_none());
            }
            _ => panic!("Expected KubeConfig variant"),
        }
    }

    #[test]
    fn test_connection_mode_incluster() {
        let mode = ConnectionMode::InCluster;
        match mode {
            ConnectionMode::InCluster => {},
            _ => panic!("Expected InCluster variant"),
        }
    }

    #[test]
    fn test_connection_mode_infer() {
        let mode = ConnectionMode::Infer;
        match mode {
            ConnectionMode::Infer => {},
            _ => panic!("Expected Infer variant"),
        }
    }

    // =========================================================================
    // 12. Тесты для serialization моделей
    // =========================================================================

    #[test]
    fn test_pod_info_json_roundtrip() {
        let mut labels = BTreeMap::new();
        labels.insert("app".into(), "web".into());
        let pod = PodInfo {
            name: "web-123".into(),
            namespace: "default".into(),
            phase: "Running".into(),
            reason: None,
            node_name: Some("node-1".into()),
            pod_ip: Some("10.0.0.5".into()),
            host_ip: Some("192.168.1.10".into()),
            containers: vec![ContainerStatus {
                name: "nginx".into(),
                image: "nginx:1.25".into(),
                ready: true,
                restart_count: 0,
                state: "running".into(),
                reason: None,
            }],
            labels,
            created_at: Some("2024-01-01T00:00:00Z".into()),
            ready_count: 1,
            total_count: 1,
        };
        let json = serde_json::to_string(&pod).unwrap();
        let decoded: PodInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "web-123");
        assert_eq!(decoded.containers.len(), 1);
        assert_eq!(decoded.containers[0].image, "nginx:1.25");
        assert_eq!(decoded.labels.get("app"), Some(&"web".into()));
    }

    #[test]
    fn test_secret_info_skip_serializing_none_revealed() {
        let secret = SecretInfo {
            name: "sec".into(),
            namespace: "default".into(),
            type_: "Opaque".into(),
            data_keys: vec!["k".into()],
            data_revealed: None,
            labels: BTreeMap::new(),
            created_at: None,
        };
        let json = serde_json::to_string(&secret).unwrap();
        assert!(!json.contains("data_revealed"));
    }

    #[test]
    fn test_secret_info_serialize_revealed() {
        let mut revealed = BTreeMap::new();
        revealed.insert("k".into(), "v".into());
        let secret = SecretInfo {
            name: "sec".into(),
            namespace: "default".into(),
            type_: "Opaque".into(),
            data_keys: vec!["k".into()],
            data_revealed: Some(revealed),
            labels: BTreeMap::new(),
            created_at: None,
        };
        let json = serde_json::to_string(&secret).unwrap();
        assert!(json.contains("data_revealed"));
    }

    #[test]
    fn test_event_info_json_roundtrip() {
        let ev = EventInfo {
            name: "ev1".into(),
            namespace: "default".into(),
            type_: "Warning".into(),
            reason: "OOMKilling".into(),
            message: "Container killed due to OOM".into(),
            involved_object_name: "pod-x".into(),
            involved_object_kind: "Pod".into(),
            count: 1,
            first_time: None,
            last_time: None,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let decoded: EventInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.reason, "OOMKilling");
        assert_eq!(decoded.count, 1);
    }

    #[test]
    fn test_ingress_info_json_roundtrip() {
        let ing = IngressInfo {
            name: "my-ing".into(),
            namespace: "default".into(),
            ingress_class: Some("nginx".into()),
            rules: vec![IngressRule {
                host: Some("app.example.com".into()),
                paths: vec![IngressPath {
                    path: "/".into(),
                    path_type: "Prefix".into(),
                    backend_service: "web-svc".into(),
                    backend_port: "80".into(),
                }],
            }],
            tls: vec![IngressTls {
                hosts: vec!["app.example.com".into()],
                secret_name: Some("tls".into()),
            }],
            labels: BTreeMap::new(),
            annotations: BTreeMap::new(),
            load_balancer_ips: vec![],
            created_at: None,
        };
        let json = serde_json::to_string(&ing).unwrap();
        let decoded: IngressInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.rules.len(), 1);
        assert_eq!(decoded.rules[0].paths[0].backend_service, "web-svc");
        assert_eq!(decoded.tls[0].secret_name, Some("tls".into()));
    }

    // =========================================================================
    // 13. Тесты для Clone и Debug трейтов
    // =========================================================================

    #[test]
    fn test_pod_info_clone() {
        let pod = PodInfo {
            name: "clone-test".into(),
            namespace: "default".into(),
            phase: "Pending".into(),
            reason: Some("ContainerCreating".into()),
            node_name: None,
            pod_ip: None,
            host_ip: None,
            containers: vec![],
            labels: BTreeMap::new(),
            created_at: None,
            ready_count: 0,
            total_count: 0,
        };
        let cloned = pod.clone();
        assert_eq!(cloned.name, pod.name);
        assert_eq!(cloned.phase, pod.phase);
        assert_eq!(cloned.reason, pod.reason);
    }

    #[test]
    fn test_container_status_debug() {
        let cs = ContainerStatus {
            name: "debug-ct".into(),
            image: "alpine".into(),
            ready: false,
            restart_count: 0,
            state: "unknown".into(),
            reason: None,
        };
        let debug_str = format!("{:?}", cs);
        assert!(debug_str.contains("ContainerStatus"));
        assert!(debug_str.contains("debug-ct"));
    }

    #[test]
    fn test_deployment_info_clone() {
        let dep = DeploymentInfo {
            name: "clone-dep".into(),
            namespace: "ns".into(),
            replicas_desired: 2,
            replicas_ready: 1,
            replicas_available: 1,
            replicas_updated: 0,
            labels: BTreeMap::new(),
            selector: BTreeMap::new(),
            images: vec!["img:v1".into()],
            created_at: None,
            conditions: vec![],
        };
        let cloned = dep.clone();
        assert_eq!(cloned.name, dep.name);
        assert_eq!(cloned.replicas_desired, 2);
    }

    // =========================================================================
    // 14. Тесты для граничных случаев
    // =========================================================================

    #[test]
    fn test_pod_info_empty_name() {
        let pod = PodInfo {
            name: "".into(),
            namespace: "".into(),
            phase: "Unknown".into(),
            reason: None,
            node_name: None,
            pod_ip: None,
            host_ip: None,
            containers: vec![],
            labels: BTreeMap::new(),
            created_at: None,
            ready_count: 0,
            total_count: 0,
        };
        assert!(pod.name.is_empty());
        assert!(pod.namespace.is_empty());
    }

    #[test]
    fn test_service_port_int_and_string_target() {
        let int_target = ServicePortInfo {
            name: None, protocol: "TCP".into(), port: 443,
            target_port: "443".into(), node_port: None,
        };
        assert_eq!(int_target.target_port, "443");
    }

    #[test]
    fn test_namespace_info_terminating_phase() {
        let ns = NamespaceInfo {
            name: "deleting-ns".into(),
            phase: "Terminating".into(),
            labels: BTreeMap::new(),
        };
        assert_eq!(ns.phase, "Terminating");
    }

    #[test]
    fn test_cluster_version_info_deserialize() {
        let json = r#"{
            "major": "1",
            "minor": "29",
            "git_version": "v1.29.0",
            "platform": "darwin/arm64"
        }"#;
        let info: ClusterVersionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.major, "1");
        assert_eq!(info.minor, "29");
        assert_eq!(info.git_version, "v1.29.0");
        assert_eq!(info.platform, "darwin/arm64");
    }

    #[test]
    fn test_container_status_terminated() {
        let cs = ContainerStatus {
            name: "job".into(),
            image: "busybox".into(),
            ready: false,
            restart_count: 0,
            state: "terminated".into(),
            reason: Some("Completed".into()),
        };
        assert_eq!(cs.state, "terminated");
        assert_eq!(cs.reason, Some("Completed".into()));
        assert!(!cs.ready);
    }

    #[test]
    fn test_labels_map_ordering() {
        let mut labels = BTreeMap::new();
        labels.insert("z-label".into(), "last".into());
        labels.insert("a-label".into(), "first".into());
        labels.insert("m-label".into(), "middle".into());

        let pod = PodInfo {
            name: "ordered".into(),
            namespace: "default".into(),
            phase: "Running".into(),
            reason: None, node_name: None, pod_ip: None, host_ip: None,
            containers: vec![], labels, created_at: None,
            ready_count: 1, total_count: 1,
        };

        let json = serde_json::to_string(&pod).unwrap();
        // BTreeMap гарантирует сортировку по ключам
        let a_pos = json.find("a-label").unwrap();
        let m_pos = json.find("m-label").unwrap();
        let z_pos = json.find("z-label").unwrap();
        assert!(a_pos < m_pos && m_pos < z_pos);
    }

    // =========================================================================
    // 15. Тесты для KubernetesClusterService Clone
    // =========================================================================

    #[test]
    fn test_kubernetes_cluster_service_is_clone() {
        // Проверяем, что KubernetesClusterService реализует Clone
        fn assert_clone<T: Clone>() {}
        assert_clone::<KubernetesClusterService>();
    }

    #[test]
    fn test_kubernetes_cluster_service_is_send_sync() {
        // Проверяем, что тип безопасен для отправки между потоками
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<KubernetesClusterService>();
        assert_sync::<KubernetesClusterService>();
    }
}
