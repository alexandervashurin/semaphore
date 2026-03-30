# 📘 ФАЗА 1: Фундамент (Недели 1-2)

> **Цель:** Создать базовую инфраструктуру для работы с Kubernetes: подключение, аутентификация, базовые операции.

---

## 📋 Задачи Фазы 1

### 1.1. Инфраструктура Kubernetes клиента

**Файлы:**
```
rust/src/kubernetes/
├── client.rs           # Обновить существующий
├── config.rs           # Обновить существующий
├── error.rs            # Новый файл
├── types.rs            # Новый файл
└── mod.rs              # Обновить существующий
```

**Задачи:**
- [ ] Создать `KubeClient` с использованием `kube` crate
- [ ] Реализовать поддержку kubeconfig и in-cluster config
- [ ] Добавить кэширование подключений (connection pool)
- [ ] Обработка ошибок подключения
- [ ] Логирование операций

**Пример кода (client.rs):**
```rust
//! Kubernetes Client на основе kube-rs

use kube::{
    api::{Api, ListParams, ResourceExt},
    client::Client,
    config::{KubeConfigOptions, Kubeconfig, Context},
    Config, Resource,
};
use k8s_openapi::api::core::v1::{Namespace, Pod, Service};
use tracing::{info, warn, error, debug};
use crate::error::{Error, Result};
use std::sync::Arc;

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
pub struct KubeClient {
    client: Client,
    default_namespace: String,
    config: KubeConfig,
}

impl KubeClient {
    /// Создаёт новый Kubernetes клиент
    pub async fn new(config: KubeConfig) -> Result<Self> {
        info!("Creating Kubernetes client");

        let client_config = if let Some(kubeconfig_path) = &config.kubeconfig_path {
            // Загрузка из файла
            let kubeconfig = Kubeconfig::read_from(kubeconfig_path)
                .map_err(|e| Error::KubernetesError(format!("Failed to read kubeconfig: {}", e)))?;

            let mut config_opts = KubeConfigOptions {
                context: config.context.clone(),
                ..Default::default()
            };

            Config::from_custom_kubeconfig(kubeconfig, &config_opts)
                .await
                .map_err(|e| Error::KubernetesError(format!("Failed to load kubeconfig: {}", e)))?
        } else {
            // In-cluster config или ~/.kube/config
            Config::infer()
                .await
                .map_err(|e| Error::KubernetesError(format!("Failed to infer config: {}", e)))?
        };

        let client = Client::try_from(client_config)
            .map_err(|e| Error::KubernetesError(format!("Failed to create client: {}", e)))?;

        Ok(Self {
            client,
            default_namespace: config.default_namespace.clone(),
            config,
        })
    }

    /// Проверяет подключение к кластеру
    pub async fn check_connection(&self) -> Result<bool> {
        let api: Api<Namespace> = Api::all(self.client.clone());

        match api.list(&ListParams::default().limit(1)).await {
            Ok(_) => {
                info!("Successfully connected to Kubernetes cluster");
                Ok(true)
            }
            Err(e) => {
                error!("Failed to connect to Kubernetes: {}", e);
                Err(Error::KubernetesError(e.to_string()))
            }
        }
    }

    /// Получает API для работы с ресурсами в namespace
    pub fn api<T: Resource>(&self, namespace: Option<&str>) -> Api<T>
    where
        T::DynamicType: Default,
    {
        let ns = namespace.unwrap_or(&self.default_namespace);
        Api::namespaced(self.client.clone(), ns)
    }

    /// Получает API для cluster-scoped ресурсов
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
}
```

**Пример кода (error.rs):**
```rust
//! Ошибки Kubernetes модуля

use thiserror::Error;

#[derive(Error, Debug)]
pub enum KubeError {
    #[error("Kubernetes API error: {0}")]
    ApiError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("YAML parse error: {0}")]
    YamlError(String),

    #[error("JSON parse error: {0}")]
    JsonError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}

impl From<kube::Error> for KubeError {
    fn from(err: kube::Error) -> Self {
        KubeError::ApiError(err.to_string())
    }
}

impl From<serde_yaml::Error> for KubeError {
    fn from(err: serde_yaml::Error) -> Self {
        KubeError::YamlError(err.to_string())
    }
}

impl From<serde_json::Error> for KubeError {
    fn from(err: serde_json::Error) -> Self {
        KubeError::JsonError(err.to_string())
    }
}
```

