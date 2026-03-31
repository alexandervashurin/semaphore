//! Rate Limiting Middleware
//!
//! Поддерживает:
//! - Разные лимиты для разных endpoints (API, Auth, Sensitive)
//! - HTTP заголовки (X-RateLimit-*)
//! - Очистку старых записей (каждые 10 минут)
//! - Извлечение IP из X-Forwarded-For

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Конфигурация rate limiter
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u64,
    pub period_secs: u64,
    pub burst_size: Option<u64>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            period_secs: 60,
            burst_size: None,
        }
    }
}

#[derive(Debug, Clone)]
struct ClientInfo {
    requests: u64,
    period_start: Instant,
    last_request: Instant,
}

/// Rate Limiter с поддержкой очистки
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
    cleanup_interval: Duration,
    last_cleanup: Arc<RwLock<Instant>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let now = Instant::now();
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval: Duration::from_secs(600), // 10 минут
            last_cleanup: Arc::new(RwLock::new(now)),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(RateLimitConfig::default())
    }

    pub fn for_api() -> Self {
        Self::new(RateLimitConfig {
            max_requests: 100,
            period_secs: 60,
            burst_size: Some(20),
        })
    }

    pub fn for_auth() -> Self {
        Self::new(RateLimitConfig {
            max_requests: 5,
            period_secs: 60,
            burst_size: None,
        })
    }

    pub fn for_sensitive() -> Self {
        Self::new(RateLimitConfig {
            max_requests: 10,
            period_secs: 60,
            burst_size: None,
        })
    }

    async fn maybe_cleanup(&self) {
        let now = Instant::now();
        let mut last_cleanup_guard = self.last_cleanup.write().await;

        if now.duration_since(*last_cleanup_guard) > self.cleanup_interval {
            drop(last_cleanup_guard);

            let mut clients = self.clients.write().await;
            let period = Duration::from_secs(self.config.period_secs);
            let before = clients.len();

            clients.retain(|_, info| now.duration_since(info.period_start) < period);

            let removed = before - clients.len();
            if removed > 0 {
                debug!("RateLimiter: cleaned up {} stale entries", removed);
            }

            *self.last_cleanup.write().await = now;
        }
    }

    pub async fn is_allowed(&self, key: &str) -> bool {
        self.maybe_cleanup().await;

        let now = Instant::now();
        let period = Duration::from_secs(self.config.period_secs);

        let mut clients = self.clients.write().await;
        let client = clients
            .entry(key.to_string())
            .or_insert_with(|| ClientInfo {
                requests: 0,
                period_start: now,
                last_request: now,
            });

        // Проверка периода
        if now.duration_since(client.period_start) > period {
            client.requests = 0;
            client.period_start = now;
        }

        client.last_request = now;

        // Проверка лимита
        if client.requests >= self.config.max_requests {
            return false;
        }

        client.requests += 1;
        true
    }

    pub async fn get_remaining(&self, key: &str) -> u64 {
        let now = Instant::now();
        let period = Duration::from_secs(self.config.period_secs);

        let clients = self.clients.read().await;
        if let Some(client) = clients.get(key) {
            if now.duration_since(client.period_start) > period {
                return self.config.max_requests;
            }
            return self.config.max_requests.saturating_sub(client.requests);
        }
        self.config.max_requests
    }
}

/// State для middleware
#[derive(Clone)]
pub struct RateLimitState {
    pub limiter: Arc<RateLimiter>,
}

fn extract_ip<B>(req: &Request<B>) -> String {
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(ip) = value.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    if let Some(addr) = req.extensions().get::<std::net::SocketAddr>() {
        return addr.ip().to_string();
    }

    "unknown".to_string()
}

