//! Prometheus Metrics - сервис экспорта метрик
//!
//! Предоставляет метрики для мониторинга Velum UI:
//! - Количество задач (всего, успешных, проваленных)
//! - Длительность задач
//! - Активные раннеры
//! - Пользователи
//! - Проекты
//! - Использование ресурсов

use lazy_static::lazy_static;
use prometheus::{
    register_counter, register_gauge, register_histogram, Counter, Encoder, Gauge, Histogram,
    Registry, TextEncoder,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Глобальный реестр метрик
lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    /// Счётчик всех задач
    pub static ref TASK_TOTAL: Counter = register_counter!(
        "semaphore_tasks_total",
        "Общее количество задач"
    ).unwrap();

    /// Счётчик успешных задач
    pub static ref TASK_SUCCESS: Counter = register_counter!(
        "semaphore_tasks_success_total",
        "Количество успешных задач"
    ).unwrap();

    /// Счётчик проваленных задач
    pub static ref TASK_FAILED: Counter = register_counter!(
        "semaphore_tasks_failed_total",
        "Количество проваленных задач"
    ).unwrap();

    /// Счётчик остановленных задач
    pub static ref TASK_STOPPED: Counter = register_counter!(
        "semaphore_tasks_stopped_total",
        "Количество остановленных задач"
    ).unwrap();

    /// Гистограмма длительности задач
    pub static ref TASK_DURATION: Histogram = register_histogram!(
        "semaphore_task_duration_seconds",
        "Длительность выполнения задач в секундах",
        vec![0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0]
    ).unwrap();

    /// Гистограмма времени в очереди
    pub static ref TASK_QUEUE_TIME: Histogram = register_histogram!(
        "semaphore_task_queue_time_seconds",
        "Время ожидания задачи в очереди в секундах",
        vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0]
    ).unwrap();

    /// Количество запущенных задач
    pub static ref TASKS_RUNNING: Gauge = register_gauge!(
        "semaphore_tasks_running",
        "Количество запущенных задач"
    ).unwrap();

    /// Количество задач в очереди
    pub static ref TASKS_QUEUED: Gauge = register_gauge!(
        "semaphore_tasks_queued",
        "Количество задач в очереди"
    ).unwrap();

    /// Количество активных раннеров
    pub static ref RUNNERS_ACTIVE: Gauge = register_gauge!(
        "semaphore_runners_active",
        "Количество активных раннеров"
    ).unwrap();

    /// Количество проектов
    pub static ref PROJECTS_TOTAL: Gauge = register_gauge!(
        "semaphore_projects_total",
        "Общее количество проектов"
    ).unwrap();

    /// Количество пользователей
    pub static ref USERS_TOTAL: Gauge = register_gauge!(
        "semaphore_users_total",
        "Общее количество пользователей"
    ).unwrap();

    /// Количество шаблонов
    pub static ref TEMPLATES_TOTAL: Gauge = register_gauge!(
        "semaphore_templates_total",
        "Общее количество шаблонов"
    ).unwrap();

    /// Количество инвентарей
    pub static ref INVENTORIES_TOTAL: Gauge = register_gauge!(
        "semaphore_inventories_total",
        "Общее количество инвентарей"
    ).unwrap();

    /// Количество репозиториев
    pub static ref REPOSITORIES_TOTAL: Gauge = register_gauge!(
        "semaphore_repositories_total",
        "Общее количество репозиториев"
    ).unwrap();

    /// Использование CPU (проценты)
    pub static ref CPU_USAGE: Gauge = register_gauge!(
        "semaphore_system_cpu_usage_percent",
        "Использование CPU в процентах"
    ).unwrap();

    /// Использование памяти (MB)
    pub static ref MEMORY_USAGE: Gauge = register_gauge!(
        "semaphore_system_memory_usage_mb",
        "Использование памяти в MB"
    ).unwrap();

    /// Время работы (секунды)
    pub static ref UPTIME: Gauge = register_gauge!(
        "semaphore_system_uptime_seconds",
        "Время работы системы в секундах"
    ).unwrap();

    /// Статус системы (1 = здоров, 0 = нездоров)
    pub static ref SYSTEM_HEALTHY: Gauge = register_gauge!(
        "semaphore_system_healthy",
        "Статус здоровья системы"
    ).unwrap();

    // ========================================================================
    // Multi-Tenancy Metrics (v4.0)
    // ========================================================================

    /// Количество организаций
    pub static ref ORGANIZATIONS_TOTAL: Gauge = register_gauge!(
        "semaphore_organizations_total",
        "Общее количество организаций"
    ).unwrap();

    /// Количество пользователей в организациях
    pub static ref ORGANIZATION_USERS_TOTAL: Gauge = register_gauge!(
        "semaphore_organization_users_total",
        "Общее количество пользователей в организациях"
    ).unwrap();

    /// Количество проектов в организациях
    pub static ref ORGANIZATION_PROJECTS_TOTAL: Gauge = register_gauge!(
        "semaphore_organization_projects_total",
        "Общее количество проектов в организациях"
    ).unwrap();

    // ========================================================================
    // Audit Log Metrics (v4.0)
    // ========================================================================

    /// Счётчик audit записей
    pub static ref AUDIT_LOG_TOTAL: Counter = register_counter!(
        "semaphore_audit_log_events_total",
        "Общее количество audit записей"
    ).unwrap();

    /// Audit записей по уровням
    pub static ref AUDIT_LOG_BY_LEVEL: prometheus::CounterVec = prometheus::register_counter_vec!(
        "semaphore_audit_log_events_by_level",
        "Количество audit записей по уровням",
        &["level"]
    ).unwrap();

    /// Audit записей по действиям
    pub static ref AUDIT_LOG_BY_ACTION: prometheus::CounterVec = prometheus::register_counter_vec!(
        "semaphore_audit_log_events_by_action",
        "Количество audit записей по действиям",
        &["action"]
    ).unwrap();

    // ========================================================================
    // HTTP Request Metrics
    // ========================================================================

    /// Счётчик HTTP запросов
    pub static ref HTTP_REQUESTS_TOTAL: prometheus::CounterVec = prometheus::register_counter_vec!(
        "semaphore_http_requests_total",
        "Общее количество HTTP запросов",
        &["method", "endpoint", "status"]
    ).unwrap();

    /// Гистограмма длительности HTTP запросов
    pub static ref HTTP_REQUEST_DURATION: Histogram = register_histogram!(
        "semaphore_http_request_duration_seconds",
        "Длительность HTTP запросов в секундах",
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    ).unwrap();

    /// Количество активных сессий
    pub static ref ACTIVE_SESSIONS: Gauge = register_gauge!(
        "semaphore_active_sessions",
        "Количество активных сессий"
    ).unwrap();

    // ========================================================================
    // Task Metrics with Labels (TD-06)
    // ========================================================================

    /// Счётчик задач по проектам
    pub static ref TASK_TOTAL_BY_PROJECT: prometheus::CounterVec = prometheus::register_counter_vec!(
        "semaphore_tasks_total_by_project",
        "Общее количество задач по проектам",
        &["project_id", "project_name"]
    ).unwrap();

    /// Счётчик успешных задач по проектам
    pub static ref TASK_SUCCESS_BY_PROJECT: prometheus::CounterVec = prometheus::register_counter_vec!(
        "semaphore_tasks_success_total_by_project",
        "Количество успешных задач по проектам",
        &["project_id", "project_name"]
    ).unwrap();

    /// Счётчик проваленных задач по проектам
    pub static ref TASK_FAILED_BY_PROJECT: prometheus::CounterVec = prometheus::register_counter_vec!(
        "semaphore_tasks_failed_total_by_project",
        "Количество проваленных задач по проектам",
        &["project_id", "project_name"]
    ).unwrap();

    /// Счётчик задач по шаблонам
    pub static ref TASK_TOTAL_BY_TEMPLATE: prometheus::CounterVec = prometheus::register_counter_vec!(
        "semaphore_tasks_total_by_template",
        "Общее количество задач по шаблонам",
        &["template_id", "template_name"]
    ).unwrap();

    /// Счётчик успешных задач по шаблонам
    pub static ref TASK_SUCCESS_BY_TEMPLATE: prometheus::CounterVec = prometheus::register_counter_vec!(
        "semaphore_tasks_success_total_by_template",
        "Количество успешных задач по шаблонам",
        &["template_id", "template_name"]
    ).unwrap();

    /// Счётчик задач по пользователям
    pub static ref TASK_TOTAL_BY_USER: prometheus::CounterVec = prometheus::register_counter_vec!(
        "semaphore_tasks_total_by_user",
        "Общее количество задач по пользователям",
        &["user_id", "username"]
    ).unwrap();

    /// Гистограмма длительности задач по типам приложений
    pub static ref TASK_DURATION_BY_APP: prometheus::HistogramVec = prometheus::register_histogram_vec!(
        "semaphore_task_duration_seconds_by_app",
        "Длительность выполнения задач по типам приложений",
        &["app_type"],
        vec![0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0]
    ).unwrap();
}

