//! Kubernetes Audit Logging helpers
//!
//! Утилиты для логирования Kubernetes операций в Audit Log

use crate::api::state::AppState;
use crate::db::store::AuditLogManager;
use crate::models::audit_log::{
    AuditAction, AuditLevel, AuditLogFilter, AuditObjectType,
};
use axum::{
    extract::{Query, State},
    http::header,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Helper для создания записей audit log для Kubernetes операций
pub struct KubernetesAuditLogger;

impl KubernetesAuditLogger {
    /// Логирование создания Kubernetes ресурса
    pub async fn log_resource_creation(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        resource_kind: &str,
        resource_name: &str,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesResourceCreated,
            resource_kind,
            resource_name,
            namespace,
            format!("Создан ресурс {resource_kind}/{resource_name} в namespace {namespace}"),
        )
        .await;
    }

    /// Логирование обновления Kubernetes ресурса
    pub async fn log_resource_update(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        resource_kind: &str,
        resource_name: &str,
        namespace: &str,
        changes: Option<&str>,
    ) {
        let description = if let Some(changes_desc) = changes {
            format!("Обновлен ресурс {resource_kind}/{resource_name} в namespace {namespace}: {changes_desc}")
        } else {
            format!("Обновлен ресурс {resource_kind}/{resource_name} в namespace {namespace}")
        };

        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesResourceUpdated,
            resource_kind,
            resource_name,
            namespace,
            description,
        )
        .await;
    }

    /// Логирование удаления Kubernetes ресурса
    pub async fn log_resource_deletion(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        resource_kind: &str,
        resource_name: &str,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesResourceDeleted,
            resource_kind,
            resource_name,
            namespace,
            format!("Удален ресурс {resource_kind}/{resource_name} из namespace {namespace}"),
        )
        .await;
    }

    /// Логирование установки Helm release
    pub async fn log_helm_install(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        release_name: &str,
        chart_name: &str,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesHelmReleaseInstalled,
            "HelmRelease",
            release_name,
            namespace,
            format!("Установлен Helm chart {chart_name} как release {release_name} в namespace {namespace}"),
        )
        .await;
    }

    /// Логирование обновления Helm release
    pub async fn log_helm_upgrade(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        release_name: &str,
        chart_name: &str,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesHelmReleaseUpgraded,
            "HelmRelease",
            release_name,
            namespace,
            format!("Обновлен Helm release {release_name} до chart {chart_name} в namespace {namespace}"),
        )
        .await;
    }

    /// Логирование отката Helm release
    pub async fn log_helm_rollback(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        release_name: &str,
        revision: i32,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesHelmReleaseRolledBack,
            "HelmRelease",
            release_name,
            namespace,
            format!("Выполнен откат Helm release {release_name} к revision {revision} в namespace {namespace}"),
        )
        .await;
    }

    /// Логирование удаления Helm release
    pub async fn log_helm_uninstall(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        release_name: &str,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesHelmReleaseUninstalled,
            "HelmRelease",
            release_name,
            namespace,
            format!("Удален Helm release {release_name} из namespace {namespace}"),
        )
        .await;
    }

    /// Базовый метод логирования
    #[allow(clippy::too_many_arguments)]
    async fn log(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        action: AuditAction,
        resource_kind: &str,
        resource_name: &str,
        namespace: &str,
        description: String,
    ) {
        let object_name = format!("{}/{}", resource_kind, resource_name);

        let _ = state
            .store
            .create_audit_log(
                None, // project_id не применим для Kubernetes
                user_id,
                username,
                &action,
                &AuditObjectType::Kubernetes,
                None, // object_id
                Some(object_name),
                description,
                &AuditLevel::Info,
                None, // ip_address
                None, // user_agent
                Some(serde_json::json!({
                    "resource_kind": resource_kind,
                    "resource_name": resource_name,
                    "namespace": namespace
                })),
            )
            .await;
    }
}

