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
use prometheus::{register_counter, register_gauge, register_histogram, Counter, Gauge, Histogram, Registry, Encoder, TextEncoder};
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
    
    /// Отмечает завершение задачи успешно
    pub fn task_completed(&self, duration_secs: f64, queue_time_secs: f64) {
        TASK_SUCCESS.inc();
        TASKS_RUNNING.dec();
        TASK_DURATION.observe(duration_secs);
        TASK_QUEUE_TIME.observe(queue_time_secs);
    }
    
    /// Отмечает провал задачи
    pub fn task_failed(&self, duration_secs: f64) {
        TASK_FAILED.inc();
        TASKS_RUNNING.dec();
        TASK_DURATION.observe(duration_secs);
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
    pub fn update_cpu_usage(&self, percent: f64) {
        CPU_USAGE.set(percent);
    }
    
    /// Обновляет использование памяти
    pub fn update_memory_usage(&self, mb: f64) {
        MEMORY_USAGE.set(mb);
    }
    
    /// Обновляет статус здоровья
    pub fn update_health(&self, healthy: bool) {
        SYSTEM_HEALTHY.set(if healthy { 1.0 } else { 0.0 });
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
    
    /// Получает счётчики задач
    pub async fn get_task_counters(&self) -> TaskCounters {
        self.task_counters.read().await.clone()
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
}