/// Менеджер метрик
#[derive(Clone)]
pub struct MetricsManager {
    start_time: std::time::Instant,
    task_counters: Arc<RwLock<TaskCounters>>,
}

/// Счётчики задач по проектам
#[derive(Debug, Clone, Default)]
pub struct TaskCounters {
    pub by_project: HashMap<i64, ProjectTaskCounters>,
    pub by_template: HashMap<i64, TemplateTaskCounters>,
    pub by_user: HashMap<i64, UserTaskCounters>,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectTaskCounters {
    pub total: u64,
    pub success: u64,
    pub failed: u64,
    pub stopped: u64,
}

#[derive(Debug, Clone, Default)]
pub struct TemplateTaskCounters {
    pub total: u64,
    pub success: u64,
    pub failed: u64,
}

#[derive(Debug, Clone, Default)]
pub struct UserTaskCounters {
    pub total: u64,
    pub success: u64,
    pub failed: u64,
}

/// Labels для агрегации метрик задач (TD-06)
#[derive(Debug, Clone)]
pub struct TaskMetricLabels {
    pub project_id: String,
    pub project_name: String,
    pub template_id: String,
    pub template_name: String,
    pub user_id: String,
    pub username: String,
    pub app_type: String,
}

impl TaskMetricLabels {
    pub fn new(
        project_id: impl Into<String>,
        project_name: impl Into<String>,
        template_id: impl Into<String>,
        template_name: impl Into<String>,
        user_id: impl Into<String>,
        username: impl Into<String>,
        app_type: impl Into<String>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            project_name: project_name.into(),
            template_id: template_id.into(),
            template_name: template_name.into(),
            user_id: user_id.into(),
            username: username.into(),
            app_type: app_type.into(),
        }
    }
}

