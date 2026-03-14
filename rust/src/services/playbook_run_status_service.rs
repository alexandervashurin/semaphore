//! Сервис обновления статуса запуска Playbook
//!
//! Этот модуль предоставляет функциональность для обновления
//! статуса и результатов выполнения playbook run.

use crate::db::store::*;
use crate::error::Result;
use crate::models::playbook_run_history::{PlaybookRunStatus, PlaybookRunUpdate};
use crate::services::task_logger::TaskStatus;
use chrono::Utc;
use tracing::info;

/// Сервис для обновления статуса playbook run
pub struct PlaybookRunStatusService;

impl PlaybookRunStatusService {
    /// Обновляет статус playbook run при изменении статуса задачи
    ///
    /// # Arguments
    /// * `task_id` - ID задачи
    /// * `new_status` - Новый статус задачи
    /// * `store` - Хранилище данных
    pub async fn update_from_task_status<S>(
        task_id: i32,
        new_status: &TaskStatus,
        store: &S,
    ) -> Result<()>
    where
        S: PlaybookRunManager,
    {
        // Находим playbook run по task_id
        // TODO: Нужен метод get_playbook_run_by_task_id
        
        // Маппинг статусов TaskStatus -> PlaybookRunStatus
        let playbook_status = match new_status {
            TaskStatus::Waiting => PlaybookRunStatus::Waiting,
            TaskStatus::Starting => PlaybookRunStatus::Waiting,
            TaskStatus::Running => PlaybookRunStatus::Running,
            TaskStatus::Success => PlaybookRunStatus::Success,
            TaskStatus::Error => PlaybookRunStatus::Failed,
            TaskStatus::Stopped => PlaybookRunStatus::Cancelled,
            TaskStatus::Confirmed => PlaybookRunStatus::Running,
            TaskStatus::Rejected => PlaybookRunStatus::Failed,
            TaskStatus::WaitingConfirmation => PlaybookRunStatus::Waiting,
            TaskStatus::Stopping => PlaybookRunStatus::Running,
            TaskStatus::NotExecuted => PlaybookRunStatus::Waiting,
        };

        // TODO: Обновить запись playbook_run
        // store.update_playbook_run(id, project_id, PlaybookRunUpdate {
        //     status: Some(playbook_status),
        //     end_time: if matches!(new_status, TaskStatus::Success | TaskStatus::Error | TaskStatus::Stopped) {
        //         Some(Utc::now())
        //     } else {
        //         None
        //     },
        //     ..Default::default()
        // }).await?;

        info!("Task {} status updated to {:?}", task_id, new_status);

        Ok(())
    }

    /// Обновляет статистику выполнения playbook run
    ///
    /// # Arguments
    /// * `run_id` - ID записи playbook_run
    /// * `project_id` - ID проекта
    /// * `hosts_total` - Всего хостов
    /// * `hosts_changed` - Изменено хостов
    /// * `hosts_unreachable` - Недоступных хостов
    /// * `hosts_failed` - Хостов с ошибками
    /// * `store` - Хранилище данных
    pub async fn update_run_statistics<S>(
        run_id: i32,
        project_id: i32,
        hosts_total: i32,
        hosts_changed: i32,
        hosts_unreachable: i32,
        hosts_failed: i32,
        store: &S,
    ) -> Result<()>
    where
        S: PlaybookRunManager,
    {
        // TODO: Реализовать обновление статистики
        info!(
            "Playbook run {} statistics updated: total={}, changed={}, unreachable={}, failed={}",
            run_id, hosts_total, hosts_changed, hosts_unreachable, hosts_failed
        );

        Ok(())
    }

    /// Вычисляет длительность выполнения в секундах
    pub fn calculate_duration(
        start_time: Option<chrono::DateTime<Utc>>,
        end_time: Option<chrono::DateTime<Utc>>,
    ) -> Option<i32> {
        match (start_time, end_time) {
            (Some(start), Some(end)) => {
                let duration = end.signed_duration_since(start);
                Some(duration.num_seconds() as i32)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_duration() {
        let start = Utc::now();
        let end = start + chrono::Duration::seconds(30);
        
        let duration = PlaybookRunStatusService::calculate_duration(Some(start), Some(end));
        assert_eq!(duration, Some(30));
    }

    #[test]
    fn test_calculate_duration_none() {
        let duration = PlaybookRunStatusService::calculate_duration(None, None);
        assert_eq!(duration, None);
    }
}
