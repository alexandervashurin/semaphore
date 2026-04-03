//! Task Queue abstraction
//!
//! Два бэкенда:
//! - `InMemoryTaskQueue` — VecDeque в памяти (default, без Redis)
//! - `RedisTaskQueue`    — Redis LIST (LPUSH / RPOP) для HA / персистентности

use crate::error::{Error, Result};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

// ============================================================================
// Trait
// ============================================================================

/// Абстракция очереди task-id'ов.
///
/// Реализации обязаны быть Send + Sync — пул использует их из нескольких потоков.
#[async_trait]
pub trait TaskQueue: Send + Sync {
    /// Добавить task_id в конец очереди (idempotent: не добавляет если уже есть).
    async fn push(&self, task_id: i32) -> Result<()>;

    /// Извлечь следующий task_id из начала очереди.
    async fn pop(&self) -> Result<Option<i32>>;

    /// Количество элементов в очереди.
    async fn len(&self) -> Result<usize>;

    /// Быстрая проверка, что очередь пуста.
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }

    /// Проверить наличие task_id в очереди (без извлечения).
    async fn contains(&self, task_id: i32) -> Result<bool>;

    /// Имя реализации (для логов).
    fn backend_name(&self) -> &'static str;
}

// ============================================================================
// In-memory implementation
// ============================================================================

/// In-memory очередь на основе `VecDeque`.
///
/// Используется когда Redis недоступен или не настроен.
/// Данные теряются при рестарте процесса.
pub struct InMemoryTaskQueue {
    inner: Arc<Mutex<VecDeque<i32>>>,
}

impl InMemoryTaskQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl Default for InMemoryTaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskQueue for InMemoryTaskQueue {
    async fn push(&self, task_id: i32) -> Result<()> {
        let mut q = self.inner.lock().await;
        if !q.contains(&task_id) {
            q.push_back(task_id);
            debug!("[queue:memory] pushed task {task_id}");
        }
        Ok(())
    }

    async fn pop(&self) -> Result<Option<i32>> {
        let mut q = self.inner.lock().await;
        Ok(q.pop_front())
    }

    async fn len(&self) -> Result<usize> {
        Ok(self.inner.lock().await.len())
    }

    async fn contains(&self, task_id: i32) -> Result<bool> {
        Ok(self.inner.lock().await.contains(&task_id))
    }

    fn backend_name(&self) -> &'static str {
        "in-memory"
    }
}

// ============================================================================
// Redis implementation
// ============================================================================

const QUEUE_KEY: &str = "velum:task_queue";

/// Redis-backed очередь на основе Redis LIST.
///
/// Операции:
/// - `push`     → `LPUSH velum:task_queue <task_id>`  (если ещё нет через LPOS)
/// - `pop`      → `RPOP  velum:task_queue`
/// - `len`      → `LLEN  velum:task_queue`
/// - `contains` → `LPOS  velum:task_queue <task_id>`
///
/// При ошибке Redis методы логируют предупреждение и возвращают safe-значение,
/// чтобы не уронить весь job_pool (graceful degradation).
pub struct RedisTaskQueue {
    conn: Arc<Mutex<ConnectionManager>>,
}

impl RedisTaskQueue {
    /// Создать очередь из уже открытого `ConnectionManager`.
    pub fn new(conn: ConnectionManager) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    /// Попытаться подключиться к Redis и вернуть `RedisTaskQueue`.
    ///
    /// Таймаут подключения — 2 секунды (защита от зависания при недоступном сервере).
    pub async fn connect(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)
            .map_err(|e| Error::Other(format!("Redis client error: {e}")))?;
        let conn = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            client.get_connection_manager(),
        )
        .await
        .map_err(|_| Error::Other("Redis connection timeout (2s)".to_string()))?
        .map_err(|e| Error::Other(format!("Redis connection error: {e}")))?;
        Ok(Self::new(conn))
    }
}

#[async_trait]
impl TaskQueue for RedisTaskQueue {
    async fn push(&self, task_id: i32) -> Result<()> {
        let mut conn = self.conn.lock().await;

        // LPOS возвращает None если элемента нет — тогда добавляем
        let pos: redis::RedisResult<Option<i64>> =
            redis::cmd("LPOS").arg(QUEUE_KEY).arg(task_id).query_async(&mut *conn).await;

        let already_exists = matches!(pos, Ok(Some(_)));
        if already_exists {
            return Ok(());
        }

        let result: redis::RedisResult<i64> = conn.lpush(QUEUE_KEY, task_id).await;
        match result {
            Ok(_) => {
                debug!("[queue:redis] pushed task {task_id}");
                Ok(())
            }
            Err(e) => {
                warn!("[queue:redis] push error for task {task_id}: {e}");
                Err(Error::Other(format!("Redis push error: {e}")))
            }
        }
    }

