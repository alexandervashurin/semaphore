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
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};

/// Middleware функция для добавления security headers
pub async fn security_headers(req: Request<Body>, next: Next) -> Response {
    let is_api = req.uri().path().starts_with("/api/");
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // X-Frame-Options (защита от clickjacking)
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));

    // X-Content-Type-Options (защита от MIME sniffing)
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    // X-XSS-Protection (XSS filter)
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

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
pub async fn cors_headers(req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));

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
    let origin_value = req
        .headers()
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
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
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

    #[tokio::test]
    async fn test_security_headers_xframe_deny() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(security_headers));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let xframe = response.headers().get("X-Frame-Options").unwrap();
        assert_eq!(xframe, "DENY");
    }

    #[tokio::test]
    async fn test_security_headers_nosniff() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(security_headers));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let nosniff = response.headers().get("X-Content-Type-Options").unwrap();
        assert_eq!(nosniff, "nosniff");
    }

    #[tokio::test]
    async fn test_security_headers_xss_protection() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(security_headers));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let xss = response.headers().get("X-XSS-Protection").unwrap();
        assert_eq!(xss, "1; mode=block");
    }

    #[tokio::test]
    async fn test_security_headers_hsts() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(security_headers));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let hsts = response.headers().get("Strict-Transport-Security").unwrap();
        assert_eq!(hsts, "max-age=31536000; includeSubDomains");
    }

    #[tokio::test]
    async fn test_security_headers_csp() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(security_headers));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let csp = response.headers().get("Content-Security-Policy").unwrap();
        assert!(csp.to_str().unwrap().contains("default-src 'self'"));
        assert!(csp.to_str().unwrap().contains("frame-ancestors 'none'"));
    }

    #[tokio::test]
    async fn test_security_headers_referrer_policy() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(security_headers));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let rp = response.headers().get("Referrer-Policy").unwrap();
        assert_eq!(rp, "strict-origin-when-cross-origin");
    }

    #[tokio::test]
    async fn test_security_headers_permissions_policy() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(security_headers));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let pp = response.headers().get("Permissions-Policy").unwrap();
        assert!(pp.to_str().unwrap().contains("geolocation=()"));
        assert!(pp.to_str().unwrap().contains("microphone=()"));
        assert!(pp.to_str().unwrap().contains("camera=()"));
    }

    #[tokio::test]
    async fn test_security_headers_cache_control_for_api() {
        let app = Router::new()
            .route("/api/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(security_headers));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let headers = response.headers();
        assert!(headers.contains_key("Cache-Control"));
        let cc = headers.get("Cache-Control").unwrap();
        assert_eq!(cc, "no-store, no-cache, must-revalidate");
        assert_eq!(headers.get("Pragma").unwrap(), "no-cache");
        assert_eq!(headers.get("Expires").unwrap(), "0");
    }

    #[tokio::test]
    async fn test_security_headers_no_cache_for_non_api() {
        let app = Router::new()
            .route("/page", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(security_headers));

        let response = app
            .oneshot(Request::builder().uri("/page").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let headers = response.headers();
        assert!(!headers.contains_key("Cache-Control"));
        assert!(!headers.contains_key("Pragma"));
        assert!(!headers.contains_key("Expires"));
    }

    #[tokio::test]
    async fn test_strict_cors_allows_matching_origin() {
        const ORIGINS: &[&str] = &["https://example.com"];
        let app =
            Router::new()
                .route("/test", get(|| async { "OK" }))
                .layer(axum::middleware::from_fn(move |req, next| {
                    strict_cors_headers(ORIGINS, req, next)
                }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let headers = response.headers();
        assert_eq!(
            headers.get("Access-Control-Allow-Origin").unwrap(),
            "https://example.com"
        );
        assert!(headers.contains_key("Access-Control-Allow-Methods"));
        assert!(headers.contains_key("Access-Control-Allow-Headers"));
        assert_eq!(headers.get("Access-Control-Max-Age").unwrap(), "86400");
    }

    #[tokio::test]
    async fn test_strict_cors_blocks_non_matching_origin() {
        const ORIGINS: &[&str] = &["https://example.com"];
        let app =
            Router::new()
                .route("/test", get(|| async { "OK" }))
                .layer(axum::middleware::from_fn(move |req, next| {
                    strict_cors_headers(ORIGINS, req, next)
                }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://evil.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let headers = response.headers();
        assert!(!headers.contains_key("Access-Control-Allow-Origin"));
    }

    #[tokio::test]
    async fn test_strict_cors_no_origin_header() {
        const ORIGINS: &[&str] = &["https://example.com"];
        let app =
            Router::new()
                .route("/test", get(|| async { "OK" }))
                .layer(axum::middleware::from_fn(move |req, next| {
                    strict_cors_headers(ORIGINS, req, next)
                }));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert!(
            !response
                .headers()
                .contains_key("Access-Control-Allow-Origin")
        );
    }

    #[tokio::test]
    async fn test_cors_allows_all_methods() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(cors_headers));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let methods = response
            .headers()
            .get("Access-Control-Allow-Methods")
            .unwrap();
        assert_eq!(methods, "GET, POST, PUT, DELETE, PATCH, OPTIONS");
    }

    #[tokio::test]
    async fn test_cors_allows_correct_headers() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(cors_headers));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let allowed = response
            .headers()
            .get("Access-Control-Allow-Headers")
            .unwrap();
        assert_eq!(allowed, "Content-Type, Authorization, X-Requested-With");
    }

    #[tokio::test]
    async fn test_strict_cors_additional_headers() {
        const ORIGINS: &[&str] = &["https://allowed.com"];
        let app =
            Router::new()
                .route("/test", get(|| async { "OK" }))
                .layer(axum::middleware::from_fn(move |req, next| {
                    strict_cors_headers(ORIGINS, req, next)
                }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://allowed.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let allowed = response
            .headers()
            .get("Access-Control-Allow-Headers")
            .unwrap();
        assert!(allowed.to_str().unwrap().contains("X-RateLimit-Limit"));
        assert!(allowed.to_str().unwrap().contains("X-RateLimit-Remaining"));
    }
}
