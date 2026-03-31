//! Kubernetes Troubleshooting Dashboard
//!
//! Агрегация событий, метрик, логов и audit записей для диагностики проблем

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc, Duration};
use k8s_openapi::api::core::v1::Event;
use kube::{api::{Api, ListParams, ResourceExt}};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::db::store::AuditLogManager;
use crate::error::{Error, Result};
use super::client::KubeClient;
use super::prometheus::MetricValue;

// ============================================================================
// Troubleshooting Dashboard Types
// ============================================================================

/// Основная структура ответа Troubleshooting Dashboard
#[derive(Debug, Serialize, Deserialize)]
pub struct TroubleshootingReport {
    /// Объект для диагностики
    pub target: ResourceTarget,
    
    /// Временная шкала событий
    pub timeline: Vec<TimelineEvent>,
    
    /// Последние audit записи
    pub audit_records: Vec<AuditRecord>,
    
    /// Метрики объекта (если доступны)
    pub metrics: Option<ResourceMetrics>,
    
    /// Рекомендации по диагностике
    pub recommendations: Vec<Recommendation>,
    
    /// Статус здоровья
    pub health_status: HealthStatus,
}

/// Целевой ресурс для диагностики
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceTarget {
    pub kind: String,
    pub name: String,
    pub namespace: String,
    pub api_version: Option<String>,
    pub uid: Option<String>,
}

/// Событие временной шкалы (объединяет Kubernetes Events и Audit)
#[derive(Debug, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: TimelineEventType,
    pub source: EventSource,
    pub summary: String,
    pub details: Option<String>,
    pub severity: Severity,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TimelineEventType {
    KubernetesEvent,
    AuditLog,
    MetricAlert,
    StateChange,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    Kubernetes,
    VelumAudit,
    Metrics,
    User,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    Warning,
    Normal,
    Info,
}

/// Audit запись (упрощённая версия)
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: String,
    pub resource_kind: String,
    pub resource_name: String,
    pub namespace: String,
    pub description: String,
    pub level: String,
}

/// Метрики ресурса
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_usage: Option<String>,
    pub memory_usage: Option<String>,
    pub cpu_request: Option<String>,
    pub memory_request: Option<String>,
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub restart_count: Option<i32>,
}

/// Рекомендация по диагностике
#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub title: String,
    pub description: String,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// Статус здоровья
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TroubleshootQuery {
    pub namespace: String,
    pub kind: String,
    pub name: String,
    pub lookback_hours: Option<i64>,
    pub include_metrics: Option<bool>,
    pub include_audit: Option<bool>,
}

// ============================================================================
// Troubleshooting API Handlers
// ============================================================================

/// Получить отчёт по диагностике для ресурса
pub async fn get_troubleshooting_report(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TroubleshootQuery>,
) -> Result<Json<TroubleshootingReport>> {
    let lookback = query.lookback_hours.unwrap_or(24);
    let include_metrics = query.include_metrics.unwrap_or(true);
    let include_audit = query.include_audit.unwrap_or(true);

    // Собираем timeline событий
    let timeline = build_timeline(&state, &query, lookback).await?;
    
    // Получаем audit записи
    let audit_records = if include_audit {
        get_audit_records(&state, &query, lookback).await?
    } else {
        Vec::new()
    };
    
    // Получаем метрики
    let metrics = if include_metrics {
        get_resource_metrics(&state, &query).await.ok()
    } else {
        None
    };
    
    // Генерируем рекомендации
    let recommendations = generate_recommendations(&timeline, &metrics, &query);
    
    // Определяем статус здоровья
    let health_status = calculate_health_status(&timeline, &metrics);

    let target = ResourceTarget {
        kind: query.kind.clone(),
        name: query.name.clone(),
        namespace: query.namespace.clone(),
        api_version: None,
        uid: None,
    };

    Ok(Json(TroubleshootingReport {
        target,
        timeline,
        audit_records,
        metrics,
        recommendations,
        health_status,
    }))
}

