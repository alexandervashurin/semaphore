//! Remote Runners Module
//!
//! Модуль для поддержки удалённых раннеров в Velum.
//! Предоставляет инфраструктуру для:
//! - Регистрации раннеров
//! - Heartbeat механизма
//! - Распределения задач
//! - Мониторинга здоровья раннеров

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::models::Runner;

/// Конфигурация Remote Runners
#[derive(Debug, Clone)]
pub struct RemoteRunnersConfig {
    /// Включить remote runners
    pub enabled: bool,
    /// Интервал heartbeat (секунды)
    pub heartbeat_interval_secs: u64,
    /// Таймаут раннера без heartbeat (секунды)
    pub runner_timeout_secs: u64,
    /// Максимальное количество раннеров на проект
    pub max_runners_per_project: usize,
}

impl Default for RemoteRunnersConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            heartbeat_interval_secs: 30,
            runner_timeout_secs: 90,
            max_runners_per_project: 10,
        }
    }
}

/// Информация о зарегистрированном раннере
#[derive(Debug, Clone)]
pub struct RegisteredRunner {
    pub runner: Runner,
    pub last_heartbeat: DateTime<Utc>,
    pub current_tasks: Vec<i32>,
    pub capabilities: RunnerCapabilities,
}

/// Возможности раннера
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunnerCapabilities {
    pub ansible: bool,
    pub terraform: bool,
    pub bash: bool,
    pub powershell: bool,
    pub kubernetes: bool,
    pub max_parallel_tasks: i32,
    pub tags: Vec<String>,
}

/// Менеджер Remote Runners
pub struct RemoteRunnersManager {
    config: RemoteRunnersConfig,
    /// Зарегистрированные раннеры по token
    runners: Arc<RwLock<HashMap<String, RegisteredRunner>>>,
    /// Очередь задач для раннеров
    task_queue: Arc<RwLock<Vec<i32>>>,
}

impl RemoteRunnersManager {
    /// Создаёт новый менеджер
    pub fn new(config: RemoteRunnersConfig) -> Self {
        Self {
            config,
            runners: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Регистрирует новый раннер
    pub async fn register_runner(&self, runner: Runner, capabilities: RunnerCapabilities) -> Result<String, String> {
        if !self.config.enabled {
            return Err("Remote runners disabled".to_string());
        }

        let token = runner.token.clone();
        let registered = RegisteredRunner {
            runner,
            last_heartbeat: Utc::now(),
            current_tasks: Vec::new(),
            capabilities,
        };

        let mut runners = self.runners.write().await;
        runners.insert(token.clone(), registered);
        
        info!("Runner registered: {}", token);
        Ok(token)
    }

    /// Обновляет heartbeat раннера
    pub async fn heartbeat(&self, token: &str) -> Result<(), String> {
        let mut runners = self.runners.write().await;
        
        match runners.get_mut(token) {
            Some(runner) => {
                runner.last_heartbeat = Utc::now();
                debug!("Heartbeat received from runner: {}", token);
                Ok(())
            }
            None => Err(format!("Runner not found: {}", token)),
        }
    }

    /// Отменяет регистрацию раннера
    pub async fn unregister_runner(&self, token: &str) -> Result<(), String> {
        let mut runners = self.runners.write().await;
        
        match runners.remove(token) {
            Some(_) => {
                info!("Runner unregistered: {}", token);
                Ok(())
            }
            None => Err(format!("Runner not found: {}", token)),
        }
    }

    /// Получает список активных раннеров
    pub async fn get_active_runners(&self) -> Vec<RegisteredRunner> {
        let runners = self.runners.read().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(self.config.runner_timeout_secs as i64);
        
        runners
            .values()
            .filter(|r| r.last_heartbeat > cutoff)
            .cloned()
            .collect()
    }

    /// Добавляет задачу в очередь
    pub async fn queue_task(&self, task_id: i32) {
        let mut queue = self.task_queue.write().await;
        queue.push(task_id);
        debug!("Task {} queued", task_id);
    }

    /// Получает следующую задачу для раннера
    pub async fn dequeue_task(&self) -> Option<i32> {
        let mut queue = self.task_queue.write().await;
        if queue.is_empty() {
            None
        } else {
            Some(queue.remove(0))
        }
    }

    /// Назначает задачу раннеру
    pub async fn assign_task(&self, token: &str, task_id: i32) -> Result<(), String> {
        let mut runners = self.runners.write().await;
        
        match runners.get_mut(token) {
            Some(runner) => {
                runner.current_tasks.push(task_id);
                info!("Task {} assigned to runner {}", task_id, token);
                Ok(())
            }
            None => Err(format!("Runner not found: {}", token)),
        }
    }

    /// Завершает задачу на раннере
    pub async fn complete_task(&self, token: &str, task_id: i32) -> Result<(), String> {
        let mut runners = self.runners.write().await;
        
        match runners.get_mut(token) {
            Some(runner) => {
                runner.current_tasks.retain(|&id| id != task_id);
                debug!("Task {} completed on runner {}", task_id, token);
                Ok(())
            }
            None => Err(format!("Runner not found: {}", token)),
        }
    }

    /// Запускает мониторинг здоровья раннеров
    pub async fn start_health_monitor(self: Arc<Self>) {
        info!("Starting remote runners health monitor");
        
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.heartbeat_interval_secs)
        );
        
        loop {
            interval.tick().await;
            self.check_health().await;
        }
    }