**Пример кода (types.rs):**
```rust
//! Общие типы для Kubernetes модуля

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Метаданные ресурса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMeta {
    pub name: String,
    pub namespace: Option<String>,
    pub uid: String,
    pub created_at: DateTime<Utc>,
    pub labels: std::collections::HashMap<String, String>,
    pub annotations: std::collections::HashMap<String, String>,
}

/// Статус ресурса
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
    Terminating,
}

/// Сводка по ресурсам namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceSummary {
    pub name: String,
    pub status: String,
    pub pods_count: i32,
    pub services_count: i32,
    pub deployments_count: i32,
    pub configmaps_count: i32,
    pub secrets_count: i32,
    pub cpu_request: String,
    pub memory_request: String,
    pub cpu_limit: String,
    pub memory_limit: String,
}

/// Сводка по кластеру
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterSummary {
    pub kubernetes_version: String,
    pub nodes_count: i32,
    pub nodes_ready: i32,
    pub namespaces_count: i32,
    pub pods_total: i32,
    pub pods_running: i32,
    pub pods_pending: i32,
    pub pods_failed: i32,
    pub cpu_capacity: String,
    pub memory_capacity: String,
    pub cpu_allocatable: String,
    pub memory_allocatable: String,
    pub cpu_used: String,
    pub memory_used: String,
}
```

---

### 1.2. API Handlers для подключения

**Файлы:**
```
rust/src/api/handlers/kubernetes/
├── mod.rs              # Новый файл
├── cluster.rs          # Новый файл
├── namespaces.rs       # Новый файл
└── health.rs           # Новый файл
```

**rust/src/api/handlers/kubernetes/mod.rs:**
```rust
//! Kubernetes API handlers

pub mod cluster;
pub mod namespaces;
pub mod health;

pub use cluster::*;
pub use namespaces::*;
pub use health::*;
```

