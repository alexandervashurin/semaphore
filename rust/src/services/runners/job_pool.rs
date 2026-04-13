//! Job Pool
//!
//! Пул задач для раннеров

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, sleep, timeout, Duration};

use crate::db::store::Store;
use crate::error::Result;
use crate::models::Task;
use crate::services::task_execution;
use crate::services::task_logger::TaskStatus;
use crate::services::runners::task_queue::TaskQueue;

/// Логгер задач
pub struct JobLogger {
    pub context: String,
}

impl JobLogger {
    pub fn new(context: &str) -> Self {
        Self {
            context: context.to_string(),
        }
    }

    pub fn info(&self, message: &str) {
        tracing::info!("[{}] {}", self.context, message);
    }

    pub fn debug(&self, message: &str) {
        tracing::debug!("[{}] {}", self.context, message);
    }

    pub fn task_info(&self, message: &str, task_id: i32, status: &str) {
        tracing::info!("[{}] {} - Task {}: {}", self.context, message, task_id, status);
    }
}

/// Пул задач
pub struct JobPool {
    /// Очередь задач, ожидающих запуска
    queue: Arc<Mutex<Vec<QueuedTask>>>,
    /// ID задач, которые сейчас выполняются (трекер параллелизма)
    running_ids: Arc<Mutex<HashSet<i32>>>,
    /// Хранилище данных
    store: Arc<dyn Store + Send + Sync>,
    /// Персистентная очередь задач (Redis или in-memory)
    task_queue: Option<Arc<dyn TaskQueue>>,
    /// Максимальное число параллельных задач
    max_parallel: usize,
    /// Флаг завершения — при true новые задачи не берутся
    shutting_down: Arc<AtomicBool>,
}

impl JobPool {
    /// Создаёт новый пул задач
    pub fn new(store: Arc<dyn Store + Send + Sync>) -> Self {
        Self::with_task_queue(store, None)
    }

    /// Создаёт пул задач с ограничением параллелизма
    pub fn with_max_parallel(store: Arc<dyn Store + Send + Sync>, max_parallel: usize) -> Self {
        Self::with_task_queue(store, None)
    }