/// Middleware функция для API rate limiting
pub async fn api_rate_limit(
    State(state): State<RateLimitState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = extract_ip(&req);

    if !state.limiter.is_allowed(&ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}

/// Middleware функция для auth rate limiting
pub async fn auth_rate_limit(
    State(state): State<RateLimitState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = extract_ip(&req);

    if !state.limiter.is_allowed(&ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}

/// Middleware для API rate limiting через AppState
pub async fn app_api_rate_limit(
    State(state): State<std::sync::Arc<crate::api::state::AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_ip(&req);

    if !state.rate_limiter_api.is_allowed(&ip).await {
        let remaining = state.rate_limiter_api.get_remaining(&ip).await;
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "Rate limit exceeded. Try again later.",
                "retry_after": state.rate_limiter_api.config.period_secs
            })),
        )
            .into_response();

        let headers = response.headers_mut();
        headers.insert(
            "X-RateLimit-Limit",
            state
                .rate_limiter_api
                .config
                .max_requests
                .to_string()
                .parse()
                .unwrap(),
        );
        headers.insert(
            "X-RateLimit-Remaining",
            remaining.to_string().parse().unwrap(),
        );
        headers.insert(
            "Retry-After",
            state
                .rate_limiter_api
                .config
                .period_secs
                .to_string()
                .parse()
                .unwrap(),
        );

        return response;
    }

    let remaining = state.rate_limiter_api.get_remaining(&ip).await;
    let mut response = next.run(req).await;

    let headers = response.headers_mut();
    headers.insert(
        "X-RateLimit-Limit",
        state
            .rate_limiter_api
            .config
            .max_requests
            .to_string()
            .parse()
            .unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        remaining.to_string().parse().unwrap(),
    );

    response
}

/// Middleware для auth rate limiting через AppState
pub async fn app_auth_rate_limit(
    State(state): State<std::sync::Arc<crate::api::state::AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_ip(&req);

    if !state.rate_limiter_auth.is_allowed(&ip).await {
        let remaining = state.rate_limiter_auth.get_remaining(&ip).await;
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "Too many authentication attempts. Try again in 1 minute.",
                "retry_after": state.rate_limiter_auth.config.period_secs
            })),
        )
            .into_response();

        let headers = response.headers_mut();
        headers.insert(
            "X-RateLimit-Limit",
            state
                .rate_limiter_auth
                .config
                .max_requests
                .to_string()
                .parse()
                .unwrap(),
        );
        headers.insert(
            "X-RateLimit-Remaining",
            remaining.to_string().parse().unwrap(),
        );
        headers.insert(
            "Retry-After",
            state
                .rate_limiter_auth
                .config
                .period_secs
                .to_string()
                .parse()
                .unwrap(),
        );

        return response;
    }

    let remaining = state.rate_limiter_auth.get_remaining(&ip).await;
    let mut response = next.run(req).await;

    let headers = response.headers_mut();
    headers.insert(
        "X-RateLimit-Limit",
        state
            .rate_limiter_auth
            .config
            .max_requests
            .to_string()
            .parse()
            .unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        remaining.to_string().parse().unwrap(),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 5,
            period_secs: 60,
            burst_size: None,
        });

        for i in 0..5 {
            assert!(
                limiter.is_allowed("test").await,
                "Request {} should be allowed",
                i
            );
        }

        assert!(
            !limiter.is_allowed("test").await,
            "Request 6 should be blocked"
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_get_remaining() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 10,
            period_secs: 60,
            burst_size: None,
        });

        assert_eq!(limiter.get_remaining("test2").await, 10);

        limiter.is_allowed("test2").await;
        limiter.is_allowed("test2").await;
        limiter.is_allowed("test2").await;

        assert_eq!(limiter.get_remaining("test2").await, 7);
    }

    #[tokio::test]
    async fn test_rate_limiter_auth_strict() {
        let limiter = RateLimiter::for_auth();

        for i in 0..5 {
            assert!(
                limiter.is_allowed("auth_user").await,
                "Auth request {} should be allowed",
                i
            );
        }

        assert!(
            !limiter.is_allowed("auth_user").await,
            "6th auth request should be blocked"
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_sensitive() {
        let limiter = RateLimiter::for_sensitive();

        for i in 0..10 {
            assert!(
                limiter.is_allowed("sensitive_op").await,
                "Sensitive request {} should be allowed",
                i
            );
        }

        assert!(
            !limiter.is_allowed("sensitive_op").await,
            "11th sensitive request should be blocked"
        );
    }
}