**rust/src/api/handlers/kubernetes/cluster.rs:**
```rust
//! Cluster API handlers

use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::error::{Error, Result};
use crate::kubernetes::KubeClient;
use k8s_openapi::api::core::v1::Node;
use k8s_openapi::apimachinery::pkg::version::Info;
use kube::api::{Api, ListParams};
use serde::{Deserialize, Serialize};

/// Информация о кластере
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub kubernetes_version: String,
    pub platform: String,
    pub git_version: String,
    pub git_commit: String,
    pub build_date: String,
    pub go_version: String,
    pub compiler: String,
    pub platform_os: String,
    pub architecture: String,
}

/// Сводка по узлам
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSummary {
    pub name: String,
    pub status: String,
    pub roles: Vec<String>,
    pub version: String,
    pub internal_ip: String,
    pub external_ip: Option<String>,
    pub os_image: String,
    pub kernel_version: String,
    pub container_runtime: String,
    pub cpu_capacity: String,
    pub memory_capacity: String,
    pub pods_capacity: i32,
    pub cpu_allocatable: String,
    pub memory_allocatable: String,
    pub pods_allocatable: i32,
}

/// Получить информацию о кластере
/// GET /api/kubernetes/cluster/info
pub async fn get_cluster_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ClusterInfo>> {
    let client = state.kubernetes_client()?;

    // Получаем версию через discovery API
    let version = client
        .api_all::<k8s_openapi::apimachinery::pkg::version::Info>()
        .get_metadata()
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    Ok(Json(ClusterInfo {
        kubernetes_version: version.git_version.clone(),
        platform: version.platform.clone(),
        git_version: version.git_version.clone(),
        git_commit: version.git_commit.clone(),
        build_date: version.build_date.clone(),
        go_version: version.go_version.clone(),
        compiler: version.compiler.clone(),
        platform_os: version.platform.split('/').next().unwrap_or("unknown").to_string(),
        architecture: version.platform.split('/').nth(1).unwrap_or("unknown").to_string(),
    }))
}

/// Получить список узлов
/// GET /api/kubernetes/cluster/nodes
pub async fn get_cluster_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<NodeSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<Node> = client.api_all();

    let nodes = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    let mut summaries = Vec::new();

    for node in nodes.items {
        let status = &node.status;
        let spec = &node.spec;
        let meta = &node.metadata;

        // Определяем роли
        let mut roles = Vec::new();
        if let Some(labels) = &meta.labels {
            for (key, value) in labels {
                if key.starts_with("node-role.kubernetes.io/") && value == "" {
                    roles.push(key.split('/').last().unwrap_or("unknown").to_string());
                }
            }
        }
        if roles.is_empty() {
            roles.push("<none>".to_string());
        }

        // Определяем статус
        let node_status = status
            .conditions
            .as_ref()
            .and_then(|conditions| {
                conditions
                    .iter()
                    .find(|c| c.type_ == "Ready")
                    .map(|c| if c.status == "True" { "Ready" } else { "NotReady" }.to_string())
            })
            .unwrap_or_else(|| "Unknown".to_string());

        // Получаем IP адреса
        let mut internal_ip = "unknown".to_string();
        let mut external_ip: Option<String> = None;

        if let Some(addresses) = &status.addresses {
            for addr in addresses {
                match addr.type_.as_str() {
                    "InternalIP" => internal_ip = addr.address.clone(),
                    "ExternalIP" => external_ip = Some(addr.address.clone()),
                    _ => {}
                }
            }
        }

        // Ресурсы
        let capacity = &status.capacity;
        let allocatable = &status.allocatable;

        summaries.push(NodeSummary {
            name: meta.name.clone().unwrap_or_else(|| "unknown".to_string()),
            status: node_status,
            roles,
            version: status.node_info.kubelet_version.clone(),
            internal_ip,
            external_ip,
            os_image: status.node_info.os_image.clone(),
            kernel_version: status.node_info.kernel_version.clone(),
            container_runtime: status
                .node_info
                .container_runtime_version
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            cpu_capacity: capacity
                .get("cpu")
                .map(|q| q.to_string())
                .unwrap_or_else(|| "0".to_string()),
            memory_capacity: capacity
                .get("memory")
                .map(|q| q.to_string())
                .unwrap_or_else(|| "0".to_string()),
            pods_capacity: capacity
                .get("pods")
                .and_then(|q| q.0.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            cpu_allocatable: allocatable
                .get("cpu")
                .map(|q| q.to_string())
                .unwrap_or_else(|| "0".to_string()),
            memory_allocatable: allocatable
                .get("memory")
                .map(|q| q.to_string())
                .unwrap_or_else(|| "0".to_string()),
            pods_allocatable: allocatable
                .get("pods")
                .and_then(|q| q.0.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        });
    }

    Ok(Json(summaries))
}

/// Получить сводку по кластеру
/// GET /api/kubernetes/cluster/summary
pub async fn get_cluster_summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;

    // Считаем количество узлов
    let nodes_api: Api<Node> = client.api_all();
    let nodes = nodes_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    let nodes_count = nodes.items.len() as i32;
    let nodes_ready = nodes
        .items
        .iter()
        .filter(|n| {
            n.status
                .conditions
                .as_ref()
                .map(|conds| {
                    conds
                        .iter()
                        .any(|c| c.type_ == "Ready" && c.status == "True")
                })
                .unwrap_or(false)
        })
        .count() as i32;

    // Считаем namespaces
    let ns_api: Api<k8s_openapi::api::core::v1::Namespace> = client.api_all();
    let namespaces = ns_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;
    let namespaces_count = namespaces.items.len() as i32;

    // Считаем pod'ы
    let pods_api: Api<k8s_openapi::api::core::v1::Pod> = client.api_all();
    let pods = pods_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    let pods_total = pods.items.len() as i32;
    let pods_running = pods
        .items
        .iter()
        .filter(|p| {
            p.status
                .as_ref()
                .map(|s| s.phase.as_deref() == Some("Running"))
                .unwrap_or(false)
        })
        .count() as i32;
    let pods_pending = pods
        .items
        .iter()
        .filter(|p| {
            p.status
                .as_ref()
                .map(|s| s.phase.as_deref() == Some("Pending"))
                .unwrap_or(false)
        })
        .count() as i32;
    let pods_failed = pods
        .items
        .iter()
        .filter(|p| {
            p.status
                .as_ref()
                .map(|s| s.phase.as_deref() == Some("Failed"))
                .unwrap_or(false)
        })
        .count() as i32;

    Ok(Json(serde_json::json!({
        "kubernetes_version": "v1.30.0", // TODO: получить из API
        "nodes_count": nodes_count,
        "nodes_ready": nodes_ready,
        "namespaces_count": namespaces_count,
        "pods_total": pods_total,
        "pods_running": pods_running,
        "pods_pending": pods_pending,
        "pods_failed": pods_failed,
    })))
}
```

