//! Kubernetes RBAC UX helpers
//!
//! SelfSubjectAccessReview с кэшированием (5 минут) для производительности

use axum::{extract::{State, Path, Query}, Json};
use k8s_openapi::api::authorization::v1::{
    ResourceAttributes, SelfSubjectAccessReview, SelfSubjectAccessReviewSpec,
};
use kube::api::{Api, PostParams};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

use crate::api::state::AppState;
use crate::error::{Error, Result};
use once_cell::sync::OnceCell;

// ── RBAC Cache ────────────────────────────────────────────────────

/// Кэш для RBAC проверок (TTL 5 минут)
pub struct RbacCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    ttl: Duration,
}

struct CacheEntry {
    allowed: bool,
    expires: Instant,
}

impl RbacCache {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            ttl: Duration::from_secs(300), // 5 минут
        }
    }

    async fn get(&self, key: &str) -> Option<bool> {
        let entries = self.entries.read().await;
        entries.get(key).filter(|e| e.expires > Instant::now()).map(|e| e.allowed)
    }

    async fn set(&self, key: String, allowed: bool) {
        let mut entries = self.entries.write().await;
        entries.insert(key, CacheEntry {
            allowed,
            expires: Instant::now() + self.ttl,
        });
    }

    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }
}

impl Default for RbacCache {
    fn default() -> Self {
        Self::new()
    }
}

// Глобальный кэш с безопасной инициализацией
static RBAC_CACHE: OnceCell<Arc<RbacCache>> = OnceCell::new();

fn get_rbac_cache() -> Arc<RbacCache> {
    RBAC_CACHE
        .get_or_init(|| Arc::new(RbacCache::new()))
        .clone()
}

