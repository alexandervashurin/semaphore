//! Health check handlers для Kubernetes
//!
//! Handlers для проверки здоровья подключения к Kubernetes кластеру

use crate::api::state::AppState;
use crate::error::Result;
use axum::{extract::State, Json};
use kube::api::ListParams;
use std::sync::Arc;

use super::types::KubernetesHealth;

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
                    let nodes_count = match client
                        .api_all::<k8s_openapi::api::core::v1::Node>()
                        .list(&Default::default())
                        .await
                    {
                        Ok(nodes) => Some(nodes.items.len() as i32),
                        Err(_) => None,
                    };

                    // Получаем версию кластера
                    let version = match client.raw().apiserver_version().await {
                        Ok(v) => Some(v.git_version),
                        Err(_) => None,
                    };

                    Ok(Json(KubernetesHealth {
                        connected: true,
                        cluster_name: Some("default".to_string()),
                        kubernetes_version: version,
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

/// Detailed health check с проверкой всех компонентов
/// GET /api/kubernetes/health/detailed
pub async fn kubernetes_health_detailed(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>> {
    let mut checks = serde_json::json!({
        "connected": false,
        "checks": {}
    });

    match state.kubernetes_client() {
        Ok(client) => {
            // Проверка подключения
            let api_check = match client.check_connection().await {
                Ok(_) => {
                    checks["checks"]["api_server"] = serde_json::json!({
                        "status": "healthy",
                        "message": "API server is reachable"
                    });
                    true
                }
                Err(e) => {
                    checks["checks"]["api_server"] = serde_json::json!({
                        "status": "unhealthy",
                        "message": e.to_string()
                    });
                    false
                }
            };

            if api_check {
                // Проверка списка узлов
                match client
                    .api_all::<k8s_openapi::api::core::v1::Node>()
                    .list(&ListParams::default().limit(1))
                    .await
                {
                    Ok(nodes) => {
                        checks["checks"]["nodes"] = serde_json::json!({
                            "status": "healthy",
                            "message": format!("{} nodes accessible", nodes.items.len())
                        });
                    }
                    Err(e) => {
                        checks["checks"]["nodes"] = serde_json::json!({
                            "status": "unhealthy",
                            "message": e.to_string()
                        });
                    }
                }

                // Проверка списка namespace'ов
                match client
                    .api_all::<k8s_openapi::api::core::v1::Namespace>()
                    .list(&ListParams::default().limit(1))
                    .await
                {
                    Ok(namespaces) => {
                        checks["checks"]["namespaces"] = serde_json::json!({
                            "status": "healthy",
                            "message": format!("{} namespaces accessible", namespaces.items.len())
                        });
                    }
                    Err(e) => {
                        checks["checks"]["namespaces"] = serde_json::json!({
                            "status": "unhealthy",
                            "message": e.to_string()
                        });
                    }
                }

                checks["connected"] = serde_json::json!(true);
            }
        }
        Err(e) => {
            checks["checks"]["client"] = serde_json::json!({
                "status": "unhealthy",
                "message": e.to_string()
            });
        }
    }

    Ok(Json(checks))
}
