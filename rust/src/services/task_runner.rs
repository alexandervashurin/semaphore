//! Исполнитель задач (TaskRunner)
//!
//! Основной компонент для выполнения задач.
//! Управляет выполнением Task, логированием, статусами.

use std::sync::Arc;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error, debug};

use crate::error::{Error, Result};
use crate::models::{Task, Template, Inventory, Repository, Environment, Event};
use crate::services::task_logger::{TaskStatus, TaskLogger};
use crate::db::store::Store;

/// Стадия выполнения задачи
#[derive(Debug, Clone, Copy)]
pub enum TaskStageType {
    Init,
    CloneRepository,
    InstallDependencies,
    Run,
    Cleanup,
}

/// Стадия задачи
#[derive(Debug, Clone)]
pub struct TaskStage {
    pub r#type: TaskStageType,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

/// Job - интерфейс для выполняемой работы
#[async_trait::async_trait]
pub trait Job: Send + Sync {
    /// Запускает задачу
    async fn run(&mut self) -> Result<()>;
    
    /// Останавливает задачу
    fn kill(&mut self);
    
    /// Проверяет, остановлена ли задача
    fn is_killed(&self) -> bool;
}

/// TaskRunner - исполнитель одной задачи
pub struct TaskRunner {
    /// Задача
    pub task: Task,
    /// Шаблон
    pub template: Template,
    /// Инвентарь
    pub inventory: Option<Inventory>,
    /// Репозиторий
    pub repository: Option<Repository>,
    /// Окружение
    pub environment: Option<Environment>,
    
    /// Текущая стадия
    current_stage: Option<TaskStage>,
    
    /// Пул задач
    pool: Option<Arc<super::task_pool::TaskPool>>,
    
    /// Хранилище
    store: Arc<dyn Store>,
    
    /// ID раннера
    pub runner_id: Option<i32>,
    
    /// Имя пользователя
    pub username: String,
    
    /// Job для выполнения
    job: Option<Box<dyn Job>>,
    
    /// Флаг отправки уведомлений
    alert: bool,
    
    /// Alert chat
    alert_chat: Option<String>,
    
    /// Список пользователей для уведомлений
    users: Vec<i32>,
}

impl TaskRunner {
    /// Создаёт новый TaskRunner
    pub fn new(
        task: Task,
        template: Template,
        store: Arc<dyn Store>,
        username: String,
    ) -> Self {
        Self {
            task,
            template,
            inventory: None,
            repository: None,
            environment: None,
            current_stage: None,
            pool: None,
            store,
            runner_id: None,
            username,
            job: None,
            alert: false,
            alert_chat: None,
            users: Vec::new(),
        }
    }

    /// Загружает детали задачи (инвентарь, репозиторий, окружение)
    pub async fn load_details(&mut self) -> Result<()> {
        debug!("Загрузка деталей для задачи {}", self.task.id);
        
        // Загружаем инвентарь
        if let Some(inventory_id) = self.task.inventory_id {
            let inventory = self.store.get_inventory(self.task.project_id, inventory_id).await?;
            self.inventory = Some(inventory);
        }
        
        // Загружаем репозиторий
        let repo = self.store.get_repository(self.task.project_id, self.template.repository_id).await?;
        self.repository = Some(repo);
        
        // Загружаем окружение
        if self.template.environment_id > 0 {
            let env = self.store.get_environment(self.task.project_id, self.template.environment_id).await?;
            self.environment = Some(env);
        }
        
        // Загружаем пользователей для уведомлений
        self.users = self.load_notification_users().await?;
        
        Ok(())
    }

    /// Загружает список пользователей для уведомлений
    async fn load_notification_users(&self) -> Result<Vec<i32>> {
        // Получаем администраторов проекта
        let admins = self.store.get_all_admins().await?;
        Ok(admins.iter().map(|u| u.id).collect())
    }

    /// Запускает задачу
    pub async fn run(&mut self) -> Result<()> {
        info!("Запуск задачи {}", self.task.id);
        
        // Устанавливаем статус Running
        self.set_status(TaskStatus::Running).await?;
        
        // Загружаем детали
        self.load_details().await?;
        
        // Начинаем стадию инициализации
        self.begin_stage(TaskStageType::Init).await?;
        
        // Создаём Job для выполнения
        self.job = Some(self.create_job()?);
        
        // Запускаем Job
        if let Some(ref mut job) = self.job {
            match job.run().await {
                Ok(_) => {
                    self.set_status(TaskStatus::Success).await?;
                    info!("Задача {} успешно завершена", self.task.id);
                }
                Err(e) => {
                    error!("Ошибка выполнения задачи {}: {}", self.task.id, e);
                    self.set_status(TaskStatus::Error).await?;
                }
            }
        }
        
        // Завершаем стадию
        self.end_stage().await?;
        
        // Создаём событие
        self.create_task_event().await?;
        
        // Отправляем уведомления если нужно
        if self.alert {
            self.send_alerts().await?;
        }
        
        Ok(())
    }