    /// Проверяет здоровье раннеров
    async fn check_health(&self) {
        let cutoff = Utc::now() - chrono::Duration::seconds(self.config.runner_timeout_secs as i64);
        let mut runners = self.runners.write().await;
        
        let mut to_remove = Vec::new();
        for (token, runner) in runners.iter() {
            if runner.last_heartbeat < cutoff {
                warn!("Runner {} timed out (last heartbeat: {})", token, runner.last_heartbeat);
                to_remove.push(token.clone());
            }
        }
        
        for token in to_remove {
            runners.remove(&token);
            error!("Runner {} removed due to timeout", token);
        }
    }

    /// Получает статистику раннеров
    pub async fn get_stats(&self) -> RemoteRunnersStats {
        let runners = self.runners.read().await;
        let active = self.get_active_runners().await.len();
        let total_tasks: usize = runners.values().map(|r| r.current_tasks.len()).sum();
        
        RemoteRunnersStats {
            total_runners: runners.len(),
            active_runners: active,
            total_tasks_running: total_tasks,
            tasks_queued: self.task_queue.read().await.len(),
        }
    }
}

/// Статистика Remote Runners
#[derive(Debug, Clone, Default)]
pub struct RemoteRunnersStats {
    pub total_runners: usize,
    pub active_runners: usize,
    pub total_tasks_running: usize,
    pub tasks_queued: usize,
}

/// Request для регистрации раннера
#[derive(Debug, Deserialize)]
pub struct RunnerRegisterRequest {
    pub name: String,
    pub project_id: Option<i32>,
    pub capabilities: RunnerCapabilities,
    pub webhook: Option<String>,
    pub max_parallel_tasks: Option<i32>,
    pub tags: Option<Vec<String>>,
}

/// Response при регистрации раннера
#[derive(Debug, Serialize)]
pub struct RunnerRegisterResponse {
    pub runner_id: i32,
    pub token: String,
    pub message: String,
}

/// Request для heartbeat
#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    pub token: String,
    pub current_tasks: Vec<i32>,
    pub progress: Option<RunnerProgress>,
}

/// Прогресс выполнения задачи
#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerProgress {
    pub task_id: i32,
    pub percent: f64,
    pub message: String,
}

