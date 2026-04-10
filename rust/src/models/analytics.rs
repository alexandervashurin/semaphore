//! Analytics - Модели для аналитики и дашбордов
//!
//! Предоставляет структуры для:
//! - Статистики проектов
//! - Метрик задач
//! - Активности пользователей
//! - Использования ресурсов

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Основная статистика проекта
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectStats {
    pub project_id: i64,
    pub project_name: String,
    pub total_tasks: i64,
    pub successful_tasks: i64,
    pub failed_tasks: i64,
    pub stopped_tasks: i64,
    pub pending_tasks: i64,
    pub running_tasks: i64,
    pub total_templates: i64,
    pub total_users: i64,
    pub total_inventories: i64,
    pub total_repositories: i64,
    pub total_environments: i64,
    pub total_keys: i64,
    pub total_schedules: i64,
    pub success_rate: f64,
    pub avg_task_duration_secs: f64,
}

/// Статистика задач за период
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub period: String,
    pub total: i64,
    pub success: i64,
    pub failed: i64,
    pub stopped: i64,
    pub avg_duration_secs: f64,
    pub max_duration_secs: f64,
    pub min_duration_secs: f64,
    pub total_duration_secs: i64,
}

/// Активность пользователей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub user_id: i64,
    pub username: String,
    pub total_actions: i64,
    pub tasks_created: i64,
    pub tasks_run: i64,
    pub templates_created: i64,
    pub last_activity: DateTime<Utc>,
}

/// Метрики производительности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_queue_time_secs: f64,
    pub avg_execution_time_secs: f64,
    pub tasks_per_hour: f64,
    pub tasks_per_day: f64,
    pub concurrent_tasks_avg: f64,
    pub concurrent_tasks_max: i64,
    pub resource_usage: ResourceUsage,
}

/// Использование ресурсов
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub disk_usage_mb: f64,
    pub network_rx_bytes: i64,
    pub network_tx_bytes: i64,
}

/// Данные для графика
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub label: String,
    pub value: f64,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Временной ряд для графиков
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    pub metric: String,
    pub data: Vec<ChartData>,
}

/// Статус системы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub healthy: bool,
    pub version: String,
    pub uptime_secs: i64,
    pub active_runners: i64,
    pub running_tasks: i64,
    pub queued_tasks: i64,
    pub database_status: String,
    pub last_check: DateTime<Utc>,
}

/// Топ элементов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopItem {
    pub id: i64,
    pub name: String,
    pub value: i64,
    pub r#type: String,
}

/// Топ задач по длительности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopSlowTask {
    pub task_id: i64,
    pub task_name: String,
    pub template_name: String,
    pub duration_secs: f64,
    pub status: String,
    pub created: DateTime<Utc>,
}

/// Топ пользователей по активности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopUser {
    pub user_id: i64,
    pub username: String,
    pub tasks_count: i64,
    pub success_rate: f64,
}

/// Параметры запроса аналитики
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyticsQueryParams {
    pub project_id: Option<i64>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub period: Option<String>, // hour, day, week, month
    pub limit: Option<i64>,
    pub group_by: Option<String>, // user, template, status
}

/// Сводная аналитика проекта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalytics {
    pub stats: ProjectStats,
    pub task_stats: TaskStats,
    pub performance: PerformanceMetrics,
    pub top_users: Vec<TopUser>,
    pub top_templates: Vec<TopItem>,
    pub recent_activity: Vec<ChartData>,
}

/// Метрики раннера
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunnerMetrics {
    pub runner_id: i64,
    pub runner_name: String,
    pub active: bool,
    pub tasks_completed: i64,
    pub tasks_failed: i64,
    pub avg_execution_time_secs: f64,
    pub last_seen: Option<DateTime<Utc>>,
    pub uptime_secs: i64,
}

