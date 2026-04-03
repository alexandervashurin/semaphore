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
}