/// Request для результата задачи
#[derive(Debug, Deserialize)]
pub struct TaskResultRequest {
    pub token: String,
    pub task_id: i32,
    pub status: String,
    pub output: String,
    pub duration_secs: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_remote_runners_manager_creation() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);
        assert_eq!(manager.get_active_runners().await.len(), 0);
    }

    #[tokio::test]
    async fn test_runner_registration() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);
        
        let runner = Runner {
            id: 1,
            project_id: Some(1),
            token: "test-token".to_string(),
            name: "Test Runner".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        
        let capabilities = RunnerCapabilities::default();
        let result = manager.register_runner(runner, capabilities).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-token");
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);
        
        let runner = Runner {
            id: 1,
            project_id: Some(1),
            token: "test-token".to_string(),
            name: "Test Runner".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };
        
        let capabilities = RunnerCapabilities::default();
        manager.register_runner(runner, capabilities).await.unwrap();
        
        let result = manager.heartbeat("test-token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_task_queue() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);
        
        manager.queue_task(1).await;
        manager.queue_task(2).await;
        
        let task1 = manager.dequeue_task().await;
        let task2 = manager.dequeue_task().await;
        let task3 = manager.dequeue_task().await;
        
        assert_eq!(task1, Some(1));
        assert_eq!(task2, Some(2));
        assert_eq!(task3, None);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_runners, 0);
        assert_eq!(stats.active_runners, 0);
        assert_eq!(stats.total_tasks_running, 0);
        assert_eq!(stats.tasks_queued, 0);
    }

    #[tokio::test]
    async fn test_register_runner_disabled() {
        let config = RemoteRunnersConfig {
            enabled: false,
            ..RemoteRunnersConfig::default()
        };
        let manager = RemoteRunnersManager::new(config);

        let runner = Runner {
            id: 1,
            project_id: Some(1),
            token: "test-token".to_string(),
            name: "Test Runner".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };

        let result = manager.register_runner(runner, RunnerCapabilities::default()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Remote runners disabled");
    }

    #[tokio::test]
    async fn test_heartbeat_runner_not_found() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);

        let result = manager.heartbeat("nonexistent-token").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Runner not found: nonexistent-token");
    }

    #[tokio::test]
    async fn test_unregister_runner_success() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);

        let runner = Runner {
            id: 1,
            project_id: Some(1),
            token: "test-token".to_string(),
            name: "Test Runner".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };

        manager.register_runner(runner, RunnerCapabilities::default()).await.unwrap();

        let result = manager.unregister_runner("test-token").await;
        assert!(result.is_ok());

        let active = manager.get_active_runners().await;
        assert_eq!(active.len(), 0);
    }

    #[tokio::test]
    async fn test_unregister_runner_not_found() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);

        let result = manager.unregister_runner("nonexistent").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Runner not found: nonexistent");
    }

    #[tokio::test]
    async fn test_assign_and_complete_task() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);

        let runner = Runner {
            id: 1,
            project_id: Some(1),
            token: "test-token".to_string(),
            name: "Test Runner".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };

        manager.register_runner(runner, RunnerCapabilities::default()).await.unwrap();

        let result = manager.assign_task("test-token", 42).await;
        assert!(result.is_ok());

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_tasks_running, 1);

        let result = manager.complete_task("test-token", 42).await;
        assert!(result.is_ok());

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_tasks_running, 0);
    }

    #[tokio::test]
    async fn test_assign_task_runner_not_found() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);

        let result = manager.assign_task("nonexistent", 42).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Runner not found: nonexistent");
    }

    #[tokio::test]
    async fn test_complete_task_runner_not_found() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);

        let result = manager.complete_task("nonexistent", 42).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Runner not found: nonexistent");
    }

    #[tokio::test]
    async fn test_get_stats_with_runners_and_tasks() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);

        for i in 1..=2 {
            let runner = Runner {
                id: i,
                project_id: Some(1),
                token: format!("token-{}", i),
                name: format!("Runner {}", i),
                active: true,
                last_active: None,
                webhook: None,
                max_parallel_tasks: None,
                tag: None,
                cleaning_requested: None,
                touched: None,
                created: None,
            };
            manager.register_runner(runner, RunnerCapabilities::default()).await.unwrap();
        }

        manager.assign_task("token-1", 1).await.unwrap();
        manager.assign_task("token-1", 2).await.unwrap();
        manager.assign_task("token-2", 3).await.unwrap();

        manager.queue_task(4).await;
        manager.queue_task(5).await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_runners, 2);
        assert_eq!(stats.active_runners, 2);
        assert_eq!(stats.total_tasks_running, 3);
        assert_eq!(stats.tasks_queued, 2);
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = RemoteRunnersConfig {
            enabled: true,
            heartbeat_interval_secs: 60,
            runner_timeout_secs: 180,
            max_runners_per_project: 5,
        };
        let manager = RemoteRunnersManager::new(config);

        let active = manager.get_active_runners().await;
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_runner_capabilities_serialization() {
        let capabilities = RunnerCapabilities {
            ansible: true,
            terraform: true,
            bash: false,
            powershell: false,
            kubernetes: true,
            max_parallel_tasks: 5,
            tags: vec!["linux".to_string(), "production".to_string()],
        };

        let json = serde_json::to_string(&capabilities).unwrap();
        let deserialized: RunnerCapabilities = serde_json::from_str(&json).unwrap();

        assert!(deserialized.ansible);
        assert!(deserialized.terraform);
        assert!(!deserialized.bash);
        assert!(deserialized.kubernetes);
        assert_eq!(deserialized.max_parallel_tasks, 5);
        assert_eq!(deserialized.tags, vec!["linux", "production"]);
    }

    #[test]
    fn test_remote_runners_config_default() {
        let config = RemoteRunnersConfig::default();
        assert!(config.enabled);
        assert_eq!(config.heartbeat_interval_secs, 30);
        assert_eq!(config.runner_timeout_secs, 90);
        assert_eq!(config.max_runners_per_project, 10);
    }

    #[test]
    fn test_remote_runners_config_custom() {
        let config = RemoteRunnersConfig {
            enabled: false,
            heartbeat_interval_secs: 60,
            runner_timeout_secs: 180,
            max_runners_per_project: 5,
        };
        assert!(!config.enabled);
        assert_eq!(config.heartbeat_interval_secs, 60);
        assert_eq!(config.runner_timeout_secs, 180);
        assert_eq!(config.max_runners_per_project, 5);
    }

    #[test]
    fn test_runner_capabilities_default() {
        let caps = RunnerCapabilities::default();
        assert!(!caps.ansible);
        assert!(!caps.terraform);
        assert!(!caps.bash);
        assert!(!caps.powershell);
        assert!(!caps.kubernetes);
        assert_eq!(caps.max_parallel_tasks, 0);
        assert!(caps.tags.is_empty());
    }

    #[test]
    fn test_runner_capabilities_clone() {
        let caps = RunnerCapabilities {
            ansible: true,
            terraform: false,
            bash: true,
            powershell: false,
            kubernetes: false,
            max_parallel_tasks: 3,
            tags: vec!["tag1".to_string()],
        };
        let cloned = caps.clone();
        assert_eq!(cloned.ansible, caps.ansible);
        assert_eq!(cloned.tags, caps.tags);
    }

    #[tokio::test]
    async fn test_register_disabled_returns_error() {
        let config = RemoteRunnersConfig {
            enabled: false,
            ..RemoteRunnersConfig::default()
        };
        let manager = RemoteRunnersManager::new(config);

        let runner = Runner {
            id: 1,
            project_id: Some(1),
            token: "token".to_string(),
            name: "Test".to_string(),
            active: true,
            last_active: None,
            webhook: None,
            max_parallel_tasks: None,
            tag: None,
            cleaning_requested: None,
            touched: None,
            created: None,
        };

        let result = manager.register_runner(runner, RunnerCapabilities::default()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Remote runners disabled");
    }

    #[tokio::test]
    async fn test_heartbeat_nonexistent_runner() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);

        let result = manager.heartbeat("does-not-exist").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unregister_nonexistent_runner() {
        let config = RemoteRunnersConfig::default();
        let manager = RemoteRunnersManager::new(config);

        let result = manager.unregister_runner("does-not-exist").await;
        assert!(result.is_err());
    }
}
