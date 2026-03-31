//! Kubernetes Inventory Sync
//!
//! Синхронизация Kubernetes нод и Pod в Ansible инвентарь Velum
//! для запуска playbook на кластере

use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::core::v1::{Node, Pod};
use kube::{api::{Api, ListParams}, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::db::store::InventoryManager;
use crate::error::{Error, Result};
use crate::models::Inventory;
use super::client::KubeClient;

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncType {
    /// Синхронизировать Node (кластерные ноды)
    Nodes,
    /// Синхронизировать Pod (в namespace)
    Pods,
    /// Синхронизировать всё
    All,
}

impl Default for SyncType {
    fn default() -> Self {
        SyncType::Nodes
    }
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
        SyncType::Nodes => {
            get_nodes_preview(&kube_client, &params).await
        }
        SyncType::Pods => {
            get_pods_preview(&kube_client, &params).await
        }
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
    
    let nodes = api.list(&lp).await
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
        warnings.push(format!("Большое количество нод: {}. Рекомендуется использовать label_selector.", nodes.items.len()));
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
    
    let pods = api.list(&lp).await
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
            let ip = pod.status.as_ref()?.pod_ip.clone().unwrap_or_else(|| "unknown".to_string());
            
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
        warnings.push(format!("Большое количество pod: {}. Рекомендуется использовать label_selector.", pods.items.len()));
    }
    if namespace == "default" {
        warnings.push("Синхронизация из namespace 'default'. Укажите нужный namespace.".to_string());
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
                            let safe_key = key.replace('/', "_").replace('-', "_");
                            inventory.push_str(&format!(" k8s_label_{}={}", safe_key, sanitize_value(&value)));
                        }
                    }
                    
                    // Annotations
                    if let Some(annotations) = &node.metadata.annotations {
                        for (key, value) in annotations {
                            let safe_key = key.replace('/', "_").replace('-', "_");
                            inventory.push_str(&format!(" k8s_annotation_{}={}", safe_key, sanitize_value(&value)));
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
                (k == "node-role.kubernetes.io/master" || k == "node-role.kubernetes.io/control-plane") && !v.is_empty()
            }) {
                inventory.push_str(&format!("{}\n", name));
            }
        }
    }
    
    inventory.push_str("\n[k8s_workers]\n");
    for node in nodes {
        if let (Some(name), Some(labels)) = (&node.metadata.name, &node.metadata.labels) {
            let is_worker = labels.iter().any(|(k, v)| {
                k == "node-role.kubernetes.io/worker" && !v.is_empty()
            });
            let is_not_master = !labels.iter().any(|(k, v)| {
                (k == "node-role.kubernetes.io/master" || k == "node-role.kubernetes.io/control-plane") && !v.is_empty()
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
    inventory.push_str(&format!("# Kubernetes Pods Inventory - Namespace: {}\n", namespace));
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
                        let safe_key = key.replace('/', "_").replace('-', "_");
                        inventory.push_str(&format!(" k8s_label_{}={}", safe_key, sanitize_value(&value)));
                    }
                }
                
                inventory.push('\n');
            }
        }
    }
    
    // Группы по labels
    let mut groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for pod in pods {
        if let (Some(name), Some(labels)) = (&pod.metadata.name, &pod.metadata.labels) {
            for (key, value) in labels {
                if key == "app" || key == "application" {
                    groups.entry(format!("k8s_app_{}", sanitize_value(value)))
                        .or_insert_with(Vec::new)
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
        .replace(' ', "_")
        .replace('/', "_")
        .replace('-', "_")
        .replace('.', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
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
    
    let created_inventory = if params.create_new || params.inventory_id.is_none() {
        state.store.create_inventory(inventory).await
            .map_err(|e| Error::Other(format!("Failed to create inventory: {}", e)))?
    } else {
        // TODO: update inventory
        state.store.create_inventory(inventory).await
            .map_err(|e| Error::Other(format!("Failed to update inventory: {}", e)))?
    };
    
    let inventory_name_result = created_inventory.name.clone();
    
    Ok(Json(InventorySyncResult {
        inventory_id: created_inventory.id,
        inventory_name: inventory_name_result,
        sync_type: params.sync_type,
        synced_count: preview.resource_count,
        message: format!(
            "Синхронизировано {} ресурсов в инвентарь '{}'",
            preview.resource_count,
            created_inventory.name
        ),
    }))
}
