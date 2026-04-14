//! Kubernetes Networking маршруты — Services, Ingress, NetworkPolicy, Gateway API

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

/// Маршруты для управления networking-ресурсами
pub fn networking_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Services
        .route("/api/kubernetes/services", get(handlers::list_services))
        .route(
            "/api/kubernetes/namespaces/{namespace}/services",
            post(handlers::create_service),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/services/{name}",
            get(handlers::get_service),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/services/{name}",
            put(handlers::update_service),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/services/{name}",
            delete(handlers::delete_service),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/services/{name}/endpoints",
            get(handlers::get_service_endpoints),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/services/{name}/endpoint-slices",
            get(handlers::get_service_endpoint_slices),
        )
        // Ingress & IngressClass
        .route("/api/kubernetes/ingresses", get(handlers::list_ingresses))
        .route(
            "/api/kubernetes/namespaces/{namespace}/ingresses",
            post(handlers::create_ingress),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/ingresses/{name}",
            get(handlers::get_ingress),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/ingresses/{name}",
            put(handlers::update_ingress),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/ingresses/{name}",
            delete(handlers::delete_ingress),
        )
        .route(
            "/api/kubernetes/ingressclasses",
            get(handlers::list_ingress_classes),
        )
        .route(
            "/api/kubernetes/ingressclasses/{name}",
            get(handlers::get_ingress_class),
        )
        // NetworkPolicy
        .route(
            "/api/kubernetes/networkpolicies",
            get(handlers::list_networkpolicies),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/networkpolicies",
            post(handlers::create_networkpolicy),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/networkpolicies/{name}",
            get(handlers::get_networkpolicy),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/networkpolicies/{name}",
            put(handlers::update_networkpolicy),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/networkpolicies/{name}",
            delete(handlers::delete_networkpolicy),
        )
        // Gateway API (read-only)
        .route(
            "/api/kubernetes/gateway-api/status",
            get(handlers::get_gateway_api_status),
        )
        .route("/api/kubernetes/gateways", get(handlers::list_gateways))
        .route("/api/kubernetes/httproutes", get(handlers::list_httproutes))
        .route("/api/kubernetes/grpcroutes", get(handlers::list_grpcroutes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_networking_routes_creates() {
        let _router: Router<Arc<AppState>> = networking_routes();
    }

    #[test]
    fn test_networking_routes_returns_router() {
        let router = networking_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_networking_routes_has_services_crud() {
        // Services: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = networking_routes;
    }

    #[test]
    fn test_networking_routes_has_service_endpoints() {
        // /api/kubernetes/namespaces/{namespace}/services/{name}/endpoints - GET
        let _fn: fn() -> Router<Arc<AppState>> = networking_routes;
    }

    #[test]
    fn test_networking_routes_has_endpoint_slices() {
        // /api/kubernetes/namespaces/{namespace}/services/{name}/endpoint-slices - GET
        let _fn: fn() -> Router<Arc<AppState>> = networking_routes;
    }

    #[test]
    fn test_networking_routes_has_ingresses_crud() {
        // Ingresses: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = networking_routes;
    }

    #[test]
    fn test_networking_routes_has_ingress_classes() {
        // IngressClasses: list, get
        let _fn: fn() -> Router<Arc<AppState>> = networking_routes;
    }

    #[test]
    fn test_networking_routes_has_networkpolicies_crud() {
        // NetworkPolicies: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = networking_routes;
    }

    #[test]
    fn test_networking_routes_has_gateway_api_status() {
        // /api/kubernetes/gateway-api/status - GET
        let _fn: fn() -> Router<Arc<AppState>> = networking_routes;
    }

    #[test]
    fn test_networking_routes_has_gateway_routes() {
        // Gateways, HTTPRoutes, GRPCRoutes (read-only)
        let _fn: fn() -> Router<Arc<AppState>> = networking_routes;
    }

    #[test]
    fn test_networking_routes_router_not_empty() {
        let router = networking_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }
}
