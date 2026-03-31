//! Prometheus Client для получения метрик Kubernetes
//!
//! Интеграция с Prometheus API для получения метрик CPU, Memory, Network

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Prometheus Types
// ============================================================================

/// Prometheus метрика
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrometheusMetric {
    /// Название метрики
    pub metric: String,
    
    /// Значение
    pub value: MetricValue,
    
    /// Labels метрики
    #[serde(default)]
    pub labels: std::collections::HashMap<String, String>,
}

/// Значение метрики
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum MetricValue {
    /// Одиночное значение (gauge/counter)
    Single(f64),
    /// Временной ряд
    TimeSeries(Vec<(DateTime<Utc>, f64)>),
}

/// Query параметры для Prometheus
#[derive(Debug, Deserialize)]
pub struct PrometheusQuery {
    /// Query expression (PromQL)
    pub query: String,
    
    /// Время (RFC3339 или unix timestamp)
    #[serde(default)]
    pub time: Option<String>,
    
    /// Timeout для запроса
    #[serde(default)]
    pub timeout: Option<u32>,
}

/// Range query параметры
#[derive(Debug, Deserialize)]
pub struct PrometheusRangeQuery {
    /// Query expression (PromQL)
    pub query: String,
    
    /// Начало периода (RFC3339 или unix timestamp)
    pub start: String,
    
    /// Конец периода
    pub end: String,
    
    /// Шаг (duration string: 1m, 1h, 1d)
    pub step: String,
    
    /// Timeout
    #[serde(default)]
    pub timeout: Option<u32>,
}

/// Ответ Prometheus API
#[derive(Debug, Serialize, Deserialize)]
pub struct PrometheusResponse {
    pub status: String,
    pub data: PrometheusData,
}

/// Данные Prometheus
#[derive(Debug, Serialize, Deserialize)]
pub struct PrometheusData {
    #[serde(default)]
    pub result_type: String,
    
    #[serde(default)]
    pub result: Vec<PrometheusResult>,
}

/// Результат запроса
#[derive(Debug, Serialize, Deserialize)]
pub struct PrometheusResult {
    pub metric: std::collections::HashMap<String, String>,
    pub value: Option<(f64, String)>,
    #[serde(default)]
    pub values: Vec<(f64, String)>,
}

// ============================================================================
// Prometheus Client
// ============================================================================

/// Клиент Prometheus
pub struct PrometheusClient {
    client: Client,
    base_url: String,
}

impl PrometheusClient {
    /// Создаёт новый клиент Prometheus
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    /// Instant query — текущее значение
    pub async fn query(&self, query: &str) -> Result<Vec<PrometheusMetric>> {
        let url = format!("{}/api/v1/query", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[("query", query)])
            .send()
            .await
            .map_err(|e| Error::Other(format!("Prometheus query failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "Prometheus API error: {}",
                response.status()
            )));
        }

        let prom_response: PrometheusResponse = response
            .json()
            .await
            .map_err(|e| Error::Other(format!("Failed to parse Prometheus response: {}", e)))?;

        if prom_response.status != "success" {
            return Err(Error::Other("Prometheus returned error status".to_string()));
        }

        let metrics = prom_response
            .data
            .result
            .into_iter()
            .filter_map(|r| {
                r.value.map(|(ts, val)| {
                    let value = val.parse::<f64>().unwrap_or(0.0);
                    let metric_name = r.metric.get("__name__").cloned().unwrap_or_default();
                    
                    PrometheusMetric {
                        metric: metric_name,
                        value: MetricValue::Single(value),
                        labels: r.metric,
                    }
                })
            })
            .collect();

        Ok(metrics)
    }

    /// Range query — временной ряд
    pub async fn query_range(
        &self,
        query: &str,
        start: i64,
        end: i64,
        step: &str,
    ) -> Result<Vec<PrometheusMetric>> {
        let url = format!("{}/api/v1/query_range", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[
                ("query", query),
                ("start", &start.to_string()),
                ("end", &end.to_string()),
                ("step", step),
            ])
            .send()
            .await
            .map_err(|e| Error::Other(format!("Prometheus range query failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "Prometheus API error: {}",
                response.status()
            )));
        }

        let prom_response: PrometheusResponse = response
            .json()
            .await
            .map_err(|e| Error::Other(format!("Failed to parse Prometheus response: {}", e)))?;

        if prom_response.status != "success" {
            return Err(Error::Other("Prometheus returned error status".to_string()));
        }

        let metrics = prom_response
            .data
            .result
            .into_iter()
            .map(|r| {
                let metric_name = r.metric.get("__name__").cloned().unwrap_or_default();
                
                let time_series: Vec<(DateTime<Utc>, f64)> = r
                    .values
                    .into_iter()
                    .filter_map(|(ts, val)| {
                        let dt = DateTime::from_timestamp(ts as i64, 0);
                        let value = val.parse::<f64>().ok()?;
                        dt.map(|t| (t.into(), value))
                    })
                    .collect();

                PrometheusMetric {
                    metric: metric_name,
                    value: MetricValue::TimeSeries(time_series),
                    labels: r.metric,
                }
            })
            .collect();

        Ok(metrics)
    }

    /// Получить метрики CPU для Pod
    pub async fn get_pod_cpu(&self, namespace: &str, pod: &str) -> Result<Vec<PrometheusMetric>> {
        let query = format!(
            "rate(container_cpu_usage_seconds_total{{namespace=\"{}\", pod=\"{}\"}}[5m])",
            namespace, pod
        );
        self.query(&query).await
    }

    /// Получить метрики Memory для Pod
    pub async fn get_pod_memory(&self, namespace: &str, pod: &str) -> Result<Vec<PrometheusMetric>> {
        let query = format!(
            "container_memory_usage_bytes{{namespace=\"{}\", pod=\"{}\"}}",
            namespace, pod
        );
        self.query(&query).await
    }

    /// Получить метрики Network для Pod
    pub async fn get_pod_network(&self, namespace: &str, pod: &str) -> Result<Vec<PrometheusMetric>> {
        let query_rx = format!(
            "rate(container_network_receive_bytes_total{{namespace=\"{}\", pod=\"{}\"}}[5m])",
            namespace, pod
        );
        let query_tx = format!(
            "rate(container_network_transmit_bytes_total{{namespace=\"{}\", pod=\"{}\"}}[5m])",
            namespace, pod
        );
        
        let (rx, tx) = tokio::join!(
            self.query(&query_rx),
            self.query(&query_tx)
        );
        
        let mut metrics = rx?;
        metrics.extend(tx?);
        Ok(metrics)
    }
}

