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
        let run = store.get_playbook_run_by_task_id(task_id).await?;
        let run = match run {
            Some(r) => r,
            None => {
                // Нет связанного playbook run — это обычная задача, не через playbook
                info!("Task {} has no associated playbook run", task_id);
                return Ok(());
            }
        };

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

        store
            .update_playbook_run_status(run.id, playbook_status)
            .await?;

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
        let update = PlaybookRunUpdate {
            status: None,
            start_time: None,
            end_time: None,
            duration_seconds: None,
            hosts_total: Some(hosts_total),
            hosts_changed: Some(hosts_changed),
            hosts_unreachable: Some(hosts_unreachable),
            hosts_failed: Some(hosts_failed),
            output: None,
            error_message: None,
        };

        store
            .update_playbook_run(run_id, project_id, update)
            .await?;

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
    use crate::db::mock::MockStore;
    use crate::models::playbook_run_history::{PlaybookRun, PlaybookRunStatus};

    fn create_test_playbook_run(task_id: i32) -> PlaybookRun {
        PlaybookRun {
            id: 1,
            project_id: 1,
            playbook_id: 1,
            task_id: Some(task_id),
            template_id: Some(1),
            inventory_id: None,
            environment_id: None,
            extra_vars: None,
            limit_hosts: None,
            tags: None,
            skip_tags: None,
            user_id: Some(1),
            status: PlaybookRunStatus::Waiting,
            output: None,
            error_message: None,
            created: Utc::now(),
            updated: Utc::now(),
            start_time: None,
            end_time: None,
            duration_seconds: None,
            hosts_total: Some(0),
            hosts_changed: Some(0),
            hosts_unreachable: Some(0),
            hosts_failed: Some(0),
        }
    }

    #[tokio::test]
    async fn test_update_from_task_status_waiting() {
        let store = MockStore::new();
        let run = create_test_playbook_run(1);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_from_task_status(1, &TaskStatus::Waiting, &store)
            .await
            .unwrap();

        let updated = store.get_playbook_run_by_task_id(1).await.unwrap().unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Waiting);
    }

    #[tokio::test]
    async fn test_update_from_task_status_starting() {
        let store = MockStore::new();
        let run = create_test_playbook_run(2);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_from_task_status(2, &TaskStatus::Starting, &store)
            .await
            .unwrap();

        let updated = store.get_playbook_run_by_task_id(2).await.unwrap().unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Waiting);
    }

    #[tokio::test]
    async fn test_update_from_task_status_running() {
        let store = MockStore::new();
        let run = create_test_playbook_run(3);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_from_task_status(3, &TaskStatus::Running, &store)
            .await
            .unwrap();

        let updated = store.get_playbook_run_by_task_id(3).await.unwrap().unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Running);
    }

    #[tokio::test]
    async fn test_update_from_task_status_success() {
        let store = MockStore::new();
        let run = create_test_playbook_run(4);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_from_task_status(4, &TaskStatus::Success, &store)
            .await
            .unwrap();

        let updated = store.get_playbook_run_by_task_id(4).await.unwrap().unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Success);
    }

    #[tokio::test]
    async fn test_update_from_task_status_error() {
        let store = MockStore::new();
        let run = create_test_playbook_run(5);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_from_task_status(5, &TaskStatus::Error, &store)
            .await
            .unwrap();

        let updated = store.get_playbook_run_by_task_id(5).await.unwrap().unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Failed);
    }

    #[tokio::test]
    async fn test_update_from_task_status_stopped() {
        let store = MockStore::new();
        let run = create_test_playbook_run(6);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_from_task_status(6, &TaskStatus::Stopped, &store)
            .await
            .unwrap();

        let updated = store.get_playbook_run_by_task_id(6).await.unwrap().unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_update_from_task_status_confirmed() {
        let store = MockStore::new();
        let run = create_test_playbook_run(7);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_from_task_status(7, &TaskStatus::Confirmed, &store)
            .await
            .unwrap();

        let updated = store.get_playbook_run_by_task_id(7).await.unwrap().unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Running);
    }

    #[tokio::test]
    async fn test_update_from_task_status_rejected() {
        let store = MockStore::new();
        let run = create_test_playbook_run(8);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_from_task_status(8, &TaskStatus::Rejected, &store)
            .await
            .unwrap();

        let updated = store.get_playbook_run_by_task_id(8).await.unwrap().unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Failed);
    }

    #[tokio::test]
    async fn test_update_from_task_status_waiting_confirmation() {
        let store = MockStore::new();
        let run = create_test_playbook_run(9);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_from_task_status(9, &TaskStatus::WaitingConfirmation, &store)
            .await
            .unwrap();

        let updated = store.get_playbook_run_by_task_id(9).await.unwrap().unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Waiting);
    }

    #[tokio::test]
    async fn test_update_from_task_status_stopping() {
        let store = MockStore::new();
        let run = create_test_playbook_run(10);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_from_task_status(10, &TaskStatus::Stopping, &store)
            .await
            .unwrap();

        let updated = store.get_playbook_run_by_task_id(10).await.unwrap().unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Running);
    }

    #[tokio::test]
    async fn test_update_from_task_status_not_executed() {
        let store = MockStore::new();
        let run = create_test_playbook_run(11);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_from_task_status(11, &TaskStatus::NotExecuted, &store)
            .await
            .unwrap();

        let updated = store.get_playbook_run_by_task_id(11).await.unwrap().unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Waiting);
    }

    #[tokio::test]
    async fn test_update_from_task_status_no_run_found() {
        let store = MockStore::new();

        // Task без связанного playbook run — должно вернуть Ok(())
        let result = PlaybookRunStatusService::update_from_task_status(999, &TaskStatus::Running, &store)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_run_statistics() {
        let store = MockStore::new();
        let run = create_test_playbook_run(12);
        store.seed_playbook_run(run);

        PlaybookRunStatusService::update_run_statistics(
            1, 1, 10, 5, 1, 2, &store
        )
        .await
        .unwrap();

        let updated = store.get_playbook_run(1, 1).await.unwrap();
        assert_eq!(updated.hosts_total, Some(10));
        assert_eq!(updated.hosts_changed, Some(5));
        assert_eq!(updated.hosts_unreachable, Some(1));
        assert_eq!(updated.hosts_failed, Some(2));
    }

    #[tokio::test]
    async fn test_update_run_statistics_multiple_updates() {
        let store = MockStore::new();
        let run = create_test_playbook_run(13);
        store.seed_playbook_run(run);

        // Первое обновление
        PlaybookRunStatusService::update_run_statistics(
            1, 1, 5, 3, 0, 1, &store
        )
        .await
        .unwrap();

        // Второе обновление
        PlaybookRunStatusService::update_run_statistics(
            1, 1, 10, 7, 2, 3, &store
        )
        .await
        .unwrap();

        let updated = store.get_playbook_run(1, 1).await.unwrap();
        assert_eq!(updated.hosts_total, Some(10));
        assert_eq!(updated.hosts_changed, Some(7));
        assert_eq!(updated.hosts_unreachable, Some(2));
        assert_eq!(updated.hosts_failed, Some(3));
    }

    #[test]
    fn test_calculate_duration_negative() {
        let end = Utc::now();
        let start = end + chrono::Duration::seconds(30);

        let duration = PlaybookRunStatusService::calculate_duration(Some(start), Some(end));
        assert_eq!(duration, Some(-30));
    }

    #[test]
    fn test_calculate_duration_zero() {
        let t = Utc::now();
        let duration = PlaybookRunStatusService::calculate_duration(Some(t), Some(t));
        assert_eq!(duration, Some(0));
    }

    #[test]
    fn test_calculate_duration_start_only() {
        let duration = PlaybookRunStatusService::calculate_duration(Some(Utc::now()), None);
        assert_eq!(duration, None);
    }

    #[test]
    fn test_calculate_duration_end_only() {
        let duration = PlaybookRunStatusService::calculate_duration(None, Some(Utc::now()));
        assert_eq!(duration, None);
    }

    #[test]
    fn test_calculate_duration_both_none() {
        let duration = PlaybookRunStatusService::calculate_duration(None, None);
        assert_eq!(duration, None);
    }

    #[test]
    fn test_playbook_run_status_display() {
        assert_eq!(PlaybookRunStatus::Waiting.to_string(), "waiting");
        assert_eq!(PlaybookRunStatus::Running.to_string(), "running");
        assert_eq!(PlaybookRunStatus::Success.to_string(), "success");
        assert_eq!(PlaybookRunStatus::Failed.to_string(), "failed");
        assert_eq!(PlaybookRunStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_playbook_run_update_all_fields() {
        let update = PlaybookRunUpdate {
            status: Some(PlaybookRunStatus::Running),
            start_time: Some(Utc::now()),
            end_time: Some(Utc::now()),
            duration_seconds: Some(60),
            hosts_total: Some(10),
            hosts_changed: Some(5),
            hosts_unreachable: Some(1),
            hosts_failed: Some(2),
            output: Some("output".to_string()),
            error_message: Some("error".to_string()),
        };
        assert_eq!(update.status, Some(PlaybookRunStatus::Running));
        assert_eq!(update.duration_seconds, Some(60));
        assert_eq!(update.hosts_total, Some(10));
    }

    #[test]
    fn test_playbook_run_update_minimal() {
        let update = PlaybookRunUpdate {
            status: None,
            start_time: None,
            end_time: None,
            duration_seconds: None,
            hosts_total: None,
            hosts_changed: None,
            hosts_unreachable: None,
            hosts_failed: None,
            output: None,
            error_message: None,
        };
        assert!(update.status.is_none());
        assert!(update.hosts_total.is_none());
    }

    #[test]
    fn test_playbook_run_status_clone() {
        let status = PlaybookRunStatus::Failed;
        let cloned = status.clone();
        assert_eq!(cloned, status);
    }

    #[test]
    fn test_playbook_run_serialize() {
        let run = create_test_playbook_run(99);
        let json = serde_json::to_string(&run).unwrap();
        assert!(json.contains("99"));
        assert!(json.contains("project_id"));
        assert!(json.contains("task_id"));
    }

    #[test]
    fn test_playbook_run_update_all_fields_set() {
        let update = PlaybookRunUpdate {
            status: Some(PlaybookRunStatus::Success),
            start_time: Some(Utc::now()),
            end_time: Some(Utc::now()),
            duration_seconds: Some(120),
            hosts_total: Some(10),
            hosts_changed: Some(8),
            hosts_unreachable: Some(1),
            hosts_failed: Some(1),
            output: Some("Full output".to_string()),
            error_message: None,
        };
        assert_eq!(update.status, Some(PlaybookRunStatus::Success));
        assert_eq!(update.duration_seconds, Some(120));
        assert!(update.error_message.is_none());
    }

    #[test]
    fn test_calculate_duration_one_hour() {
        let now = Utc::now();
        let start = now - chrono::Duration::seconds(3600);
        let duration = PlaybookRunStatusService::calculate_duration(Some(start), Some(now));
        assert_eq!(duration, Some(3600));
    }

    #[test]
    fn test_calculate_duration_one_day() {
        let now = Utc::now();
        let start = now - chrono::Duration::seconds(86400);
        let duration = PlaybookRunStatusService::calculate_duration(Some(start), Some(now));
        assert_eq!(duration, Some(86400));
    }
}
