//! Kubernetes Multi-Cluster Management API handlers
//!
//! Управление несколькими Kubernetes кластерами

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use kube::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Cluster Contexts
// ============================================================================

#[derive(Debug, Serialize)]
pub struct KubernetesCluster {
    pub name: String,
    pub context: String,
    pub server: String,
    pub version: Option<String>,
    pub is_current: bool,
    pub is_reachable: bool,
    pub namespaces_count: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ClusterList {
    pub clusters: Vec<KubernetesCluster>,
    pub current_context: String,
}

#[derive(Debug, Deserialize)]
pub struct AddClusterRequest {
    pub name: String,
    pub kubeconfig: String, // Base64 encoded kubeconfig или путь к файлу
    pub set_current: Option<bool>,
}

pub async fn list_kubernetes_clusters(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ClusterList>> {
    // В production это читало бы ~/.kube/config
    // Для demo возвращаем mock данные
    
    let clusters = vec![
        KubernetesCluster {
            name: "default".to_string(),
            context: "default".to_string(),
            server: "https://kubernetes.default.svc".to_string(),
            version: Some("v1.28.0".to_string()),
            is_current: true,
            is_reachable: true,
            namespaces_count: Some(5),
        },
        KubernetesCluster {
            name: "production".to_string(),
            context: "production".to_string(),
            server: "https://prod-cluster.example.com:6443".to_string(),
            version: Some("v1.27.0".to_string()),
            is_current: false,
            is_reachable: false,
            namespaces_count: None,
        },
        KubernetesCluster {
            name: "staging".to_string(),
            context: "staging".to_string(),
            server: "https://staging-cluster.example.com:6443".to_string(),
            version: Some("v1.28.0".to_string()),
            is_current: false,
            is_reachable: false,
            namespaces_count: None,
        },
    ];
    
    Ok(Json(ClusterList {
        clusters,
        current_context: "default".to_string(),
    }))
}

pub async fn add_kubernetes_cluster(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddClusterRequest>,
) -> Result<Json<KubernetesCluster>> {
    // Encrypt kubeconfig blob with AES-256-GCM before storing in memory
    let encrypted = encrypt_kubeconfig(&state, &payload.kubeconfig)?;
    state.kubeconfigs.insert(payload.name.clone(), encrypted);

    Ok(Json(KubernetesCluster {
        name: payload.name.clone(),
        context: payload.name.clone(),
        server: "https://new-cluster.example.com:6443".to_string(),
        version: Some("v1.28.0".to_string()),
        is_current: payload.set_current.unwrap_or(false),
        is_reachable: true,
        namespaces_count: Some(0),
    }))
}

/// Encrypts a kubeconfig string with AES-256-GCM using the app encryption key
fn encrypt_kubeconfig(state: &AppState, kubeconfig: &str) -> Result<String> {
    use crate::utils::encryption::aes256_encrypt;
    let key = derive_encryption_key(&state.config);
    aes256_encrypt(kubeconfig.as_bytes(), &key)
        .map_err(|e| Error::Other(format!("Kubeconfig encryption failed: {e}")))
}

/// Decrypts a stored kubeconfig string
#[allow(dead_code)]
fn decrypt_kubeconfig(state: &AppState, encrypted: &str) -> Result<String> {
    use crate::utils::encryption::aes256_decrypt;
    let key = derive_encryption_key(&state.config);
    let bytes = aes256_decrypt(encrypted, &key)
        .map_err(|e| Error::Other(format!("Kubeconfig decryption failed: {e}")))?;
    String::from_utf8(bytes).map_err(|e| Error::Other(format!("Kubeconfig UTF-8 error: {e}")))
}

/// Derives a 32-byte AES key from env var SEMAPHORE_KUBECONFIG_KEY or cookie_hash
fn derive_encryption_key(config: &crate::config::Config) -> [u8; 32] {
    let mut key = [0u8; 32];
    if let Ok(raw) = std::env::var("SEMAPHORE_KUBECONFIG_KEY")
        .or_else(|_| std::env::var("SEMAPHORE_ACCESS_KEY_ENCRYPTION"))
    {
        let b = raw.as_bytes();
        let len = b.len().min(32);
        key[..len].copy_from_slice(&b[..len]);
    } else {
        // Fall back to cookie_hash (Vec<u8>, already 32 bytes or less)
        let len = config.cookie_hash.len().min(32);
        key[..len].copy_from_slice(&config.cookie_hash[..len]);
    }
    key
}

pub async fn switch_kubernetes_cluster(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    // В реальной реализации переключает текущий контекст
    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": format!("Switched to cluster '{}'", name),
        "current_context": name
    })))
}

