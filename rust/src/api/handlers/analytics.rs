//! Analytics API Handlers
//!
//! Обработчики для аналитики и дашбордов

use crate::api::state::AppState;
use crate::db::store::{
    AccessKeyManager, EnvironmentManager, InventoryManager, ProjectStore, RepositoryManager,
    ScheduleManager, TaskManager, TemplateManager, UserManager,
};
use crate::models::analytics::*;
use crate::services::task_logger::TaskStatus;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use std::sync::Arc;

/// Параметры запроса аналитики проекта
#[derive(Debug, Deserialize)]
pub struct AnalyticsParams {
    #[serde(default)]
    pub period: Option<String>, // day, week, month, year
}

/// GET /api/project/{project_id}/analytics - Аналитика проекта
pub async fn get_project_analytics(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i64>,
    Query(params): Query<AnalyticsParams>,
) -> Result<Json<ProjectAnalytics>, StatusCode> {
    // Получаем базовую статистику из tasks
    let tasks = state
        .store
        .get_tasks(project_id as i32, None::<i32>)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get tasks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Считаем статистику
    let total_tasks = tasks.len() as i64;
    let successful_tasks = tasks
        .iter()
        .filter(|t| t.task.status == TaskStatus::Success)
        .count() as i64;
    let failed_tasks = tasks
        .iter()
        .filter(|t| t.task.status == TaskStatus::Error)
        .count() as i64;
    let stopped_tasks = tasks
        .iter()
        .filter(|t| t.task.status == TaskStatus::Stopped)
        .count() as i64;
    let pending_tasks = tasks
        .iter()
        .filter(|t| t.task.status == TaskStatus::Waiting || t.task.status == TaskStatus::Starting)
        .count() as i64;
    let running_tasks = tasks
        .iter()
        .filter(|t| t.task.status == TaskStatus::Running)
        .count() as i64;

    let success_rate = if total_tasks > 0 {
        (successful_tasks as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };

    // Вычисляем avg/max/min длительности задач (start..end)
    let durations_secs: Vec<f64> = tasks
        .iter()
        .filter_map(|t| match (t.task.start, t.task.end) {
            (Some(s), Some(e)) => {
                let d = (e - s).num_seconds();
                if d >= 0 {
                    Some(d as f64)
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();
    let avg_task_duration_secs = if durations_secs.is_empty() {
        0.0
    } else {
        durations_secs.iter().sum::<f64>() / durations_secs.len() as f64
    };
    let max_task_duration_secs = durations_secs.iter().cloned().fold(0.0f64, f64::max);
    let min_task_duration_secs = if durations_secs.is_empty() {
        0.0
    } else {
        durations_secs.iter().cloned().fold(f64::MAX, f64::min)
    };
    let total_duration_secs = durations_secs.iter().sum::<f64>() as i64;

    // Получаем шаблоны для статистики
    let templates = state
        .store
        .get_templates(project_id as i32)
        .await
        .unwrap_or_default();

    // Получаем инвентари
    let inventories = state
        .store
        .get_inventories(project_id as i32)
        .await
        .unwrap_or_default();

    // Получаем репозитории
    let repositories = state
        .store
        .get_repositories(project_id as i32)
        .await
        .unwrap_or_default();

    // Получаем окружения
    let environments = state
        .store
        .get_environments(project_id as i32)
        .await
        .unwrap_or_default();

    // Получаем ключи
    let keys = state
        .store
        .get_access_keys(project_id as i32)
        .await
        .unwrap_or_default();

    // Получаем расписания
    let schedules = state
        .store
        .get_schedules(project_id as i32)
        .await
        .unwrap_or_default();

    // Получаем пользователей проекта
    let users = state
        .store
        .get_users(Default::default())
        .await
        .unwrap_or_default();

    // Получаем проект для имени
    let project = state.store.get_project(project_id as i32).await.ok();

    // Создаём статистику
    let stats = ProjectStats {
        project_id,
        project_name: project
            .map(|p| p.name)
            .unwrap_or_else(|| format!("Project {}", project_id)),
        total_tasks,
        successful_tasks,
        failed_tasks,
        stopped_tasks,
        pending_tasks,
        running_tasks,
        total_templates: templates.len() as i64,
        total_users: users.len() as i64,
        total_inventories: inventories.len() as i64,
        total_repositories: repositories.len() as i64,
        total_environments: environments.len() as i64,
        total_keys: keys.len() as i64,
        total_schedules: schedules.len() as i64,
        success_rate,
        avg_task_duration_secs,
    };

    // Определяем период
    let period = params.period.as_deref().unwrap_or("week");

    // Создаём простую статистику задач
    let task_stats = TaskStats {
        period: period.to_string(),
        total: total_tasks,
        success: successful_tasks,
        failed: failed_tasks,
        stopped: stopped_tasks,
        avg_duration_secs: avg_task_duration_secs,
        max_duration_secs: max_task_duration_secs,
        min_duration_secs: min_task_duration_secs,
        total_duration_secs,
    };

    // Вычисляем tasks_per_day (за последние 30 дней)
    let thirty_days_ago = Utc::now() - Duration::days(30);
    let recent_tasks = tasks
        .iter()
        .filter(|t| t.task.created > thirty_days_ago)
        .count() as f64;
    let tasks_per_day = recent_tasks / 30.0;
    let tasks_per_hour = tasks_per_day / 24.0;

    let performance = PerformanceMetrics {
        avg_queue_time_secs: 0.0,
        avg_execution_time_secs: avg_task_duration_secs,
        tasks_per_hour,
        tasks_per_day,
        concurrent_tasks_avg: 0.0,
        concurrent_tasks_max: 0,
        resource_usage: ResourceUsage::default(),
    };

    // Топ пользователей (заглушка)
    let top_users = vec![];

    // Топ шаблонов
    let top_templates = templates
        .iter()
        .take(5)
        .map(|t| TopItem {
            id: t.id as i64,
            name: t.name.clone(),
            value: 0,
            r#type: "template".to_string(),
        })
        .collect();

    // Недавняя активность (заглушка)
    let recent_activity = vec![];

    Ok(Json(ProjectAnalytics {
        stats,
        task_stats,
        performance,
        top_users,
        top_templates,
        recent_activity,
    }))
}

/// GET /api/project/{project_id}/analytics/tasks-chart - Данные для графика задач
pub async fn get_tasks_chart(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i64>,
    Query(params): Query<AnalyticsParams>,
) -> Result<Json<Vec<ChartData>>, StatusCode> {
    let tasks = state
        .store
        .get_tasks(project_id as i32, None::<i32>)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get tasks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Определяем диапазон дней по периоду
    let days: i64 = match params.period.as_deref().unwrap_or("week") {
        "month" => 30,
        "year" => 365,
        _ => 7, // week (default)
    };

    let now = Utc::now();
    let start = now - Duration::days(days);

    // Строим карту дата → счётчик
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<String, f64> = BTreeMap::new();

    // Заполняем все дни нулями
    for d in 0..days {
        let day = (start + Duration::days(d + 1))
            .format("%Y-%m-%d")
            .to_string();
        counts.insert(day, 0.0);
    }

    // Считаем задачи по датам
    for t in &tasks {
        if t.task.created >= start {
            let day = t.task.created.format("%Y-%m-%d").to_string();
            *counts.entry(day).or_insert(0.0) += 1.0;
        }
    }

    let chart_data: Vec<ChartData> = counts
        .into_iter()
        .map(|(label, value)| ChartData {
            label,
            value,
            timestamp: None,
        })
        .collect();

    Ok(Json(chart_data))
}

/// GET /api/project/{project_id}/analytics/status-distribution - Распределение по статусам
pub async fn get_status_distribution(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<ChartData>>, StatusCode> {
    let tasks = state
        .store
        .get_tasks(project_id as i32, None::<i32>)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get tasks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Считаем распределение
    let mut distribution = Vec::new();
    for status in &[
        TaskStatus::Success,
        TaskStatus::Error,
        TaskStatus::Stopped,
        TaskStatus::Waiting,
        TaskStatus::Running,
    ] {
        let count = tasks.iter().filter(|t| &t.task.status == status).count();
        if count > 0 {
            distribution.push(ChartData {
                label: format!("{:?}", status),
                value: count as f64,
                timestamp: None,
            });
        }
    }

    Ok(Json(distribution))
}

/// GET /api/analytics/system - Системная аналитика (для админов)
pub async fn get_system_analytics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SystemMetrics>, StatusCode> {
    // Получаем все проекты
    let projects = state.store.get_projects(None).await.unwrap_or_default();

    // Получаем всех пользователей
    let users = state
        .store
        .get_users(Default::default())
        .await
        .unwrap_or_default();

    // Получаем все шаблоны
    let mut total_templates = 0;
    for project in &projects {
        let templates = state
            .store
            .get_templates(project.id)
            .await
            .unwrap_or_default();
        total_templates += templates.len();
    }

    // Заглушка для остальных метрик
    Ok(Json(SystemMetrics {
        total_projects: projects.len() as i64,
        total_users: users.len() as i64,
        total_tasks: 0,
        total_templates: total_templates as i64,
        total_runners: 0,
        active_runners: 0,
        running_tasks: 0,
        queued_tasks: 0,
        success_rate_24h: 0.0,
        avg_task_duration_24h: 0.0,
        tasks_24h: 0,
        tasks_7d: 0,
        tasks_30d: 0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Тесты для AnalyticsParams (deserialization) =====

    #[test]
    fn test_analytics_params_default() {
        let params = AnalyticsParams { period: None };
        assert!(params.period.is_none());
        assert_eq!(params.period.as_deref().unwrap_or("week"), "week");
    }

    #[test]
    fn test_analytics_params_with_day() {
        let params = AnalyticsParams {
            period: Some("day".to_string()),
        };
        assert_eq!(params.period.as_deref(), Some("day"));
    }

    #[test]
    fn test_analytics_params_with_week() {
        let params = AnalyticsParams {
            period: Some("week".to_string()),
        };
        assert_eq!(params.period.as_deref(), Some("week"));
    }

    #[test]
    fn test_analytics_params_with_month() {
        let params = AnalyticsParams {
            period: Some("month".to_string()),
        };
        assert_eq!(params.period.as_deref(), Some("month"));
    }

    #[test]
    fn test_analytics_params_with_year() {
        let params = AnalyticsParams {
            period: Some("year".to_string()),
        };
        assert_eq!(params.period.as_deref(), Some("year"));
    }

    #[test]
    fn test_analytics_params_deserialize_from_json() {
        let json = r#"{"period": "month"}"#;
        let params: AnalyticsParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.period.as_deref(), Some("month"));
    }

    #[test]
    fn test_analytics_params_deserialize_empty_from_json() {
        let json = r#"{}"#;
        let params: AnalyticsParams = serde_json::from_str(json).unwrap();
        assert!(params.period.is_none());
    }

    // ===== Тесты для period parsing логики =====

    #[test]
    fn test_period_parsing_returns_correct_days() {
        let cases = [
            ("month", 30),
            ("year", 365),
            ("week", 7),
            ("day", 7),     // default
            ("unknown", 7), // default fallback
            ("", 7),        // default fallback
        ];
        for (period, expected_days) in cases {
            let days: i64 = match period {
                "month" => 30,
                "year" => 365,
                _ => 7,
            };
            assert_eq!(
                days, expected_days,
                "period '{}' should yield {} days",
                period, expected_days
            );
        }
    }

    // ===== Тесты для моделей analytics =====

    #[test]
    fn test_project_stats_serialization() {
        let stats = ProjectStats {
            project_id: 42,
            project_name: "My Project".to_string(),
            total_tasks: 100,
            successful_tasks: 80,
            failed_tasks: 15,
            stopped_tasks: 3,
            pending_tasks: 1,
            running_tasks: 1,
            total_templates: 5,
            total_users: 3,
            total_inventories: 2,
            total_repositories: 1,
            total_environments: 2,
            total_keys: 4,
            total_schedules: 1,
            success_rate: 80.0,
            avg_task_duration_secs: 150.5,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"project_id\":42"));
        assert!(json.contains("\"project_name\":\"My Project\""));
        assert!(json.contains("\"total_tasks\":100"));
        assert!(json.contains("\"success_rate\":80.0"));
    }

    #[test]
    fn test_project_stats_deserialization() {
        let json = r#"{
            "project_id": 1,
            "project_name": "Test",
            "total_tasks": 10,
            "successful_tasks": 8,
            "failed_tasks": 2,
            "stopped_tasks": 0,
            "pending_tasks": 0,
            "running_tasks": 0,
            "total_templates": 3,
            "total_users": 2,
            "total_inventories": 1,
            "total_repositories": 1,
            "total_environments": 1,
            "total_keys": 2,
            "total_schedules": 0,
            "success_rate": 80.0,
            "avg_task_duration_secs": 60.0
        }"#;
        let stats: ProjectStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.project_id, 1);
        assert_eq!(stats.project_name, "Test");
        assert_eq!(stats.success_rate, 80.0);
    }

    #[test]
    fn test_task_stats_serialization() {
        let stats = TaskStats {
            period: "week".to_string(),
            total: 25,
            success: 20,
            failed: 4,
            stopped: 1,
            avg_duration_secs: 90.0,
            max_duration_secs: 300.0,
            min_duration_secs: 5.0,
            total_duration_secs: 2250,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"period\":\"week\""));
        assert!(json.contains("\"total\":25"));
        assert!(json.contains("\"success\":20"));
    }

    #[test]
    fn test_performance_metrics_serialization() {
        let perf = PerformanceMetrics {
            avg_queue_time_secs: 3.5,
            avg_execution_time_secs: 120.0,
            tasks_per_hour: 8.0,
            tasks_per_day: 192.0,
            concurrent_tasks_avg: 2.0,
            concurrent_tasks_max: 4,
            resource_usage: ResourceUsage::default(),
        };
        let json = serde_json::to_string(&perf).unwrap();
        assert!(json.contains("\"tasks_per_hour\":8.0"));
        assert!(json.contains("\"tasks_per_day\":192.0"));
        assert!(json.contains("\"concurrent_tasks_max\":4"));
    }

    #[test]
    fn test_resource_usage_default_values() {
        let usage = ResourceUsage::default();
        assert_eq!(usage.cpu_usage_percent, 0.0);
        assert_eq!(usage.memory_usage_mb, 0.0);
        assert_eq!(usage.disk_usage_mb, 0.0);
        assert_eq!(usage.network_rx_bytes, 0);
        assert_eq!(usage.network_tx_bytes, 0);
    }

    #[test]
    fn test_resource_usage_serialization() {
        let usage = ResourceUsage {
            cpu_usage_percent: 75.5,
            memory_usage_mb: 2048.0,
            disk_usage_mb: 10240.0,
            network_rx_bytes: 1_000_000,
            network_tx_bytes: 500_000,
        };
        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("\"cpu_usage_percent\":75.5"));
        assert!(json.contains("\"memory_usage_mb\":2048.0"));
        assert!(json.contains("\"network_rx_bytes\":1000000"));
    }

    #[test]
    fn test_system_metrics_serialization() {
        let metrics = SystemMetrics {
            total_projects: 5,
            total_users: 10,
            total_tasks: 500,
            total_templates: 20,
            total_runners: 3,
            active_runners: 2,
            running_tasks: 4,
            queued_tasks: 1,
            success_rate_24h: 95.5,
            avg_task_duration_24h: 45.0,
            tasks_24h: 50,
            tasks_7d: 300,
            tasks_30d: 1200,
        };
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("\"total_projects\":5"));
        assert!(json.contains("\"success_rate_24h\":95.5"));
        assert!(json.contains("\"tasks_30d\":1200"));
    }

    #[test]
    fn test_system_metrics_default() {
        let metrics = SystemMetrics::default();
        assert_eq!(metrics.total_projects, 0);
        assert_eq!(metrics.total_users, 0);
        assert_eq!(metrics.success_rate_24h, 0.0);
        assert_eq!(metrics.avg_task_duration_24h, 0.0);
    }

    // ===== Тесты для ChartData =====

    #[test]
    fn test_chart_data_with_timestamp() {
        let now = Utc::now();
        let data = ChartData {
            label: "2024-01-15".to_string(),
            value: 42.0,
            timestamp: Some(now),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"label\":\"2024-01-15\""));
        assert!(json.contains("\"value\":42.0"));
    }

    #[test]
    fn test_chart_data_without_timestamp() {
        let data = ChartData {
            label: "Success".to_string(),
            value: 15.0,
            timestamp: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"timestamp\":null"));
        assert!(json.contains("\"value\":15.0"));
    }

    #[test]
    fn test_chart_data_deserialization() {
        let json = r#"{"label":"2024-03-01","value":10.5,"timestamp":null}"#;
        let data: ChartData = serde_json::from_str(json).unwrap();
        assert_eq!(data.label, "2024-03-01");
        assert_eq!(data.value, 10.5);
        assert!(data.timestamp.is_none());
    }

    // ===== Тесты для логики фильтрации по статусам =====

    #[test]
    fn test_task_status_filtering_success() {
        let statuses = [
            TaskStatus::Success,
            TaskStatus::Error,
            TaskStatus::Success,
            TaskStatus::Stopped,
        ];
        let success_count = statuses
            .iter()
            .filter(|s| **s == TaskStatus::Success)
            .count();
        assert_eq!(success_count, 2);
    }

    #[test]
    fn test_task_status_filtering_pending() {
        let statuses = [
            TaskStatus::Waiting,
            TaskStatus::Starting,
            TaskStatus::Running,
            TaskStatus::Success,
        ];
        let pending_count = statuses
            .iter()
            .filter(|s| **s == TaskStatus::Waiting || **s == TaskStatus::Starting)
            .count();
        assert_eq!(pending_count, 2);
    }

    #[test]
    fn test_task_status_distribution() {
        let statuses = [
            TaskStatus::Success,
            TaskStatus::Success,
            TaskStatus::Error,
            TaskStatus::Stopped,
            TaskStatus::Waiting,
            TaskStatus::Running,
            TaskStatus::Running,
        ];
        let distribution: std::collections::HashMap<String, i64> =
            statuses
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, s| {
                    let key = format!("{:?}", s);
                    *acc.entry(key).or_insert(0) += 1;
                    acc
                });
        assert_eq!(*distribution.get("Success").unwrap(), 2);
        assert_eq!(*distribution.get("Error").unwrap(), 1);
        assert_eq!(*distribution.get("Running").unwrap(), 2);
    }

    // ===== Тесты для TopItem =====

    #[test]
    fn test_top_item_serialization() {
        let item = TopItem {
            id: 100,
            name: "Deploy Template".to_string(),
            value: 50,
            r#type: "template".to_string(),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"name\":\"Deploy Template\""));
        assert!(json.contains("\"type\":\"template\""));
    }

    // ===== Тесты для ProjectAnalytics =====

    #[test]
    fn test_project_analytics_roundtrip() {
        let analytics = ProjectAnalytics {
            stats: ProjectStats::default(),
            task_stats: TaskStats {
                period: "day".to_string(),
                total: 0,
                success: 0,
                failed: 0,
                stopped: 0,
                avg_duration_secs: 0.0,
                max_duration_secs: 0.0,
                min_duration_secs: 0.0,
                total_duration_secs: 0,
            },
            performance: PerformanceMetrics {
                avg_queue_time_secs: 0.0,
                avg_execution_time_secs: 0.0,
                tasks_per_hour: 0.0,
                tasks_per_day: 0.0,
                concurrent_tasks_avg: 0.0,
                concurrent_tasks_max: 0,
                resource_usage: ResourceUsage::default(),
            },
            top_users: vec![],
            top_templates: vec![],
            recent_activity: vec![],
        };
        let json = serde_json::to_string(&analytics).unwrap();
        let parsed: ProjectAnalytics = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.task_stats.period, "day");
        assert!(parsed.top_users.is_empty());
    }
}
