//! Планировщик задач
//!
//! Предоставляет инфраструктуру для автоматического запуска задач по расписанию (cron).

use chrono::{DateTime, Utc};
use cron::Schedule as CronSchedule;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};
use tracing::{error, info};

use crate::db::store::Store;
use crate::error::{Error, Result};
use crate::models::Schedule;
use crate::services::task_execution;

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
    store: Arc<dyn Store + Send + Sync>,
    jobs: Arc<RwLock<HashMap<i32, ScheduledJob>>>,
    running: Arc<RwLock<bool>>,
}

impl SchedulePool {
    /// Создаёт новый пул планировщика
    pub fn new(store: Arc<dyn Store + Send + Sync>) -> Self {
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
        let schedules = self.store.get_all_schedules().await?;

        let mut jobs = self.jobs.write().await;
        jobs.clear();

        for schedule in schedules {
            if schedule.active {
                let run_at_only = schedule.cron_format.as_deref() == Some("run_at")
                    && schedule.cron.trim().is_empty();
                if run_at_only {
                    continue;
                }
                if let Ok(next_run) = Self::calculate_next_run(&schedule.cron) {
                    jobs.insert(
                        schedule.id,
                        ScheduledJob {
                            schedule_id: schedule.id,
                            template_id: schedule.template_id,
                            project_id: schedule.project_id,
                            cron: schedule.cron.clone(),
                            name: schedule.name.clone(),
                            active: schedule.active,
                            next_run: Some(next_run),
                        },
                    );
                }
            }
        }

        Ok(())
    }

    /// Проверяет расписания и запускает задачи
    async fn check_schedules(
        jobs: &Arc<RwLock<HashMap<i32, ScheduledJob>>>,
        store: &Arc<dyn Store + Send + Sync>,
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
        store: &Arc<dyn Store + Send + Sync>,
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
            repository_id: None,
            environment_id: None,
            params: None,
        };

        let created_task = store.create_task(task).await?;

        info!(
            "Создана задача {} по расписанию {}",
            created_task.id, schedule_id
        );

        // Запускаем задачу в фоновом потоке
        let store_clone = store.clone();
        tokio::spawn(async move {
            task_execution::execute_task(store_clone, created_task).await;
        });

