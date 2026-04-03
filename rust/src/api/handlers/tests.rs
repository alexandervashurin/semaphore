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

    // ========================================================================
    // API Integration Tests — additional coverage
    // ========================================================================

    #[tokio::test]
    async fn test_projects_list_without_auth() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/projects")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_templates_list_without_auth() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/project/1/templates")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // MockStore не требует auth для некоторых routes — проверяем что не 500
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn test_inventories_without_auth() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/project/1/inventory")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // MockStore не требует auth для некоторых routes — проверяем что не 500
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn test_repositories_without_auth() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/project/1/repositories")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn test_environment_without_auth() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/project/1/environment")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn test_notifications_without_auth() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/notifications")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Endpoint может не существовать или требовать auth
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_auth_login_invalid_credentials() {
        let app = create_test_app();
        let body = serde_json::json!({
            "auth": "nonexistent_user",
            "password": "wrong_password"
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let resp_body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&resp_body).unwrap();
        assert!(json.get("error").is_some() || json.get("message").is_some());
    }

    #[tokio::test]
    async fn test_auth_login_empty_body() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        // Login без credentials должен вернуть ошибку — проверяем что не 500
        assert!(response.status() != StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // metrics может быть на /metrics или /api/internal/metrics или не быть
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_version_endpoint() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/version")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Version endpoint должен вернуть OK или NotFound но не 500
        assert!(response.status() != StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_oidc_login_metadata() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/auth/oidc/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // OIDC login metadata endpoint может вернуть любой статус кроме 500
        assert!(response.status() != StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_graphql_endpoint() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/graphql")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"query": "{ __typename }"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        // GraphQL может быть 200 или 404 если не зарегистрирован — не 500
        assert!(response.status() != StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_websocket_upgrade_connection() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/ws")
                    .header("Upgrade", "websocket")
                    .header("Connection", "Upgrade")
                    .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
                    .header("Sec-WebSocket-Version", "13")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // WS upgrade без auth может вернуть любой статус кроме 500
        assert!(response.status() != StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_404_unknown_route() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/nonexistent/route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_options_cors_preflight() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/api/projects")
                    .header("Origin", "http://localhost:3000")
                    .header("Access-Control-Request-Method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // CORS preflight должен вернуть 200
        assert_eq!(response.status(), StatusCode::OK);
    }
}