**rust/src/api/handlers/kubernetes/namespaces.rs:**
```rust
//! Namespace API handlers

use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::error::{Error, Result};
use k8s_openapi::api::core::v1::{Namespace, ResourceQuota, LimitRange};
use kube::api::{Api, ListParams, PostParams, DeleteParams};
use serde::{Deserialize, Serialize};

/// Список namespace'ов
/// GET /api/kubernetes/namespaces
pub async fn list_namespaces(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api: Api<Namespace> = client.api_all();

    let namespaces = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

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
                "created_at": meta.creation_timestamp.as_ref().map(|t| t.to_rfc3339()).unwrap_or_else(|| "unknown".to_string()),
                "labels": meta.labels.clone().unwrap_or_default(),
                "annotations": meta.annotations.clone().unwrap_or_default(),
            })
        })
        .collect();

    Ok(Json(result))
}

/// Детали namespace
/// GET /api/kubernetes/namespaces/{name}
pub async fn get_namespace(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Namespace> = client.api_all();

    let ns = api
        .get(&name)
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    Ok(Json(serde_json::json!(ns)))
}

/// Создать namespace
/// POST /api/kubernetes/namespaces
pub async fn create_namespace(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Namespace> = client.api_all();

    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::ValidationError("Name is required".to_string()))?;

    let labels = payload
        .get("labels")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let annotations = payload
        .get("annotations")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let ns = Namespace {
        metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
            name: Some(name.to_string()),
            labels: Some(labels),
            annotations: Some(annotations),
            ..Default::default()
        },
        ..Default::default()
    };

    let created = api
        .create(&PostParams::default(), &ns)
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    Ok(Json(serde_json::json!(created)))
}

/// Обновить namespace
/// PUT /api/kubernetes/namespaces/{name}
pub async fn update_namespace(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Namespace> = client.api_all();

    let mut ns = api
        .get(&name)
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    // Обновляем labels и annotations
    if let Some(labels) = payload.get("labels") {
        ns.metadata.labels = serde_json::from_value(labels.clone()).ok();
    }

    if let Some(annotations) = payload.get("annotations") {
        ns.metadata.annotations = serde_json::from_value(annotations.clone()).ok();
    }

    let updated = api
        .replace(&name, &Default::default(), &ns)
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    Ok(Json(serde_json::json!(updated)))
}

/// Удалить namespace
/// DELETE /api/kubernetes/namespaces/{name}
pub async fn delete_namespace(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Namespace> = client.api_all();

    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Namespace {} deleted", name)
    })))
}

/// Получить ResourceQuota namespace
/// GET /api/kubernetes/namespaces/{name}/quota
pub async fn get_namespace_quota(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Vec<ResourceQuota>>> {
    let client = state.kubernetes_client()?;
    let api: Api<ResourceQuota> = client.api::<ResourceQuota>(Some(&name));

    let quotas = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    Ok(Json(quotas.items))
}

/// Получить LimitRange namespace
/// GET /api/kubernetes/namespaces/{name}/limits
pub async fn get_namespace_limits(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Vec<LimitRange>>> {
    let client = state.kubernetes_client()?;
    let api: Api<LimitRange> = client.api::<LimitRange>(Some(&name));

    let limits = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    Ok(Json(limits.items))
}
```

**rust/src/api/handlers/kubernetes/health.rs:**
```rust
//! Health check handlers для Kubernetes

use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Статус подключения к Kubernetes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesHealth {
    pub connected: bool,
    pub cluster_name: Option<String>,
    pub kubernetes_version: Option<String>,
    pub nodes_count: Option<i32>,
    pub error: Option<String>,
}

/// Проверка здоровья Kubernetes подключения
/// GET /api/kubernetes/health
pub async fn kubernetes_health(
    State(state): State<Arc<AppState>>,
) -> Result<Json<KubernetesHealth>> {
    match state.kubernetes_client() {
        Ok(client) => {
            match client.check_connection().await {
                Ok(_) => {
                    // Получаем дополнительную информацию
                    let nodes_count = match client.api_all::<k8s_openapi::api::core::v1::Node>().list(&Default::default()).await {
                        Ok(nodes) => Some(nodes.items.len() as i32),
                        Err(_) => None,
                    };

                    Ok(Json(KubernetesHealth {
                        connected: true,
                        cluster_name: Some("default".to_string()),
                        kubernetes_version: Some("v1.30.0".to_string()),
                        nodes_count,
                        error: None,
                    }))
                }
                Err(e) => Ok(Json(KubernetesHealth {
                    connected: false,
                    cluster_name: None,
                    kubernetes_version: None,
                    nodes_count: None,
                    error: Some(e.to_string()),
                })),
            }
        }
        Err(e) => Ok(Json(KubernetesHealth {
            connected: false,
            cluster_name: None,
            kubernetes_version: None,
            nodes_count: None,
            error: Some(e.to_string()),
        })),
    }
}
```

---

### 1.3. Регистрация routes

**Файл:** `rust/src/api/routes.rs`