/// Сводные метрики системы
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMetrics {
    pub total_projects: i64,
    pub total_users: i64,
    pub total_tasks: i64,
    pub total_templates: i64,
    pub total_runners: i64,
    pub active_runners: i64,
    pub running_tasks: i64,
    pub queued_tasks: i64,
    pub success_rate_24h: f64,
    pub avg_task_duration_24h: f64,
    pub tasks_24h: i64,
    pub tasks_7d: i64,
    pub tasks_30d: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_stats_default() {
        let stats = ProjectStats::default();
        assert_eq!(stats.project_id, 0);
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.success_rate, 0.0);
    }

    #[test]
    fn test_project_stats_serialization() {
        let stats = ProjectStats {
            project_id: 10,
            project_name: "Test Project".to_string(),
            total_tasks: 100,
            successful_tasks: 80,
            failed_tasks: 15,
            stopped_tasks: 5,
            pending_tasks: 0,
            running_tasks: 0,
            total_templates: 10,
            total_users: 5,
            total_inventories: 3,
            total_repositories: 2,
            total_environments: 4,
            total_keys: 6,
            total_schedules: 2,
            success_rate: 80.0,
            avg_task_duration_secs: 120.5,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"project_name\":\"Test Project\""));
        assert!(json.contains("\"total_tasks\":100"));
        assert!(json.contains("\"success_rate\":80.0"));
    }

    #[test]
    fn test_task_stats_serialization() {
        let stats = TaskStats {
            period: "daily".to_string(),
            total: 50,
            success: 40,
            failed: 8,
            stopped: 2,
            avg_duration_secs: 95.0,
            max_duration_secs: 300.0,
            min_duration_secs: 10.0,
            total_duration_secs: 4750,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"period\":\"daily\""));
        assert!(json.contains("\"total\":50"));
    }

    #[test]
    fn test_resource_usage_default() {
        let usage = ResourceUsage::default();
        assert_eq!(usage.cpu_usage_percent, 0.0);
        assert_eq!(usage.memory_usage_mb, 0.0);
    }

    #[test]
    fn test_system_status_serialization() {
        let status = SystemStatus {
            healthy: true,
            version: "2.5.0".to_string(),
            uptime_secs: 86400,
            active_runners: 3,
            running_tasks: 5,
            queued_tasks: 2,
            database_status: "connected".to_string(),
            last_check: Utc::now(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"version\":\"2.5.0\""));
        assert!(json.contains("\"healthy\":true"));
    }

    #[test]
    fn test_system_metrics_default() {
        let metrics = SystemMetrics::default();
        assert_eq!(metrics.total_projects, 0);
        assert_eq!(metrics.success_rate_24h, 0.0);
    }

    #[test]
    fn test_user_activity_serialization() {
        let activity = UserActivity {
            user_id: 1,
            username: "admin".to_string(),
            total_actions: 50,
            tasks_created: 30,
            tasks_run: 20,
            templates_created: 5,
            last_activity: Utc::now(),
        };
        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("\"username\":\"admin\""));
        assert!(json.contains("\"total_actions\":50"));
    }

    #[test]
    fn test_performance_metrics_serialization() {
        let perf = PerformanceMetrics {
            avg_queue_time_secs: 5.0,
            avg_execution_time_secs: 120.0,
            tasks_per_hour: 10.0,
            tasks_per_day: 240.0,
            concurrent_tasks_avg: 2.5,
            concurrent_tasks_max: 5,
            resource_usage: ResourceUsage::default(),
        };
        let json = serde_json::to_string(&perf).unwrap();
        assert!(json.contains("\"tasks_per_hour\":10.0"));
        assert!(json.contains("\"tasks_per_day\":240.0"));
    }

    #[test]
    fn test_chart_data_serialization() {
        let data = ChartData {
            label: "CPU".to_string(),
            value: 75.5,
            timestamp: Some(Utc::now()),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"label\":\"CPU\""));
        assert!(json.contains("\"value\":75.5"));
    }

    #[test]
    fn test_chart_data_null_timestamp() {
        let data = ChartData { label: "Mem".to_string(), value: 60.0, timestamp: None };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"timestamp\":null"));
    }

    #[test]
    fn test_time_series_serialization() {
        let series = TimeSeries {
            metric: "cpu".to_string(),
            data: vec![ChartData { label: "point1".to_string(), value: 1.0, timestamp: None }],
        };
        let json = serde_json::to_string(&series).unwrap();
        assert!(json.contains("\"metric\":\"cpu\""));
        assert!(json.contains("\"data\":["));
    }

    #[test]
    fn test_top_item_serialization() {
        let item = TopItem { id: 1, name: "Top Template".to_string(), value: 100, r#type: "template".to_string() };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"name\":\"Top Template\""));
        assert!(json.contains("\"value\":100"));
    }

    #[test]
    fn test_top_slow_task_serialization() {
        let task = TopSlowTask {
            task_id: 1, task_name: "Slow Task".to_string(),
            template_name: "Slow Template".to_string(), duration_secs: 300.0,
            status: "success".to_string(), created: Utc::now(),
        };
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("\"task_name\":\"Slow Task\""));
        assert!(json.contains("\"duration_secs\":300.0"));
    }

    #[test]
    fn test_top_user_serialization() {
        let user = TopUser { user_id: 1, username: "top_user".to_string(), tasks_count: 500, success_rate: 95.0 };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"username\":\"top_user\""));
        assert!(json.contains("\"tasks_count\":500"));
    }

    #[test]
    fn test_analytics_query_params_serialization() {
        let params = AnalyticsQueryParams {
            project_id: Some(10), date_from: Some(Utc::now()), date_to: None,
            period: Some("daily".to_string()), limit: Some(100), group_by: Some("template".to_string()),
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"project_id\":10"));
        assert!(json.contains("\"limit\":100"));
    }

    #[test]
    fn test_runner_metrics_serialization() {
        let metrics = RunnerMetrics {
            runner_id: 1, runner_name: "Runner 1".to_string(), active: true,
            tasks_completed: 100, tasks_failed: 5, avg_execution_time_secs: 60.0,
            last_seen: Some(Utc::now()), uptime_secs: 86400,
        };
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("\"runner_name\":\"Runner 1\""));
        assert!(json.contains("\"tasks_completed\":100"));
    }

    #[test]
    fn test_project_analytics_serialization() {
        let analytics = ProjectAnalytics {
            stats: ProjectStats::default(),
            task_stats: TaskStats {
                period: "daily".to_string(), total: 10, success: 8, failed: 2, stopped: 0,
                avg_duration_secs: 60.0, max_duration_secs: 120.0, min_duration_secs: 10.0,
                total_duration_secs: 600,
            },
            performance: PerformanceMetrics {
                avg_queue_time_secs: 5.0, avg_execution_time_secs: 60.0,
                tasks_per_hour: 10.0, tasks_per_day: 240.0, concurrent_tasks_avg: 2.0,
                concurrent_tasks_max: 5, resource_usage: ResourceUsage::default(),
            },
            top_users: vec![], top_templates: vec![], recent_activity: vec![],
        };
        let json = serde_json::to_string(&analytics).unwrap();
        assert!(json.contains("\"task_stats\":"));
        assert!(json.contains("\"performance\":"));
    }
}
