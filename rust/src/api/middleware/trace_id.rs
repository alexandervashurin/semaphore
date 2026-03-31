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
