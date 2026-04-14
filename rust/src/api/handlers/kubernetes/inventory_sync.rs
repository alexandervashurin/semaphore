//! Kubernetes Inventory Sync
//!
//! Синхронизация Kubernetes нод и Pod в Ansible инвентарь Velum
//! для запуска playbook на кластере

use axum::{
    Json,
    extract::{Path, Query, State},
};
use k8s_openapi::api::core::v1::{Node, Pod};
use kube::{
    Client,
    api::{Api, ListParams},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::client::KubeClient;
use crate::api::state::AppState;
use crate::db::store::InventoryManager;
use crate::error::{Error, Result};
use crate::models::Inventory;

// ============================================================================
// Inventory Sync Types
// ============================================================================

/// Параметры синхронизации инвентаря
#[derive(Debug, Deserialize)]
pub struct InventorySyncParams {
    /// ID проекта для создания инвентаря
    pub project_id: i32,

    /// Тип синхронизации
    #[serde(default)]
    pub sync_type: SyncType,

    /// Namespace (только для pod)
    #[serde(default)]
    pub namespace: Option<String>,

    /// Label selector для фильтрации
    #[serde(default)]
    pub label_selector: Option<String>,

    /// Префикс для имени инвентаря
    #[serde(default)]
    pub name_prefix: Option<String>,

    /// Создать новый инвентарь или обновить существующий
    #[serde(default)]
    pub create_new: bool,

    /// ID существующего инвентаря для обновления
    #[serde(default)]
    pub inventory_id: Option<i32>,
}

/// Тип синхронизации
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SyncType {
    /// Синхронизировать Node (кластерные ноды)
    #[default]
    Nodes,
    /// Синхронизировать Pod (в namespace)
    Pods,
    /// Синхронизировать всё
    All,
}

/// Предпросмотр синхронизации
#[derive(Debug, Serialize)]
pub struct InventorySyncPreview {
    /// Тип синхронизации
    pub sync_type: SyncType,

    /// Количество ресурсов для синхронизации
    pub resource_count: usize,

    /// Примеры ресурсов
    pub examples: Vec<ResourcePreview>,

    /// Генерируемый инвентарь (YAML/INI)
    pub inventory_content: String,

    /// Предупреждения
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Предпросмотр ресурса
#[derive(Debug, Serialize)]
pub struct ResourcePreview {
    pub name: String,
    pub ip: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
}

/// Результат синхронизации
#[derive(Debug, Serialize)]
pub struct InventorySyncResult {
    /// ID созданного/обновленного инвентаря
    pub inventory_id: i32,

    /// Название инвентаря
    pub inventory_name: String,

    /// Тип синхронизации
    pub sync_type: SyncType,

    /// Количество синхронизированных ресурсов
    pub synced_count: usize,

    /// Сообщение
    pub message: String,
}

// ============================================================================
// Inventory Sync Logic
// ============================================================================

/// Получить предпросмотр синхронизации инвентаря
pub async fn get_inventory_sync_preview(
    State(state): State<Arc<AppState>>,
    Query(params): Query<InventorySyncParams>,
) -> Result<Json<InventorySyncPreview>> {
    let kube_client = state.kubernetes_client()?;

    match params.sync_type {
        SyncType::Nodes => get_nodes_preview(&kube_client, &params).await,
        SyncType::Pods => get_pods_preview(&kube_client, &params).await,
        SyncType::All => {
            // Для All показываем только Node (как базовый вариант)
            get_nodes_preview(&kube_client, &params).await
        }
    }
}

/// Предпросмотр для Node
async fn get_nodes_preview(
    kube_client: &Arc<KubeClient>,
    params: &InventorySyncParams,
) -> Result<Json<InventorySyncPreview>> {
    let client = kube_client.raw().clone();
    let api: Api<Node> = Api::all(client);

    let mut lp = ListParams::default();
    if let Some(selector) = &params.label_selector {
        lp.label_selector = Some(selector.clone());
    }

    let nodes = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to list nodes: {}", e)))?;

    if nodes.items.is_empty() {
        return Err(Error::NotFound("No nodes found".to_string()));
    }

    // Собираем примеры
    let examples: Vec<ResourcePreview> = nodes
        .items
        .iter()
        .take(5)
        .filter_map(|node| {
            let name = node.metadata.name.clone()?;
            let addresses = node.status.as_ref()?.addresses.as_ref()?;

            // Ищем InternalIP
            let ip = addresses
                .iter()
                .find(|a| a.type_ == "InternalIP")
                .or_else(|| addresses.iter().find(|a| a.type_ == "ExternalIP"))
                .map(|a| a.address.clone())
                .unwrap_or_else(|| "unknown".to_string());

            Some(ResourcePreview {
                name,
                ip,
                labels: node.metadata.labels.clone().unwrap_or_default(),
                annotations: node.metadata.annotations.clone().unwrap_or_default(),
            })
        })
        .collect();

    // Генерируем инвентарь
    let inventory_content = generate_nodes_inventory(&nodes.items);

    let mut warnings = Vec::new();
    if nodes.items.len() > 100 {
        warnings.push(format!(
            "Большое количество нод: {}. Рекомендуется использовать label_selector.",
            nodes.items.len()
        ));
    }

    Ok(Json(InventorySyncPreview {
        sync_type: SyncType::Nodes,
        resource_count: nodes.items.len(),
        examples,
        inventory_content,
        warnings,
    }))
}

/// Предпросмотр для Pod
async fn get_pods_preview(
    kube_client: &Arc<KubeClient>,
    params: &InventorySyncParams,
) -> Result<Json<InventorySyncPreview>> {
    let client = kube_client.raw().clone();
    let namespace = params.namespace.as_deref().unwrap_or("default");
    let api: Api<Pod> = Api::namespaced(client, namespace);

    let mut lp = ListParams::default();
    if let Some(selector) = &params.label_selector {
        lp.label_selector = Some(selector.clone());
    }

    let pods = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to list pods: {}", e)))?;

    if pods.items.is_empty() {
        return Err(Error::NotFound("No pods found".to_string()));
    }

    // Собираем примеры
    let examples: Vec<ResourcePreview> = pods
        .items
        .iter()
        .take(5)
        .filter_map(|pod| {
            let name = pod.metadata.name.clone()?;
            let ip = pod
                .status
                .as_ref()?
                .pod_ip
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

            Some(ResourcePreview {
                name,
                ip,
                labels: pod.metadata.labels.clone().unwrap_or_default(),
                annotations: pod.metadata.annotations.clone().unwrap_or_default(),
            })
        })
        .collect();

    // Генерируем инвентарь
    let inventory_content = generate_pods_inventory(&pods.items, namespace);

    let mut warnings = Vec::new();
    if pods.items.len() > 200 {
        warnings.push(format!(
            "Большое количество pod: {}. Рекомендуется использовать label_selector.",
            pods.items.len()
        ));
    }
    if namespace == "default" {
        warnings
            .push("Синхронизация из namespace 'default'. Укажите нужный namespace.".to_string());
    }

    Ok(Json(InventorySyncPreview {
        sync_type: SyncType::Pods,
        resource_count: pods.items.len(),
        examples,
        inventory_content,
        warnings,
    }))
}

/// Сгенерировать Ansible инвентарь для Node
fn generate_nodes_inventory(nodes: &[Node]) -> String {
    let mut inventory = String::new();
    inventory.push_str("# Kubernetes Nodes Inventory\n");
    inventory.push_str("# Auto-generated by Velum Kubernetes Sync\n\n");

    // Все ноды в группе [k8s_nodes]
    inventory.push_str("[k8s_nodes]\n");

    for node in nodes {
        if let (Some(name), Some(status)) = (&node.metadata.name, &node.status) {
            if let Some(addresses) = &status.addresses {
                if let Some(ip) = addresses
                    .iter()
                    .find(|a| a.type_ == "InternalIP")
                    .or_else(|| addresses.iter().find(|a| a.type_ == "ExternalIP"))
                {
                    // Добавляем переменные
                    inventory.push_str(&format!("{} ansible_host={}", name, ip.address));

                    // Добавляем labels как переменные
                    if let Some(labels) = &node.metadata.labels {
                        for (key, value) in labels {
                            let safe_key = key.replace(['/', '-'], "_");
                            inventory.push_str(&format!(
                                " k8s_label_{}={}",
                                safe_key,
                                sanitize_value(value)
                            ));
                        }
                    }

                    // Annotations
                    if let Some(annotations) = &node.metadata.annotations {
                        for (key, value) in annotations {
                            let safe_key = key.replace(['/', '-'], "_");
                            inventory.push_str(&format!(
                                " k8s_annotation_{}={}",
                                safe_key,
                                sanitize_value(value)
                            ));
                        }
                    }

                    inventory.push('\n');
                }
            }
        }
    }

    // Группы по ролям (master/worker)
    inventory.push_str("\n[k8s_masters]\n");
    for node in nodes {
        if let (Some(name), Some(labels)) = (&node.metadata.name, &node.metadata.labels) {
            if labels.iter().any(|(k, v)| {
                (k == "node-role.kubernetes.io/master"
                    || k == "node-role.kubernetes.io/control-plane")
                    && !v.is_empty()
            }) {
                inventory.push_str(&format!("{}\n", name));
            }
        }
    }

    inventory.push_str("\n[k8s_workers]\n");
    for node in nodes {
        if let (Some(name), Some(labels)) = (&node.metadata.name, &node.metadata.labels) {
            let is_worker = labels
                .iter()
                .any(|(k, v)| k == "node-role.kubernetes.io/worker" && !v.is_empty());
            let is_not_master = !labels.iter().any(|(k, v)| {
                (k == "node-role.kubernetes.io/master"
                    || k == "node-role.kubernetes.io/control-plane")
                    && !v.is_empty()
            });

            if is_worker || is_not_master {
                inventory.push_str(&format!("{}\n", name));
            }
        }
    }

    inventory
}

/// Сгенерировать Ansible инвентарь для Pod
fn generate_pods_inventory(pods: &[Pod], namespace: &str) -> String {
    let mut inventory = String::new();
    inventory.push_str(&format!(
        "# Kubernetes Pods Inventory - Namespace: {}\n",
        namespace
    ));
    inventory.push_str("# Auto-generated by Velum Kubernetes Sync\n\n");

    inventory.push_str("[k8s_pods]\n");

    for pod in pods {
        if let (Some(name), Some(status)) = (&pod.metadata.name, &pod.status) {
            if let Some(ip) = &status.pod_ip {
                inventory.push_str(&format!("{} ansible_host={}", name, ip));
                inventory.push_str(&format!(" k8s_namespace={}", namespace));

                // Labels
                if let Some(labels) = &pod.metadata.labels {
                    for (key, value) in labels {
                        let safe_key = key.replace(['/', '-'], "_");
                        inventory.push_str(&format!(
                            " k8s_label_{}={}",
                            safe_key,
                            sanitize_value(value)
                        ));
                    }
                }

                inventory.push('\n');
            }
        }
    }

    // Группы по labels
    let mut groups: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for pod in pods {
        if let (Some(name), Some(labels)) = (&pod.metadata.name, &pod.metadata.labels) {
            for (key, value) in labels {
                if key == "app" || key == "application" {
                    groups
                        .entry(format!("k8s_app_{}", sanitize_value(value)))
                        .or_default()
                        .push(name.clone());
                }
            }
        }
    }

    for (group_name, members) in groups {
        inventory.push_str(&format!("\n[{}]\n", group_name));
        for member in members {
            inventory.push_str(&format!("{}\n", member));
        }
    }

    inventory
}

/// Очистить значение для использования в инвентаре
fn sanitize_value(value: &str) -> String {
    value
        .replace([' ', '/', '-', '.'], "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // --- SyncType tests ---

    #[test]
    fn test_sync_type_default_is_nodes() {
        let st = SyncType::default();
        assert_eq!(st, SyncType::Nodes);
    }

    #[test]
    fn test_sync_type_serialize_nodes() {
        let json = serde_json::to_string(&SyncType::Nodes).unwrap();
        assert_eq!(json, "\"nodes\"");
    }

    #[test]
    fn test_sync_type_serialize_pods() {
        let json = serde_json::to_string(&SyncType::Pods).unwrap();
        assert_eq!(json, "\"pods\"");
    }

    #[test]
    fn test_sync_type_serialize_all() {
        let json = serde_json::to_string(&SyncType::All).unwrap();
        assert_eq!(json, "\"all\"");
    }

    #[test]
    fn test_sync_type_deserialize_nodes() {
        let st: SyncType = serde_json::from_str("\"nodes\"").unwrap();
        assert_eq!(st, SyncType::Nodes);
    }

    #[test]
    fn test_sync_type_deserialize_pods() {
        let st: SyncType = serde_json::from_str("\"pods\"").unwrap();
        assert_eq!(st, SyncType::Pods);
    }

    #[test]
    fn test_sync_type_deserialize_all() {
        let st: SyncType = serde_json::from_str("\"all\"").unwrap();
        assert_eq!(st, SyncType::All);
    }

    #[test]
    fn test_sync_type_deserialize_invalid_returns_error() {
        let result: serde_json::Result<SyncType> = serde_json::from_str("\"invalid\"");
        assert!(result.is_err());
    }

    // --- InventorySyncParams tests ---

    #[test]
    fn test_inventory_sync_params_deserialize_minimal() {
        let json = r#"{"project_id": 1}"#;
        let params: InventorySyncParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.project_id, 1);
        assert_eq!(params.sync_type, SyncType::Nodes);
        assert!(params.namespace.is_none());
        assert!(params.label_selector.is_none());
        assert!(params.name_prefix.is_none());
        assert!(!params.create_new);
        assert!(params.inventory_id.is_none());
    }

    #[test]
    fn test_inventory_sync_params_deserialize_full() {
        let json = r#"{
            "project_id": 42,
            "sync_type": "pods",
            "namespace": "kube-system",
            "label_selector": "app=nginx",
            "name_prefix": "my-inventory",
            "create_new": true,
            "inventory_id": 7
        }"#;
        let params: InventorySyncParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.project_id, 42);
        assert_eq!(params.sync_type, SyncType::Pods);
        assert_eq!(params.namespace, Some("kube-system".to_string()));
        assert_eq!(params.label_selector, Some("app=nginx".to_string()));
        assert_eq!(params.name_prefix, Some("my-inventory".to_string()));
        assert!(params.create_new);
        assert_eq!(params.inventory_id, Some(7));
    }

    #[test]
    fn test_inventory_sync_params_missing_project_id_errors() {
        let json = r#"{}"#;
        let result: serde_json::Result<InventorySyncParams> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // --- sanitize_value tests ---

    #[test]
    fn test_sanitize_value_spaces() {
        assert_eq!(sanitize_value("my value"), "my_value");
    }

    #[test]
    fn test_sanitize_value_slashes() {
        assert_eq!(sanitize_value("my/value"), "my_value");
    }

    #[test]
    fn test_sanitize_value_dashes() {
        assert_eq!(sanitize_value("my-value"), "my_value");
    }

    #[test]
    fn test_sanitize_value_dots() {
        assert_eq!(sanitize_value("my.value"), "my_value");
    }

    #[test]
    fn test_sanitize_value_mixed() {
        assert_eq!(sanitize_value("my-app.k8s.io/name"), "my_app_k8s_io_name");
    }

    #[test]
    fn test_sanitize_value_already_clean() {
        assert_eq!(sanitize_value("alphanumeric123"), "alphanumeric123");
    }

    #[test]
    fn test_sanitize_value_empty() {
        assert_eq!(sanitize_value(""), "");
    }

    #[test]
    fn test_sanitize_value_removes_special_chars() {
        assert_eq!(sanitize_value("val@#$%^&*!"), "val");
    }

    // --- ResourcePreview / InventorySyncPreview / InventorySyncResult ---

    #[test]
    fn test_resource_preview_serialize() {
        let mut labels = std::collections::BTreeMap::new();
        labels.insert("app".to_string(), "nginx".to_string());
        let mut annotations = std::collections::BTreeMap::new();
        annotations.insert("description".to_string(), "test".to_string());

        let preview = ResourcePreview {
            name: "node-1".to_string(),
            ip: "10.0.0.1".to_string(),
            labels,
            annotations,
        };
        let json = serde_json::to_string(&preview).unwrap();
        assert!(json.contains("\"name\":\"node-1\""));
        assert!(json.contains("\"ip\":\"10.0.0.1\""));
        assert!(json.contains("\"app\""));
        assert!(json.contains("\"nginx\""));
    }

    #[test]
    fn test_inventory_sync_preview_serialize() {
        let preview = InventorySyncPreview {
            sync_type: SyncType::Nodes,
            resource_count: 3,
            examples: vec![],
            inventory_content: "# test\n".to_string(),
            warnings: vec!["warn1".to_string()],
        };
        let json = serde_json::to_string(&preview).unwrap();
        assert!(json.contains("\"sync_type\":\"nodes\""));
        assert!(json.contains("\"resource_count\":3"));
        assert!(json.contains("\"warnings\":[\"warn1\"]"));
    }

    #[test]
    fn test_inventory_sync_result_serialize() {
        let result = InventorySyncResult {
            inventory_id: 10,
            inventory_name: "K8s Nodes".to_string(),
            sync_type: SyncType::Nodes,
            synced_count: 5,
            message: "ok".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"inventory_id\":10"));
        assert!(json.contains("\"inventory_name\":\"K8s Nodes\""));
        assert!(json.contains("\"sync_type\":\"nodes\""));
        assert!(json.contains("\"synced_count\":5"));
        assert!(json.contains("\"message\":\"ok\""));
    }

    // --- Inventory content generation (pure function tests) ---

    #[test]
    fn test_generate_nodes_inventory_empty() {
        let nodes: Vec<Node> = vec![];
        let content = generate_nodes_inventory(&nodes);
        assert!(content.contains("# Kubernetes Nodes Inventory"));
        assert!(content.contains("[k8s_nodes]"));
        assert!(content.contains("[k8s_masters]"));
        assert!(content.contains("[k8s_workers]"));
    }

    #[test]
    fn test_generate_pods_inventory_empty() {
        let pods: Vec<Pod> = vec![];
        let content = generate_pods_inventory(&pods, "default");
        assert!(content.contains("# Kubernetes Pods Inventory - Namespace: default"));
        assert!(content.contains("[k8s_pods]"));
    }

    #[test]
    fn test_generate_nodes_inventory_has_section_headers() {
        let nodes: Vec<Node> = vec![];
        let content = generate_nodes_inventory(&nodes);
        // Verify all three group sections are present
        assert!(content.contains("[k8s_nodes]"));
        assert!(content.contains("[k8s_masters]"));
        assert!(content.contains("[k8s_workers]"));
        // Verify order: nodes section comes before masters
        let nodes_pos = content.find("[k8s_nodes]").unwrap();
        let masters_pos = content.find("[k8s_masters]").unwrap();
        assert!(nodes_pos < masters_pos);
    }
}

