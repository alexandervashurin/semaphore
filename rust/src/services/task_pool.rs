//! Пул задач (TaskPool)
//!
//! Центральный компонент управления очередью задач.
//! Реализует очередь задач, логирование, управление состоянием.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Semaphore};
use chrono::{DateTime, Utc};
use tracing::{info, warn, error, debug};

use crate::error::{Error, Result};
use crate::models::{Task, Project};
use crate::services::task_logger::TaskStatus;
use crate::db::store::Store;

/// Событие пула задач
#[derive(Debug, Clone)]
pub enum TaskPoolEvent {
    /// Новая задача создана
    TaskCreated(Task),
    /// Задача завершена
    TaskFinished { task_id: i32, success: bool },
    /// Задача не удалась
    TaskFailed { task_id: i32, error: String },
    /// Задача возвращена в очередь
    TaskRequeued(Task),
    /// Очередь пуста
    QueueEmpty,
}

/// Запись лога задачи
#[derive(Debug, Clone)]
pub struct TaskLogRecord {
    pub task_id: i32,
    pub output: String,
    pub time: DateTime<Utc>,
}

/// Информация о выполняемой задаче
#[derive(Debug, Clone)]
pub struct RunningTask {
    pub task: Task,
    pub project_id: i32,
    pub started_at: DateTime<Utc>,
    pub runner_id: Option<i32>,
}

/// Состояние пула задач
pub struct TaskPoolState {
    /// Очередь задач по проектам
    pub queue: HashMap<i32, Vec<Task>>,
    /// Активные задачи по проектам
    pub running: HashMap<i32, Vec<RunningTask>>,
    /// Активные проекты
    pub active_projects: HashMap<i32, Project>,
    /// Блокировки для параллельных задач
    pub blocks: HashMap<i32, Arc<Semaphore>>,
}

impl Default for TaskPoolState {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskPoolState {
    pub fn new() -> Self {
        Self {
            queue: HashMap::new(),
            running: HashMap::new(),
            active_projects: HashMap::new(),
            blocks: HashMap::new(),
        }
    }
}

/// Пул задач
pub struct TaskPool {
    /// Канал для добавления задач
    register: mpsc::Sender<Task>,
    /// Канал для логов задач
    logger: mpsc::Sender<TaskLogRecord>,
    /// Канал для событий пула
    events: mpsc::Sender<TaskPoolEvent>,
    /// Состояние
    state: Arc<RwLock<TaskPoolState>>,
    /// Хранилище данных
    store: Arc<dyn Store>,
    /// Флаг работы
    running: Arc<RwLock<bool>>,
    /// Максимум задач на проект
    max_tasks_per_project: usize,
}

impl TaskPool {
    /// Создаёт новый пул задач
    pub fn new(store: Arc<dyn Store>, max_tasks_per_project: usize) -> Self {
        let (register_tx, mut register_rx) = mpsc::channel::<Task>(100);
        let (logger_tx, mut logger_rx) = mpsc::channel::<TaskLogRecord>(1000);
        let (events_tx, mut events_rx) = mpsc::channel::<TaskPoolEvent>(100);
        
        let state = Arc::new(RwLock::new(TaskPoolState::new()));
        let running = Arc::new(RwLock::new(false));

        // Запускаем обработчик регистрации задач
        let state_clone = state.clone();
        let store_clone = store.clone();
        let events_tx_clone = events_tx.clone();
        
        tokio::spawn(async move {
            while let Some(task) = register_rx.recv().await {
                debug!("Получена новая задача: {}", task.id);
                
                let mut state = state_clone.write().await;
                let project_id = task.project_id;
                
                // Инициализируем очередь для проекта если нужно
                state.queue.entry(project_id).or_insert_with(Vec::new);
                
                // Добавляем задачу в очередь
                state.queue.get_mut(&project_id).unwrap().push(task.clone());
                
                // Отправляем событие
                let _ = events_tx_clone.send(TaskPoolEvent::TaskCreated(task)).await;
            }
        });

        // Запускаем обработчик логов
        let _store_clone = store.clone();
        tokio::spawn(async move {
            while let Some(record) = logger_rx.recv().await {
                // Сохраняем лог в БД
                // TODO: Реализовать сохранение в БД
                debug!("Лог задачи {}: {}", record.task_id, record.output);
            }
        });

        // Запускаем обработчик событий
        let state_events = state.clone();
        tokio::spawn(async move {
            while let Some(event) = events_rx.recv().await {
                match event {
                    TaskPoolEvent::TaskFinished { task_id, success } => {
                        info!("Задача {} завершена: {}", task_id, if success { "успешно" } else { "с ошибками" });

                        // Удаляем из running
                        let mut state = state_events.write().await;
                        for (_, running_list) in state.running.iter_mut() {
                            running_list.retain(|rt| rt.task.id != task_id);
                        }
                    }
                    TaskPoolEvent::TaskFailed { task_id, error } => {
                        error!("Задача {} не удалась: {}", task_id, error);
                    }
                    TaskPoolEvent::QueueEmpty => {
                        debug!("Очередь пуста");
                    }
                    _ => {}
                }
            }
        });

        Self {
            register: register_tx,
            logger: logger_tx,
            events: events_tx,
            state,
            store,
            running,
            max_tasks_per_project,
        }
    }

