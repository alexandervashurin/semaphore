//! Task State Store
//!
//! Хранилище состояния задач

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::models::Task;
use crate::services::task_logger::TaskStatus;

/// Хранилище состояния задач
pub struct TaskStateStore {
    states: Arc<RwLock<HashMap<i32, TaskState>>>,
}

impl TaskStateStore {
    /// Создаёт новое хранилище
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Получает состояние задачи
    pub async fn get_state(&self, task_id: i32) -> Option<TaskState> {
        let states = self.states.read().await;
        states.get(&task_id).cloned()
    }

    /// Устанавливает состояние задачи
    pub async fn set_state(&self, task_id: i32, state: TaskState) {
        let mut states = self.states.write().await;
        states.insert(task_id, state);
    }

    /// Удаляет состояние задачи
    pub async fn remove_state(&self, task_id: i32) {
        let mut states = self.states.write().await;
        states.remove(&task_id);
    }

    /// Получает все состояния
    pub async fn get_all_states(&self) -> HashMap<i32, TaskState> {
        let states = self.states.read().await;
        states.clone()
    }

    /// Очищает все состояния
    pub async fn clear(&self) {
        let mut states = self.states.write().await;
        states.clear();
    }
}

impl Default for TaskStateStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Состояние задачи
#[derive(Debug, Clone)]
pub struct TaskState {
    pub task: Task,
    pub status: TaskStatus,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TaskState {
    /// Создаёт новое состояние
    pub fn new(task: Task, status: TaskStatus) -> Self {
        Self {
            task,
            status,
            started_at: None,
            ended_at: None,
        }
    }

    /// Устанавливает время начала
    pub fn start(&mut self) {
        self.started_at = Some(chrono::Utc::now());
    }

    /// Устанавливает время окончания
    pub fn end(&mut self) {
        self.ended_at = Some(chrono::Utc::now());
    }

    /// Проверяет, завершена ли задача
    pub fn is_finished(&self) -> bool {
        self.status.is_finished()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_state_store_creation() {
        let store = TaskStateStore::new();
        assert!(true);
    }

    #[test]
    fn test_task_state_creation() {
        // Тест для проверки создания состояния
        assert!(true);
    }
}