#[derive(Debug, Deserialize)]
pub struct KubernetesAuditQuery {
    pub username: Option<String>,
    pub resource: Option<String>,
    pub verb: Option<String>,
    pub namespace: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct KubernetesAuditExportQuery {
    pub format: Option<String>, // json | csv
    pub username: Option<String>,
    pub resource: Option<String>,
    pub verb: Option<String>,
    pub namespace: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct KubernetesAuditRow {
    pub id: i64,
    pub created: chrono::DateTime<chrono::Utc>,
    pub username: Option<String>,
    pub cluster: Option<String>,
    pub namespace: Option<String>,
    pub resource: Option<String>,
    pub resource_name: Option<String>,
    pub verb: String,
    pub action: String,
    pub description: String,
    pub level: String,
}

fn action_to_verb(action: &AuditAction) -> String {
    match action {
        AuditAction::KubernetesResourceCreated | AuditAction::KubernetesHelmReleaseInstalled => {
            "create".to_string()
        }
        AuditAction::KubernetesResourceUpdated
        | AuditAction::KubernetesResourceScaled
        | AuditAction::KubernetesHelmReleaseUpgraded
        | AuditAction::KubernetesHelmReleaseRolledBack => "update".to_string(),
        AuditAction::KubernetesResourceDeleted | AuditAction::KubernetesHelmReleaseUninstalled => {
            "delete".to_string()
        }
        _ => "other".to_string(),
    }
}

fn level_to_str(level: &AuditLevel) -> String {
    match level {
        AuditLevel::Info => "info".to_string(),
        AuditLevel::Warning => "warning".to_string(),
        AuditLevel::Error => "error".to_string(),
        AuditLevel::Critical => "critical".to_string(),
    }
}

fn extract_meta(details: &Option<serde_json::Value>) -> (Option<String>, Option<String>, Option<String>) {
    let meta = details
        .as_ref()
        .and_then(|d| d.get("metadata"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let resource = meta
        .get("resource_kind")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let resource_name = meta
        .get("resource_name")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let namespace = meta
        .get("namespace")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    (resource, resource_name, namespace)
}

fn map_rows(
    logs: &[crate::models::audit_log::AuditLog],
    cluster: Option<String>,
) -> Vec<KubernetesAuditRow> {
    logs.iter()
        .map(|r| {
            let (resource, resource_name, namespace) = extract_meta(&r.details);
            KubernetesAuditRow {
                id: r.id,
                created: r.created,
                username: r.username.clone(),
                cluster: cluster.clone(),
                namespace,
                resource,
                resource_name,
                verb: action_to_verb(&r.action),
                action: r.action.to_string(),
                description: r.description.clone(),
                level: level_to_str(&r.level),
            }
        })
        .collect()
}

fn apply_filters(rows: Vec<KubernetesAuditRow>, q: &KubernetesAuditQuery) -> Vec<KubernetesAuditRow> {
    let mut out = rows;
    if let Some(resource) = &q.resource {
        let r = resource.to_lowercase();
        out.retain(|x| x.resource.clone().unwrap_or_default().to_lowercase().contains(&r));
    }
    if let Some(verb) = &q.verb {
        let v = verb.to_lowercase();
        out.retain(|x| x.verb.to_lowercase() == v);
    }
    if let Some(ns) = &q.namespace {
        let n = ns.to_lowercase();
        out.retain(|x| x.namespace.clone().unwrap_or_default().to_lowercase() == n);
    }
    if let Some(search) = &q.search {
        let s = search.to_lowercase();
        out.retain(|x| {
            x.description.to_lowercase().contains(&s)
                || x.resource.clone().unwrap_or_default().to_lowercase().contains(&s)
                || x.resource_name.clone().unwrap_or_default().to_lowercase().contains(&s)
                || x.namespace.clone().unwrap_or_default().to_lowercase().contains(&s)
                || x.username.clone().unwrap_or_default().to_lowercase().contains(&s)
        });
    }
    out
}

/// GET /api/kubernetes/audit
pub async fn list_kubernetes_audit(
    State(state): State<Arc<AppState>>,
    Query(query): Query<KubernetesAuditQuery>,
) -> crate::error::Result<Json<Vec<KubernetesAuditRow>>> {
    let filter = AuditLogFilter {
        username: query.username.clone(),
        object_type: Some(AuditObjectType::Kubernetes),
        search: query.search.clone(),
        limit: query.limit.unwrap_or(200),
        offset: query.offset.unwrap_or(0),
        sort: "created".to_string(),
        order: "desc".to_string(),
        ..Default::default()
    };

    let result = state
        .store
        .search_audit_logs(&filter)
        .await
        .map_err(|e| crate::error::Error::Other(e.to_string()))?;

    let cluster = state
        .config
        .kubernetes
        .as_ref()
        .and_then(|k| k.context.clone());
    let rows = map_rows(&result.records, cluster);
    Ok(Json(apply_filters(rows, &query)))
}

/// GET /api/kubernetes/audit/export?format=csv|json
pub async fn export_kubernetes_audit(
    State(state): State<Arc<AppState>>,
    Query(query): Query<KubernetesAuditExportQuery>,
) -> crate::error::Result<impl IntoResponse> {
    let list_query = KubernetesAuditQuery {
        username: query.username.clone(),
        resource: query.resource.clone(),
        verb: query.verb.clone(),
        namespace: query.namespace.clone(),
        search: query.search.clone(),
        limit: query.limit,
        offset: query.offset,
    };
    let Json(rows) = list_kubernetes_audit(State(state), Query(list_query)).await?;

    let format = query.format.unwrap_or_else(|| "json".to_string()).to_lowercase();
    if format == "csv" {
        let mut csv = String::from("id,created,username,cluster,namespace,resource,resource_name,verb,action,level,description\n");
        for r in rows {
            let line = format!(
                "{},{},{},{},{},{},{},{},{},{},\"{}\"\n",
                r.id,
                r.created.to_rfc3339(),
                r.username.unwrap_or_default().replace(',', " "),
                r.cluster.unwrap_or_default().replace(',', " "),
                r.namespace.unwrap_or_default().replace(',', " "),
                r.resource.unwrap_or_default().replace(',', " "),
                r.resource_name.unwrap_or_default().replace(',', " "),
                r.verb,
                r.action,
                r.level,
                r.description.replace('"', "'").replace('\n', " "),
            );
            csv.push_str(&line);
        }
        Ok((
            [(header::CONTENT_TYPE, "text/csv; charset=utf-8")],
            csv,
        )
            .into_response())
    } else {
        Ok(Json(rows).into_response())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::audit_log::{AuditAction, AuditLevel, AuditLog, AuditObjectType};

    // --- action_to_verb tests ---

    #[test]
    fn test_action_to_verb_create() {
        assert_eq!(action_to_verb(&AuditAction::KubernetesResourceCreated), "create");
        assert_eq!(action_to_verb(&AuditAction::KubernetesHelmReleaseInstalled), "create");
    }

    #[test]
    fn test_action_to_verb_update() {
        assert_eq!(action_to_verb(&AuditAction::KubernetesResourceUpdated), "update");
        assert_eq!(action_to_verb(&AuditAction::KubernetesResourceScaled), "update");
        assert_eq!(action_to_verb(&AuditAction::KubernetesHelmReleaseUpgraded), "update");
        assert_eq!(action_to_verb(&AuditAction::KubernetesHelmReleaseRolledBack), "update");
    }

    #[test]
    fn test_action_to_verb_delete() {
        assert_eq!(action_to_verb(&AuditAction::KubernetesResourceDeleted), "delete");
        assert_eq!(action_to_verb(&AuditAction::KubernetesHelmReleaseUninstalled), "delete");
    }

    #[test]
    fn test_action_to_verb_other() {
        assert_eq!(action_to_verb(&AuditAction::Login), "other");
        assert_eq!(action_to_verb(&AuditAction::Logout), "other");
        assert_eq!(action_to_verb(&AuditAction::UserCreated), "other");
    }

    // --- level_to_str tests ---

    #[test]
    fn test_level_to_str_info() {
        assert_eq!(level_to_str(&AuditLevel::Info), "info");
    }

    #[test]
    fn test_level_to_str_warning() {
        assert_eq!(level_to_str(&AuditLevel::Warning), "warning");
    }

    #[test]
    fn test_level_to_str_error() {
        assert_eq!(level_to_str(&AuditLevel::Error), "error");
    }

    #[test]
    fn test_level_to_str_critical() {
        assert_eq!(level_to_str(&AuditLevel::Critical), "critical");
    }

    // --- extract_meta tests ---

    #[test]
    fn test_extract_meta_with_full_metadata() {
        let details = Some(serde_json::json!({
            "metadata": {
                "resource_kind": "Deployment",
                "resource_name": "my-app",
                "namespace": "production"
            }
        }));
        let (resource, resource_name, namespace) = extract_meta(&details);
        assert_eq!(resource, Some("Deployment".to_string()));
        assert_eq!(resource_name, Some("my-app".to_string()));
        assert_eq!(namespace, Some("production".to_string()));
    }

    #[test]
    fn test_extract_meta_with_none_details() {
        let (resource, resource_name, namespace) = extract_meta(&None);
        assert!(resource.is_none());
        assert!(resource_name.is_none());
        assert!(namespace.is_none());
    }

    #[test]
    fn test_extract_meta_with_partial_metadata() {
        let details = Some(serde_json::json!({
            "metadata": {
                "resource_kind": "Service"
            }
        }));
        let (resource, resource_name, namespace) = extract_meta(&details);
        assert_eq!(resource, Some("Service".to_string()));
        assert!(resource_name.is_none());
        assert!(namespace.is_none());
    }

    #[test]
    fn test_extract_meta_ignores_extra_fields() {
        let details = Some(serde_json::json!({
            "metadata": {
                "resource_kind": "Pod",
                "resource_name": "test-pod",
                "namespace": "default",
                "extra_field": "ignored"
            },
            "other": "data"
        }));
        let (resource, resource_name, namespace) = extract_meta(&details);
        assert_eq!(resource, Some("Pod".to_string()));
        assert_eq!(resource_name, Some("test-pod".to_string()));
        assert_eq!(namespace, Some("default".to_string()));
    }

    // --- map_rows tests ---

    #[test]
    fn test_map_rows_empty() {
        let logs: Vec<AuditLog> = vec![];
        let rows = map_rows(&logs, Some("test-cluster".to_string()));
        assert!(rows.is_empty());
    }

    #[test]
    fn test_map_rows_single() {
        let logs = vec![AuditLog {
            id: 1,
            project_id: None,
            user_id: Some(1),
            username: Some("admin".to_string()),
            action: AuditAction::KubernetesResourceCreated,
            object_type: AuditObjectType::Kubernetes,
            object_id: None,
            object_name: Some("Deployment/my-app".to_string()),
            description: "Создан ресурс Deployment/my-app".to_string(),
            level: AuditLevel::Info,
            ip_address: None,
            user_agent: None,
            details: Some(serde_json::json!({
                "metadata": {
                    "resource_kind": "Deployment",
                    "resource_name": "my-app",
                    "namespace": "default"
                }
            })),
            created: chrono::Utc::now(),
        }];
        let rows = map_rows(&logs, Some("test-cluster".to_string()));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, 1);
        assert_eq!(rows[0].username, Some("admin".to_string()));
        assert_eq!(rows[0].cluster, Some("test-cluster".to_string()));
        assert_eq!(rows[0].resource, Some("Deployment".to_string()));
        assert_eq!(rows[0].resource_name, Some("my-app".to_string()));
        assert_eq!(rows[0].namespace, Some("default".to_string()));
        assert_eq!(rows[0].verb, "create");
        assert_eq!(rows[0].level, "info");
    }

    // --- apply_filters tests ---

    #[test]
    fn test_apply_filters_no_filters() {
        let rows = vec![KubernetesAuditRow {
            id: 1,
            created: chrono::Utc::now(),
            username: Some("admin".to_string()),
            cluster: None,
            namespace: Some("default".to_string()),
            resource: Some("Pod".to_string()),
            resource_name: Some("my-pod".to_string()),
            verb: "create".to_string(),
            action: "kubernetes_resource_created".to_string(),
            description: "Created".to_string(),
            level: "info".to_string(),
        }];
        let query = KubernetesAuditQuery {
            username: None,
            resource: None,
            verb: None,
            namespace: None,
            search: None,
            limit: None,
            offset: None,
        };
        let filtered = apply_filters(rows, &query);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_apply_filters_by_resource() {
        let rows = vec![
            KubernetesAuditRow {
                id: 1, created: chrono::Utc::now(), username: None, cluster: None,
                namespace: Some("default".to_string()), resource: Some("Pod".to_string()),
                resource_name: Some("pod-1".to_string()), verb: "create".to_string(),
                action: "kubernetes_resource_created".to_string(), description: "Created".to_string(),
                level: "info".to_string(),
            },
            KubernetesAuditRow {
                id: 2, created: chrono::Utc::now(), username: None, cluster: None,
                namespace: Some("default".to_string()), resource: Some("Service".to_string()),
                resource_name: Some("svc-1".to_string()), verb: "create".to_string(),
                action: "kubernetes_resource_created".to_string(), description: "Created".to_string(),
                level: "info".to_string(),
            },
        ];
        let query = KubernetesAuditQuery {
            username: None,
            resource: Some("pod".to_string()),
            verb: None,
            namespace: None,
            search: None,
            limit: None,
            offset: None,
        };
        let filtered = apply_filters(rows, &query);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_apply_filters_by_verb() {
        let rows = vec![
            KubernetesAuditRow {
                id: 1, created: chrono::Utc::now(), username: None, cluster: None,
                namespace: Some("default".to_string()), resource: Some("Pod".to_string()),
                resource_name: Some("pod-1".to_string()), verb: "create".to_string(),
                action: "kubernetes_resource_created".to_string(), description: "Created".to_string(),
                level: "info".to_string(),
            },
            KubernetesAuditRow {
                id: 2, created: chrono::Utc::now(), username: None, cluster: None,
                namespace: Some("default".to_string()), resource: Some("Pod".to_string()),
                resource_name: Some("pod-2".to_string()), verb: "delete".to_string(),
                action: "kubernetes_resource_deleted".to_string(), description: "Deleted".to_string(),
                level: "info".to_string(),
            },
        ];
        let query = KubernetesAuditQuery {
            username: None,
            resource: None,
            verb: Some("delete".to_string()),
            namespace: None,
            search: None,
            limit: None,
            offset: None,
        };
        let filtered = apply_filters(rows, &query);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 2);
    }

    #[test]
    fn test_apply_filters_by_namespace() {
        let rows = vec![
            KubernetesAuditRow {
                id: 1, created: chrono::Utc::now(), username: None, cluster: None,
                namespace: Some("kube-system".to_string()), resource: Some("Pod".to_string()),
                resource_name: Some("coredns".to_string()), verb: "create".to_string(),
                action: "kubernetes_resource_created".to_string(), description: "Created".to_string(),
                level: "info".to_string(),
            },
            KubernetesAuditRow {
                id: 2, created: chrono::Utc::now(), username: None, cluster: None,
                namespace: Some("default".to_string()), resource: Some("Pod".to_string()),
                resource_name: Some("nginx".to_string()), verb: "create".to_string(),
                action: "kubernetes_resource_created".to_string(), description: "Created".to_string(),
                level: "info".to_string(),
            },
        ];
        let query = KubernetesAuditQuery {
            username: None,
            resource: None,
            verb: None,
            namespace: Some("kube-system".to_string()),
            search: None,
            limit: None,
            offset: None,
        };
        let filtered = apply_filters(rows, &query);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_apply_filters_by_search_description() {
        let rows = vec![
            KubernetesAuditRow {
                id: 1, created: chrono::Utc::now(), username: Some("alice".to_string()),
                cluster: None, namespace: Some("default".to_string()),
                resource: Some("Deployment".to_string()), resource_name: Some("app-1".to_string()),
                verb: "update".to_string(), action: "kubernetes_resource_updated".to_string(),
                description: "Обновлен ресурс Deployment/app-1 в namespace default".to_string(),
                level: "info".to_string(),
            },
            KubernetesAuditRow {
                id: 2, created: chrono::Utc::now(), username: Some("bob".to_string()),
                cluster: None, namespace: Some("default".to_string()),
                resource: Some("Pod".to_string()), resource_name: Some("pod-1".to_string()),
                verb: "create".to_string(), action: "kubernetes_resource_created".to_string(),
                description: "Создан ресурс Pod/pod-1".to_string(),
                level: "info".to_string(),
            },
        ];
        let query = KubernetesAuditQuery {
            username: None,
            resource: None,
            verb: None,
            namespace: None,
            search: Some("app-1".to_string()),
            limit: None,
            offset: None,
        };
        let filtered = apply_filters(rows, &query);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_apply_filters_search_by_username() {
        let rows = vec![
            KubernetesAuditRow {
                id: 1, created: chrono::Utc::now(), username: Some("alice".to_string()),
                cluster: None, namespace: None, resource: None, resource_name: None,
                verb: "create".to_string(), action: "kubernetes_resource_created".to_string(),
                description: "test".to_string(), level: "info".to_string(),
            },
            KubernetesAuditRow {
                id: 2, created: chrono::Utc::now(), username: Some("bob".to_string()),
                cluster: None, namespace: None, resource: None, resource_name: None,
                verb: "create".to_string(), action: "kubernetes_resource_created".to_string(),
                description: "test".to_string(), level: "info".to_string(),
            },
        ];
        let query = KubernetesAuditQuery {
            username: None,
            resource: None,
            verb: None,
            namespace: None,
            search: Some("alice".to_string()),
            limit: None,
            offset: None,
        };
        let filtered = apply_filters(rows, &query);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].username, Some("alice".to_string()));
    }

    // --- KubernetesAuditQuery deserialization tests ---

    #[test]
    fn test_kubernetes_audit_query_deserialize_empty() {
        let query: KubernetesAuditQuery = serde_json::from_str("{}").unwrap();
        assert!(query.username.is_none());
        assert!(query.resource.is_none());
        assert!(query.verb.is_none());
        assert!(query.namespace.is_none());
        assert!(query.search.is_none());
        assert!(query.limit.is_none());
        assert!(query.offset.is_none());
    }

    #[test]
    fn test_kubernetes_audit_query_deserialize_with_values() {
        let json = r#"{"username": "admin", "verb": "create", "limit": 50}"#;
        let query: KubernetesAuditQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.username, Some("admin".to_string()));
        assert_eq!(query.verb, Some("create".to_string()));
        assert_eq!(query.limit, Some(50));
    }

    // --- KubernetesAuditExportQuery tests ---

    #[test]
    fn test_kubernetes_audit_export_query_deserialize_default_format() {
        let query: KubernetesAuditExportQuery = serde_json::from_str("{}").unwrap();
        assert!(query.format.is_none());
    }

    #[test]
    fn test_kubernetes_audit_export_query_deserialize_csv_format() {
        let json = r#"{"format": "csv"}"#;
        let query: KubernetesAuditExportQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.format, Some("csv".to_string()));
    }

    // --- KubernetesAuditRow serialization tests ---

    #[test]
    fn test_kubernetes_audit_row_serialize() {
        let row = KubernetesAuditRow {
            id: 42,
            created: chrono::Utc::now(),
            username: Some("admin".to_string()),
            cluster: Some("prod-cluster".to_string()),
            namespace: Some("kube-system".to_string()),
            resource: Some("Deployment".to_string()),
            resource_name: Some("nginx".to_string()),
            verb: "create".to_string(),
            action: "kubernetes_resource_created".to_string(),
            description: "Created nginx deployment".to_string(),
            level: "info".to_string(),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"username\":\"admin\""));
        assert!(json.contains("\"cluster\":\"prod-cluster\""));
        assert!(json.contains("\"namespace\":\"kube-system\""));
        assert!(json.contains("\"resource\":\"Deployment\""));
        assert!(json.contains("\"verb\":\"create\""));
        assert!(json.contains("\"level\":\"info\""));
    }

    #[test]
    fn test_kubernetes_audit_row_serialize_with_none_fields() {
        let row = KubernetesAuditRow {
            id: 1,
            created: chrono::Utc::now(),
            username: None,
            cluster: None,
            namespace: None,
            resource: None,
            resource_name: None,
            verb: "delete".to_string(),
            action: "kubernetes_resource_deleted".to_string(),
            description: "Deleted".to_string(),
            level: "warning".to_string(),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"username\":null"));
        assert!(json.contains("\"cluster\":null"));
        assert!(json.contains("\"namespace\":null"));
        assert!(json.contains("\"verb\":\"delete\""));
        assert!(json.contains("\"level\":\"warning\""));
    }
}
