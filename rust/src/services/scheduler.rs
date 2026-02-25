//! Планировщик задач
//!
//! Предоставляет инфраструктуру для автоматического запуска задач по расписанию (cron).

use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use cron::Schedule as CronSchedule;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

use crate::error::{Error, Result};
use crate::models::Schedule;
use crate::db::store::Store;

/// Задача планировщика
#[derive(Debug, Clone)]
pub struct ScheduledJob {
    pub schedule_id: i32,
    pub template_id: i32,
    pub project_id: i32,
    pub cron: String,
    pub name: String,
    pub active: bool,
    pub next_run: Option<DateTime<Utc>>,
}

/// Менеджер пула планировщика
pub struct SchedulePool {
    store: Arc<dyn Store>,
    jobs: Arc<RwLock<HashMap<i32, ScheduledJob>>>,
    running: Arc<RwLock<bool>>,
}

impl SchedulePool {
    /// Создаёт новый пул планировщика
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self {
            store,
            jobs: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Запускает планировщик
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(Error::Other("Планировщик уже запущен".to_string()));
        }
        *running = true;
        drop(running);

        // Загружаем все активные расписания
        self.load_schedules().await?;

        // Запускаем фоновую задачу для проверки расписаний
        let jobs = self.jobs.clone();
        let running = self.running.clone();
        let store = self.store.clone();

        tokio::spawn(async move {
            while *running.read().await {
                Self::check_schedules(&jobs, &store).await;
                sleep(Duration::from_secs(10)).await;
            }
        });

        Ok(())
    }

    /// Останавливает планировщик
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        drop(running);

        // Очищаем все задачи
        let mut jobs = self.jobs.write().await;
        jobs.clear();

        Ok(())
    }

    /// Загружает все активные расписания из БД
    async fn load_schedules(&self) -> Result<()> {
        // Получаем все расписания (перебираем проекты)
        // TODO: Реализовать метод get_all_schedules() в Store
        let schedules = self.store.get_schedules(0).await?;
        
        let mut jobs = self.jobs.write().await;
        jobs.clear();

        for schedule in schedules {
            if schedule.active {
                if let Ok(next_run) = Self::calculate_next_run(&schedule.cron) {
                    jobs.insert(schedule.id, ScheduledJob {
                        schedule_id: schedule.id,
                        template_id: schedule.template_id,
                        project_id: schedule.project_id,
                        cron: schedule.cron.clone(),
                        name: schedule.name.clone(),
                        active: schedule.active,
                        next_run: Some(next_run),
                    });
                }
            }
        }

        Ok(())
    }

    /// Проверяет расписания и запускает задачи
    async fn check_schedules(
        jobs: &Arc<RwLock<HashMap<i32, ScheduledJob>>>,
        store: &Arc<dyn Store>,
    ) {
        let now = Utc::now();
        let mut jobs_to_run = Vec::new();

        {
            let mut jobs_write = jobs.write().await;
            for (id, job) in jobs_write.iter_mut() {
                if !job.active {
                    continue;
                }

                if let Some(next_run) = job.next_run {
                    if now >= next_run {
                        jobs_to_run.push((*id, job.template_id, job.project_id));
                        
                        // Обновляем следующее время запуска
                        if let Ok(new_next) = Self::calculate_next_run(&job.cron) {
                            job.next_run = Some(new_next);
                        }
                    }
                }
            }
        }

        // Запускаем задачи
        for (schedule_id, template_id, project_id) in jobs_to_run {
            if let Err(e) = Self::trigger_task(store, schedule_id, template_id, project_id).await {
                error!("Ошибка запуска задачи по расписанию {}: {}", schedule_id, e);
            }
        }
    }

    /// Запускает задачу
    async fn trigger_task(
        store: &Arc<dyn Store>,
        schedule_id: i32,
        template_id: i32,
        project_id: i32,
    ) -> Result<()> {
        // Создаём новую задачу
        let task = crate::models::Task {
            id: 0,
            template_id,
            project_id,
            status: crate::services::task_logger::TaskStatus::Waiting,
            playbook: None,
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: Some(schedule_id),
            created: Utc::now(),
            start: None,
            end: None,
            message: Some("Запущено по расписанию".to_string()),
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            params: None,
        };

        let created_task = store.create_task(task).await?;
        
        info!("Создана задача {} по расписанию {}", created_task.id, schedule_id);

        // Здесь должна быть логика отправки задачи в TaskPool
        // TODO: Интеграция с TaskPool

        Ok(())
    }

    /// Вычисляет следующее время запуска по cron выражению
    fn calculate_next_run(cron: &str) -> Result<DateTime<Utc>> {
        let schedule: CronSchedule = cron.parse()
            .map_err(|e| Error::Other(format!("Неверное cron выражение '{}': {}", cron, e)))?;
        
        let next = schedule.upcoming(Utc).next()
            .ok_or_else(|| Error::Other(format!("Не удалось вычислить следующее время для '{}'", cron)))?;

        Ok(next)
    }

    /// Добавляет расписание в пул
    pub async fn add_schedule(&self, schedule: Schedule) -> Result<()> {
        if !schedule.active {
            return Ok(());
        }

        let next_run = Self::calculate_next_run(&schedule.cron)?;
        
        let mut jobs = self.jobs.write().await;
        jobs.insert(schedule.id, ScheduledJob {
            schedule_id: schedule.id,
            template_id: schedule.template_id,
            project_id: schedule.project_id,
            cron: schedule.cron.clone(),
            name: schedule.name.clone(),
            active: schedule.active,
            next_run: Some(next_run),
        });

        Ok(())
    }

    /// Удаляет расписание из пула
    pub async fn remove_schedule(&self, schedule_id: i32) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.remove(&schedule_id);
        Ok(())
    }

    /// Обновляет расписание в пуле
    pub async fn update_schedule(&self, schedule: Schedule) -> Result<()> {
        self.remove_schedule(schedule.id).await?;
        self.add_schedule(schedule).await?;
        Ok(())
    }

    /// Получает все задачи
    pub async fn get_jobs(&self) -> Vec<ScheduledJob> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_parse_valid() {
        let result = SchedulePool::calculate_next_run("0 0 * * * *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_cron_parse_invalid() {
        let result = SchedulePool::calculate_next_run("invalid cron");
        assert!(result.is_err());
    }

    #[test]
    fn test_scheduled_job_creation() {
        let job = ScheduledJob {
            schedule_id: 1,
            template_id: 2,
            project_id: 3,
            cron: "0 0 * * * *".to_string(),
            name: "Test Job".to_string(),
            active: true,
            next_run: Some(Utc::now()),
        };

        assert_eq!(job.schedule_id, 1);
        assert!(job.active);
    }
}