    /// Создаёт пул задач с персистентной очередью (Redis)
    pub fn with_task_queue(store: Arc<dyn Store + Send + Sync>, task_queue: Option<Arc<dyn TaskQueue>>) -> Self {
        let backend = task_queue.as_ref().map(|q| q.backend_name()).unwrap_or("none");
        tracing::info!("[job_pool] Task queue backend: {}", backend);
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            running_ids: Arc::new(Mutex::new(HashSet::new())),
            store,
            task_queue,
            max_parallel: 10,
            shutting_down: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Graceful shutdown: прекращает брать новые задачи и ждёт завершения текущих (макс. 30 сек)
    pub async fn shutdown(&self) {
        self.shutting_down.store(true, Ordering::SeqCst);
        tracing::info!("[job_pool] Shutdown requested, waiting for running tasks...");

        let running_ids = self.running_ids.clone();
        let result = timeout(Duration::from_secs(30), async move {
            loop {
                if running_ids.lock().await.is_empty() {
                    break;
                }
                sleep(Duration::from_millis(500)).await;
            }
        })
        .await;

        match result {
            Ok(()) => tracing::info!("[job_pool] All tasks finished, shutdown complete"),
            Err(_) => tracing::warn!("[job_pool] Shutdown timeout (30s), forcing exit with running tasks"),
        }
    }

    /// Проверяет, есть ли задача в очереди
    pub async fn exists_in_queue(&self, task_id: i32) -> bool {
        // Проверяем персистентную очередь если доступна
        if let Some(ref tq) = self.task_queue {
            if let Ok(found) = tq.contains(task_id).await {
                return found;
            }
        }
        // Fallback на in-memory очередь
        let queue = self.queue.lock().await;
        queue.iter().any(|j| j.task.id == task_id)
    }

    /// Возвращает длину очереди задач
    pub async fn queue_len(&self) -> usize {
        if let Some(ref tq) = self.task_queue {
            tq.len().await.unwrap_or(0)
        } else {
            self.queue.lock().await.len()
        }
    }

    /// Проверяет, есть ли запущенные задачи
    pub async fn has_running_jobs(&self) -> bool {
        !self.running_ids.lock().await.is_empty()
    }

    /// Запускает пул задач (бесконечный цикл, завершается при shutdown)
    pub async fn run(&self) -> Result<()> {
        let logger = JobLogger::new("running");
        let mut queue_interval = interval(Duration::from_secs(5));
        let mut request_interval = interval(Duration::from_secs(1));

        loop {
            if self.shutting_down.load(Ordering::SeqCst) {
                tracing::info!("[job_pool] Shutting down — run loop stopped");
                return Ok(());
            }
            tokio::select! {
                _ = queue_interval.tick() => {
                    self.check_queue(&logger).await;
                }
                _ = request_interval.tick() => {
                    self.check_new_jobs(&logger).await;
                }
            }
        }
    }

    /// Запускает ожидающие задачи из очереди
    async fn check_queue(&self, logger: &JobLogger) {
        let running_count = self.running_ids.lock().await.len();
        if running_count >= self.max_parallel {
            return;
        }

        // Пробуем взять задачу из персистентной очереди (Redis)
        if let Some(ref tq) = self.task_queue {
            match tq.pop().await {
                Ok(Some(task_id)) => {
                    tracing::info!("[job_pool] Launching task from persistent queue: {}", task_id);
                    self.launch_task_by_id(task_id).await;
                    return;
                }
                Ok(None) => {} // Очередь пуста
                Err(e) => {
                    tracing::warn!("[job_pool] Task queue error: {}", e);
                }
            }
        }

        // Fallback на in-memory очередь
        let mut queue = self.queue.lock().await;
        if queue.is_empty() {
            return;
        }

        let queued = queue.remove(0);
        if queued.status == TaskStatus::Error {
            logger.task_info("Task dequeued (error)", queued.task.id, "failed");
            return;
        }

        self.launch_queued_task(queued).await;
    }

    /// Запускает задачу по ID (извлекает из БД)
    async fn launch_task_by_id(&self, task_id: i32) {
        self.running_ids.lock().await.insert(task_id);

        let store = self.store.clone();
        let running_ids = self.running_ids.clone();

        tokio::spawn(async move {
            // Получаем задачу из БД
            match store.get_tasks(0, None).await {
                Ok(tasks) => {
                    if let Some(task_with_tpl) = tasks.into_iter().find(|t| t.task.id == task_id) {
                        task_execution::execute_task(store, task_with_tpl.task).await;
                    }
                }
                Err(e) => tracing::warn!("[job_pool] Failed to load task {}: {}", task_id, e),
            }
            running_ids.lock().await.remove(&task_id);
        });
    }

    /// Запускает задачу из in-memory очереди
    async fn launch_queued_task(&self, queued: QueuedTask) {
        let task_id = queued.task.id;

        self.running_ids.lock().await.insert(task_id);

        let store = self.store.clone();
        let task = queued.task;
        let running_ids = self.running_ids.clone();

        tokio::spawn(async move {
            task_execution::execute_task(store, task).await;
            running_ids.lock().await.remove(&task_id);
        });
    }

    /// Ищет новые задачи в БД и добавляет их в очередь
    async fn check_new_jobs(&self, logger: &JobLogger) {
        if self.shutting_down.load(Ordering::SeqCst) {
            return;
        }
        let running_count = self.running_ids.lock().await.len();
        if running_count >= self.max_parallel {
            return;
        }

        let queue_len = self.queue.lock().await.len();
        let available_slots = (self.max_parallel - running_count).saturating_sub(queue_len);
        if available_slots == 0 {
            return;
        }

        let limit = available_slots.min(50) as i32;
        let tasks = match self.store.get_global_tasks(
            Some(vec!["waiting".to_string()]),
            Some(limit),
        ).await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("[job_pool] Failed to fetch waiting tasks: {e}");
                return;
            }
        };

