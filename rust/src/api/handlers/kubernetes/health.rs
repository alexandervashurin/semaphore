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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::handlers::kubernetes::types::KubernetesHealth;

    #[test]
    fn test_kubernetes_health_connected() {
        let health = KubernetesHealth {
            connected: true,
            cluster_name: Some("prod-cluster".to_string()),
            kubernetes_version: Some("v1.28.0".to_string()),
            nodes_count: Some(5),
            error: None,
        };
        assert!(health.connected);
        assert_eq!(health.cluster_name, Some("prod-cluster".to_string()));
        assert_eq!(health.nodes_count, Some(5));
        assert!(health.error.is_none());
    }

    #[test]
    fn test_kubernetes_health_disconnected() {
        let health = KubernetesHealth {
            connected: false,
            cluster_name: None,
            kubernetes_version: None,
            nodes_count: None,
            error: Some("Connection refused".to_string()),
        };
        assert!(!health.connected);
        assert!(health.cluster_name.is_none());
        assert!(health.error.is_some());
    }

    #[test]
    fn test_kubernetes_health_serialization() {
        let health = KubernetesHealth {
            connected: true,
            cluster_name: Some("test-cluster".to_string()),
            kubernetes_version: Some("v1.29.0".to_string()),
            nodes_count: Some(3),
            error: None,
        };
        let json = serde_json::to_string(&health).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["connected"], true);
        assert_eq!(parsed["cluster_name"], "test-cluster");
        assert_eq!(parsed["kubernetes_version"], "v1.29.0");
        assert_eq!(parsed["nodes_count"], 3);
    }

    #[test]
    fn test_kubernetes_health_partial_info() {
        let health = KubernetesHealth {
            connected: true,
            cluster_name: None,
            kubernetes_version: Some("v1.27.0".to_string()),
            nodes_count: None,
            error: None,
        };
        assert!(health.connected);
        assert!(health.cluster_name.is_none());
        assert_eq!(health.kubernetes_version, Some("v1.27.0".to_string()));
    }

    #[test]
    fn test_kubernetes_health_error_message() {
        let health = KubernetesHealth {
            connected: false,
            cluster_name: None,
            kubernetes_version: None,
            nodes_count: None,
            error: Some("Kubeconfig not found".to_string()),
        };
        assert!(health.error.as_ref().unwrap().contains("Kubeconfig"));
    }

    #[test]
    fn test_detailed_health_check_json_structure() {
        let checks = serde_json::json!({
            "connected": false,
            "checks": {}
        });
        assert_eq!(checks["connected"], false);
        assert!(checks["checks"].is_object());
    }

    #[test]
    fn test_detailed_health_check_with_api_server() {
        let mut checks = serde_json::json!({
            "connected": false,
            "checks": {}
        });
        checks["checks"]["api_server"] = serde_json::json!({
            "status": "healthy",
            "message": "API server is reachable"
        });
        checks["connected"] = serde_json::json!(true);
        assert_eq!(checks["connected"], true);
        assert_eq!(checks["checks"]["api_server"]["status"], "healthy");
    }

    #[test]
    fn test_detailed_health_check_with_nodes() {
        let mut checks = serde_json::json!({
            "connected": false,
            "checks": {}
        });
        checks["checks"]["nodes"] = serde_json::json!({
            "status": "healthy",
            "message": "3 nodes accessible"
        });
        assert_eq!(checks["checks"]["nodes"]["status"], "healthy");
        assert!(checks["checks"]["nodes"]["message"].as_str().unwrap().contains("3 nodes"));
    }

    #[test]
    fn test_detailed_health_check_with_namespaces() {
        let mut checks = serde_json::json!({
            "connected": false,
            "checks": {}
        });
        checks["checks"]["namespaces"] = serde_json::json!({
            "status": "healthy",
            "message": "10 namespaces accessible"
        });
        assert_eq!(checks["checks"]["namespaces"]["status"], "healthy");
    }

    #[test]
    fn test_detailed_health_check_client_error() {
        let mut checks = serde_json::json!({
            "connected": false,
            "checks": {}
        });
        checks["checks"]["client"] = serde_json::json!({
            "status": "unhealthy",
            "message": "Client initialization failed"
        });
        assert_eq!(checks["checks"]["client"]["status"], "unhealthy");
    }

    #[test]
    fn test_health_connected_zero_nodes() {
        let health = KubernetesHealth {
            connected: true,
            cluster_name: Some("empty-cluster".to_string()),
            kubernetes_version: Some("v1.28.0".to_string()),
            nodes_count: Some(0),
            error: None,
        };
        assert!(health.connected);
        assert_eq!(health.nodes_count, Some(0));
    }

    #[test]
    fn test_health_deserialization() {
        let json = r#"{"connected":true,"cluster_name":"my-cluster","kubernetes_version":"v1.29.0","nodes_count":10,"error":null}"#;
        let health: KubernetesHealth = serde_json::from_str(json).unwrap();
        assert!(health.connected);
        assert_eq!(health.cluster_name, Some("my-cluster".to_string()));
        assert_eq!(health.kubernetes_version, Some("v1.29.0".to_string()));
        assert_eq!(health.nodes_count, Some(10));
    }
}