        Ok(())
    }

    /// Нормализует cron: UI передаёт 5 полей (`мин час DOM M DOW`), библиотека ожидает секунды первым полем.
    fn normalize_cron_expression(cron: &str) -> String {
        let cron = cron.trim();
        if cron.is_empty() {
            return String::new();
        }
        if cron.split_whitespace().count() == 5 {
            format!("0 {}", cron)
        } else {
            cron.to_string()
        }
    }

    /// Вычисляет следующее время запуска по cron выражению
    fn calculate_next_run(cron: &str) -> Result<DateTime<Utc>> {
        let cron = cron.trim();
        if cron.is_empty() {
            return Err(Error::Other("Пустое cron выражение".to_string()));
        }

        let expr = Self::normalize_cron_expression(cron);
        let schedule: CronSchedule = expr
            .parse()
            .map_err(|e| Error::Other(format!("Неверное cron выражение '{}': {}", cron, e)))?;

        let next = schedule.upcoming(Utc).next().ok_or_else(|| {
            Error::Other(format!(
                "Не удалось вычислить следующее время для '{}'",
                cron
            ))
        })?;

        Ok(next)
    }

    /// Проверка cron перед сохранением в API (тот же пайплайн, что у планировщика).
    pub fn validate_cron_for_storage(cron: &str) -> Result<()> {
        Self::calculate_next_run(cron)?;
        Ok(())
    }

    /// Добавляет расписание в пул
    pub async fn add_schedule(&self, schedule: Schedule) -> Result<()> {
        if !schedule.active {
            return Ok(());
        }

        let run_at_only =
            schedule.cron_format.as_deref() == Some("run_at") && schedule.cron.trim().is_empty();
        if run_at_only {
            return Ok(());
        }

        let next_run = Self::calculate_next_run(&schedule.cron)?;

        let mut jobs = self.jobs.write().await;
        jobs.insert(
            schedule.id,
            ScheduledJob {
                schedule_id: schedule.id,
                template_id: schedule.template_id,
                project_id: schedule.project_id,
                cron: schedule.cron.clone(),
                name: schedule.name.clone(),
                active: schedule.active,
                next_run: Some(next_run),
            },
        );

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
    fn test_cron_parse_valid_five_fields_from_ui() {
        let result = SchedulePool::calculate_next_run("0 9 * * *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_cron_parse_invalid() {
        let result = SchedulePool::calculate_next_run("invalid cron");
        assert!(result.is_err());
    }

    #[test]
    fn test_cron_parse_rejects_empty() {
        let result = SchedulePool::calculate_next_run("  ");
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

    #[test]
    fn test_scheduled_job_inactive() {
        let job = ScheduledJob {
            schedule_id: 2,
            template_id: 5,
            project_id: 1,
            cron: "*/5 * * * *".to_string(),
            name: "Inactive Job".to_string(),
            active: false,
            next_run: None,
        };

        assert!(!job.active);
        assert!(job.next_run.is_none());
    }

    #[test]
    fn test_normalize_cron_expression_preserves_valid() {
        // 5-полевые cron из UI дополняются секундным полем
        assert_eq!(
            SchedulePool::normalize_cron_expression("0 * * * *"),
            "0 0 * * * *"
        );
        assert_eq!(
            SchedulePool::normalize_cron_expression("*/5 * * * *"),
            "0 */5 * * * *"
        );
        assert_eq!(
            SchedulePool::normalize_cron_expression("0 0 * * *"),
            "0 0 0 * * *"
        );
        assert_eq!(
            SchedulePool::normalize_cron_expression("0 12 * * 1-5"),
            "0 0 12 * * 1-5"
        );

        // 6-полевые (уже с секундами) не изменяются
        assert_eq!(
            SchedulePool::normalize_cron_expression("0 0 * * * *"),
            "0 0 * * * *"
        );
        assert_eq!(
            SchedulePool::normalize_cron_expression("30 0 * * * *"),
            "30 0 * * * *"
        );
    }

    #[test]
    fn test_calculate_next_run_valid_cron() {
        let result = SchedulePool::calculate_next_run("0 * * * *");
        assert!(result.is_ok());
        let next = result.unwrap();
        // Следующий запуск должен быть в будущем
        assert!(next > Utc::now());
    }

    #[test]
    fn test_calculate_next_run_invalid_cron() {
        let result = SchedulePool::calculate_next_run("invalid cron");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_cron_for_storage_rejects_invalid() {
        let result = SchedulePool::validate_cron_for_storage("not a cron");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_cron_for_storage_accepts_valid() {
        let result = SchedulePool::validate_cron_for_storage("0 * * * *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_schedule_validates_cron() {
        use crate::models::Schedule;

        // Создаём schedule с невалидным cron
        let schedule = Schedule {
            id: 1,
            project_id: 1,
            template_id: 1,
            cron: "not a cron".to_string(),
            cron_format: None,
            name: "Bad Schedule".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };

        // Проверяем что валидация cron работает
        let validation = SchedulePool::validate_cron_for_storage(&schedule.cron);
        assert!(validation.is_err());
    }

    #[test]
    fn test_normalize_cron_every_minute() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("* * * * *"),
            "0 * * * * *"
        );
    }

    #[test]
    fn test_normalize_cron_hourly() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("0 * * * *"),
            "0 0 * * * *"
        );
    }

    #[test]
    fn test_normalize_cron_daily() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("0 0 * * *"),
            "0 0 0 * * *"
        );
    }

    #[test]
    fn test_normalize_cron_weekday_range() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("30 8 * * 1-5"),
            "0 30 8 * * 1-5"
        );
    }

    #[test]
    fn test_calculate_next_run_specific_time() {
        // Каждые 5 минут
        let result = SchedulePool::calculate_next_run("*/5 * * * *");
        assert!(result.is_ok());
        let next = result.unwrap();
        assert!(next > Utc::now());
    }

    #[test]
    fn test_scheduled_job_properties() {
        let job = ScheduledJob {
            schedule_id: 10,
            template_id: 20,
            project_id: 30,
            cron: "0 */2 * * *".to_string(),
            name: "Every 2 hours".to_string(),
            active: true,
            next_run: Some(Utc::now()),
        };

        assert_eq!(job.schedule_id, 10);
        assert_eq!(job.template_id, 20);
        assert_eq!(job.project_id, 30);
        assert!(job.active);
        assert!(job.next_run.is_some());
    }

    #[test]
    fn test_schedule_structure() {
        use crate::models::Schedule;

        let schedule = Schedule {
            id: 1,
            project_id: 1,
            template_id: 1,
            cron: "0 9 * * 1-5".to_string(),
            cron_format: None,
            name: "Workday 9AM".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: Some(5),
            created: Some("2026-01-01T00:00:00Z".to_string()),
            run_at: None,
            delete_after_run: false,
        };

        assert_eq!(schedule.cron, "0 9 * * 1-5");
        assert!(schedule.active);
        assert_eq!(schedule.repository_id, Some(5));
    }

    #[tokio::test]
    async fn test_schedule_pool_new() {
        use crate::db::mock::MockStore;
        let store = Arc::new(MockStore::new());
        let pool = SchedulePool::new(store);

        // Проверим что pool создан
        let jobs = pool.get_jobs().await;
        assert!(jobs.is_empty());
    }

    #[tokio::test]
    async fn test_schedule_pool_start_stop() {
        use crate::db::mock::MockStore;
        let store = Arc::new(MockStore::new());
        let pool = SchedulePool::new(store);

        // Start
        let result = pool.start().await;
        assert!(result.is_ok());

        // Повторный start должен вернуть ошибку
        let result = pool.start().await;
        assert!(result.is_err());

        // Stop
        let result = pool.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_schedule_pool_add_remove_schedule() {
        use crate::db::mock::MockStore;
        use crate::models::Schedule;

        let store = Arc::new(MockStore::new());
        let pool = SchedulePool::new(store);
        pool.start().await.unwrap();

        let schedule = Schedule {
            id: 1,
            project_id: 1,
            template_id: 1,
            cron: "0 * * * *".to_string(),
            cron_format: None,
            name: "Test Schedule".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };

        // Add
        let result = pool.add_schedule(schedule).await;
        assert!(result.is_ok());

        // Check что задача добавлена
        let jobs = pool.get_jobs().await;
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "Test Schedule");

        // Remove
        let result = pool.remove_schedule(1).await;
        assert!(result.is_ok());

        let jobs = pool.get_jobs().await;
        assert!(jobs.is_empty());

        pool.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_schedule_pool_add_inactive_schedule() {
        use crate::db::mock::MockStore;
        use crate::models::Schedule;

        let store = Arc::new(MockStore::new());
        let pool = SchedulePool::new(store);
        pool.start().await.unwrap();

        let schedule = Schedule {
            id: 2,
            project_id: 1,
            template_id: 1,
            cron: "0 * * * *".to_string(),
            cron_format: None,
            name: "Inactive Schedule".to_string(),
            active: false,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };

        // Inactive schedule не должна быть добавлена
        let result = pool.add_schedule(schedule).await;
        assert!(result.is_ok());

        let jobs = pool.get_jobs().await;
        assert!(jobs.is_empty());

        pool.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_schedule_pool_add_run_at_only_schedule() {
        use crate::db::mock::MockStore;
        use crate::models::Schedule;

        let store = Arc::new(MockStore::new());
        let pool = SchedulePool::new(store);
        pool.start().await.unwrap();

        let schedule = Schedule {
            id: 3,
            project_id: 1,
            template_id: 1,
            cron: "".to_string(),
            cron_format: Some("run_at".to_string()),
            name: "Run At Schedule".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: Some("2026-05-01T00:00:00Z".to_string()),
            delete_after_run: false,
        };

        // run_at с пустым cron должна быть пропущена
        let result = pool.add_schedule(schedule).await;
        assert!(result.is_ok());

        let jobs = pool.get_jobs().await;
        assert!(jobs.is_empty());

        pool.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_schedule_pool_update_schedule() {
        use crate::db::mock::MockStore;
        use crate::models::Schedule;

        let store = Arc::new(MockStore::new());
        let pool = SchedulePool::new(store);
        pool.start().await.unwrap();

        let schedule = Schedule {
            id: 4,
            project_id: 1,
            template_id: 1,
            cron: "0 * * * *".to_string(),
            cron_format: None,
            name: "Original".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };

        pool.add_schedule(schedule).await.unwrap();

        // Обновим расписание
        let updated = Schedule {
            id: 4,
            project_id: 1,
            template_id: 1,
            cron: "0 0 * * *".to_string(),
            cron_format: None,
            name: "Updated".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };

        let result = pool.update_schedule(updated).await;
        assert!(result.is_ok());

        let jobs = pool.get_jobs().await;
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "Updated");
        assert_eq!(jobs[0].cron, "0 0 * * *");

        pool.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_schedule_pool_remove_schedule() {
        use crate::db::mock::MockStore;
        use crate::models::Schedule;

        let store = Arc::new(MockStore::new());
        let pool = SchedulePool::new(store);
        pool.start().await.unwrap();

        let schedule = Schedule {
            id: 10,
            project_id: 1,
            template_id: 1,
            cron: "0 * * * *".to_string(),
            cron_format: None,
            name: "Remove Me".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };

        pool.add_schedule(schedule).await.unwrap();
        assert_eq!(pool.get_jobs().await.len(), 1);

        // Remove the schedule
        pool.remove_schedule(10).await.unwrap();
        assert_eq!(pool.get_jobs().await.len(), 0);

        pool.stop().await.unwrap();
    }

    #[test]
    fn test_scheduled_job_clone() {
        let job = ScheduledJob {
            schedule_id: 1,
            template_id: 2,
            project_id: 3,
            cron: "0 * * * *".to_string(),
            name: "Clone Test".to_string(),
            active: true,
            next_run: Some(Utc::now()),
        };
        let cloned = job.clone();
        assert_eq!(cloned.schedule_id, job.schedule_id);
        assert_eq!(cloned.name, job.name);
    }

    #[tokio::test]
    async fn test_schedule_pool_new_empty_jobs() {
        use crate::db::mock::MockStore;
        let store = Arc::new(MockStore::new());
        let pool = SchedulePool::new(store);
        // New pool should have no jobs
        let jobs = pool.get_jobs().await;
        assert!(jobs.is_empty());
    }

    #[tokio::test]
    async fn test_schedule_pool_stop_idempotent() {
        use crate::db::mock::MockStore;
        let store = Arc::new(MockStore::new());
        let pool = SchedulePool::new(store);

        // Stop without start should be fine
        let result = pool.stop().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalize_cron_already_six_fields() {
        // 6-field cron should not be modified
        assert_eq!(
            SchedulePool::normalize_cron_expression("0 0 0 * * *"),
            "0 0 0 * * *"
        );
        assert_eq!(
            SchedulePool::normalize_cron_expression("30 12 1 1 * *"),
            "30 12 1 1 * *"
        );
    }

    #[test]
    fn test_calculate_next_run_every_minute() {
        let result = SchedulePool::calculate_next_run("* * * * *");
        assert!(result.is_ok());
        let next = result.unwrap();
        assert!(next > Utc::now());
    }

    #[test]
    fn test_calculate_next_run_invalid_returns_error() {
        let result = SchedulePool::calculate_next_run("not valid cron");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_cron_for_storage_rejects_empty() {
        let result = SchedulePool::validate_cron_for_storage("");
        assert!(result.is_err());
    }

    #[test]
    fn test_scheduled_job_inactive_flag() {
        let job = ScheduledJob {
            schedule_id: 1,
            template_id: 1,
            project_id: 1,
            cron: "0 * * * *".to_string(),
            name: "Inactive".to_string(),
            active: false,
            next_run: Some(Utc::now()),
        };
        assert!(!job.active);
        assert!(job.next_run.is_some());
    }

    #[test]
    fn test_scheduled_job_debug_format() {
        let job = ScheduledJob {
            schedule_id: 42,
            template_id: 10,
            project_id: 5,
            cron: "0 0 * * * *".to_string(),
            name: "Debug Test".to_string(),
            active: true,
            next_run: Some(Utc::now()),
        };
        // Проверяем что структура имеет Debug
        let debug_str = format!("{:?}", job);
        assert!(debug_str.contains("ScheduledJob"));
        assert!(debug_str.contains("Debug Test"));
    }

    #[test]
    fn test_normalize_cron_with_leading_whitespace() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("  * * * * *"),
            "0 * * * * *"
        );
    }

    #[test]
    fn test_normalize_cron_with_trailing_whitespace() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("0 0 * * *  "),
            "0 0 0 * * *"
        );
    }

    #[test]
    fn test_normalize_cron_preserves_six_fields() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("0 0 0 1 1 *"),
            "0 0 0 1 1 *"
        );
    }

    #[test]
    fn test_calculate_next_run_cron_with_range() {
        // Cron с диапазоном дней недели (пн-пт)
        let result = SchedulePool::calculate_next_run("0 9 * * 1-5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_next_run_cron_with_step() {
        // Cron с шагом каждые 2 часа
        let result = SchedulePool::calculate_next_run("0 */2 * * *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_next_run_cron_specific_day() {
        // Cron на 1-е число каждого месяца
        let result = SchedulePool::calculate_next_run("0 0 1 * *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalize_cron_every_second() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("* * * * * *"),
            "* * * * * *"
        );
    }

    #[test]
    fn test_scheduled_job_with_all_fields() {
        let now = Utc::now();
        let job = ScheduledJob {
            schedule_id: 100,
            template_id: 200,
            project_id: 300,
            cron: "0 0 1 1 *".to_string(),
            name: "New Year Job".to_string(),
            active: true,
            next_run: Some(now),
        };

        assert_eq!(job.schedule_id, 100);
        assert_eq!(job.template_id, 200);
        assert_eq!(job.project_id, 300);
        assert_eq!(job.cron, "0 0 1 1 *");
        assert_eq!(job.name, "New Year Job");
        assert!(job.active);
        assert!(job.next_run.is_some());
    }

    #[test]
    fn test_validate_cron_for_storage_valid_every_minute() {
        let result = SchedulePool::validate_cron_for_storage("* * * * *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_cron_for_storage_valid_complex() {
        let result = SchedulePool::validate_cron_for_storage("0 0 1,15 * *");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_schedule_pool_new_state() {
        use crate::db::mock::MockStore;
        let store = Arc::new(MockStore::new());
        let pool = SchedulePool::new(store);
        let jobs = pool.get_jobs().await;
        assert_eq!(jobs.len(), 0);
    }

    #[test]
    fn test_normalize_cron_month_range() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("0 0 1 1-6 *"),
            "0 0 0 1 1-6 *"
        );
    }

    #[test]
    fn test_normalize_cron_list_of_values() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("15 3,6,9,12,18,21 * * *"),
            "0 15 3,6,9,12,18,21 * * *"
        );
    }

    #[test]
    fn test_normalize_cron_every_15_minutes() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("*/15 * * * *"),
            "0 */15 * * * *"
        );
    }

    #[test]
    fn test_normalize_cron_quarterly() {
        assert_eq!(
            SchedulePool::normalize_cron_expression("0 0 1 1,4,7,10 *"),
            "0 0 0 1 1,4,7,10 *"
        );
    }

    #[test]
    fn test_calculate_next_run_midnight_daily() {
        let result = SchedulePool::calculate_next_run("0 0 * * *");
        assert!(result.is_ok());
        let next = result.unwrap();
        assert!(next > Utc::now());
    }

    #[test]
    fn test_calculate_next_run_weekends_only() {
        // Saturday at midnight
        let result = SchedulePool::calculate_next_run("0 0 * * 6");
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_next_run_first_of_year() {
        let result = SchedulePool::calculate_next_run("0 0 1 1 *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_next_run_every_hour_with_minutes() {
        let result = SchedulePool::calculate_next_run("30 * * * *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_cron_rejects_completely_invalid() {
        let result = SchedulePool::validate_cron_for_storage("zzz yyy xxx");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_cron_accepts_semi_colon_separated_list() {
        // Cron with list values
        let result = SchedulePool::validate_cron_for_storage("0 9,17 * * *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_scheduled_job_with_none_next_run() {
        let job = ScheduledJob {
            schedule_id: 5,
            template_id: 10,
            project_id: 15,
            cron: "invalid".to_string(),
            name: "Broken Job".to_string(),
            active: true,
            next_run: None,
        };
        assert!(job.next_run.is_none());
        assert!(job.active);
    }

    #[test]
    fn test_normalize_cron_no_whitespace_change() {
        let input = "  0  0  *  *  *  ";
        let normalized = SchedulePool::normalize_cron_expression(input);
        // normalize_cron_expression trims and prepends "0 " for 5-field cron
        assert_eq!(normalized, "0 0  0  *  *  *");
    }

    #[test]
    fn test_calculate_next_run_every_3_hours() {
        let result = SchedulePool::calculate_next_run("0 */3 * * *");
        assert!(result.is_ok());
        let next = result.unwrap();
        assert!(next > Utc::now());
    }
}