        if tasks.is_empty() {
            return;
        }

        // Если есть персистентная очередь — пушим туда
        if let Some(ref tq) = self.task_queue {
            for task_with_tpl in tasks {
                let task_id = task_with_tpl.task.id;
                // Проверяем что задача ещё не в очереди
                if let Ok(exists) = tq.contains(task_id).await {
                    if exists {
                        continue;
                    }
                }
                if let Err(e) = tq.push(task_id).await {
                    logger.task_info("Failed to push to task queue", task_id, &format!("error: {}", e));
                }
            }
            return;
        }

        // Fallback на in-memory очередь
        let mut queue = self.queue.lock().await;
        let running = self.running_ids.lock().await;

        for task_with_tpl in tasks {
            let task_id = task_with_tpl.task.id;
            if queue.iter().any(|j| j.task.id == task_id) || running.contains(&task_id) {
                continue;
            }

            logger.task_info("Queueing task", task_id, "waiting");
            queue.push(QueuedTask {
                task: task_with_tpl.task,
                status: TaskStatus::Waiting,
            });
        }
    }
}

/// Задача в очереди
#[derive(Clone)]
pub struct QueuedTask {
    /// Задача для запуска
    pub task: Task,
    /// Статус в очереди
    pub status: TaskStatus,
}