    /// Запускает пул задач
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(Error::Other("Пул задач уже запущен".to_string()));
        }
        *running = true;
        drop(running);

        info!("Пул задач запущен");

        // Запускаем цикл обработки очереди
        let state = self.state.clone();
        let events = self.events.clone();
        let running = self.running.clone();
        let max_tasks = self.max_tasks_per_project;

        tokio::spawn(async move {
            while *running.read().await {
                TaskPool::process_queue(&state, &events, max_tasks).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        Ok(())
    }

    /// Останавливает пул задач
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        drop(running);

        info!("Пул задач остановлен");
        Ok(())
    }

    /// Добавляет задачу в очередь
    pub async fn add_task(&self, task: Task) -> Result<()> {
        self.register.send(task).await
            .map_err(|e| Error::Other(format!("Ошибка добавления задачи: {}", e)))
    }

    /// Записывает лог задачи
    pub async fn log(&self, record: TaskLogRecord) -> Result<()> {
        self.logger.send(record).await
            .map_err(|e| Error::Other(format!("Ошибка логирования: {}", e)))
    }

    /// Обрабатывает очередь задач
    async fn process_queue(
        state: &Arc<RwLock<TaskPoolState>>,
        events: &mpsc::Sender<TaskPoolEvent>,
        max_tasks: usize,
    ) {
        // Собираем информацию о задачах для запуска
        let mut tasks_to_start: Vec<(Task, i32)> = {
            let state = state.read().await;
            
            let mut result = Vec::new();
            
            // Копируем данные для итерации
            let project_ids: Vec<i32> = state.queue.keys().copied().collect();
            
            for project_id in project_ids {
                let queue = match state.queue.get(&project_id) {
                    Some(q) if !q.is_empty() => q,
                    _ => continue,
                };

                // Проверяем лимит задач на проект
                let running_count = state.running
                    .get(&project_id)
                    .map(|r| r.len())
                    .unwrap_or(0);

                if running_count >= max_tasks {
                    debug!("Проект {} достиг лимита задач ({})", project_id, max_tasks);
                    continue;
                }

                // Берём первую задачу из очереди
                if let Some(task) = queue.first() {
                    // Проверяем блокировки
                    let has_blocks = state.blocks
                        .get(&task.template_id)
                        .map(|s| s.available_permits() == 0)
                        .unwrap_or(false);
                    
                    if has_blocks {
                        debug!("Задача {} заблокирована", task.id);
                        continue;
                    }

                    result.push((task.clone(), project_id));
                }
            }
            
            result
        };

        // Обрабатываем задачи вне блокировки
        {
            let mut state = state.write().await;
            
            for (task, project_id) in tasks_to_start.drain(..) {
                // Удаляем из очереди
                if let Some(queue) = state.queue.get_mut(&project_id) {
                    if !queue.is_empty() {
                        queue.remove(0);
                    }
                }

                // Добавляем в running
                let running_task = RunningTask {
                    task: task.clone(),
                    project_id,
                    started_at: Utc::now(),
                    runner_id: None,
                };

                state.running.entry(project_id).or_insert_with(Vec::new);
                if let Some(running_list) = state.running.get_mut(&project_id) {
                    running_list.push(running_task);
                }

                // Отправляем событие запуска
                let _ = events.send(TaskPoolEvent::TaskCreated(task)).await;
            }
        }

        // Проверяем, пуста ли очередь
        let state = state.read().await;
        let all_empty = state.queue.values().all(|q| q.is_empty());
        if all_empty {
            drop(state);
            let _ = events.send(TaskPoolEvent::QueueEmpty).await;
        }
    }

    /// Получает количество задач в очереди
    pub async fn get_queue_size(&self) -> usize {
        let state = self.state.read().await;
        state.queue.values().map(|q| q.len()).sum()
    }

    /// Получает количество выполняемых задач
    pub async fn get_running_count(&self) -> usize {
        let state = self.state.read().await;
        state.running.values().map(|r| r.len()).sum()
    }

    /// Получает задачи проекта
    pub async fn get_project_tasks(&self, project_id: i32) -> Vec<Task> {
        let state = self.state.read().await;
        let mut tasks = Vec::new();
        
        if let Some(queue) = state.queue.get(&project_id) {
            tasks.extend(queue.clone());
        }
        
        if let Some(running) = state.running.get(&project_id) {
            tasks.extend(running.iter().map(|rt| rt.task.clone()));
        }
        
        tasks
    }

    /// Устанавливает блокировку для шаблона
    pub async fn set_block(&self, template_id: i32, permits: usize) {
        let mut state = self.state.write().await;
        state.blocks.insert(template_id, Arc::new(Semaphore::new(permits)));
    }

    /// Снимает блокировку
    pub async fn remove_block(&self, template_id: i32) {
        let mut state = self.state.write().await;
        state.blocks.remove(&template_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_pool_event_serialization() {
        let event = TaskPoolEvent::TaskCreated(Task {
            id: 1,
            template_id: 1,
            project_id: 1,
            status: TaskStatus::Waiting,
            playbook: None,
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: None,
            created: Utc::now(),
            start: None,
            end: None,
            message: None,
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            params: None,
        });
        
        match event {
            TaskPoolEvent::TaskCreated(t) => {
                assert_eq!(t.id, 1);
                assert_eq!(t.project_id, 1);
            }
            _ => panic!("Ожидалось событие TaskCreated"),
        }
    }

    #[test]
    fn test_running_task_creation() {
        let task = Task {
            id: 1,
            template_id: 1,
            project_id: 1,
            status: TaskStatus::Waiting,
            playbook: None,
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: None,
            created: Utc::now(),
            start: None,
            end: None,
            message: None,
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            params: None,
        };

        let running = RunningTask {
            task: task.clone(),
            project_id: 1,
            started_at: Utc::now(),
            runner_id: None,
        };

        assert_eq!(running.task.id, 1);
        assert_eq!(running.project_id, 1);
        assert!(running.runner_id.is_none());
    }

    #[test]
    fn test_task_log_record() {
        let record = TaskLogRecord {
            task_id: 1,
            output: "Test output".to_string(),
            time: Utc::now(),
        };

        assert_eq!(record.task_id, 1);
        assert_eq!(record.output, "Test output");
    }
}
