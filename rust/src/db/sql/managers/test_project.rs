//! Тесты ProjectManager через MockStore

#[cfg(test)]
mod tests {
    use crate::db::mock::MockStore;
    use crate::db::store::ProjectStore;
    use crate::models::Project;
    use chrono::Utc;

    fn create_test_project(id: i32, name: &str) -> Project {
        Project {
            id,
            name: name.to_string(),
            created: Utc::now(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 0,
            r#type: String::new(),
            default_secret_storage_id: None,
        }
    }

    #[tokio::test]
    async fn test_create_project() {
        let store = MockStore::new();
        let project = create_test_project(0, "test-project");
        let result = store.create_project(project).await.unwrap();
        assert!(result.id > 0 || result.name == "test-project");
    }

    #[tokio::test]
    async fn test_get_projects_empty() {
        let store = MockStore::new();
        let projects = store.get_projects(None).await.unwrap();
        assert!(projects.is_empty());
    }

    #[tokio::test]
    async fn test_get_project_not_found() {
        let store = MockStore::new();
        let result = store.get_project(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_and_get_project() {
        let store = MockStore::new();
        let project = create_test_project(1, "my-project");
        store.create_project(project).await.unwrap();

        let retrieved = store.get_project(1).await.unwrap();
        assert_eq!(retrieved.name, "my-project");
    }

    #[tokio::test]
    async fn test_update_project() {
        let store = MockStore::new();
        let project = create_test_project(1, "old-name");
        store.create_project(project).await.unwrap();

        let mut updated = store.get_project(1).await.unwrap();
        updated.name = "new-name".to_string();
        store.update_project(updated).await.unwrap();

        let retrieved = store.get_project(1).await.unwrap();
        assert_eq!(retrieved.name, "new-name");
    }

    #[tokio::test]
    async fn test_delete_project() {
        let store = MockStore::new();
        let project = create_test_project(1, "to-delete");
        store.create_project(project).await.unwrap();

        store.delete_project(1).await.unwrap();
        let result = store.get_project(1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_projects() {
        let store = MockStore::new();
        for i in 1..=5 {
            store.create_project(create_test_project(0, &format!("project-{}", i))).await.unwrap();
        }

        let projects = store.get_projects(None).await.unwrap();
        assert_eq!(projects.len(), 5);
    }

    #[tokio::test]
    async fn test_project_alert_flag() {
        let store = MockStore::new();
        let mut project = create_test_project(1, "with-alert");
        project.alert = true;
        project.alert_chat = Some("chat123".to_string());
        store.create_project(project).await.unwrap();

        let retrieved = store.get_project(1).await.unwrap();
        assert!(retrieved.alert);
        assert_eq!(retrieved.alert_chat, Some("chat123".to_string()));
    }
}
