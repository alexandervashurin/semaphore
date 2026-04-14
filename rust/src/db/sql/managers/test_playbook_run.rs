//! Тесты PlaybookRunManager через MockStore

#[cfg(test)]
mod tests {
    use crate::db::mock::MockStore;
    use crate::db::store::PlaybookRunManager;
    use crate::models::playbook_run_history::{
        PlaybookRunCreate, PlaybookRunFilter, PlaybookRunStatus, PlaybookRunUpdate,
    };

    fn create_test_run_create(project_id: i32, playbook_id: i32) -> PlaybookRunCreate {
        PlaybookRunCreate {
            project_id,
            playbook_id,
            task_id: None,
            template_id: None,
            inventory_id: None,
            environment_id: None,
            extra_vars: None,
            limit_hosts: None,
            tags: None,
            skip_tags: None,
            user_id: None,
        }
    }

    #[tokio::test]
    async fn test_create_playbook_run() {
        let store = MockStore::new();
        let run = create_test_run_create(1, 1);
        let result = store.create_playbook_run(run).await.unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.project_id, 1);
        assert_eq!(result.playbook_id, 1);
        assert_eq!(result.status, PlaybookRunStatus::Waiting);
    }

    #[tokio::test]
    async fn test_get_playbook_run_found() {
        let store = MockStore::new();
        store
            .create_playbook_run(create_test_run_create(1, 1))
            .await
            .unwrap();
        let run = store.get_playbook_run(1, 1).await.unwrap();
        assert_eq!(run.id, 1);
    }

    #[tokio::test]
    async fn test_get_playbook_run_wrong_project() {
        let store = MockStore::new();
        store
            .create_playbook_run(create_test_run_create(1, 1))
            .await
            .unwrap();
        let result = store.get_playbook_run(1, 2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_playbook_runs_empty() {
        let store = MockStore::new();
        let runs = store
            .get_playbook_runs(PlaybookRunFilter::default())
            .await
            .unwrap();
        assert!(runs.is_empty());
    }

    #[tokio::test]
    async fn test_get_playbook_runs_filter_project() {
        let store = MockStore::new();
        store
            .create_playbook_run(create_test_run_create(1, 1))
            .await
            .unwrap();
        store
            .create_playbook_run(create_test_run_create(2, 1))
            .await
            .unwrap();

        let p1 = store
            .get_playbook_runs(PlaybookRunFilter {
                project_id: Some(1),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(p1.len(), 1);
        assert_eq!(p1[0].project_id, 1);
    }

    #[tokio::test]
    async fn test_get_playbook_runs_filter_status() {
        let store = MockStore::new();
        store
            .create_playbook_run(create_test_run_create(1, 1))
            .await
            .unwrap();
        store
            .update_playbook_run_status(1, PlaybookRunStatus::Success)
            .await
            .unwrap();

        let success = store
            .get_playbook_runs(PlaybookRunFilter {
                status: Some(PlaybookRunStatus::Success),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(success.len(), 1);

        let pending = store
            .get_playbook_runs(PlaybookRunFilter {
                status: Some(PlaybookRunStatus::Waiting),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(pending.is_empty());
    }

    #[tokio::test]
    async fn test_get_playbook_runs_pagination() {
        let store = MockStore::new();
        for _ in 0..5 {
            store
                .create_playbook_run(create_test_run_create(1, 1))
                .await
                .unwrap();
        }

        let all = store
            .get_playbook_runs(PlaybookRunFilter::default())
            .await
            .unwrap();
        assert_eq!(all.len(), 5);

        let page = store
            .get_playbook_runs(PlaybookRunFilter {
                limit: Some(2),
                offset: Some(2),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(page.len(), 2);
    }

    #[tokio::test]
    async fn test_update_playbook_run() {
        let store = MockStore::new();
        store
            .create_playbook_run(create_test_run_create(1, 1))
            .await
            .unwrap();

        let updated = store
            .update_playbook_run(
                1,
                1,
                PlaybookRunUpdate {
                    status: Some(PlaybookRunStatus::Running),
                    output: Some("output log".to_string()),
                    error_message: None,
                    start_time: None,
                    end_time: None,
                    duration_seconds: None,
                    hosts_total: None,
                    hosts_changed: None,
                    hosts_unreachable: None,
                    hosts_failed: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.status, PlaybookRunStatus::Running);
        assert_eq!(updated.output, Some("output log".to_string()));
    }

    #[tokio::test]
    async fn test_update_playbook_run_status() {
        let store = MockStore::new();
        store
            .create_playbook_run(create_test_run_create(1, 1))
            .await
            .unwrap();

        store
            .update_playbook_run_status(1, PlaybookRunStatus::Running)
            .await
            .unwrap();
        store
            .update_playbook_run_status(1, PlaybookRunStatus::Success)
            .await
            .unwrap();

        let run = store.get_playbook_run(1, 1).await.unwrap();
        assert_eq!(run.status, PlaybookRunStatus::Success);
    }

    #[tokio::test]
    async fn test_delete_playbook_run() {
        let store = MockStore::new();
        store
            .create_playbook_run(create_test_run_create(1, 1))
            .await
            .unwrap();
        store.delete_playbook_run(1, 1).await.unwrap();
        let result = store.get_playbook_run(1, 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_playbook_run_by_task_id() {
        let store = MockStore::new();
        store
            .create_playbook_run(create_test_run_create(1, 1))
            .await
            .unwrap();
        // task_id is None by default
        let found = store.get_playbook_run_by_task_id(999).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_get_playbook_run_stats() {
        let store = MockStore::new();
        store
            .create_playbook_run(create_test_run_create(1, 1))
            .await
            .unwrap();
        store
            .update_playbook_run_status(1, PlaybookRunStatus::Success)
            .await
            .unwrap();

        store
            .create_playbook_run(create_test_run_create(1, 1))
            .await
            .unwrap();
        store
            .update_playbook_run_status(2, PlaybookRunStatus::Failed)
            .await
            .unwrap();

        store
            .create_playbook_run(create_test_run_create(1, 2))
            .await
            .unwrap(); // different playbook

        let stats = store.get_playbook_run_stats(1).await.unwrap();
        assert_eq!(stats.total_runs, 2);
        assert_eq!(stats.success_runs, 1);
        assert_eq!(stats.failed_runs, 1);
    }
}
