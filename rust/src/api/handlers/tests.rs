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
    use crate::db::store::Store;
    use crate::models::User;

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
    async fn test_healthz_endpoint() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readyz_endpoint() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_logout_handler() {
        let store: Arc<dyn crate::db::Store + Send + Sync> = Arc::new(MockStore::new());
        let state = Arc::new(crate::api::state::AppState::new(
            store,
            crate::config::Config::default(),
            None,
        ));
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/auth/logout")
            .body(axum::body::Body::empty())
            .unwrap();
        let result = handlers::logout(axum::extract::State(state), req).await;
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

    /// SEC-03: после logout тот же JWT не принимается (blacklist).
    #[tokio::test]
    async fn test_logout_then_same_token_rejected() {
        let pwd = "secret123";
        let hash = crate::api::auth_local::hash_password(pwd).unwrap();
        let user = User {
            id: 1,
            name: "Logout Test".to_string(),
            username: "logouttest".to_string(),
            email: "logouttest@example.com".to_string(),
            password: hash,
            admin: true,
            external: false,
            alert: false,
            pro: false,
            created: chrono::Utc::now(),
            totp: None,
            email_otp: None,
        };

        let store: std::sync::Arc<dyn crate::db::Store + Send + Sync> =
            std::sync::Arc::new(MockStore::new());
        store.create_user(user, pwd).await.unwrap();

        let app = create_app(store);

        let login_body = serde_json::json!({
            "username": "logouttest",
            "password": pwd
        });
        let login_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(login_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(login_resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(login_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let token = v["token"].as_str().expect("login returns token");

        let logout_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/logout")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(logout_resp.status(), StatusCode::OK);

        let projects_resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/projects")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(projects_resp.status(), StatusCode::UNAUTHORIZED);
        let err_body = axum::body::to_bytes(projects_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let err: serde_json::Value = serde_json::from_slice(&err_body).unwrap();
        assert_eq!(err["code"].as_str(), Some("TOKEN_REVOKED"));
    }
}