impl MetricsManager {
    /// Создаёт новый MetricsManager
    pub fn new() -> Self {
        // Force lazy_static initialization so all metrics are registered in the global registry
        lazy_static::initialize(&TASK_TOTAL);
        lazy_static::initialize(&TASK_SUCCESS);
        lazy_static::initialize(&TASK_FAILED);
        lazy_static::initialize(&TASK_STOPPED);
        lazy_static::initialize(&TASK_DURATION);
        lazy_static::initialize(&TASK_QUEUE_TIME);
        lazy_static::initialize(&TASKS_RUNNING);
        lazy_static::initialize(&TASKS_QUEUED);
        lazy_static::initialize(&RUNNERS_ACTIVE);
        lazy_static::initialize(&PROJECTS_TOTAL);
        lazy_static::initialize(&USERS_TOTAL);
        lazy_static::initialize(&TEMPLATES_TOTAL);
        lazy_static::initialize(&INVENTORIES_TOTAL);
        lazy_static::initialize(&REPOSITORIES_TOTAL);
        lazy_static::initialize(&CPU_USAGE);
        lazy_static::initialize(&MEMORY_USAGE);
        lazy_static::initialize(&UPTIME);
        lazy_static::initialize(&SYSTEM_HEALTHY);
        // Multi-Tenancy metrics
        lazy_static::initialize(&ORGANIZATIONS_TOTAL);
        lazy_static::initialize(&ORGANIZATION_USERS_TOTAL);
        lazy_static::initialize(&ORGANIZATION_PROJECTS_TOTAL);
        // Audit Log metrics
        lazy_static::initialize(&AUDIT_LOG_TOTAL);
        lazy_static::initialize(&AUDIT_LOG_BY_LEVEL);
        lazy_static::initialize(&AUDIT_LOG_BY_ACTION);
        // HTTP metrics
        lazy_static::initialize(&HTTP_REQUESTS_TOTAL);
        lazy_static::initialize(&HTTP_REQUEST_DURATION);
        lazy_static::initialize(&ACTIVE_SESSIONS);
        Self {
            start_time: std::time::Instant::now(),
            task_counters: Arc::new(RwLock::new(TaskCounters::default())),
        }
    }

