//! Security Headers Middleware (упрощённая версия для axum 0.8)
//!
//! Добавляет security headers ко всем ответам:
//! - X-Frame-Options: DENY (защита от clickjacking)
//! - X-Content-Type-Options: nosniff (защита от MIME sniffing)
//! - X-XSS-Protection: 1; mode=block (XSS filter)
//! - Strict-Transport-Security: HSTS (HTTPS enforcement)
//! - Content-Security-Policy: CSP (источники контента)
//! - Referrer-Policy: strict-origin-when-cross-origin
//! - Permissions-Policy: отключение опасных функций
//! - Cache-Control: no-store для API

use axum::{
    http::{Request, HeaderValue},
    middleware::Next,
    response::Response,
    body::Body,
};

/// Middleware функция для добавления security headers
pub async fn security_headers(
    req: Request<Body>,
    next: Next,
) -> Response {
    let is_api = req.uri().path().starts_with("/api/");
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // X-Frame-Options (защита от clickjacking)
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));

    // X-Content-Type-Options (защита от MIME sniffing)
    headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));

    // X-XSS-Protection (XSS filter)
    headers.insert("X-XSS-Protection", HeaderValue::from_static("1; mode=block"));

    // Strict-Transport-Security (HSTS)
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Content-Security-Policy
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com https://cdnjs.cloudflare.com; img-src 'self' data: https:; font-src 'self' https://fonts.gstatic.com https://cdnjs.cloudflare.com; connect-src 'self' wss:; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"),
    );

    // Referrer-Policy
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions-Policy
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    // Cache-Control для API endpoints
    if is_api {
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("no-store, no-cache, must-revalidate"),
        );
        headers.insert("Pragma", HeaderValue::from_static("no-cache"));
        headers.insert("Expires", HeaderValue::from_static("0"));
    }

    response
}

/// Middleware для CORS (Cross-Origin Resource Sharing)
///
/// Разрешает запросы с любых доменов (для development)
/// Для production рекомендуется настроить конкретные домены
pub async fn cors_headers(
    req: Request<Body>,
    next: Next,
) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert(
        "Access-Control-Allow-Origin",
        HeaderValue::from_static("*"),
    );

    headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, PATCH, OPTIONS"),
    );

    headers.insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_static("Content-Type, Authorization, X-Requested-With"),
    );

    response
}

/// Строгий CORS middleware для production
///
/// Разрешает запросы только с указанных доменов
pub async fn strict_cors_headers(
    allowed_origins: &'static [&'static str],
    req: Request<Body>,
    next: Next,
) -> Response {
    // Сохраняем Origin до вызова next.run()
    let origin_value = req.headers()
        .get("Origin")
        .and_then(|h| h.to_str().ok())
        .filter(|origin_str| allowed_origins.contains(origin_str))
        .map(|s| s.to_string());

    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Установка Origin только если он в списке разрешённых
    if let Some(origin) = origin_value {
        headers.insert(
            "Access-Control-Allow-Origin",
            HeaderValue::from_str(&origin).unwrap(),
        );
    }

    headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, PATCH, OPTIONS"),
    );

    headers.insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_static("Content-Type, Authorization, X-Requested-With, X-RateLimit-Limit, X-RateLimit-Remaining"),
    );

    headers.insert(
        "Access-Control-Max-Age",
        HeaderValue::from_static("86400"), // 24 hours
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        routing::get,
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_security_headers() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(security_headers));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        assert!(headers.contains_key("X-Frame-Options"));
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("Strict-Transport-Security"));
        assert!(headers.contains_key("Content-Security-Policy"));
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(cors_headers));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let headers = response.headers();
        assert!(headers.contains_key("Access-Control-Allow-Origin"));
        assert!(headers.contains_key("Access-Control-Allow-Methods"));
        assert!(headers.contains_key("Access-Control-Allow-Headers"));
    }
}