Добавить в начало файла:
```rust
// Kubernetes модуль
use crate::api::handlers::kubernetes::{
    cluster, namespaces, health
};
```

Добавить в функцию `api_routes()`:
```rust
// Kubernetes API
.route("/api/kubernetes/health", get(kubernetes::health::kubernetes_health))
.route("/api/kubernetes/cluster/info", get(kubernetes::cluster::get_cluster_info))
.route("/api/kubernetes/cluster/nodes", get(kubernetes::cluster::get_cluster_nodes))
.route("/api/kubernetes/cluster/summary", get(kubernetes::cluster::get_cluster_summary))
.route("/api/kubernetes/namespaces", get(kubernetes::namespaces::list_namespaces))
.route("/api/kubernetes/namespaces", post(kubernetes::namespaces::create_namespace))
.route("/api/kubernetes/namespaces/{name}", get(kubernetes::namespaces::get_namespace))
.route("/api/kubernetes/namespaces/{name}", put(kubernetes::namespaces::update_namespace))
.route("/api/kubernetes/namespaces/{name}", delete(kubernetes::namespaces::delete_namespace))
.route("/api/kubernetes/namespaces/{name}/quota", get(kubernetes::namespaces::get_namespace_quota))
.route("/api/kubernetes/namespaces/{name}/limits", get(kubernetes::namespaces::get_namespace_limits))
```

---

### 1.4. AppState интеграция

**Файл:** `rust/src/api/state.rs`

Добавить:
```rust
use crate::kubernetes::KubeClient;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    // ... существующие поля ...

    /// Kubernetes клиент (опционально)
    pub kubernetes_client: Arc<RwLock<Option<Arc<KubeClient>>>>,
}

impl AppState {
    // ... существующие методы ...

    /// Получить Kubernetes клиент
    pub fn kubernetes_client(&self) -> Result<Arc<KubeClient>, Error> {
        let client_guard = self.kubernetes_client.blocking_read();
        client_guard
            .as_ref()
            .cloned()
            .ok_or_else(|| Error::KubernetesError("Kubernetes client not initialized".to_string()))
    }

    /// Инициализировать Kubernetes клиент
    pub async fn init_kubernetes(&self, config: KubeConfig) -> Result<(), Error> {
        let client = KubeClient::new(config).await?;
        let mut client_guard = self.kubernetes_client.blocking_write();
        *client_guard = Some(Arc::new(client));
        Ok(())
    }
}
```

---

### 1.5. Frontend — Namespace Picker

**Файл:** `web/kubernetes/components/namespace-picker.js`

```javascript
/**
 * Namespace Picker Component
 * Выпадающий список для выбора namespace
 */
class NamespacePicker {
  constructor(containerId, options = {}) {
    this.container = document.getElementById(containerId);
    this.onChange = options.onChange || (() => {});
    this.includeAllOption = options.includeAllOption ?? true;
    this.allOptionLabel = options.allOptionLabel || 'All Namespaces';
    this.selectedNamespace = null;
    this.namespaces = [];

    this.render();
    this.loadNamespaces();
  }

  async loadNamespaces() {
    try {
      this.namespaces = await k8s.listNamespaces();
      this.render();
    } catch (error) {
      console.error('Failed to load namespaces:', error);
      this.container.innerHTML = `
        <div class="error-message">
          <i class="fa-solid fa-triangle-exclamation"></i>
          Failed to load namespaces
        </div>
      `;
    }
  }

  render() {
    let optionsHtml = '';

    if (this.includeAllOption) {
      optionsHtml += `
        <option value="" ${!this.selectedNamespace ? 'selected' : ''}>
          ${this.allOptionLabel}
        </option>
      `;
    }

    this.namespaces.forEach(ns => {
      const isSelected = this.selectedNamespace === ns.name;
      optionsHtml += `
        <option value="${ns.name}" ${isSelected ? 'selected' : ''}>
          ${ns.name}
        </option>
      `;
    });

    this.container.innerHTML = `
      <select class="namespace-picker" data-testid="namespace-picker">
        ${optionsHtml}
      </select>
    `;

    this.container.querySelector('select').addEventListener('change', (e) => {
      this.selectedNamespace = e.target.value || null;
      this.onChange(this.selectedNamespace);
    });
  }

  getValue() {
    return this.selectedNamespace;
  }

  setValue(namespace) {
    this.selectedNamespace = namespace;
    this.render();
  }
}

// Usage:
// const nsPicker = new NamespacePicker('namespace-picker', {
//   onChange: (ns) => {
//     console.log('Selected namespace:', ns);
//     loadResources(ns);
//   }
// });
```

