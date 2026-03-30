//! Kubernetes Audit Logging helpers
//!
//! Утилиты для логирования Kubernetes операций в Audit Log

use crate::api::state::AppState;
use crate::db::store::AuditLogManager;
use crate::models::audit_log::{AuditAction, AuditLevel, AuditObjectType};
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