    /// Создаёт Job для выполнения задачи
    fn create_job(&self) -> Result<Box<dyn Job>> {
        // TODO: Создать соответствующий Job на основе типа шаблона
        // Ansible -> AnsibleJob
        // Terraform -> TerraformJob
        // Shell -> ShellJob
        
        // Временно возвращаем заглушку
        Err(Error::Other("Job not implemented".to_string()))
    }

    /// Устанавливает статус задачи
    pub async fn set_status(&mut self, status: TaskStatus) -> Result<()> {
        let old_status = self.task.status.clone();
        self.task.status = status.clone();
        
        // Обновляем время
        match status {
            TaskStatus::Running => {
                self.task.start = Some(Utc::now());
            }
            TaskStatus::Success | TaskStatus::Error | TaskStatus::Stopped => {
                self.task.end = Some(Utc::now());
            }
            _ => {}
        }
        
        // Сохраняем статус
        self.save_status().await?;
        
        // Уведомляем слушателей
        self.notify_status_listeners(status, old_status).await?;
        
        Ok(())
    }

    /// Сохраняет статус задачи в БД
    async fn save_status(&self) -> Result<()> {
        self.store.update_task(self.task.clone()).await?;
        
        // Отправляем WebSocket уведомление
        // TODO: Интеграция с WebSocket
        debug!("Статус задачи {} сохранён: {:?}", self.task.id, self.task.status);
        
        Ok(())
    }

    /// Уведомляет слушателей статуса
    async fn notify_status_listeners(
        &self,
        new_status: TaskStatus,
        old_status: TaskStatus,
    ) -> Result<()> {
        // TODO: Реализовать уведомление слушателей
        debug!("Смена статуса: {:?} -> {:?}", old_status, new_status);
        Ok(())
    }

    /// Начинает новую стадию
    async fn begin_stage(&mut self, stage_type: TaskStageType) -> Result<()> {
        debug!("Начата стадия {:?}", stage_type);
        
        self.current_stage = Some(TaskStage {
            r#type: stage_type,
            start: Utc::now(),
            end: None,
        });
        
        Ok(())
    }

    /// Завершает текущую стадию
    async fn end_stage(&mut self) -> Result<()> {
        if let Some(ref mut stage) = self.current_stage {
            stage.end = Some(Utc::now());
            debug!("Завершена стадия {:?}", stage.r#type);
        }
        
        // TODO: Сохранить стадию в БД
        self.current_stage = None;
        Ok(())
    }

    /// Создаёт событие задачи
    async fn create_task_event(&self) -> Result<()> {
        let desc = format!(
            "Task ID {} ({}) finished with status {}",
            self.task.id,
            self.template.name,
            self.task.status
        );
        
        let event = Event {
            id: 0,
            user_id: self.task.user_id,
            project_id: Some(self.task.project_id),
            object_id: Some(self.task.id),
            object_type: "task".to_string(),
            description: desc,
            created: Utc::now(),
        };
        
        self.store.create_event(event).await?;
        
        Ok(())
    }

    /// Отправляет уведомления
    async fn send_alerts(&self) -> Result<()> {
        // TODO: Отправка email/webhook уведомлений
        warn!("Send alerts not implemented");
        Ok(())
    }

    /// Останавливает задачу
    pub async fn kill(&mut self) -> Result<()> {
        if let Some(ref mut job) = self.job {
            job.kill();
            self.set_status(TaskStatus::Stopped).await?;
        }
        Ok(())
    }

    /// Проверяет, остановлена ли задача
    pub fn is_killed(&self) -> bool {
        self.job.as_ref().map(|j| j.is_killed()).unwrap_or(false)
    }

    /// Устанавливает пул задач
    pub fn set_pool(&mut self, pool: Arc<super::task_pool::TaskPool>) {
        self.pool = Some(pool);
    }

    /// Устанавливает ID раннера
    pub fn set_runner_id(&mut self, runner_id: i32) {
        self.runner_id = Some(runner_id);
    }

    /// Включает уведомления
    pub fn set_alert(&mut self, alert: bool, alert_chat: Option<String>) {
        self.alert = alert;
        self.alert_chat = alert_chat;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_stage_creation() {
        let stage = TaskStage {
            r#type: TaskStageType::Init,
            start: Utc::now(),
            end: None,
        };

        assert!(matches!(stage.r#type, TaskStageType::Init));
        assert!(stage.end.is_none());
    }

    #[test]
    fn test_task_stage_completion() {
        let mut stage = TaskStage {
            r#type: TaskStageType::Run,
            start: Utc::now(),
            end: None,
        };

        stage.end = Some(Utc::now());
        assert!(stage.end.is_some());
    }
}
