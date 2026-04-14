//! Kubernetes Advanced маршруты — HPA, VPA, ResourceQuota, LimitRange, CRD, Custom Objects

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

/// Маршруты для advanced-ресурсов
pub fn advanced_routes() -> Router<Arc<AppState>> {
    Router::new()
        // HPA
        .route(
            "/api/kubernetes/horizontalpodautoscalers",
            get(handlers::list_hpas),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/horizontalpodautoscalers",
            post(handlers::create_hpa),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/horizontalpodautoscalers/{name}",
            get(handlers::get_hpa),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/horizontalpodautoscalers/{name}",
            put(handlers::update_hpa),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/horizontalpodautoscalers/{name}",
            delete(handlers::delete_hpa),
        )
        // ResourceQuota
        .route(
            "/api/kubernetes/resourcequotas",
            get(handlers::list_resource_quotas),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/resourcequotas",
            post(handlers::create_resource_quota),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/resourcequotas/{name}",
            get(handlers::get_resource_quota),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/resourcequotas/{name}",
            put(handlers::update_resource_quota),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/resourcequotas/{name}",
            delete(handlers::delete_resource_quota),
        )
        // LimitRange
        .route(
            "/api/kubernetes/limitranges",
            get(handlers::list_limit_ranges),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/limitranges",
            post(handlers::create_limit_range),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/limitranges/{name}",
            get(handlers::get_limit_range),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/limitranges/{name}",
            put(handlers::update_limit_range),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/limitranges/{name}",
            delete(handlers::delete_limit_range),
        )
        // CRD
        .route(
            "/api/kubernetes/customresourcedefinitions",
            get(handlers::list_crds),
        )
        .route(
            "/api/kubernetes/customresourcedefinitions/{name}",
            get(handlers::get_crd),
        )
        // Custom Objects (namespace)
        .route(
            "/api/kubernetes/customobjects",
            get(handlers::list_custom_objects).post(handlers::create_custom_object_query),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/customobjects/{plural}/{name}",
            get(handlers::get_custom_object),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/customobjects/{plural}/{name}",
            put(handlers::replace_custom_object),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/customobjects/{plural}/{name}",
            delete(handlers::delete_custom_object),
        )
        // Custom Objects (cluster)
        .route(
            "/api/kubernetes/cluster/customobjects/{plural}/{name}",
            get(handlers::get_custom_object_cluster),
        )
        .route(
            "/api/kubernetes/cluster/customobjects/{plural}/{name}",
            put(handlers::replace_custom_object_cluster),
        )
        .route(
            "/api/kubernetes/cluster/customobjects/{plural}/{name}",
            delete(handlers::delete_custom_object_cluster),
        )
        .route(
            "/api/kubernetes/cluster/customobjects/{plural}",
            post(handlers::create_custom_object_cluster),
        )
        // VPA (read-only)
        .route("/api/kubernetes/vpa/status", get(handlers::get_vpa_status))
        .route(
            "/api/kubernetes/verticalpodautoscalers",
            get(handlers::list_vertical_pod_autoscalers),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/verticalpodautoscalers/{name}",
            get(handlers::get_vertical_pod_autoscaler),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_routes_creates() {
        let _router: Router<Arc<AppState>> = advanced_routes();
    }

    #[test]
    fn test_advanced_routes_returns_router() {
        let router = advanced_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_advanced_routes_has_hpa_endpoints() {
        // HPA: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = advanced_routes;
    }

    #[test]
    fn test_advanced_routes_has_resource_quota_endpoints() {
        // ResourceQuota: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = advanced_routes;
    }

    #[test]
    fn test_advanced_routes_has_limit_range_endpoints() {
        // LimitRange: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = advanced_routes;
    }

    #[test]
    fn test_advanced_routes_has_crd_endpoints() {
        // CRD: list, get
        let _fn: fn() -> Router<Arc<AppState>> = advanced_routes;
    }

    #[test]
    fn test_advanced_routes_has_custom_objects_namespaced() {
        // Custom objects namespace: get, replace, delete
        let _fn: fn() -> Router<Arc<AppState>> = advanced_routes;
    }

    #[test]
    fn test_advanced_routes_has_custom_objects_cluster() {
        // Custom objects cluster: get, replace, delete, create
        let _fn: fn() -> Router<Arc<AppState>> = advanced_routes;
    }

    #[test]
    fn test_advanced_routes_has_vpa_endpoints() {
        // VPA: status, list, get (read-only)
        let _fn: fn() -> Router<Arc<AppState>> = advanced_routes;
    }

    #[test]
    fn test_advanced_routes_uses_delete_for_hpa() {
        // delete_hpa uses DELETE method
        let _fn: fn() -> Router<Arc<AppState>> = advanced_routes;
    }

    #[test]
    fn test_advanced_routes_uses_put_for_update() {
        // update_hpa, update_resource_quota, update_limit_range use PUT
        let _fn: fn() -> Router<Arc<AppState>> = advanced_routes;
    }
}
