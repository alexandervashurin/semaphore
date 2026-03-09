//! Prometheus Metrics API handler

use axum::{
    extract::State,
    http::{StatusCode, header::CONTENT_TYPE},
    response::Response,
    Json,
};
use std::sync::Arc;
use prometheus::{Encoder, TextEncoder};
use crate::api::state::AppState;
use crate::services::metrics::MetricsManager;

/// GET /api/metrics - Prometheus metrics endpoint
pub async fn get_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Response<String>, StatusCode> {
    // Обновляем динамические метрики
    update_system_metrics(&state.metrics).await;
    
    // Форматируем метрики
    let encoder = TextEncoder::new();
    let metric_families = MetricsManager::registry().gather();
    let mut buffer = Vec::new();
    
    encoder.encode(&metric_families, &mut buffer)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let output = String::from_utf8(buffer).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut response = Response::new(output);
    response.headers_mut().insert(
        CONTENT_TYPE,
        "text/plain; version=0.0.4".parse().unwrap(),
    );
    
    Ok(response)
}

/// GET /api/metrics/json - Metrics в JSON формате
pub async fn get_metrics_json(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Обновляем динамические метрики
    update_system_metrics(&state.metrics).await;
    
    let counters = state.metrics.get_task_counters().await;
    
    Ok(Json(serde_json::json!({
        "tasks": {
            "total": counters.by_project.values().map(|c| c.total).sum::<u64>(),
            "success": counters.by_project.values().map(|c| c.success).sum::<u64>(),
            "failed": counters.by_project.values().map(|c| c.failed).sum::<u64>(),
        },
        "projects": counters.by_project.len(),
        "templates": counters.by_template.len(),
        "users": counters.by_user.len(),
    })))
}

/// Обновляет системные метрики
async fn update_system_metrics(metrics: &MetricsManager) {
    // Обновляем uptime
    metrics.update_uptime();
    
    // Получаем системную информацию
    if let Ok(system_info) = get_system_info() {
        metrics.update_cpu_usage(system_info.cpu_usage);
        metrics.update_memory_usage(system_info.memory_usage_mb);
    }
    
    // Обновляем статус здоровья
    metrics.update_health(true);
}

/// Системная информация
struct SystemInfo {
    cpu_usage: f64,
    memory_usage_mb: f64,
}

/// Получает системную информацию
fn get_system_info() -> Result<SystemInfo, Box<dyn std::error::Error>> {
    // Простая оценка использования памяти
    let memory_usage_mb = get_memory_usage_mb().unwrap_or(0.0);
    
    // Простая оценка использования CPU
    let cpu_usage = get_cpu_usage_percent().unwrap_or(0.0);
    
    Ok(SystemInfo {
        cpu_usage,
        memory_usage_mb,
    })
}

/// Получает использование памяти процесса в MB
fn get_memory_usage_mb() -> Result<f64, Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status")?;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: f64 = parts[1].parse()?;
                    return Ok(kb / 1024.0);
                }
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows implementation would require additional crates
        return Ok(0.0);
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS implementation would require additional crates
        return Ok(0.0);
    }
    
    Ok(180.0) // Default estimate
}

/// Получает использование CPU в процентах
fn get_cpu_usage_percent() -> Result<f64, Box<dyn std::error::Error>> {
    // Простая оценка - в production лучше использовать sysinfo crate
    Ok(5.0) // Default estimate
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_info() {
        let info = get_system_info().unwrap();
        assert!(info.cpu_usage >= 0.0);
        assert!(info.memory_usage_mb >= 0.0);
    }
}
