//! Тесты OrganizationManager через MockStore

#[cfg(test)]
mod tests {
    use crate::db::mock::MockStore;
    use crate::db::store::OrganizationManager;
    use crate::models::organization::{OrganizationCreate, OrganizationUpdate, OrganizationUserCreate};

    fn create_org_create(name: &str) -> OrganizationCreate {
        OrganizationCreate {
            name: name.to_string(),
            slug: None,
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        }
    }

    #[tokio::test]
    async fn test_create_organization_auto_slug() {
        let store = MockStore::new();
        let org = store.create_organization(create_org_create("My Organization")).await.unwrap();
        assert_eq!(org.slug, "my-organization");
        assert!(org.active);
    }

    #[tokio::test]
    async fn test_create_organization_custom_slug() {
        let store = MockStore::new();
        let mut payload = create_org_create("Test Org");
        payload.slug = Some("custom-slug".to_string());
        let org = store.create_organization(payload).await.unwrap();
        assert_eq!(org.slug, "custom-slug");
    }

    #[tokio::test]
    async fn test_get_organization_found() {
        let store = MockStore::new();
        store.create_organization(create_org_create("Test")).await.unwrap();
        let org = store.get_organization(1).await.unwrap();
        assert_eq!(org.name, "Test");
    }

    #[tokio::test]
    async fn test_get_organization_not_found() {
        let store = MockStore::new();
        let result = store.get_organization(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_organization_by_slug() {
        let store = MockStore::new();
        store.create_organization(create_org_create("Test Org")).await.unwrap();
        let org = store.get_organization_by_slug("test-org").await.unwrap();
        assert_eq!(org.name, "Test Org");
    }

    #[tokio::test]
    async fn test_update_organization() {
        let store = MockStore::new();
        store.create_organization(create_org_create("Old Name")).await.unwrap();

        let updated = store.update_organization(1, OrganizationUpdate {
            name: Some("New Name".to_string()),
            description: Some("New description".to_string()),
            active: Some(false),
            ..Default::default()
        }).await.unwrap();
        assert_eq!(updated.name, "New Name");
        assert!(!updated.active);
        assert!(updated.updated.is_some());
    }

    #[tokio::test]
    async fn test_delete_organization() {
        let store = MockStore::new();
        store.create_organization(create_org_create("ToDelete")).await.unwrap();
        store.delete_organization(1).await.unwrap();
        let result = store.get_organization(1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_user_to_organization() {
        let store = MockStore::new();
        store.create_organization(create_org_create("Test")).await.unwrap();

        let ou = store.add_user_to_organization(OrganizationUserCreate {
            org_id: 1,
            user_id: 42,
            role: "member".to_string(),
        }).await.unwrap();
        assert_eq!(ou.org_id, 1);
        assert_eq!(ou.user_id, 42);
        assert_eq!(ou.role, "member");
    }

    #[tokio::test]
    async fn test_get_organization_users() {
        let store = MockStore::new();
        store.create_organization(create_org_create("Test")).await.unwrap();
        store.add_user_to_organization(OrganizationUserCreate { org_id: 1, user_id: 1, role: "member".to_string() }).await.unwrap();
        store.add_user_to_organization(OrganizationUserCreate { org_id: 1, user_id: 2, role: "admin".to_string() }).await.unwrap();

        let users = store.get_organization_users(1).await.unwrap();
        assert_eq!(users.len(), 2);
    }

    #[tokio::test]
    async fn test_remove_user_from_organization() {
        let store = MockStore::new();
        store.create_organization(create_org_create("Test")).await.unwrap();
        store.add_user_to_organization(OrganizationUserCreate { org_id: 1, user_id: 1, role: "member".to_string() }).await.unwrap();

        store.remove_user_from_organization(1, 1).await.unwrap();
        let users = store.get_organization_users(1).await.unwrap();
        assert!(users.is_empty());
    }

    #[tokio::test]
    async fn test_update_user_organization_role() {
        let store = MockStore::new();
        store.create_organization(create_org_create("Test")).await.unwrap();
        store.add_user_to_organization(OrganizationUserCreate { org_id: 1, user_id: 1, role: "member".to_string() }).await.unwrap();

        store.update_user_organization_role(1, 1, "admin").await.unwrap();
        let users = store.get_organization_users(1).await.unwrap();
        assert_eq!(users[0].role, "admin");
    }

    #[tokio::test]
    async fn test_get_user_organizations() {
        let store = MockStore::new();
        store.create_organization(create_org_create("Org1")).await.unwrap();
        store.create_organization(create_org_create("Org2")).await.unwrap();
        store.add_user_to_organization(OrganizationUserCreate { org_id: 1, user_id: 10, role: "member".to_string() }).await.unwrap();
        store.add_user_to_organization(OrganizationUserCreate { org_id: 2, user_id: 10, role: "member".to_string() }).await.unwrap();

        let orgs = store.get_user_organizations(10).await.unwrap();
        assert_eq!(orgs.len(), 2);
    }

    #[tokio::test]
    async fn test_check_quota_projects_under_limit() {
        let store = MockStore::new();
        let mut payload = create_org_create("Limited");
        payload.quota_max_projects = Some(5);
        store.create_organization(payload).await.unwrap();

        let ok = store.check_organization_quota(1, "projects").await.unwrap();
        assert!(ok); // 0 < 5
    }

    #[tokio::test]
    async fn test_check_quota_users_no_limit() {
        let store = MockStore::new();
        store.create_organization(create_org_create("NoQuota")).await.unwrap();
        let ok = store.check_organization_quota(1, "users").await.unwrap();
        assert!(ok); // No quota set
    }

    #[tokio::test]
    async fn test_check_quota_unknown_type() {
        let store = MockStore::new();
        store.create_organization(create_org_create("Test")).await.unwrap();
        let ok = store.check_organization_quota(1, "unknown").await.unwrap();
        assert!(ok);
    }

    #[tokio::test]
    async fn test_multiple_organizations() {
        let store = MockStore::new();
        for name in &["Alpha", "Beta", "Gamma"] {
            store.create_organization(create_org_create(name)).await.unwrap();
        }
        let orgs = store.get_organizations().await.unwrap();
        assert_eq!(orgs.len(), 3);
    }
}
