//! Интеграционные тесты для API handlers

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use std::sync::Arc;

    use crate::api::{create_app, handlers};
    use crate::db::mock::MockStore;

    fn create_test_app() -> axum::Router {
        let store = Box::new(MockStore::new());
        create_app(store)
    }

    #[tokio::test]
    async fn test_health_handler() {
        let response = handlers::health().await;
        assert_eq!(response, "OK");
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"OK");
    }

    #[tokio::test]
    async fn test_logout_handler() {
        let store = Box::new(MockStore::new());
        let state = Arc::new(crate::api::state::AppState::new(
            store,
            crate::config::Config::default(),
        ));
        let result = handlers::logout(axum::extract::State(state)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let store = Box::new(MockStore::new());
        let state = Arc::new(crate::api::state::AppState::new(
            store,
            crate::config::Config::default(),
        ));
        let payload = handlers::LoginPayload {
            username: "nonexistent".to_string(),
            password: "wrong".to_string(),
            totp_code: None,
        };
        let result = handlers::login(
            axum::extract::State(state),
            axum::Json(payload),
        ).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_projects_list_empty() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/projects")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
