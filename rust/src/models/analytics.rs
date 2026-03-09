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