/// Выполнить синхронизацию инвентаря
pub async fn execute_inventory_sync(
    State(state): State<Arc<AppState>>,
    Query(params): Query<InventorySyncParams>,
) -> Result<Json<InventorySyncResult>> {
    let kube_client = state.kubernetes_client()?;

    // Получаем предпросмотр
    let preview = match params.sync_type {
        SyncType::Nodes => get_nodes_preview(&kube_client, &params).await?,
        SyncType::Pods => get_pods_preview(&kube_client, &params).await?,
        SyncType::All => get_nodes_preview(&kube_client, &params).await?,
    };

    // Формируем название инвентаря
    let inventory_name = params.name_prefix.unwrap_or_else(|| {
        format!(
            "K8s {} {}",
            match params.sync_type {
                SyncType::Nodes => "Nodes",
                SyncType::Pods => "Pods",
                SyncType::All => "All",
            },
            chrono::Utc::now().format("%Y-%m-%d %H:%M")
        )
    });

    // Создаём или обновляем инвентарь
    let inventory = crate::models::Inventory {
        id: params.inventory_id.unwrap_or(0),
        project_id: params.project_id,
        name: inventory_name.clone(),
        inventory_type: crate::models::InventoryType::StaticYaml,
        inventory_data: preview.inventory_content.clone(),
        key_id: None,
        secret_storage_id: None,
        ssh_login: "root".to_string(),
        ssh_port: 22,
        extra_vars: None,
        ssh_key_id: None,
        become_key_id: None,
        vaults: None,
        created: Some(chrono::Utc::now()),
        runner_tag: None,
    };

    let created_inventory = if params.create_new {
        state
            .store
            .create_inventory(inventory)
            .await
            .map_err(|e| Error::Other(format!("Failed to create inventory: {}", e)))?
    } else if let Some(inventory_id) = params.inventory_id {
        // Update existing inventory
        let mut inventory = inventory;
        inventory.id = inventory_id;
        state
            .store
            .update_inventory(inventory)
            .await
            .map_err(|e| Error::Other(format!("Failed to update inventory: {}", e)))?;
        state
            .store
            .get_inventory(params.project_id, inventory_id)
            .await
            .map_err(|e| Error::Other(format!("Failed to get updated inventory: {}", e)))?
    } else {
        // Upsert: try to find inventory by name and update, otherwise create new
        let existing = state
            .store
            .get_inventories(params.project_id)
            .await
            .unwrap_or_default()
            .into_iter()
            .find(|inv| inv.name == inventory_name)
            .map(|inv| inv.id);

        if let Some(existing_id) = existing {
            let mut inventory = inventory;
            inventory.id = existing_id;
            state
                .store
                .update_inventory(inventory)
                .await
                .map_err(|e| Error::Other(format!("Failed to update inventory: {}", e)))?;
            state
                .store
                .get_inventory(params.project_id, existing_id)
                .await
                .map_err(|e| Error::Other(format!("Failed to get updated inventory: {}", e)))?
        } else {
            state
                .store
                .create_inventory(inventory)
                .await
                .map_err(|e| Error::Other(format!("Failed to create inventory: {}", e)))?
        }
    };

    let inventory_name_result = created_inventory.name.clone();

    Ok(Json(InventorySyncResult {
        inventory_id: created_inventory.id,
        inventory_name: inventory_name_result,
        sync_type: params.sync_type,
        synced_count: preview.resource_count,
        message: format!(
            "Синхронизировано {} ресурсов в инвентарь '{}'",
            preview.resource_count, created_inventory.name
        ),
    }))
}
