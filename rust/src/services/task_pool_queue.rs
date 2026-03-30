//! TaskPool Queue - очередь задач
//!
//! Аналог services/tasks/TaskPool.go из Go версии (часть 2: очередь)

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

use crate::models::Task;
use crate::services::task_pool_types::TaskPool;

impl TaskPool {
    /// Добавляет задачу в очередь
    pub async fn add_task(&self, task: Task) -> Result<(), String> {
        if self.is_shutdown().await {
            return Err("TaskPool is shutdown".to_string());
        }
        
        let mut queue = self.task_queue.lock().await;
        queue.push(task);
        
        info!("Task added to queue. Queue size: {}", queue.len());
        
        Ok(())
    }
    
    /// Получает задачу из очереди
    pub async fn get_next_task(&self) -> Option<Task> {
        let mut queue = self.task_queue.lock().await;
        
        if queue.is_empty() {
            return None;
        }
        
        let task = queue.remove(0);
        info!("Task removed from queue. Queue size: {}", queue.len());
        
        Some(task)
    }
    
    /// Получает размер очереди
    pub async fn queue_size(&self) -> usize {
        let queue = self.task_queue.lock().await;
        queue.len()
    }
    
    /// Очищает очередь
    pub async fn clear_queue(&self) {
        let mut queue = self.task_queue.lock().await;
        let count = queue.len();
        queue.clear();
        
        info!("Queue cleared. Removed {} tasks.", count);
    }
    
    /// Получает все задачи из очереди
    pub async fn get_queue(&self) -> Vec<Task> {
        let queue = self.task_queue.lock().await;
        queue.clone()
    }
    
    /// Удаляет задачу из очереди по ID
    pub async fn remove_task(&self, task_id: i32) -> bool {
        let mut queue = self.task_queue.lock().await;
        
        if let Some(pos) = queue.iter().position(|t| t.id == task_id) {
            queue.remove(pos);
            info!("Task {} removed from queue", task_id);
            return true;
        }
        
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Task;
    use crate::services::task_logger::TaskStatus;
    use chrono::Utc;

    fn create_test_task(id: i32) -> Task {
        let mut task = Task::default();
        task.id = id;
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Waiting;
        task.message = Some(format!("Task {}", id));
        task
    }

    async fn create_test_pool() -> TaskPool {
        use crate::db::mock::MockStore;
        use crate::db::Store;
        use crate::models::Project;

        let store: Arc<dyn Store + Send + Sync> = Arc::new(MockStore::new());
        let project = Project {
            id: 1,
            name: "Test Project".to_string(),
            created: Utc::now(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 5,
            r#type: "default".to_string(),
            default_secret_storage_id: None,
        };
        
        TaskPool::new(store, project, Arc::new(crate::api::websocket::WebSocketManager::new()))
    }

    #[tokio::test]
    async fn test_add_task() {
        let pool = create_test_pool().await;
        let task = create_test_task(1);
        
        let result = pool.add_task(task).await;
        assert!(result.is_ok());
        
        assert_eq!(pool.queue_size().await, 1);
    }

    #[tokio::test]
    async fn test_get_next_task() {
        let pool = create_test_pool().await;
        let task = create_test_task(1);
        
        pool.add_task(task).await.unwrap();
        
        let next_task = pool.get_next_task().await;
        assert!(next_task.is_some());
        assert_eq!(next_task.unwrap().id, 1);
        
        assert_eq!(pool.queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_queue_size() {
        let pool = create_test_pool().await;
        
        assert_eq!(pool.queue_size().await, 0);
        
        pool.add_task(create_test_task(1)).await.unwrap();
        pool.add_task(create_test_task(2)).await.unwrap();
        pool.add_task(create_test_task(3)).await.unwrap();
        
        assert_eq!(pool.queue_size().await, 3);
    }

    #[tokio::test]
    async fn test_clear_queue() {
        let pool = create_test_pool().await;
        
        pool.add_task(create_test_task(1)).await.unwrap();
        pool.add_task(create_test_task(2)).await.unwrap();
        pool.add_task(create_test_task(3)).await.unwrap();
        
        pool.clear_queue().await;
        
        assert_eq!(pool.queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_remove_task() {
        let pool = create_test_pool().await;
        
        pool.add_task(create_test_task(1)).await.unwrap();
        pool.add_task(create_test_task(2)).await.unwrap();
        pool.add_task(create_test_task(3)).await.unwrap();
        
        let removed = pool.remove_task(2).await;
        assert!(removed);
        
        assert_eq!(pool.queue_size().await, 2);
        
        let removed = pool.remove_task(999).await;
        assert!(!removed);
    }

    #[tokio::test]
    async fn test_get_queue() {
        let pool = create_test_pool().await;
        
        pool.add_task(create_test_task(1)).await.unwrap();
        pool.add_task(create_test_task(2)).await.unwrap();
        
        let queue = pool.get_queue().await;
        assert_eq!(queue.len(), 2);
        assert_eq!(queue[0].id, 1);
        assert_eq!(queue[1].id, 2);
    }

    #[tokio::test]
    async fn test_add_task_after_shutdown() {
        let pool = create_test_pool().await;
        
        pool.shutdown().await;
        
        let result = pool.add_task(create_test_task(1)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "TaskPool is shutdown");
    }
}