    /// Получает глобальный реестр
    pub fn registry() -> &'static Registry {
        &REGISTRY
    }

    /// Форматирует метрики в Prometheus формат
    pub fn encode_metrics(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer).unwrap())
    }

    /// Обновляет время работы
    pub fn update_uptime(&self) {
        let uptime = self.start_time.elapsed().as_secs_f64();
        UPTIME.set(uptime);
    }

    /// Отмечает начало выполнения задачи
    pub fn task_started(&self) {
        TASK_TOTAL.inc();
        TASKS_RUNNING.inc();
    }

    /// Отмечает начало выполнения задачи с labels
    pub fn task_started_with_labels(&self, labels: &TaskMetricLabels) {
        TASK_TOTAL.inc();
        TASKS_RUNNING.inc();
        TASK_TOTAL_BY_PROJECT
            .with_label_values(&[&labels.project_id, &labels.project_name])
            .inc();
        TASK_TOTAL_BY_TEMPLATE
            .with_label_values(&[&labels.template_id, &labels.template_name])
            .inc();
        TASK_TOTAL_BY_USER
            .with_label_values(&[&labels.user_id, &labels.username])
            .inc();
    }

    /// Отмечает завершение задачи успешно
    pub fn task_completed(&self, duration_secs: f64, queue_time_secs: f64) {
        TASK_SUCCESS.inc();
        TASKS_RUNNING.dec();
        TASK_DURATION.observe(duration_secs);
        TASK_QUEUE_TIME.observe(queue_time_secs);
    }

    /// Отмечает завершение задачи успешно с labels
    pub fn task_completed_with_labels(&self, duration_secs: f64, queue_time_secs: f64, labels: &TaskMetricLabels) {
        TASK_SUCCESS.inc();
        TASKS_RUNNING.dec();
        TASK_DURATION.observe(duration_secs);
        TASK_QUEUE_TIME.observe(queue_time_secs);
        TASK_SUCCESS_BY_PROJECT
            .with_label_values(&[&labels.project_id, &labels.project_name])
            .inc();
        TASK_SUCCESS_BY_TEMPLATE
            .with_label_values(&[&labels.template_id, &labels.template_name])
            .inc();
        TASK_DURATION_BY_APP
            .with_label_values(&[&labels.app_type])
            .observe(duration_secs);
    }

    /// Отмечает провал задачи
    pub fn task_failed(&self, duration_secs: f64) {
        TASK_FAILED.inc();
        TASKS_RUNNING.dec();
        TASK_DURATION.observe(duration_secs);
    }

    /// Отмечает провал задачи с labels
    pub fn task_failed_with_labels(&self, duration_secs: f64, labels: &TaskMetricLabels) {
        TASK_FAILED.inc();
        TASKS_RUNNING.dec();
        TASK_DURATION.observe(duration_secs);
        TASK_FAILED_BY_PROJECT
            .with_label_values(&[&labels.project_id, &labels.project_name])
            .inc();
    }

    /// Отмечает остановку задачи
    pub fn task_stopped(&self) {
        TASK_STOPPED.inc();
        TASKS_RUNNING.dec();
    }

    /// Обновляет количество задач в очереди
    pub fn update_queued_tasks(&self, count: i64) {
        TASKS_QUEUED.set(count as f64);
    }

    /// Обновляет количество активных раннеров
    pub fn update_active_runners(&self, count: i64) {
        RUNNERS_ACTIVE.set(count as f64);
    }

    /// Обновляет количество проектов
    pub fn update_projects(&self, count: i64) {
        PROJECTS_TOTAL.set(count as f64);
    }

    /// Обновляет количество пользователей
    pub fn update_users(&self, count: i64) {
        USERS_TOTAL.set(count as f64);
    }

    /// Обновляет количество шаблонов
    pub fn update_templates(&self, count: i64) {
        TEMPLATES_TOTAL.set(count as f64);
    }

    /// Обновляет количество инвентарей
    pub fn update_inventories(&self, count: i64) {
        INVENTORIES_TOTAL.set(count as f64);
    }

    /// Обновляет количество репозиториев
    pub fn update_repositories(&self, count: i64) {
        REPOSITORIES_TOTAL.set(count as f64);
    }

    /// Обновляет использование CPU
    pub fn update_cpu_usage(&self, usage: f64) {
        CPU_USAGE.set(usage);
    }

    /// Обновляет использование памяти
    pub fn update_memory_usage(&self, usage_mb: f64) {
        MEMORY_USAGE.set(usage_mb);
    }

    /// Обновляет статус здоровья
    pub fn update_health(&self, healthy: bool) {
        SYSTEM_HEALTHY.set(if healthy { 1.0 } else { 0.0 });
    }

    // ========================================================================
    // Multi-Tenancy Metrics
    // ========================================================================

    /// Обновляет количество организаций
    pub fn update_organizations(&self, count: i64) {
        ORGANIZATIONS_TOTAL.set(count as f64);
    }

    /// Обновляет количество пользователей в организациях
    pub fn update_organization_users(&self, count: i64) {
        ORGANIZATION_USERS_TOTAL.set(count as f64);
    }

    /// Обновляет количество проектов в организациях
    pub fn update_organization_projects(&self, count: i64) {
        ORGANIZATION_PROJECTS_TOTAL.set(count as f64);
    }

    // ========================================================================
    // Audit Log Metrics
    // ========================================================================

    /// Отмечает audit событие
    pub fn audit_event(&self, level: &str, action: &str) {
        AUDIT_LOG_TOTAL.inc();
        AUDIT_LOG_BY_LEVEL.with_label_values(&[level]).inc();
        AUDIT_LOG_BY_ACTION.with_label_values(&[action]).inc();
    }

    // ========================================================================
    // HTTP Request Metrics
    // ========================================================================

    /// Отмечает HTTP запрос
    pub fn http_request(&self, method: &str, endpoint: &str, status: u16, duration_secs: f64) {
        HTTP_REQUESTS_TOTAL
            .with_label_values(&[method, endpoint, &status.to_string()])
            .inc();
        HTTP_REQUEST_DURATION.observe(duration_secs);
    }

    /// Обновляет количество активных сессий
    pub fn update_active_sessions(&self, count: i64) {
        ACTIVE_SESSIONS.set(count as f64);
    }

    /// Получает счётчики задач
    pub async fn get_task_counters(&self) -> TaskCounters {
        self.task_counters.read().await.clone()
    }

    /// Инкремент счётчика задач проекта
    pub async fn inc_project_task(&self, project_id: i64, success: bool) {
        let mut counters = self.task_counters.write().await;
        let project_counters = counters.by_project.entry(project_id).or_default();
        project_counters.total += 1;
        if success {
            project_counters.success += 1;
        } else {
            project_counters.failed += 1;
        }
    }

    /// Инкремент счётчика задач шаблона
    pub async fn inc_template_task(&self, template_id: i64, success: bool) {
        let mut counters = self.task_counters.write().await;
        let template_counters = counters.by_template.entry(template_id).or_default();
        template_counters.total += 1;
        if success {
            template_counters.success += 1;
        } else {
            template_counters.failed += 1;
        }
    }

    /// Инкремент счётчика задач пользователя
    pub async fn inc_user_task(&self, user_id: i64, success: bool) {
        let mut counters = self.task_counters.write().await;
        let user_counters = counters.by_user.entry(user_id).or_default();
        user_counters.total += 1;
        if success {
            user_counters.success += 1;
        } else {
            user_counters.failed += 1;
        }
    }
}