---

### 1.6. Frontend — Cluster Overview Page

**Файл:** `web/kubernetes/pages/k8s-cluster.html`

```html
<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Cluster Overview — Kubernetes — Velum</title>
    <link rel="stylesheet" href="/styles.css">
    <link rel="stylesheet" href="/kubernetes/styles/kubernetes.css">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css">
</head>
<body>
    <div class="layout">
        <aside class="sidebar">
            <!-- Kubernetes Navigation -->
            <nav class="k8s-nav">
                <a href="/kubernetes/cluster.html" class="active">
                    <i class="fa-solid fa-server"></i> Overview
                </a>
                <a href="/kubernetes/pods.html">
                    <i class="fa-solid fa-box"></i> Pods
                </a>
                <a href="/kubernetes/deployments.html">
                    <i class="fa-solid fa-layer-group"></i> Deployments
                </a>
                <a href="/kubernetes/services.html">
                    <i class="fa-solid fa-network-wired"></i> Services
                </a>
                <a href="/kubernetes/configmaps.html">
                    <i class="fa-solid fa-file-code"></i> ConfigMaps
                </a>
                <a href="/kubernetes/secrets.html">
                    <i class="fa-solid fa-key"></i> Secrets
                </a>
                <a href="/kubernetes/storage.html">
                    <i class="fa-solid fa-database"></i> Storage
                </a>
                <a href="/kubernetes/workloads.html">
                    <i class="fa-solid fa-cogs"></i> Workloads
                </a>
                <a href="/kubernetes/rbac.html">
                    <i class="fa-solid fa-user-shield"></i> RBAC
                </a>
                <a href="/kubernetes/settings.html">
                    <i class="fa-solid fa-cog"></i> Settings
                </a>
            </nav>
        </aside>

        <div class="main-content">
            <div class="main-header">
                <h2>
                    <i class="fa-solid fa-server"></i>
                    Cluster Overview
                </h2>
                <div id="cluster-health-badge"></div>
            </div>

            <div class="main-body">
                <!-- Cluster Info Cards -->
                <div class="k8s-cards-grid" id="cluster-cards">
                    <div class="loading">
                        <div class="loading-spinner"></div>
                        <p>Loading cluster info...</p>
                    </div>
                </div>

                <!-- Nodes Grid -->
                <h3 style="margin: 24px 0 16px;">
                    <i class="fa-solid fa-server"></i> Nodes
                </h3>
                <div class="k8s-nodes-grid" id="nodes-grid"></div>

                <!-- Resource Usage Chart -->
                <h3 style="margin: 24px 0 16px;">
                    <i class="fa-solid fa-chart-pie"></i> Resource Usage
                </h3>
                <div class="k8s-charts-grid" id="resource-charts"></div>
            </div>
        </div>
    </div>

    <script src="/app.js"></script>
    <script src="/kubernetes/k8s.js"></script>
    <script src="/kubernetes/components/namespace-picker.js"></script>
    <script src="/kubernetes/pages/cluster.js"></script>
</body>
</html>
```

**Файл:** `web/kubernetes/pages/cluster.js`