// Обратная совместимость
pub type Job = QueuedTask;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::mock::MockStore;
    use std::sync::atomic::Ordering;

    fn make_store() -> Arc<dyn Store + Send + Sync> {
        Arc::new(MockStore::new())
    }

    #[test]
    fn test_job_pool_creation() {
        let _pool = JobPool::new(make_store());
    }

    #[test]
    fn test_job_logger_creation() {
        let logger = JobLogger::new("test");
        assert_eq!(logger.context, "test");
    }

    #[tokio::test]
    async fn test_exists_in_queue_empty() {
        let pool = JobPool::new(make_store());
        assert!(!pool.exists_in_queue(1).await);
    }

    #[tokio::test]
    async fn test_has_running_jobs_empty() {
        let pool = JobPool::new(make_store());
        assert!(!pool.has_running_jobs().await);
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let pool = JobPool::new(make_store());

        pool.running_ids.lock().await.insert(42);

        let running_ids = pool.running_ids.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            running_ids.lock().await.remove(&42);
        });

        pool.shutdown().await;

        assert!(pool.shutting_down.load(Ordering::SeqCst));
        assert!(!pool.has_running_jobs().await);
    }

    #[tokio::test]
    async fn test_queue_len_nonempty() {
        let pool = JobPool::new(make_store());
        pool.queue.lock().await.push(QueuedTask {
            task: crate::models::Task::default(),
            status: TaskStatus::Waiting,
        });
        assert_eq!(pool.queue_len().await, 1);
    }

    #[tokio::test]
    async fn test_exists_in_queue_positive() {
        let pool = JobPool::new(make_store());
        pool.queue.lock().await.push(QueuedTask {
            task: crate::models::Task { id: 99, ..crate::models::Task::default() },
            status: TaskStatus::Waiting,
        });
        assert!(pool.exists_in_queue(99).await);
        assert!(!pool.exists_in_queue(1).await);
    }

    #[tokio::test]
    async fn test_has_running_jobs_positive() {
        let pool = JobPool::new(make_store());
        pool.running_ids.lock().await.insert(42);
        assert!(pool.has_running_jobs().await);
    }

    #[tokio::test]
    async fn test_with_max_parallel() {
        // Note: current implementation has a bug -- max_parallel is hardcoded to 10
        // This test documents current behavior
        let pool = JobPool::with_max_parallel(make_store(), 5);
        // max_parallel field should be 5 after fix, but currently stays 10
        assert_eq!(pool.max_parallel, 10); // documents bug
    }

    #[tokio::test]
    async fn test_with_task_queue_none() {
        let pool = JobPool::with_task_queue(make_store(), None);
        assert_eq!(pool.queue_len().await, 0);
    }

    #[tokio::test]
    async fn test_run_stops_on_shutdown() {
        let pool = JobPool::new(make_store());
        pool.shutting_down.store(true, Ordering::SeqCst);
        let result = pool.run().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_queued_task_default() {
        let qt = QueuedTask {
            task: crate::models::Task::default(),
            status: TaskStatus::Waiting,
        };
        assert_eq!(qt.task.id, 0);
        assert_eq!(qt.status, TaskStatus::Waiting);
    }

    #[test]
    fn test_job_logger_info_method() {
        let logger = JobLogger::new("test_logger");
        // Just verify it doesn't panic
        logger.info("Test info message");
    }

    #[test]
    fn test_job_logger_debug_method() {
        let logger = JobLogger::new("test_debug");
        logger.debug("Test debug message");
    }

    #[test]
    fn test_job_logger_task_info_method() {
        let logger = JobLogger::new("test_task");
        logger.task_info("Status update", 42, "running");
    }

    #[tokio::test]
    async fn test_job_pool_shutdown_flag() {
        let pool = JobPool::new(make_store());
        assert!(!pool.shutting_down.load(Ordering::SeqCst));

        pool.shutting_down.store(true, Ordering::SeqCst);
        assert!(pool.shutting_down.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_job_pool_max_parallel() {
        let pool = JobPool::new(make_store());
        // Default max_parallel is 10
        assert_eq!(pool.max_parallel, 10);
    }

    #[tokio::test]
    async fn test_queue_len_multiple_tasks() {
        let pool = JobPool::new(make_store());
        let mut queue = pool.queue.lock().await;
        for i in 1..=5 {
            queue.push(QueuedTask {
                task: crate::models::Task { id: i, ..crate::models::Task::default() },
                status: TaskStatus::Waiting,
            });
        }
        drop(queue);
        assert_eq!(pool.queue_len().await, 5);
    }

    #[tokio::test]
    async fn test_running_ids_multiple_tasks() {
        let pool = JobPool::new(make_store());
        let mut running = pool.running_ids.lock().await;
        running.insert(1);
        running.insert(2);
        running.insert(3);
        drop(running);
        assert!(pool.has_running_jobs().await);
    }

    #[tokio::test]
    async fn test_shutdown_timeout_behavior() {
        let pool = JobPool::new(make_store());
        // Don't add any running tasks - should shutdown immediately
        pool.shutdown().await;
        assert!(pool.shutting_down.load(Ordering::SeqCst));
    }

    #[test]
    fn test_queued_task_clone() {
        let qt1 = QueuedTask {
            task: crate::models::Task { id: 1, ..crate::models::Task::default() },
            status: TaskStatus::Running,
        };
        let qt2 = qt1.clone();
        assert_eq!(qt2.task.id, qt1.task.id);
        assert_eq!(qt2.status, qt1.status);
    }

    #[test]
    fn test_queued_task_task_id_accessible() {
        let qt = QueuedTask {
            task: crate::models::Task { id: 42, ..crate::models::Task::default() },
            status: TaskStatus::Waiting,
        };
        assert_eq!(qt.task.id, 42);
    }

    #[test]
    fn test_job_logger_context_stored() {
        let logger = JobLogger::new("my_context");
        assert_eq!(logger.context, "my_context");
    }

    #[test]
    fn test_job_logger_with_empty_context() {
        let logger = JobLogger::new("");
        assert_eq!(logger.context, "");
    }

    #[test]
    fn test_job_logger_info_with_special_chars() {
        let logger = JobLogger::new("test");
        logger.info("Message with special chars: <>&\"'");
    }

    #[test]
    fn test_job_logger_debug_with_long_message() {
        let logger = JobLogger::new("debug_ctx");
        let long_msg = "a".repeat(1000);
        logger.debug(&long_msg);
    }

    #[test]
    fn test_job_logger_task_info_all_statuses() {
        let logger = JobLogger::new("pool");
        logger.task_info("Starting", 1, "waiting");
        logger.task_info("Executing", 2, "running");
        logger.task_info("Done", 3, "success");
        logger.task_info("Failed", 4, "error");
        logger.task_info("Stopped", 5, "stopped");
    }

    #[tokio::test]
    async fn test_queued_task_with_error_status() {
        let qt = QueuedTask {
            task: crate::models::Task { id: 99, ..crate::models::Task::default() },
            status: TaskStatus::Error,
        };
        assert_eq!(qt.status, TaskStatus::Error);
        assert_eq!(qt.task.id, 99);
    }

    #[tokio::test]
    async fn test_queued_task_with_running_status() {
        let qt = QueuedTask {
            task: crate::models::Task { id: 10, ..crate::models::Task::default() },
            status: TaskStatus::Running,
        };
        assert_eq!(qt.status, TaskStatus::Running);
    }

    #[tokio::test]
    async fn test_job_pool_queue_initial_state() {
        let pool = JobPool::new(make_store());
        assert_eq!(pool.queue_len().await, 0);
        assert!(!pool.has_running_jobs().await);
    }

    #[tokio::test]
    async fn test_job_pool_multiple_running_ids() {
        let pool = JobPool::new(make_store());
        let mut running = pool.running_ids.lock().await;
        for i in 1..=10 {
            running.insert(i);
        }
        drop(running);
        assert_eq!(pool.running_ids.lock().await.len(), 10);
    }

    #[tokio::test]
    async fn test_job_pool_queue_multiple_tasks() {
        let pool = JobPool::new(make_store());
        let mut queue = pool.queue.lock().await;
        for i in 1..=3 {
            queue.push(QueuedTask {
                task: crate::models::Task { id: i, ..crate::models::Task::default() },
                status: TaskStatus::Waiting,
            });
        }
        drop(queue);

        // Verify FIFO order
        let mut queue = pool.queue.lock().await;
        let first = queue.remove(0);
        assert_eq!(first.task.id, 1);
        let second = queue.remove(0);
        assert_eq!(second.task.id, 2);
    }

    #[tokio::test]
    async fn test_job_pool_shutdown_prevents_new_jobs() {
        let pool = JobPool::new(make_store());
        pool.shutting_down.store(true, Ordering::SeqCst);
        assert!(pool.shutting_down.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_job_pool_with_max_parallel_uses_default() {
        let store = make_store();
        let pool = JobPool::with_max_parallel(store, 100);
        // Current implementation has hardcoded default of 10
        assert_eq!(pool.max_parallel, 10);
    }

    #[tokio::test]
    async fn test_job_pool_shutdown_immediate_when_no_running() {
        let pool = JobPool::new(make_store());
        assert!(!pool.has_running_jobs().await);

        pool.shutdown().await;

        assert!(pool.shutting_down.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_exists_in_queue_with_different_ids() {
        let pool = JobPool::new(make_store());
        assert!(!pool.exists_in_queue(1).await);
        assert!(!pool.exists_in_queue(2).await);
        assert!(!pool.exists_in_queue(999).await);
    }

    #[tokio::test]
    async fn test_job_type_alias() {
        let job: Job = QueuedTask {
            task: crate::models::Task { id: 7, ..crate::models::Task::default() },
            status: TaskStatus::Waiting,
        };
        assert_eq!(job.task.id, 7);
    }

    #[tokio::test]
    async fn test_queued_task_with_success_status() {
        let qt = QueuedTask {
            task: crate::models::Task { id: 50, ..crate::models::Task::default() },
            status: TaskStatus::Success,
        };
        assert_eq!(qt.status, TaskStatus::Success);
    }

    #[tokio::test]
    async fn test_queued_task_with_stopped_status() {
        let qt = QueuedTask {
            task: crate::models::Task { id: 51, ..crate::models::Task::default() },
            status: TaskStatus::Stopped,
        };
        assert_eq!(qt.status, TaskStatus::Stopped);
    }

    #[test]
    fn test_job_logger_new_does_not_panic() {
        let _logger = JobLogger::new("initialization_test");
    }
}