/// Построить временную шкалу событий
async fn build_timeline(
    state: &Arc<AppState>,
    query: &TroubleshootQuery,
    lookback_hours: i64,
) -> Result<Vec<TimelineEvent>> {
    let kube_client = state.kubernetes_client()?;
    let mut events = Vec::new();

    // Получаем Kubernetes Events для объекта
    let k8s_events = get_kubernetes_events(&kube_client, query, lookback_hours).await?;
    
    for event in k8s_events {
        let severity = match event.type_.as_str() {
            "Warning" => Severity::Warning,
            "Error" => Severity::Critical,
            _ => Severity::Normal,
        };

        let timestamp = event
            .last_seen
            .and_then(|t| DateTime::parse_from_rfc3339(&t).ok())
            .map(|t| t.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        events.push(TimelineEvent {
            timestamp,
            event_type: TimelineEventType::KubernetesEvent,
            source: EventSource::Kubernetes,
            summary: format!("{}: {}", event.reason, event.message),
            details: Some(event.message),
            severity,
        });
    }

    // Сортируем по времени
    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(events)
}

/// Получить Kubernetes Events для объекта
async fn get_kubernetes_events(
    kube_client: &Arc<KubeClient>,
    query: &TroubleshootQuery,
    lookback_hours: i64,
) -> Result<Vec<SimpleEvent>> {
    let client = kube_client.raw().clone();
    let api: Api<Event> = Api::namespaced(client, &query.namespace);

    let mut lp = ListParams::default();
    lp.limit = Some(100);
    
    // Field selector для фильтрации по объекту
    lp.field_selector = Some(format!(
        "involvedObject.kind={},involvedObject.name={}",
        query.kind, query.name
    ));

    let event_list = api.list(&lp).await
        .map_err(|e| Error::Kubernetes(format!("Failed to list events: {}", e)))?;

    let events = event_list.items.iter().map(|e| {
        SimpleEvent {
            name: e.metadata.name.clone().unwrap_or_default(),
            namespace: e.metadata.namespace.clone().unwrap_or_default(),
            type_: e.type_.clone().unwrap_or_default(),
            reason: e.reason.clone().unwrap_or_default(),
            message: e.message.clone().unwrap_or_default(),
            count: e.count.unwrap_or(1),
            first_seen: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            last_seen: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        }
    }).collect();

    Ok(events)
}

/// Упрощённая структура Event
#[derive(Debug)]
struct SimpleEvent {
    name: String,
    namespace: String,
    type_: String,
    reason: String,
    message: String,
    count: i32,
    first_seen: Option<String>,
    last_seen: Option<String>,
}

/// Получить audit записи из Velum
async fn get_audit_records(
    state: &Arc<AppState>,
    query: &TroubleshootQuery,
    lookback_hours: i64,
) -> Result<Vec<AuditRecord>> {
    // Вычисляем дату начала периода
    let date_from = Utc::now() - Duration::hours(lookback_hours);
    
    // Фильтр по project_id если есть
    // Для Kubernetes ресурсов пока ищем по всем проектам
    // В будущем можно связывать Kubernetes кластеры с проектами Velum
    
    use crate::models::audit_log::AuditLogFilter;
    
    let filter = AuditLogFilter {
        project_id: None, // Все проекты
        user_id: None,
        username: None,
        action: None,
        object_type: None,
        object_id: None,
        level: None,
        search: Some(format!("{} {}", query.kind, query.name)), // Поиск по имени ресурса
        date_from: Some(date_from),
        date_to: Some(Utc::now()),
        limit: 50, // Последние 50 записей
        offset: 0,
        sort: "created".to_string(),
        order: "desc".to_string(),
    };
    
    match state.store.search_audit_logs(&filter).await {
        Ok(result) => {
            let records = result.records.iter().map(|r| {
                AuditRecord {
                    id: r.id,
                    timestamp: r.created,
                    user_id: r.user_id,
                    username: r.username.clone(),
                    action: r.action.to_string(),
                    resource_kind: r.object_type.to_string(),
                    resource_name: r.object_name.clone().unwrap_or_else(|| r.object_id.map(|id| id.to_string()).unwrap_or_default()),
                    namespace: String::new(), // Audit log не хранит namespace
                    description: r.description.clone(),
                    level: r.level.to_string(),
                }
            }).collect();
            
            Ok(records)
        }
        Err(e) => {
            // Если audit log недоступен, возвращаем пустой список
            tracing::warn!("Failed to get audit logs: {}", e);
            Ok(Vec::new())
        }
    }
}

/// Получить метрики ресурса
async fn get_resource_metrics(
    state: &Arc<AppState>,
    query: &TroubleshootQuery,
) -> Result<ResourceMetrics> {
    // Пробуем получить метрики из Prometheus
    let prometheus_url = std::env::var("PROMETHEUS_URL")
        .unwrap_or_else(|_| String::new());
    
    if prometheus_url.is_empty() {
        // Prometheus не настроен, возвращаем заглушку
        return Ok(ResourceMetrics {
            cpu_usage: None,
            memory_usage: None,
            cpu_request: None,
            memory_request: None,
            cpu_limit: None,
            memory_limit: None,
            restart_count: None,
        });
    }
    
    // Получаем метрики из Prometheus
    let client = super::prometheus::PrometheusClient::new(prometheus_url);
    
    if query.kind == "Pod" {
        let (cpu_result, memory_result) = tokio::join!(
            client.get_pod_cpu(&query.namespace, &query.name),
            client.get_pod_memory(&query.namespace, &query.name)
        );
        
        let cpu_usage = cpu_result.ok()
            .and_then(|metrics| metrics.into_iter().next())
            .and_then(|m| match m.value {
                MetricValue::Single(v) => Some(format!("{:.2} cores", v)),
                _ => None,
            });
        
        let memory_usage = memory_result.ok()
            .and_then(|metrics| metrics.into_iter().next())
            .and_then(|m| match m.value {
                MetricValue::Single(v) => Some(format!("{:.2} MiB", v / 1024.0 / 1024.0)),
                _ => None,
            });
        
        Ok(ResourceMetrics {
            cpu_usage,
            memory_usage,
            cpu_request: None,
            memory_request: None,
            cpu_limit: None,
            memory_limit: None,
            restart_count: None,
        })
    } else {
        // Для других ресурсов пока возвращаем пустые метрики
        Ok(ResourceMetrics {
            cpu_usage: None,
            memory_usage: None,
            cpu_request: None,
            memory_request: None,
            cpu_limit: None,
            memory_limit: None,
            restart_count: None,
        })
    }
}

/// Сгенерировать рекомендации на основе событий
fn generate_recommendations(
    timeline: &[TimelineEvent],
    metrics: &Option<ResourceMetrics>,
    query: &TroubleshootQuery,
) -> Vec<Recommendation> {
    let mut recommendations = Vec::new();

    // Проверка на Warning/Critical события
    let warning_count = timeline
        .iter()
        .filter(|e| e.severity == Severity::Warning || e.severity == Severity::Critical)
        .count();

    if warning_count > 0 {
        recommendations.push(Recommendation {
            priority: Priority::High,
            title: "Обнаружены проблемные события".to_string(),
            description: format!(
                "Найдено {} событий с предупреждениями или ошибками за последние 24 часа",
                warning_count
            ),
            suggested_actions: vec![
                "Проверьте логи пода с помощью kubectl logs".to_string(),
                "Изучите детали событий в разделе Events".to_string(),
                "Проверьте доступность зависимостей (сервисы, configmaps)".to_string(),
            ],
        });
    }

    // Проверка на частые рестарты
    if let Some(metrics_data) = metrics {
        if let Some(restarts) = metrics_data.restart_count {
            if restarts > 3 {
                recommendations.push(Recommendation {
                    priority: Priority::High,
                    title: "Частые перезапуски пода".to_string(),
                    description: format!("Под был перезапущен {} раз", restarts),
                    suggested_actions: vec![
                        "Проверьте лимиты ресурсов (CPU/memory)".to_string(),
                        "Изучите логи на наличие panic/error".to_string(),
                        "Проверьте readiness/liveness пробы".to_string(),
                    ],
                });
            }
        }
    }

    // Если рекомендаций нет
    if recommendations.is_empty() {
        recommendations.push(Recommendation {
            priority: Priority::Low,
            title: "Проблем не обнаружено".to_string(),
            description: "Ресурс работает в нормальном режиме".to_string(),
            suggested_actions: vec![
                "Продолжайте мониторинг".to_string(),
                "Проверьте метрики производительности".to_string(),
            ],
        });
    }

    recommendations
}

/// Рассчитать статус здоровья
fn calculate_health_status(
    timeline: &[TimelineEvent],
    metrics: &Option<ResourceMetrics>,
) -> HealthStatus {
    let critical_count = timeline
        .iter()
        .filter(|e| e.severity == Severity::Critical)
        .count();

    let warning_count = timeline
        .iter()
        .filter(|e| e.severity == Severity::Warning)
        .count();

    if critical_count > 0 {
        HealthStatus::Critical
    } else if warning_count > 2 {
        HealthStatus::Degraded
    } else if timeline.is_empty() {
        HealthStatus::Unknown
    } else {
        HealthStatus::Healthy
    }
}
