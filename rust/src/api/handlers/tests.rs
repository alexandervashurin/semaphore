//! Интеграционные тесты для API handlers

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::IntoResponse,
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::api::{create_app, handlers};
    use crate::db::mock::MockStore;

    fn create_test_app() -> axum::Router {
        let store = Arc::new(MockStore::new());
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
        let store: Arc<dyn crate::db::Store + Send + Sync> = Arc::new(MockStore::new());
        let state = Arc::new(crate::api::state::AppState::new(
            store,
            crate::config::Config::default(),
            None,
        ));
        let result = handlers::logout(axum::extract::State(state)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let app = create_test_app();
        let body = serde_json::json!({
            "username": "nonexistent",
            "password": "wrong"
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_projects_list_requires_auth() {
        let app = create_test_app();

        // Проверяем что health endpoint работает (не требует авторизации)
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/projects")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Protected endpoint returns 401 without valid token
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