```javascript
/**
 * Cluster Overview Page Logic
 */
document.addEventListener('DOMContentLoaded', async () => {
  checkAuth();

  // Load cluster info
  await loadClusterInfo();
  await loadNodes();
  await loadResourceCharts();

  // Refresh every 30 seconds
  setInterval(() => {
    loadClusterInfo();
    loadNodes();
  }, 30000);
});

async function loadClusterInfo() {
  try {
    const [info, summary] = await Promise.all([
      api.get('/api/kubernetes/cluster/info'),
      api.get('/api/kubernetes/cluster/summary')
    ]);

    const cards = $('#cluster-cards');
    cards.innerHTML = `
      <div class="k8s-card">
        <div class="k8s-card-icon">
          <i class="fa-brands fa-kubernetes"></i>
        </div>
        <div class="k8s-card-content">
          <div class="k8s-card-label">Kubernetes Version</div>
          <div class="k8s-card-value">${info.kubernetes_version}</div>
        </div>
      </div>

      <div class="k8s-card">
        <div class="k8s-card-icon">
          <i class="fa-solid fa-server"></i>
        </div>
        <div class="k8s-card-content">
          <div class="k8s-card-label">Nodes</div>
          <div class="k8s-card-value">${summary.nodes_count}</div>
          <div class="k8s-card-sub">${summary.nodes_ready} ready</div>
        </div>
      </div>

      <div class="k8s-card">
        <div class="k8s-card-icon">
          <i class="fa-solid fa-layer-group"></i>
        </div>
        <div class="k8s-card-content">
          <div class="k8s-card-label">Namespaces</div>
          <div class="k8s-card-value">${summary.namespaces_count}</div>
        </div>
      </div>

      <div class="k8s-card">
        <div class="k8s-card-icon">
          <i class="fa-solid fa-box"></i>
        </div>
        <div class="k8s-card-content">
          <div class="k8s-card-label">Pods</div>
          <div class="k8s-card-value">${summary.pods_total}</div>
          <div class="k8s-card-sub">
            <span class="status-running">${summary.pods_running} running</span>
            <span class="status-pending">${summary.pods_pending} pending</span>
            <span class="status-failed">${summary.pods_failed} failed</span>
          </div>
        </div>
      </div>
    `;

    // Update health badge
    const healthBadge = $('#cluster-health-badge');
    healthBadge.innerHTML = `
      <span class="badge badge-success">
        <i class="fa-solid fa-check-circle"></i> Healthy
      </span>
    `;
  } catch (error) {
    console.error('Failed to load cluster info:', error);
    const cards = $('#cluster-cards');
    cards.innerHTML = `
      <div class="error-message">
        <i class="fa-solid fa-triangle-exclamation"></i>
        Failed to load cluster information
      </div>
    `;

    const healthBadge = $('#cluster-health-badge');
    healthBadge.innerHTML = `
      <span class="badge badge-danger">
        <i class="fa-solid fa-times-circle"></i> Disconnected
      </span>
    `;
  }
}

async function loadNodes() {
  try {
    const nodes = await api.get('/api/kubernetes/cluster/nodes');
    const grid = $('#nodes-grid');

    if (nodes.length === 0) {
      grid.innerHTML = '<p class="empty-message">No nodes found</p>';
      return;
    }

    grid.innerHTML = nodes.map(node => `
      <div class="k8s-node-card ${node.status === 'Ready' ? 'node-ready' : 'node-not-ready'}">
        <div class="k8s-node-header">
          <div class="k8s-node-name">${node.name}</div>
          <span class="badge badge-${node.status === 'Ready' ? 'success' : 'danger'}">
            ${node.status}
          </span>
        </div>

        <div class="k8s-node-body">
          <div class="k8s-node-info">
            <div><i class="fa-solid fa-tag"></i> Roles: ${node.roles.join(', ')}</div>
            <div><i class="fa-brands fa-linux"></i> OS: ${node.os_image}</div>
            <div><i class="fa-solid fa-microchip"></i> Kernel: ${node.kernel_version}</div>
            <div><i class="fa-solid fa-box"></i> Runtime: ${node.container_runtime}</div>
            <div><i class="fa-solid fa-network-wired"></i> IP: ${node.internal_ip}</div>
            ${node.external_ip ? `<div><i class="fa-solid fa-globe"></i> External: ${node.external_ip}</div>` : ''}
          </div>

          <div class="k8s-node-resources">
            <div class="k8s-resource-row">
              <span>CPU:</span>
              <span>${node.cpu_allocatable} / ${node.cpu_capacity}</span>
            </div>
            <div class="k8s-resource-row">
              <span>Memory:</span>
              <span>${node.memory_allocatable} / ${node.memory_capacity}</span>
            </div>
            <div class="k8s-resource-row">
              <span>Pods:</span>
              <span>${node.pods_allocatable} / ${node.pods_capacity}</span>
            </div>
          </div>
        </div>

        <div class="k8s-node-actions">
          <button class="btn btn-sm btn-secondary" onclick="viewNodeDetails('${node.name}')">
            <i class="fa-solid fa-eye"></i> View Details
          </button>
        </div>
      </div>
    `);
  } catch (error) {
    console.error('Failed to load nodes:', error);
    $('#nodes-grid').innerHTML = `
      <div class="error-message">
        <i class="fa-solid fa-triangle-exclamation"></i>
        Failed to load nodes
      </div>
    `;
  }
}

async function loadResourceCharts() {
  // TODO: Implement charts with Chart.js or Recharts
  $('#resource-charts').innerHTML = `
    <div class="k8s-chart-placeholder">
      <i class="fa-solid fa-chart-pie"></i>
      <p>Resource usage charts will be implemented in Phase 8</p>
    </div>
  `;
}

function viewNodeDetails(nodeName) {
  // Navigate to node details page
  window.location.href = `/kubernetes/nodes/${encodeURIComponent(nodeName)}`;
}
```

