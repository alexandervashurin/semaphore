//! Security Headers Middleware (упрощённая версия для axum 0.8)

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
    
    // X-Frame-Options
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    
    // X-Content-Type-Options
    headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
    
    // X-XSS-Protection
    headers.insert("X-XSS-Protection", HeaderValue::from_static("1; mode=block"));
    
    // Strict-Transport-Security (HSTS)
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    
    // Content-Security-Policy
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"),
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

/// Middleware для CORS
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
    }
}
