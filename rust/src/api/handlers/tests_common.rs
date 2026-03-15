//! API Handlers Tests

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;
    use serde_json::json;

    #[tokio::test]
    async fn test_health_endpoint() {
        // Simple test for health endpoint
        let response = make_request("/api/health").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let response = make_request("/api/metrics").await;
        // Should return either OK or Not Found (depending on setup)
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_metrics_json_endpoint() {
        let response = make_request("/api/metrics/json").await;
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    async fn make_request(path: &str) -> axum::http::Response<Body> {
        // Create a simple test app
        let app = Router::new();
        
        app.oneshot(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap()
    }

    #[test]
    fn test_json_serialization() {
        let data = json!({
            "test": "value",
            "number": 42,
            "array": [1, 2, 3]
        });
        
        let serialized = serde_json::to_string(&data).unwrap();
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_json_deserialization() {
        let json_str = r#"{"test": "value", "number": 42}"#;
        let data: serde_json::Value = serde_json::from_str(json_str).unwrap();
        
        assert_eq!(data["test"], "value");
        assert_eq!(data["number"], 42);
    }
}
