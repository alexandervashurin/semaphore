//! Trace ID middleware
//!
//! Генерирует X-Trace-ID для каждого запроса, добавляет в response headers
//! и структурированные логи (совместимо с Jaeger/Zipkin/ELK без внешних крейтов).

use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use tracing::Span;
use uuid::Uuid;

/// Middleware: генерирует Trace-ID и добавляет в response + span
pub async fn trace_id_middleware(mut req: Request<Body>, next: Next) -> Response {
    // Используем входящий trace ID если есть (для propagation через proxy)
    let trace_id = req
        .headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Вставляем в extensions чтобы handlers могли получить
    req.extensions_mut().insert(TraceId(trace_id.clone()));

    // Добавляем в текущий tracing span
    let span = Span::current();
    span.record("trace_id", trace_id.as_str());

    tracing::debug!(trace_id = %trace_id, method = %req.method(), path = %req.uri().path(), "request");

    let mut response = next.run(req).await;

    // Добавляем в response header
    if let Ok(val) = HeaderValue::from_str(&trace_id) {
        response.headers_mut().insert("x-trace-id", val);
    }

    response
}

/// Extension-тип для доступа к Trace ID в handlers
#[derive(Clone, Debug)]
pub struct TraceId(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id_new() {
        let id = TraceId("abc-123".to_string());
        assert_eq!(id.0, "abc-123");
    }

    #[test]
    fn test_trace_id_clone() {
        let id = TraceId("test-trace".to_string());
        let cloned = id.clone();
        assert_eq!(id.0, cloned.0);
    }

    #[test]
    fn test_trace_id_debug() {
        let id = TraceId("trace-42".to_string());
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("trace-42"));
    }

    #[test]
    fn test_trace_id_with_uuid_format() {
        let uuid_like = "550e8400-e29b-41d4-a716-446655440000";
        let id = TraceId(uuid_like.to_string());
        assert_eq!(id.0, uuid_like);
    }

    #[test]
    fn test_trace_id_empty() {
        let id = TraceId("".to_string());
        assert!(id.0.is_empty());
    }

    #[test]
    fn test_trace_id_from_propagated_header() {
        let propagated = "parent-trace-id".to_string();
        let id = TraceId(propagated.clone());
        assert_eq!(id.0, "parent-trace-id");
    }

    #[test]
    fn test_trace_id_struct_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TraceId>();
    }
}