impl Default for MetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// API handler для экспорта метрик
pub async fn metrics_handler() -> Result<String, prometheus::Error> {
    let manager = MetricsManager::new();
    manager.encode_metrics()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_counter_inc() {
        // Проверяем что счётчик задач увеличивается
        let initial = TASK_TOTAL.get();
        TASK_TOTAL.inc();
        assert!(TASK_TOTAL.get() > initial);
    }

    #[test]
    fn test_task_success_counter() {
        // Проверяем что счётчик успешных задач работает
        let initial = TASK_SUCCESS.get();
        TASK_SUCCESS.inc();
        assert!(TASK_SUCCESS.get() > initial);
    }

    #[test]
    fn test_task_failed_counter() {
        // Проверяем что счётчик неудачных задач работает
        let initial = TASK_FAILED.get();
        TASK_FAILED.inc();
        assert!(TASK_FAILED.get() > initial);
    }

    #[test]
    fn test_gauge_set() {
        // Проверяем что gauge метрики работают
        let initial = RUNNERS_ACTIVE.get();
        RUNNERS_ACTIVE.set(initial + 1.0);
        assert_eq!(RUNNERS_ACTIVE.get(), initial + 1.0);
    }

    #[test]
    fn test_histogram_observe() {
        // Проверяем что гистограммы работают
        // Просто проверяем что observe не паникует
        TASK_DURATION.observe(1.0);
        TASK_QUEUE_TIME.observe(0.5);
    }

    #[test]
    fn test_task_duration_observe_positive() {
        // Гистограмма должна принимать положительные значения
        TASK_DURATION.observe(10.0);
        TASK_DURATION.observe(100.0);
    }

    #[test]
    fn test_tasks_running_gauge() {
        // Проверяем gauge для количества запущенных задач
        let initial = TASKS_RUNNING.get();
        TASKS_RUNNING.set(initial + 1.0);
        assert_eq!(TASKS_RUNNING.get(), initial + 1.0);
    }

    #[test]
    fn test_tasks_queued_gauge() {
        // Проверяем gauge для количества задач в очереди
        let initial = TASKS_QUEUED.get();
        TASKS_QUEUED.set(initial + 5.0);
        assert_eq!(TASKS_QUEUED.get(), initial + 5.0);
    }

    #[test]
    fn test_task_stopped_counter() {
        // Проверяем счётчик остановленных задач
        let initial = TASK_STOPPED.get();
        TASK_STOPPED.inc();
        assert!(TASK_STOPPED.get() > initial);
    }

    #[test]
    fn test_metrics_manager_new_and_encode() {
        let manager = MetricsManager::new();
        let output = manager.encode_metrics();
        assert!(output.is_ok());
        assert!(output.unwrap().contains("semaphore_"));
    }

    #[test]
    fn test_update_uptime() {
        let manager = MetricsManager::new();
        manager.update_uptime();
        assert!(UPTIME.get() >= 0.0);
    }

    #[test]
    fn test_update_queued_tasks() {
        let manager = MetricsManager::new();
        manager.update_queued_tasks(42);
        assert_eq!(TASKS_QUEUED.get(), 42.0);
    }

    #[test]
    fn test_update_active_runners() {
        let manager = MetricsManager::new();
        manager.update_active_runners(7);
        assert_eq!(RUNNERS_ACTIVE.get(), 7.0);
    }

    #[test]
    fn test_update_projects() {
        let manager = MetricsManager::new();
        manager.update_projects(10);
        assert_eq!(PROJECTS_TOTAL.get(), 10.0);
    }

    #[test]
    fn test_update_users() {
        let manager = MetricsManager::new();
        manager.update_users(5);
        assert_eq!(USERS_TOTAL.get(), 5.0);
    }

    #[test]
    fn test_update_templates() {
        let manager = MetricsManager::new();
        manager.update_templates(3);
        assert_eq!(TEMPLATES_TOTAL.get(), 3.0);
    }

    #[test]
    fn test_update_inventories() {
        let manager = MetricsManager::new();
        manager.update_inventories(8);
        assert_eq!(INVENTORIES_TOTAL.get(), 8.0);
    }

    #[test]
    fn test_update_repositories() {
        let manager = MetricsManager::new();
        manager.update_repositories(2);
        assert_eq!(REPOSITORIES_TOTAL.get(), 2.0);
    }

    #[test]
    fn test_update_cpu_usage() {
        let manager = MetricsManager::new();
        manager.update_cpu_usage(45.5);
        assert_eq!(CPU_USAGE.get(), 45.5);
    }

    #[test]
    fn test_update_memory_usage() {
        let manager = MetricsManager::new();
        manager.update_memory_usage(512.0);
        assert_eq!(MEMORY_USAGE.get(), 512.0);
    }

    #[test]
    fn test_update_health_true() {
        let manager = MetricsManager::new();
        manager.update_health(true);
        assert_eq!(SYSTEM_HEALTHY.get(), 1.0);
    }

    #[test]
    fn test_update_health_false() {
        let manager = MetricsManager::new();
        manager.update_health(false);
        assert_eq!(SYSTEM_HEALTHY.get(), 0.0);
    }

    #[test]
    fn test_update_organizations() {
        let manager = MetricsManager::new();
        manager.update_organizations(3);
        assert_eq!(ORGANIZATIONS_TOTAL.get(), 3.0);
    }

    #[test]
    fn test_update_organization_users() {
        let manager = MetricsManager::new();
        manager.update_organization_users(15);
        assert_eq!(ORGANIZATION_USERS_TOTAL.get(), 15.0);
    }

    #[test]
    fn test_update_organization_projects() {
        let manager = MetricsManager::new();
        manager.update_organization_projects(6);
        assert_eq!(ORGANIZATION_PROJECTS_TOTAL.get(), 6.0);
    }

    #[test]
    fn test_audit_event() {
        let manager = MetricsManager::new();
        let before = AUDIT_LOG_TOTAL.get();
        manager.audit_event("INFO", "task_created");
        assert_eq!(AUDIT_LOG_TOTAL.get(), before + 1.0);
    }

    #[test]
    fn test_update_active_sessions() {
        let manager = MetricsManager::new();
        manager.update_active_sessions(25);
        assert_eq!(ACTIVE_SESSIONS.get(), 25.0);
    }

    #[test]
    fn test_task_metric_labels_new() {
        let labels = TaskMetricLabels::new(
            "1", "proj", "10", "tpl", "5", "user", "ansible",
        );
        assert_eq!(labels.project_id, "1");
        assert_eq!(labels.project_name, "proj");
        assert_eq!(labels.template_id, "10");
        assert_eq!(labels.app_type, "ansible");
    }

    #[test]
    fn test_http_request_metric() {
        let manager = MetricsManager::new();
        let before = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "/api/tasks", "200"])
            .get();
        manager.http_request("GET", "/api/tasks", 200, 0.05);
        let after = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "/api/tasks", "200"])
            .get();
        assert_eq!(after, before + 1.0);
    }

    #[tokio::test]
    async fn test_get_task_counters_empty() {
        let manager = MetricsManager::new();
        let counters = manager.get_task_counters().await;
        assert!(counters.by_project.is_empty());
        assert!(counters.by_template.is_empty());
        assert!(counters.by_user.is_empty());
    }

    #[tokio::test]
    async fn test_inc_project_task() {
        let manager = MetricsManager::new();
        manager.inc_project_task(100, true).await;
        manager.inc_project_task(100, false).await;
        let counters = manager.get_task_counters().await;
        let pc = counters.by_project.get(&100).unwrap();
        assert_eq!(pc.total, 2);
        assert_eq!(pc.success, 1);
        assert_eq!(pc.failed, 1);
    }

    #[tokio::test]
    async fn test_inc_template_task() {
        let manager = MetricsManager::new();
        manager.inc_template_task(42, true).await;
        manager.inc_template_task(42, true).await;
        let counters = manager.get_task_counters().await;
        let tc = counters.by_template.get(&42).unwrap();
        assert_eq!(tc.total, 2);
        assert_eq!(tc.success, 2);
    }

    #[tokio::test]
    async fn test_inc_user_task() {
        let manager = MetricsManager::new();
        manager.inc_user_task(7, false).await;
        let counters = manager.get_task_counters().await;
        let uc = counters.by_user.get(&7).unwrap();
        assert_eq!(uc.total, 1);
        assert_eq!(uc.failed, 1);
    }

    #[test]
    fn test_project_task_counters_default() {
        let pc = ProjectTaskCounters::default();
        assert_eq!(pc.total, 0);
        assert_eq!(pc.success, 0);
        assert_eq!(pc.failed, 0);
        assert_eq!(pc.stopped, 0);
    }

    #[test]
    fn test_template_task_counters_default() {
        let tc = TemplateTaskCounters::default();
        assert_eq!(tc.total, 0);
        assert_eq!(tc.success, 0);
        assert_eq!(tc.failed, 0);
    }

    #[test]
    fn test_user_task_counters_default() {
        let uc = UserTaskCounters::default();
        assert_eq!(uc.total, 0);
        assert_eq!(uc.success, 0);
        assert_eq!(uc.failed, 0);
    }

    #[tokio::test]
    async fn test_metrics_handler() {
        let result = metrics_handler().await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("semaphore_") || output.contains("HELP") || output.contains("TYPE"));
    }
}