// ============================================================================
// API Handlers
// ============================================================================

/// Query параметры для get_prometheus_metrics
#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub namespace: String,
    pub kind: String,
    pub name: String,
    pub metric_type: Option<String>, // cpu, memory, network, all
    pub range: Option<String>,       // 1h, 6h, 24h, 7d
}

/// Получить метрики Prometheus для Kubernetes ресурса
pub async fn get_prometheus_metrics(
    State(state): State<Arc<AppState>>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<PrometheusMetricsResponse>> {
    // Получаем Prometheus URL из конфига
    let prometheus_url = std::env::var("PROMETHEUS_URL")
        .unwrap_or_else(|_| "http://prometheus:9090".to_string());
    
    let client = PrometheusClient::new(prometheus_url);
    
    let metric_type = query.metric_type.as_deref().unwrap_or("all");
    let range = query.range.as_deref().unwrap_or("1h");
    
    // Вычисляем временной диапазон
    let now = Utc::now();
    let start = calculate_range_start(range);
    
    let mut metrics = Vec::new();
    
    // Получаем метрики в зависимости от типа ресурса
    if query.kind == "Pod" {
        if metric_type == "all" || metric_type == "cpu" {
            if let Ok(cpu) = client.get_pod_cpu(&query.namespace, &query.name).await {
                metrics.extend(cpu);
            }
        }
        
        if metric_type == "all" || metric_type == "memory" {
            if let Ok(mem) = client.get_pod_memory(&query.namespace, &query.name).await {
                metrics.extend(mem);
            }
        }
        
        if metric_type == "all" || metric_type == "network" {
            if let Ok(net) = client.get_pod_network(&query.namespace, &query.name).await {
                metrics.extend(net);
            }
        }
    } else {
        // Для других ресурсов (Deployment, StatefulSet) — агрегированные метрики по Pod
        // TODO: реализовать агрегацию по label selectors
        return Err(Error::NotFound(
            format!("Metrics for {} not implemented", query.kind)
        ));
    }
    
    Ok(Json(PrometheusMetricsResponse {
        resource: ResourceRef {
            kind: query.kind,
            name: query.name,
            namespace: query.namespace,
        },
        metrics,
        range: range.to_string(),
        timestamp: now,
    }))
}

/// Ответ с метриками
#[derive(Debug, Serialize)]
pub struct PrometheusMetricsResponse {
    pub resource: ResourceRef,
    pub metrics: Vec<PrometheusMetric>,
    pub range: String,
    pub timestamp: DateTime<Utc>,
}

/// Ссылка на ресурс
#[derive(Debug, Serialize)]
pub struct ResourceRef {
    pub kind: String,
    pub name: String,
    pub namespace: String,
}

/// Вычислить начало временного диапазона
fn calculate_range_start(range: &str) -> i64 {
    let now = Utc::now();
    let seconds = match range {
        "1h" => 3600,
        "6h" => 6 * 3600,
        "24h" | "1d" => 24 * 3600,
        "7d" | "1w" => 7 * 24 * 3600,
        _ => 3600, // default 1h
    };
    now.timestamp() - seconds as i64
}

/// Проверить доступность Prometheus
pub async fn check_prometheus_health(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PrometheusHealthResponse>> {
    let prometheus_url = std::env::var("PROMETHEUS_URL")
        .unwrap_or_else(|_| "http://prometheus:9090".to_string());
    
    let client = reqwest::Client::new();
    
    match client.get(&format!("{}/api/v1/status/config", prometheus_url)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                Ok(Json(PrometheusHealthResponse {
                    status: "healthy".to_string(),
                    url: prometheus_url,
                    message: "Prometheus is available".to_string(),
                }))
            } else {
                Ok(Json(PrometheusHealthResponse {
                    status: "unhealthy".to_string(),
                    url: prometheus_url,
                    message: format!("Prometheus returned {}", response.status()),
                }))
            }
        }
        Err(e) => Ok(Json(PrometheusHealthResponse {
            status: "unavailable".to_string(),
            url: prometheus_url,
            message: format!("Prometheus is not available: {}", e),
        })),
    }
}

#[derive(Debug, Serialize)]
pub struct PrometheusHealthResponse {
    pub status: String,
    pub url: String,
    pub message: String,
}
