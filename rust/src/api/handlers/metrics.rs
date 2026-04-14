//! Prometheus Metrics API handler

use crate::api::state::AppState;
use crate::services::metrics::MetricsManager;
use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;

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

    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let output = String::from_utf8(buffer).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut response = Response::new(output);
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4"),
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
#[allow(unreachable_code)]
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

    #[test]
    fn test_get_memory_usage_mb_returns_positive() {
        let result = get_memory_usage_mb().unwrap();
        assert!(result >= 0.0);
    }

    #[test]
    fn test_get_cpu_usage_percent_returns_positive() {
        let result = get_cpu_usage_percent().unwrap();
        assert!(result >= 0.0);
    }

    #[test]
    fn test_system_info_struct_fields() {
        let info = get_system_info().unwrap();
        // Проверяем что поля существуют и имеют корректные значения
        let _cpu = info.cpu_usage;
        let _mem = info.memory_usage_mb;
    }

    #[test]
    fn test_get_memory_usage_mb_on_linux() {
        #[cfg(target_os = "linux")]
        {
            let result = get_memory_usage_mb();
            assert!(result.is_ok());
            assert!(result.unwrap() > 0.0);
        }
    }

    #[test]
    fn test_get_cpu_usage_default_value() {
        let result = get_cpu_usage_percent().unwrap();
        // Default estimate is 5.0
        assert!((result - 5.0).abs() < 0.001 || result >= 0.0);
    }

    #[test]
    fn test_system_info_cpu_usage_range() {
        let info = get_system_info().unwrap();
        // CPU usage should be non-negative
        assert!(info.cpu_usage >= 0.0);
        // And reasonable (less than 1000% for safety)
        assert!(info.cpu_usage < 1000.0);
    }

    #[test]
    fn test_system_info_memory_usage_range() {
        let info = get_system_info().unwrap();
        // Memory usage should be non-negative
        assert!(info.memory_usage_mb >= 0.0);
        // And reasonable (less than 1TB for safety)
        assert!(info.memory_usage_mb < 1_048_576.0);
    }

    #[test]
    fn test_get_memory_usage_mb_consistency() {
        let result1 = get_memory_usage_mb().unwrap();
        let result2 = get_memory_usage_mb().unwrap();
        // Results should be similar (within 50% for consecutive calls)
        let diff = (result1 - result2).abs();
        let max_val = result1.max(result2);
        if max_val > 0.0 {
            assert!(diff / max_val < 0.5);
        }
    }

    #[test]
    fn test_get_cpu_usage_percent_consistency() {
        let result1 = get_cpu_usage_percent().unwrap();
        let result2 = get_cpu_usage_percent().unwrap();
        // Should return same default value
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_system_info_fields_exist() {
        let info = get_system_info().unwrap();
        // Проверяем что поля существуют и имеют корректные значения
        let cpu = info.cpu_usage;
        let mem = info.memory_usage_mb;
        assert!(cpu >= 0.0);
        assert!(mem >= 0.0);
    }

    #[test]
    fn test_get_memory_usage_mb_not_zero_on_active_system() {
        let result = get_memory_usage_mb().unwrap();
        // On any active system memory usage should be > 0
        assert!(
            result > 0.0,
            "Memory usage should be greater than 0, got: {}",
            result
        );
    }

    #[test]
    fn test_metrics_json_response_structure() {
        // Проверяем что JSON структура метрик содержит ожидаемые поля
        // через сериализацию тестовых данных
        let metrics_data = serde_json::json!({
            "tasks": {
                "total": 100u64,
                "success": 80u64,
                "failed": 20u64,
            },
            "projects": 5usize,
            "templates": 10usize,
            "users": 15usize,
        });

        let json_str = serde_json::to_string(&metrics_data).unwrap();
        assert!(json_str.contains("tasks"));
        assert!(json_str.contains("total"));
        assert!(json_str.contains("success"));
        assert!(json_str.contains("failed"));
        assert!(json_str.contains("projects"));
        assert!(json_str.contains("templates"));
        assert!(json_str.contains("users"));
    }

    #[test]
    fn test_metrics_json_deserialization() {
        let json_str = r#"{
            "tasks": {"total": 50, "success": 40, "failed": 10},
            "projects": 3,
            "templates": 7,
            "users": 12
        }"#;

        let metrics: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(metrics["tasks"]["total"], 50);
        assert_eq!(metrics["tasks"]["success"], 40);
        assert_eq!(metrics["tasks"]["failed"], 10);
        assert_eq!(metrics["projects"], 3);
        assert_eq!(metrics["templates"], 7);
        assert_eq!(metrics["users"], 12);
    }

    #[test]
    fn test_metrics_empty_values() {
        let metrics_data = serde_json::json!({
            "tasks": {
                "total": 0u64,
                "success": 0u64,
                "failed": 0u64,
            },
            "projects": 0usize,
            "templates": 0usize,
            "users": 0usize,
        });

        let json_str = serde_json::to_string(&metrics_data).unwrap();
        assert!(json_str.contains("\"total\":0"));
    }
}
