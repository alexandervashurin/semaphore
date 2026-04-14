//! Kubernetes backup/restore runbook (v1) + optional Velero read-only status.

use axum::{
    extract::{Query, State},
    Json,
};
use kube::{
    api::{Api, DynamicObject, ListParams},
    core::{ApiResource, GroupVersionKind},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Serialize)]
pub struct BackupRunbook {
    pub title: String,
    pub db_steps: Vec<String>,
    pub config_steps: Vec<String>,
    pub restore_steps: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct VeleroStatus {
    pub installed: bool,
    pub backups_api: bool,
    pub restores_api: bool,
}

#[derive(Debug, Deserialize)]
pub struct VeleroQuery {
    pub namespace: Option<String>,
    pub limit: Option<u32>,
}

fn ar(group: &str, version: &str, kind: &str, plural: &str) -> ApiResource {
    ApiResource::from_gvk_with_plural(&GroupVersionKind::gvk(group, version, kind), plural)
}

fn dyn_api(
    raw: kube::Client,
    namespace: Option<&str>,
    api_res: &ApiResource,
) -> Api<DynamicObject> {
    if let Some(ns) = namespace {
        Api::namespaced_with(raw, ns, api_res)
    } else {
        Api::all_with(raw, api_res)
    }
}

pub async fn get_backup_restore_runbook() -> Result<Json<BackupRunbook>> {
    Ok(Json(BackupRunbook {
        title: "Velum backup/restore runbook (v1)".to_string(),
        db_steps: vec![
            "Остановить write-heavy операции (или включить maintenance window).".to_string(),
            "Сделать дамп БД (PostgreSQL: pg_dump; SQLite: snapshot файла + WAL).".to_string(),
            "Проверить целостность дампа и размер артефактов.".to_string(),
            "Сохранить в защищённое хранилище с ротацией и retention.".to_string(),
        ],
        config_steps: vec![
            "Сохранить env/секреты и конфиги приложения (без утечки plaintext в логи).".to_string(),
            "Сохранить версии образов, миграций и commit SHA текущего деплоя.".to_string(),
            "Проверить, что backup включает критичные каталоги и external integrations."
                .to_string(),
        ],
        restore_steps: vec![
            "Поднять чистое окружение с теми же версиями приложения/схемы.".to_string(),
            "Восстановить БД из дампа и применить миграции при необходимости.".to_string(),
            "Восстановить конфиги/секреты и перезапустить сервисы.".to_string(),
            "Провести smoke-check: login, projects, templates, task run, audit entries."
                .to_string(),
        ],
        notes: vec![
            "Velero поддерживается только как optional read-only детект в v1.".to_string(),
            "Полный UI оркестрации backup/restore вне scope v1.".to_string(),
        ],
    }))
}

pub async fn get_velero_status(State(state): State<Arc<AppState>>) -> Result<Json<VeleroStatus>> {
    let client = state.kubernetes_client()?;
    let raw = client.raw().clone();
    let lp = ListParams::default().limit(1);

    let backups: Api<DynamicObject> =
        Api::all_with(raw.clone(), &ar("velero.io", "v1", "Backup", "backups"));
    let restores: Api<DynamicObject> =
        Api::all_with(raw, &ar("velero.io", "v1", "Restore", "restores"));

    let backups_api = backups.list(&lp).await.is_ok();
    let restores_api = restores.list(&lp).await.is_ok();
    Ok(Json(VeleroStatus {
        installed: backups_api || restores_api,
        backups_api,
        restores_api,
    }))
}

pub async fn list_velero_backups(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VeleroQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api_res = ar("velero.io", "v1", "Backup", "backups");
    let api = dyn_api(client.raw().clone(), query.namespace.as_deref(), &api_res);
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let items = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Velero Backup API not available: {e}")))?;
    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_runbook_title() {
        let runbook = BackupRunbook {
            title: "Test runbook".to_string(),
            db_steps: vec![],
            config_steps: vec![],
            restore_steps: vec![],
            notes: vec![],
        };
        assert_eq!(runbook.title, "Test runbook");
    }

    #[test]
    fn test_backup_runbook_steps_not_empty() {
        let runbook = BackupRunbook {
            title: "v1".to_string(),
            db_steps: vec!["step1".to_string()],
            config_steps: vec!["cfg1".to_string()],
            restore_steps: vec!["res1".to_string()],
            notes: vec!["note1".to_string()],
        };
        assert_eq!(runbook.db_steps.len(), 1);
        assert_eq!(runbook.config_steps.len(), 1);
        assert_eq!(runbook.restore_steps.len(), 1);
        assert_eq!(runbook.notes.len(), 1);
    }

    #[test]
    fn test_backup_runbook_serialize() {
        let runbook = BackupRunbook {
            title: "test".to_string(),
            db_steps: vec!["dump".to_string()],
            config_steps: vec![],
            restore_steps: vec![],
            notes: vec![],
        };
        let json_val = serde_json::to_value(&runbook).unwrap();
        assert_eq!(json_val["title"], "test");
        assert_eq!(json_val["db_steps"][0], "dump");
    }

    #[test]
    fn test_backup_runbook_debug_format() {
        let runbook = BackupRunbook {
            title: "debug_test".to_string(),
            db_steps: vec![],
            config_steps: vec![],
            restore_steps: vec![],
            notes: vec![],
        };
        let debug_str = format!("{:?}", runbook);
        assert!(debug_str.contains("BackupRunbook"));
    }

    #[test]
    fn test_velero_status_all_false() {
        let status = VeleroStatus {
            installed: false,
            backups_api: false,
            restores_api: false,
        };
        let json_val = serde_json::to_value(&status).unwrap();
        assert_eq!(json_val["installed"], false);
        assert_eq!(json_val["backups_api"], false);
        assert_eq!(json_val["restores_api"], false);
    }

    #[test]
    fn test_velero_status_installed() {
        let status = VeleroStatus {
            installed: true,
            backups_api: true,
            restores_api: true,
        };
        let serialized = serde_json::to_string(&status).unwrap();
        assert!(serialized.contains(r#""installed":true"#));
        assert!(serialized.contains(r#""backups_api":true"#));
        assert!(serialized.contains(r#""restores_api":true"#));
    }

    #[test]
    fn test_velero_status_mixed() {
        let status = VeleroStatus {
            installed: true,
            backups_api: true,
            restores_api: false,
        };
        let json_val = serde_json::to_value(&status).unwrap();
        assert_eq!(json_val["installed"], true);
        assert_eq!(json_val["backups_api"], true);
        assert_eq!(json_val["restores_api"], false);
    }

    #[test]
    fn test_velero_query_default() {
        let q: VeleroQuery = serde_json::from_str("{}").unwrap();
        assert!(q.namespace.is_none());
        assert!(q.limit.is_none());
    }

    #[test]
    fn test_velero_query_with_namespace() {
        let q: VeleroQuery = serde_json::from_str(r#"{"namespace": "velero"}"#).unwrap();
        assert_eq!(q.namespace, Some("velero".to_string()));
        assert!(q.limit.is_none());
    }

    #[test]
    fn test_velero_query_with_limit() {
        let q: VeleroQuery = serde_json::from_str(r#"{"limit": 20}"#).unwrap();
        assert_eq!(q.limit, Some(20));
        assert!(q.namespace.is_none());
    }

    #[test]
    fn test_velero_query_full() {
        let q: VeleroQuery =
            serde_json::from_str(r#"{"namespace": "velero-ns", "limit": 15}"#).unwrap();
        assert_eq!(q.namespace, Some("velero-ns".to_string()));
        assert_eq!(q.limit, Some(15));
    }

    #[test]
    fn test_api_resource_helper() {
        let ar_result = ar("velero.io", "v1", "Backup", "backups");
        assert_eq!(ar_result.group, "velero.io");
        assert_eq!(ar_result.version, "v1");
        assert_eq!(ar_result.plural, "backups");
    }

    #[test]
    fn test_runbook_db_steps_content() {
        let runbook = BackupRunbook {
            title: "Velum backup/restore runbook (v1)".to_string(),
            db_steps: vec![
                "Остановить write-heavy операции (или включить maintenance window).".to_string(),
                "Сделать дамп БД (PostgreSQL: pg_dump; SQLite: snapshot файла + WAL).".to_string(),
            ],
            config_steps: vec![],
            restore_steps: vec![],
            notes: vec![],
        };
        let serialized = serde_json::to_string(&runbook).unwrap();
        assert!(serialized.contains("pg_dump"));
        assert!(serialized.contains("maintenance window"));
    }
}
