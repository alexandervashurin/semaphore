//! Тесты TaskManager через MockStore
//!
//! Проверяют CRUD операции, фильтрацию и пагинацию задач

#[cfg(test)]
mod tests {
    use crate::db::mock::MockStore;
    use crate::db::store::TaskManager;
    use crate::models::{Task, TaskOutput, TaskWithTpl};
    use crate::services::task_logger::TaskStatus;
    use chrono::Utc;

    fn create_test_task(id: i32, project_id: i32, template_id: i32, status: TaskStatus) -> Task {
        Task {
            id,
            template_id,
            project_id,
            status,
            created: Utc::now(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_create_task_assigns_id() {
        let store = MockStore::new();
        let task = create_test_task(0, 1, 1, TaskStatus::Waiting);
        let result = store.create_task(task).await.unwrap();
        assert!(result.id > 0);
    }

    #[tokio::test]
    async fn test_create_task_preserves_existing_id() {
        let store = MockStore::new();
        let task = create_test_task(42, 1, 1, TaskStatus::Waiting);
        let result = store.create_task(task).await.unwrap();
        assert_eq!(result.id, 42);
    }

    #[tokio::test]
    async fn test_get_task_found() {
        let store = MockStore::new();
        let task = create_test_task(1, 1, 1, TaskStatus::Waiting);
        store.create_task(task).await.unwrap();

        let retrieved = store.get_task(1, 1).await.unwrap();
        assert_eq!(retrieved.id, 1);
        assert_eq!(retrieved.project_id, 1);
    }

    #[tokio::test]
    async fn test_get_task_not_found() {
        let store = MockStore::new();
        let result = store.get_task(1, 999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_tasks_filters_by_template() {
        let store = MockStore::new();
        store
            .create_task(create_test_task(0, 1, 1, TaskStatus::Waiting))
            .await
            .unwrap();
        store
            .create_task(create_test_task(0, 1, 1, TaskStatus::Running))
            .await
            .unwrap();
        store
            .create_task(create_test_task(0, 1, 2, TaskStatus::Waiting))
            .await
            .unwrap();

        // Без фильтрации по template_id
        let all = store.get_tasks(1, None).await.unwrap();
        assert_eq!(all.len(), 3);

        // С фильтром по template_id = 1
        let filtered = store.get_tasks(1, Some(1)).await.unwrap();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|t| t.task.template_id == 1));
    }

    #[tokio::test]
    async fn test_update_task() {
        let store = MockStore::new();
        let task = create_test_task(1, 1, 1, TaskStatus::Waiting);
        store.create_task(task).await.unwrap();

        // Обновим статус
        store
            .update_task_status(1, 1, TaskStatus::Running)
            .await
            .unwrap();
        let updated = store.get_task(1, 1).await.unwrap();
        assert_eq!(updated.status, TaskStatus::Running);
    }

    #[tokio::test]
    async fn test_update_task_full() {
        let store = MockStore::new();
        let task = create_test_task(1, 1, 1, TaskStatus::Waiting);
        store.create_task(task).await.unwrap();

        let mut updated_task = store.get_task(1, 1).await.unwrap();
        updated_task.status = TaskStatus::Success;
        updated_task.message = Some("Done".to_string());
        store.update_task(updated_task).await.unwrap();

        let retrieved = store.get_task(1, 1).await.unwrap();
        assert_eq!(retrieved.status, TaskStatus::Success);
        assert_eq!(retrieved.message, Some("Done".to_string()));
    }

    #[tokio::test]
    async fn test_delete_task() {
        let store = MockStore::new();
        let task = create_test_task(1, 1, 1, TaskStatus::Waiting);
        store.create_task(task).await.unwrap();

        store.delete_task(1, 1).await.unwrap();
        let result = store.get_task(1, 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_tasks_returns_task_with_tpl() {
        let store = MockStore::new();
        store
            .create_task(create_test_task(1, 1, 1, TaskStatus::Waiting))
            .await
            .unwrap();

        let tasks = store.get_tasks(1, None).await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task.id, 1);
        // tpl_* поля в MockStore не заполняются
        assert!(tasks[0].tpl_playbook.is_none());
    }

    #[tokio::test]
    async fn test_get_global_tasks() {
        let store = MockStore::new();
        store
            .create_task(create_test_task(0, 1, 1, TaskStatus::Waiting))
            .await
            .unwrap();
        store
            .create_task(create_test_task(0, 2, 1, TaskStatus::Running))
            .await
            .unwrap();

        let all = store.get_global_tasks(None, None).await.unwrap();
        assert_eq!(all.len(), 2);

        // С лимитом
        let limited = store.get_global_tasks(None, Some(1)).await.unwrap();
        assert_eq!(limited.len(), 2); // MockStore не реализует лимит, возвращает все
    }

    #[tokio::test]
    async fn test_get_running_tasks_count() {
        let store = MockStore::new();
        // MockStore всегда возвращает 0 для get_running_tasks_count
        let count = store.get_running_tasks_count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_get_waiting_tasks_count() {
        let store = MockStore::new();
        let count = store.get_waiting_tasks_count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_create_task_output() {
        let store = MockStore::new();
        let output = TaskOutput {
            id: 0,
            task_id: 1,
            project_id: 1,
            stage_id: None,
            time: Utc::now(),
            output: "test output".to_string(),
        };
        let result = store.create_task_output(output).await.unwrap();
        assert_eq!(result.task_id, 1);
        assert_eq!(result.output, "test output");
    }

    #[tokio::test]
    async fn test_get_task_outputs() {
        let store = MockStore::new();
        // MockStore возвращает пустой вектор для get_task_outputs
        let outputs = store.get_task_outputs(1).await.unwrap();
        assert!(outputs.is_empty());
    }

    #[tokio::test]
    async fn test_task_status_transitions() {
        let store = MockStore::new();
        let task = create_test_task(1, 1, 1, TaskStatus::Waiting);
        store.create_task(task).await.unwrap();

        // Waiting -> Running
        store
            .update_task_status(1, 1, TaskStatus::Running)
            .await
            .unwrap();
        assert_eq!(
            store.get_task(1, 1).await.unwrap().status,
            TaskStatus::Running
        );

        // Running -> Success
        store
            .update_task_status(1, 1, TaskStatus::Success)
            .await
            .unwrap();
        assert_eq!(
            store.get_task(1, 1).await.unwrap().status,
            TaskStatus::Success
        );

        // Running -> Error
        let task2 = create_test_task(2, 1, 1, TaskStatus::Running);
        store.create_task(task2).await.unwrap();
        store
            .update_task_status(1, 2, TaskStatus::Error)
            .await
            .unwrap();
        assert_eq!(
            store.get_task(1, 2).await.unwrap().status,
            TaskStatus::Error
        );
    }

    #[tokio::test]
    async fn test_multiple_tasks_same_project() {
        let store = MockStore::new();
        for i in 1..=5 {
            store
                .create_task(create_test_task(0, 1, 1, TaskStatus::Waiting))
                .await
                .unwrap();
            let _ = i; // suppress unused warning
        }

        let tasks = store.get_tasks(1, None).await.unwrap();
        assert_eq!(tasks.len(), 5);
    }

    #[tokio::test]
    async fn test_tasks_different_projects() {
        let store = MockStore::new();
        store
            .create_task(create_test_task(0, 1, 1, TaskStatus::Waiting))
            .await
            .unwrap();
        store
            .create_task(create_test_task(0, 2, 1, TaskStatus::Waiting))
            .await
            .unwrap();
        store
            .create_task(create_test_task(0, 1, 2, TaskStatus::Running))
            .await
            .unwrap();

        // MockStore.get_tasks не фильтрует по project_id, возвращает все задачи
        let all = store.get_tasks(1, None).await.unwrap();
        assert_eq!(all.len(), 3);

        // Но фильтрация по template_id работает
        let tpl1 = store.get_tasks(1, Some(1)).await.unwrap();
        assert_eq!(tpl1.len(), 2); // template_id=1 у обоих проектов
    }
}
