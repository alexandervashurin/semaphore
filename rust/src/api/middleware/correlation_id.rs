//! Correlation ID middleware
//!
//! Реализует сквозной X-Request-ID (correlation ID) для трассировки запросов
//! через сервисы. Стандартный паттерн для HTTP API.
//!
//! ## Поведение
//! - Читает `X-Request-ID` из входящего запроса (если пришёл от upstream/proxy)
//! - Принимает `X-Correlation-ID` как алиас (некоторые API Gateway используют его)
//! - Генерирует новый UUID v4 если заголовок отсутствует или содержит невалидные символы
//! - Добавляет `correlation_id` в текущий tracing span (виден во всех дочерних span'ах)
//! - Делает ID доступным из handlers через `Extension<CorrelationId>`
//! - Возвращает итоговый ID в ответе через `X-Request-ID` header
//!
//! ## Пример использования в handler
//! ```rust,no_run
//! use axum::Extension;
//! use crate::api::middleware::CorrelationId;
//!
//! async fn my_handler(Extension(corr_id): Extension<CorrelationId>) {
//!     tracing::info!(correlation_id = %corr_id, "handling request");
//! }
//! ```

use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use tracing::Span;
use uuid::Uuid;

/// Extension-тип для доступа к Correlation ID в handlers.
///
/// Добавляется в `req.extensions()` middleware'ом, извлекается через
/// `Extension<CorrelationId>` в axum handlers.
#[derive(Clone, Debug)]
pub struct CorrelationId(pub String);

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl CorrelationId {
    /// Возвращает строковое значение ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Middleware: X-Request-ID correlation ID
///
/// Применяется как `axum::middleware::from_fn(correlation_id_middleware)`.
/// Должен располагаться внешним слоем (до `TraceLayer`) чтобы correlation_id
/// был доступен в span'ах для всех обработчиков.
pub async fn correlation_id_middleware(mut req: Request<Body>, next: Next) -> Response {
    // Читаем X-Request-ID, затем X-Correlation-ID как fallback
    let incoming = req
        .headers()
        .get("x-request-id")
        .or_else(|| req.headers().get("x-correlation-id"))
        .and_then(|v| v.to_str().ok())
        .filter(|s| is_valid_correlation_id(s))
        .map(|s| s.to_string());

    let correlation_id = incoming.unwrap_or_else(|| Uuid::new_v4().to_string());

    // Добавляем в extensions для handlers
    req.extensions_mut()
        .insert(CorrelationId(correlation_id.clone()));

    // Записываем в текущий tracing span — propagates во все дочерние spans
    let span = Span::current();
    span.record("correlation_id", correlation_id.as_str());

    tracing::debug!(
        correlation_id = %correlation_id,
        method = %req.method(),
        path = %req.uri().path(),
        "request started"
    );

    let mut response = next.run(req).await;

    // Возвращаем correlation ID в ответе — клиент может использовать для логов
    if let Ok(val) = HeaderValue::from_str(&correlation_id) {
        response.headers_mut().insert("x-request-id", val);
    }

    tracing::debug!(
        correlation_id = %correlation_id,
        status = %response.status().as_u16(),
        "request completed"
    );

    response
}

/// Проверяет валидность значения correlation ID.
///
/// Принимаем только ASCII printable символы, длина 1..=128.
/// Отклоняем пустые строки и потенциально опасный ввод.
fn is_valid_correlation_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes().all(|b| b.is_ascii_graphic())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_display() {
        let id = CorrelationId("abc-123".to_string());
        assert_eq!(id.to_string(), "abc-123");
        assert_eq!(id.as_str(), "abc-123");
    }

    #[test]
    fn test_correlation_id_clone() {
        let id = CorrelationId("test-id".to_string());
        let cloned = id.clone();
        assert_eq!(id.0, cloned.0);
    }

    #[test]
    fn test_valid_correlation_id_uuid() {
        let uuid = Uuid::new_v4().to_string();
        assert!(is_valid_correlation_id(&uuid));
    }

    #[test]
    fn test_valid_correlation_id_custom() {
        assert!(is_valid_correlation_id("req-12345"));
        assert!(is_valid_correlation_id("trace/abc/123"));
        assert!(is_valid_correlation_id("A"));
    }

    #[test]
    fn test_invalid_correlation_id_empty() {
        assert!(!is_valid_correlation_id(""));
    }

    #[test]
    fn test_invalid_correlation_id_too_long() {
        let long = "a".repeat(129);
        assert!(!is_valid_correlation_id(&long));
    }

    #[test]
    fn test_invalid_correlation_id_non_ascii() {
        assert!(!is_valid_correlation_id("кириллица"));
    }

    #[test]
    fn test_invalid_correlation_id_control_chars() {
        assert!(!is_valid_correlation_id("id\twith\ttabs"));
        assert!(!is_valid_correlation_id("id with spaces"));
        assert!(!is_valid_correlation_id("id\nnewline"));
    }

    #[test]
    fn test_valid_correlation_id_max_length() {
        let max = "a".repeat(128);
        assert!(is_valid_correlation_id(&max));
    }
}