pub async fn remove_kubernetes_cluster(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<StatusCode> {
    // В реальной реализации удаляет кластер из конфига
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Cluster Health & Summary
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ClusterHealth {
    pub name: String,
    pub is_healthy: bool,
    pub api_server: bool,
    pub etcd: bool,
    pub scheduler: bool,
    pub controller_manager: bool,
    pub nodes_ready: i32,
    pub nodes_total: i32,
    pub pods_running: i32,
    pub error: Option<String>,
}

pub async fn get_cluster_health(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ClusterHealth>> {
    // В production здесь была бы полная проверка компонентов
    // Для demo возвращаем mock данные
    let is_healthy = true; // state.kubernetes_client().is_ok();
    
    Ok(Json(ClusterHealth {
        name: "default".to_string(),
        is_healthy,
        api_server: is_healthy,
        etcd: is_healthy,
        scheduler: is_healthy,
        controller_manager: is_healthy,
        nodes_ready: 3,
        nodes_total: 3,
        pods_running: 25,
        error: if !is_healthy { Some("API server unreachable".to_string()) } else { None },
    }))
}

#[derive(Debug, Serialize)]
pub struct ClusterSummary {
    pub name: String,
    pub version: Option<String>,
    pub nodes: NodeSummary,
    pub resources: ResourceSummary,
}

#[derive(Debug, Serialize)]
pub struct NodeSummary {
    pub total: i32,
    pub ready: i32,
    pub not_ready: i32,
}

#[derive(Debug, Serialize)]
pub struct ResourceSummary {
    pub pods: ResourceCount,
    pub deployments: ResourceCount,
    pub services: ResourceCount,
    pub configmaps: ResourceCount,
    pub secrets: ResourceCount,
}

#[derive(Debug, Serialize)]
pub struct ResourceCount {
    pub total: i32,
    pub running: i32,
}

pub async fn get_k8s_cluster_summary(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ClusterSummary>> {
    // Mock данные для summary
    Ok(Json(ClusterSummary {
        name: "default".to_string(),
        version: Some("v1.28.0".to_string()),
        nodes: NodeSummary {
            total: 3,
            ready: 3,
            not_ready: 0,
        },
        resources: ResourceSummary {
            pods: ResourceCount { total: 50, running: 45 },
            deployments: ResourceCount { total: 15, running: 15 },
            services: ResourceCount { total: 20, running: 20 },
            configmaps: ResourceCount { total: 30, running: 30 },
            secrets: ResourceCount { total: 25, running: 25 },
        },
    }))
}

// ============================================================================
// Aggregate View (All Clusters)
// ============================================================================

#[derive(Debug, Serialize)]
pub struct AggregateView {
    pub total_clusters: i32,
    pub healthy_clusters: i32,
    pub total_nodes: i32,
    pub total_pods: i32,
    pub total_deployments: i32,
    pub clusters: Vec<ClusterStatus>,
}

#[derive(Debug, Serialize)]
pub struct ClusterStatus {
    pub name: String,
    pub status: String,
    pub nodes: i32,
    pub pods: i32,
}

pub async fn get_aggregate_view(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<AggregateView>> {
    let is_healthy = true;
    
    Ok(Json(AggregateView {
        total_clusters: 3,
        healthy_clusters: if is_healthy { 1 } else { 0 },
        total_nodes: 9,
        total_pods: 150,
        total_deployments: 45,
        clusters: vec![
            ClusterStatus {
                name: "default".to_string(),
                status: if is_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
                nodes: 3,
                pods: 50,
            },
            ClusterStatus {
                name: "production".to_string(),
                status: "unknown".to_string(),
                nodes: 5,
                pods: 80,
            },
            ClusterStatus {
                name: "staging".to_string(),
                status: "unknown".to_string(),
                nodes: 1,
                pods: 20,
            },
        ],
    }))
}
