//! Тесты UserManager через MockStore

#[cfg(test)]
mod tests {
    use crate::db::mock::MockStore;
    use crate::db::store::{RetrieveQueryParams, UserManager};
    use crate::models::User;
    use chrono::Utc;

    fn create_test_user(id: i32, username: &str) -> User {
        User {
            id,
            name: username.to_string(),
            username: username.to_string(),
            email: format!("{}@test.com", username),
            password: "hashed_password".to_string(),
            admin: false,
            created: Utc::now(),
            alert: false,
            external: false,
            pro: false,
            email_otp: None,
            totp: None,
        }
    }

    #[tokio::test]
    async fn test_get_users_empty() {
        let store = MockStore::new();
        let users = store.get_users(RetrieveQueryParams::default()).await.unwrap();
        assert!(users.is_empty());
    }

    #[tokio::test]
    async fn test_create_user() {
        let store = MockStore::new();
        let user = create_test_user(0, "testuser");
        let result = store.create_user(user, "plain_password").await.unwrap();
        assert!(result.id > 0 || result.username == "testuser");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let store = MockStore::new();
        let result = store.get_user(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let store = MockStore::new();
        let user = create_test_user(1, "alice");
        store.create_user(user, "password").await.unwrap();

        let retrieved = store.get_user(1).await.unwrap();
        assert_eq!(retrieved.username, "alice");
        assert_eq!(retrieved.email, "alice@test.com");
    }

    #[tokio::test]
    async fn test_update_user() {
        let store = MockStore::new();
        let user = create_test_user(1, "bob");
        store.create_user(user, "password").await.unwrap();

        let mut updated = store.get_user(1).await.unwrap();
        updated.name = "Bobby".to_string();
        store.update_user(updated).await.unwrap();

        let retrieved = store.get_user(1).await.unwrap();
        assert_eq!(retrieved.name, "Bobby");
    }

    #[tokio::test]
    async fn test_user_admin_flag() {
        let store = MockStore::new();
        let mut admin_user = create_test_user(1, "admin");
        admin_user.admin = true;
        store.create_user(admin_user, "password").await.unwrap();

        let retrieved = store.get_user(1).await.unwrap();
        assert!(retrieved.admin);
    }

    #[tokio::test]
    async fn test_multiple_users() {
        let store = MockStore::new();
        store.create_user(create_test_user(1, "user1"), "password").await.unwrap();
        store.create_user(create_test_user(2, "user2"), "password").await.unwrap();
        store.create_user(create_test_user(3, "user3"), "password").await.unwrap();

        let users = store.get_users(RetrieveQueryParams::default()).await.unwrap();
        assert_eq!(users.len(), 3);
    }
}
