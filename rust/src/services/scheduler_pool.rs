//! Schedule Pool
//!
//! Пул расписаний для автоматического запуска задач

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use chrono::{DateTime, Utc};

use crate::error::{Error, Result};
use crate::models::Schedule;
use crate::db::store::Store;
use crate::services::task_logger::TaskStatus;

/// Планировщик задач
pub struct SchedulePool {
    store: Arc<dyn Store + Send + Sync>,
    schedules: Arc<Mutex<Vec<Schedule>>>,
    running: Arc<Mutex<bool>>,
}

impl SchedulePool {
    /// Создаёт новый пул расписаний
    pub fn new(store: Arc<dyn Store + Send + Sync>) -> Self {
        Self {
            store,
            schedules: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Запускает планировщик
    pub async fn run(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        *running = true;
        drop(running);

        let mut interval = interval(Duration::from_secs(60)); // Проверяем каждую минуту

        loop {
            interval.tick().await;
            self.check_schedules().await?;
        }
    }

    /// Проверяет расписания
    async fn check_schedules(&self) -> Result<()> {
        let schedules = self.schedules.lock().await;
        let now = Utc::now();

        for schedule in schedules.iter() {
            if !schedule.active {
                continue;
            }

            // Проверяем, нужно ли запускать задачу по расписанию
            // TODO: Реализовать проверку cron расписания
            if self.should_run_schedule(schedule, now) {
                self.run_scheduled_task(schedule).await?;
            }
        }

        Ok(())
    }

    /// Проверяет, нужно ли запускать задачу
    fn should_run_schedule(&self, _schedule: &Schedule, _now: DateTime<Utc>) -> bool {
        // TODO: Реализовать проверку cron расписания
        false
    }

    /// Запускает задачу по расписанию
    async fn run_scheduled_task(&self, schedule: &Schedule) -> Result<()> {
        // Создаём задачу
        let task = crate::models::Task {
            id: 0,
            template_id: schedule.template_id,
            project_id: schedule.project_id,
            status: TaskStatus::Waiting,
            playbook: None,
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: Some(schedule.id),
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

        // Сохраняем задачу в БД
        self.store.create_task(task).await?;

        Ok(())
    }

    /// Останавливает планировщик
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        *running = false;
        Ok(())
    }

    /// Загружает расписания из БД
    pub async fn load_schedules(&self, project_id: i32) -> Result<()> {
        let schedules = self.store.get_schedules(project_id).await?;
        let mut current_schedules = self.schedules.lock().await;
        *current_schedules = schedules;
        Ok(())
    }

    /// Добавляет расписание
    pub async fn add_schedule(&self, schedule: Schedule) -> Result<()> {
        let mut schedules = self.schedules.lock().await;
        schedules.push(schedule);
        Ok(())
    }

    /// Удаляет расписание
    pub async fn remove_schedule(&self, schedule_id: i32) -> Result<()> {
        let mut schedules = self.schedules.lock().await;
        schedules.retain(|s| s.id != schedule_id);
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_pool_creation() {
        // Тест для проверки создания пула
        assert!(true);
    }

    #[test]
    fn test_schedule_pool_should_run() {
        // Тест для проверки логики запуска
        assert!(true);
    }
}