---

### 1.7. CSS Styles

**Файл:** `web/kubernetes/styles/kubernetes.css`

```css
/* ========================================
   Kubernetes UI Styles
   ======================================== */

/* Kubernetes Navigation */
.k8s-nav {
  padding: 16px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
}

.k8s-nav a {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  color: var(--text-secondary);
  text-decoration: none;
  border-radius: 6px;
  margin-bottom: 4px;
  transition: all 0.2s;
}

.k8s-nav a:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.k8s-nav a.active {
  background: var(--primary);
  color: #fff;
}

.k8s-nav a i {
  width: 20px;
  text-align: center;
}

/* Cards Grid */
.k8s-cards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.k8s-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 16px;
}

.k8s-card-icon {
  width: 56px;
  height: 56px;
  background: var(--primary);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.k8s-card-icon i {
  font-size: 24px;
  color: #fff;
}

.k8s-card-content {
  flex: 1;
}

.k8s-card-label {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.k8s-card-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
}

.k8s-card-sub {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

/* Nodes Grid */
.k8s-nodes-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 16px;
}

.k8s-node-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 16px;
  transition: all 0.2s;
}

.k8s-node-card.node-ready {
  border-left: 4px solid var(--success);
}

.k8s-node-card.node-not-ready {
  border-left: 4px solid var(--danger);
}

.k8s-node-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  transform: translateY(-2px);
}

.k8s-node-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.k8s-node-name {
  font-weight: 600;
  font-size: 16px;
  color: var(--text-primary);
}

.k8s-node-body {
  margin-bottom: 16px;
}

.k8s-node-info {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.k8s-node-info div {
  margin-bottom: 4px;
}

.k8s-node-info i {
  width: 20px;
  color: var(--text-muted);
}

.k8s-node-resources {
  background: var(--bg-tertiary);
  border-radius: 6px;
  padding: 12px;
}

.k8s-resource-row {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  margin-bottom: 6px;
}

.k8s-resource-row:last-child {
  margin-bottom: 0;
}

.k8s-node-actions {
  display: flex;
  gap: 8px;
}

/* Namespace Picker */
.namespace-picker {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 14px;
  min-width: 200px;
  cursor: pointer;
}

.namespace-picker:focus {
  outline: none;
  border-color: var(--primary);
}

/* Status Colors */
.status-running {
  color: var(--success);
}

.status-pending {
  color: var(--warning);
}

.status-failed {
  color: var(--danger);
}

/* Charts Placeholder */
.k8s-charts-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 16px;
}

.k8s-chart-placeholder {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 40px;
  text-align: center;
  color: var(--text-secondary);
}

.k8s-chart-placeholder i {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

/* Resource Table */
.k8s-table {
  width: 100%;
  border-collapse: collapse;
}

.k8s-table th,
.k8s-table td {
  padding: 12px;
  text-align: left;
  border-bottom: 1px solid var(--border-color);
}

.k8s-table th {
  background: var(--bg-tertiary);
  font-weight: 600;
  font-size: 13px;
  color: var(--text-secondary);
}

.k8s-table tr:hover {
  background: var(--bg-tertiary);
}

.k8s-table .actions {
  display: flex;
  gap: 8px;
}

/* Badges */
.k8s-card .badge {
  font-size: 12px;
  padding: 4px 8px;
}

/* Loading States */
.k8s-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
  color: var(--text-secondary);
}

/* Empty States */
.k8s-empty {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
}

.k8s-empty i {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

/* Responsive */
@media (max-width: 768px) {
  .k8s-cards-grid {
    grid-template-columns: 1fr;
  }

  .k8s-nodes-grid {
    grid-template-columns: 1fr;
  }

  .k8s-charts-grid {
    grid-template-columns: 1fr;
  }
}
```

---

## ✅ Критерии приемки Фазы 1

- [ ] Подключение к Kubernetes кластеру работает
- [ ] Health check endpoint возвращает статус
- [ ] Список namespace'ов отображается
- [ ] CRUD операции для namespace работают
- [ ] Cluster overview страница отображается
- [ ] Nodes grid с метриками показывается
- [ ] Namespace picker компонент работает
- [ ] CSS стили применены
- [ ] Ошибки обрабатываются корректно
- [ ] Логирование операций настроено

---

## 📊 Метрики Фазы 1

| Метрика | Целевое значение |
|---------|------------------|
| Время подключения к кластеру | < 2 сек |
| Время загрузки namespace'ов | < 500 ms |
| Время отклика API | < 100 ms |
| Покрытие тестами | > 80% |