    async fn pop(&self) -> Result<Option<i32>> {
        let mut conn = self.conn.lock().await;
        let result: redis::RedisResult<Option<i64>> = conn.rpop(QUEUE_KEY, None).await;
        match result {
            Ok(Some(id)) => Ok(Some(id as i32)),
            Ok(None) => Ok(None),
            Err(e) => {
                warn!("[queue:redis] pop error: {e}");
                Ok(None) // degraded gracefully
            }
        }
    }

    async fn len(&self) -> Result<usize> {
        let mut conn = self.conn.lock().await;
        let result: redis::RedisResult<i64> = conn.llen(QUEUE_KEY).await;
        match result {
            Ok(n) => Ok(n as usize),
            Err(e) => {
                warn!("[queue:redis] llen error: {e}");
                Ok(0)
            }
        }
    }

    async fn contains(&self, task_id: i32) -> Result<bool> {
        let mut conn = self.conn.lock().await;
        let result: redis::RedisResult<Option<i64>> =
            redis::cmd("LPOS").arg(QUEUE_KEY).arg(task_id).query_async(&mut *conn).await;
        Ok(matches!(result, Ok(Some(_))))
    }

    fn backend_name(&self) -> &'static str {
        "redis"
    }
}

// ============================================================================
// Factory
// ============================================================================

/// Создаёт нужный бэкенд в зависимости от конфигурации.
///
/// Если `redis_url` задан — пробует подключиться к Redis.
/// При ошибке подключения логирует предупреждение и возвращает in-memory.
pub async fn build_task_queue(redis_url: Option<&str>) -> Arc<dyn TaskQueue> {
    if let Some(url) = redis_url {
        match RedisTaskQueue::connect(url).await {
            Ok(q) => {
                tracing::info!("[task_queue] using Redis backend at {url}");
                return Arc::new(q);
            }
            Err(e) => {
                warn!("[task_queue] Redis unavailable ({e}), falling back to in-memory");
            }
        }
    }
    tracing::info!("[task_queue] using in-memory backend");
    Arc::new(InMemoryTaskQueue::new())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- InMemoryTaskQueue ---

    #[tokio::test]
    async fn test_inmemory_push_pop() {
        let q = InMemoryTaskQueue::new();
        q.push(1).await.unwrap();
        q.push(2).await.unwrap();
        assert_eq!(q.len().await.unwrap(), 2);
        assert_eq!(q.pop().await.unwrap(), Some(1));
        assert_eq!(q.pop().await.unwrap(), Some(2));
        assert_eq!(q.pop().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_inmemory_push_idempotent() {
        let q = InMemoryTaskQueue::new();
        q.push(42).await.unwrap();
        q.push(42).await.unwrap(); // дубль не добавляется
        assert_eq!(q.len().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_inmemory_contains() {
        let q = InMemoryTaskQueue::new();
        q.push(10).await.unwrap();
        assert!(q.contains(10).await.unwrap());
        assert!(!q.contains(99).await.unwrap());
    }

    #[tokio::test]
    async fn test_inmemory_pop_empty() {
        let q = InMemoryTaskQueue::new();
        assert_eq!(q.pop().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_inmemory_len_empty() {
        let q = InMemoryTaskQueue::new();
        assert_eq!(q.len().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_inmemory_backend_name() {
        let q = InMemoryTaskQueue::new();
        assert_eq!(q.backend_name(), "in-memory");
    }

    #[tokio::test]
    async fn test_inmemory_fifo_order() {
        let q = InMemoryTaskQueue::new();
        for i in 1..=5 {
            q.push(i).await.unwrap();
        }
        for i in 1..=5 {
            assert_eq!(q.pop().await.unwrap(), Some(i));
        }
    }

    #[tokio::test]
    async fn test_inmemory_default() {
        let q = InMemoryTaskQueue::default();
        assert_eq!(q.len().await.unwrap(), 0);
        assert_eq!(q.backend_name(), "in-memory");
    }

    // --- build_task_queue factory ---

    #[tokio::test]
    async fn test_build_queue_no_redis_returns_inmemory() {
        let q = build_task_queue(None).await;
        assert_eq!(q.backend_name(), "in-memory");
    }

    #[tokio::test]
    async fn test_build_queue_bad_url_falls_back_to_inmemory() {
        // недоступный Redis → fallback (жёстный внешний лимит на случай платформенных зависаний TCP)
        let q = tokio::time::timeout(std::time::Duration::from_secs(15), build_task_queue(Some(
            "redis://127.0.0.1:19999",
        )))
        .await
        .expect("build_task_queue should finish (Redis connect is bounded)");
        assert_eq!(q.backend_name(), "in-memory");
    }
}