#[derive(Debug, Deserialize)]
pub struct RbacCheckRequest {
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RbacVerbMatrix {
    pub get: bool,
    pub list: bool,
    pub watch: bool,
    pub create: bool,
    pub update: bool,
    pub patch: bool,
    pub delete: bool,
}

#[derive(Debug, Serialize)]
pub struct RbacResourceCheck {
    pub resource: String,
    pub namespaced: bool,
    pub verbs: RbacVerbMatrix,
}

#[derive(Debug, Serialize)]
pub struct SecretAccessHints {
    pub has_get: bool,
    pub has_list: bool,
    pub has_watch: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RbacCheckResponse {
    pub namespace: Option<String>,
    pub resources: Vec<RbacResourceCheck>,
    pub secrets_hints: SecretAccessHints,
}

async fn can_i(
    api: &Api<SelfSubjectAccessReview>,
    group: &str,
    resource: &str,
    verb: &str,
    namespace: Option<&str>,
) -> Result<bool> {
    // Создаём ключ для кэша
    let cache_key = format!(
        "{}:{}:{}:{}",
        namespace.unwrap_or("_cluster"),
        group,
        resource,
        verb
    );

    // Проверяем кэш
    let cache = get_rbac_cache();
    if let Some(allowed) = cache.get(&cache_key).await {
        return Ok(allowed);
    }

    // Выполняем запрос к API
    let review = SelfSubjectAccessReview {
        metadata: Default::default(),
        spec: SelfSubjectAccessReviewSpec {
            non_resource_attributes: None,
            resource_attributes: Some(ResourceAttributes {
                group: if group.is_empty() {
                    None
                } else {
                    Some(group.to_string())
                },
                name: None,
                namespace: namespace.map(str::to_string),
                resource: Some(resource.to_string()),
                subresource: None,
                verb: Some(verb.to_string()),
                version: None,
            }),
        },
        status: None,
    };

    let created = api
        .create(&PostParams::default(), &review)
        .await
        .map_err(|e| Error::Kubernetes(format!("RBAC SelfSubjectAccessReview failed: {e}")))?;

    let allowed = created
        .status
        .as_ref()
        .map(|s| s.allowed)
        .unwrap_or(false);

    // Сохраняем в кэш
    cache.set(cache_key, allowed).await;

    Ok(allowed)
}

async fn check_resource(
    api: &Api<SelfSubjectAccessReview>,
    group: &str,
    resource: &str,
    namespaced: bool,
    namespace: Option<&str>,
) -> Result<RbacResourceCheck> {
    let ns = if namespaced { namespace } else { None };
    Ok(RbacResourceCheck {
        resource: resource.to_string(),
        namespaced,
        verbs: RbacVerbMatrix {
            get: can_i(api, group, resource, "get", ns).await?,
            list: can_i(api, group, resource, "list", ns).await?,
            watch: can_i(api, group, resource, "watch", ns).await?,
            create: can_i(api, group, resource, "create", ns).await?,
            update: can_i(api, group, resource, "update", ns).await?,
            patch: can_i(api, group, resource, "patch", ns).await?,
            delete: can_i(api, group, resource, "delete", ns).await?,
        },
    })
}

pub async fn check_kubernetes_rbac(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RbacCheckRequest>,
) -> Result<Json<RbacCheckResponse>> {
    let client = state.kubernetes_client()?;
    let review_api: Api<SelfSubjectAccessReview> = Api::all(client.raw().clone());
    let ns = payload.namespace.as_deref();

    let mut resources = Vec::new();
    resources.push(check_resource(&review_api, "", "namespaces", false, ns).await?);
    resources.push(check_resource(&review_api, "", "services", true, ns).await?);
    resources.push(check_resource(&review_api, "", "configmaps", true, ns).await?);
    resources.push(check_resource(&review_api, "", "secrets", true, ns).await?);
    resources.push(check_resource(&review_api, "", "persistentvolumeclaims", true, ns).await?);
    resources.push(check_resource(&review_api, "", "persistentvolumes", false, ns).await?);
    resources.push(check_resource(&review_api, "storage.k8s.io", "storageclasses", false, ns).await?);
    resources.push(check_resource(&review_api, "networking.k8s.io", "ingresses", true, ns).await?);
    resources.push(check_resource(&review_api, "networking.k8s.io", "networkpolicies", true, ns).await?);
    resources.push(check_resource(&review_api, "networking.k8s.io", "ingressclasses", false, ns).await?);
    resources.push(check_resource(&review_api, "batch", "jobs", true, ns).await?);
    resources.push(check_resource(&review_api, "batch", "cronjobs", true, ns).await?);
    resources.push(check_resource(&review_api, "policy", "poddisruptionbudgets", true, ns).await?);
    resources.push(
        check_resource(
            &review_api,
            "scheduling.k8s.io",
            "priorityclasses",
            false,
            ns,
        )
        .await?,
    );
    resources.push(check_resource(&review_api, "", "serviceaccounts", true, ns).await?);
    resources.push(
        check_resource(
            &review_api,
            "rbac.authorization.k8s.io",
            "roles",
            true,
            ns,
        )
        .await?,
    );
    resources.push(
        check_resource(
            &review_api,
            "rbac.authorization.k8s.io",
            "rolebindings",
            true,
            ns,
        )
        .await?,
    );
    resources.push(
        check_resource(
            &review_api,
            "rbac.authorization.k8s.io",
            "clusterroles",
            false,
            ns,
        )
        .await?,
    );
    resources.push(
        check_resource(
            &review_api,
            "rbac.authorization.k8s.io",
            "clusterrolebindings",
            false,
            ns,
        )
        .await?,
    );
    resources.push(
        check_resource(
            &review_api,
            "autoscaling",
            "horizontalpodautoscalers",
            true,
            ns,
        )
        .await?,
    );
    resources.push(check_resource(&review_api, "", "resourcequotas", true, ns).await?);
    resources.push(check_resource(&review_api, "", "limitranges", true, ns).await?);
    resources.push(
        check_resource(
            &review_api,
            "apiextensions.k8s.io",
            "customresourcedefinitions",
            false,
            ns,
        )
        .await?,
    );

    let secrets = resources
        .iter()
        .find(|r| r.resource == "secrets")
        .ok_or_else(|| Error::Other("Secrets RBAC check missing".to_string()))?;

    let warning = if secrets.verbs.get && secrets.verbs.list && !secrets.verbs.watch {
        Some("User can get/list secrets but cannot watch; live updates should be disabled and UI should show manual refresh.".to_string())
    } else if (secrets.verbs.get || secrets.verbs.list) && !secrets.verbs.watch {
        Some("Limited secret permissions detected (no watch).".to_string())
    } else {
        None
    };
    let has_get = secrets.verbs.get;
    let has_list = secrets.verbs.list;
    let has_watch = secrets.verbs.watch;

    Ok(Json(RbacCheckResponse {
        namespace: payload.namespace,
        resources,
        secrets_hints: SecretAccessHints {
            has_get,
            has_list,
            has_watch,
            warning,
        },
    }))
}

// ── Quick RBAC Check Endpoint ────────────────────────────────────

/// Запрос на проверку конкретного действия
#[derive(Debug, Deserialize)]
pub struct RbacActionCheck {
    pub namespace: Option<String>,
    pub group: Option<String>,
    pub resource: String,
    pub verb: String,
}

/// Ответ на проверку действия
#[derive(Debug, Serialize)]
pub struct RbacActionResponse {
    pub allowed: bool,
    pub namespace: Option<String>,
    pub group: Option<String>,
    pub resource: String,
    pub verb: String,
    pub cached: bool,
}

/// Быстрая проверка одного действия (resource + verb)
/// GET /api/kubernetes/rbac/check-action?namespace=...&group=...&resource=...&verb=...
pub async fn check_rbac_action(
    State(state): State<Arc<AppState>>,
    Query(payload): Query<RbacActionCheck>,
) -> Result<Json<RbacActionResponse>> {
    let client = state.kubernetes_client()?;
    let review_api: Api<SelfSubjectAccessReview> = Api::all(client.raw().clone());
    
    // Проверяем, был ли результат в кэше
    let cache_key = format!(
        "{}:{}:{}:{}",
        payload.namespace.as_deref().unwrap_or("_cluster"),
        payload.group.as_deref().unwrap_or(""),
        payload.resource,
        payload.verb
    );
    
    let cache = get_rbac_cache();
    let cached = cache.get(&cache_key).await.is_some();
    
    let allowed = can_i(
        &review_api,
        payload.group.as_deref().unwrap_or(""),
        &payload.resource,
        &payload.verb,
        payload.namespace.as_deref(),
    ).await?;

    Ok(Json(RbacActionResponse {
        allowed,
        namespace: payload.namespace,
        group: payload.group,
        resource: payload.resource,
        verb: payload.verb,
        cached,
    }))
}

/// Очистка RBAC кэша
/// POST /api/kubernetes/rbac/cache/clear
pub async fn clear_rbac_cache() -> Result<Json<serde_json::Value>> {
    let cache = get_rbac_cache();
    cache.clear().await;
    Ok(Json(serde_json::json!({"cleared": true, "message": "RBAC cache cleared"})))
}
