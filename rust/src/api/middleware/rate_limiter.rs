//! Rate Limiting Middleware (упрощённая версия для axum 0.8)

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse, Json},
    body::Body,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// Конфигурация rate limiter
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u64,
    pub period_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            period_secs: 60,
        }
    }
}

#[derive(Debug, Clone)]
struct ClientInfo {
    requests: u64,
    period_start: Instant,
}

/// Rate Limiter
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(RateLimitConfig::default())
    }
    
    pub fn for_api() -> Self {
        Self::new(RateLimitConfig {
            max_requests: 100,
            period_secs: 60,
        })
    }
    
    pub fn for_auth() -> Self {
        Self::new(RateLimitConfig {
            max_requests: 5,
            period_secs: 60,
        })
    }
    
    pub async fn is_allowed(&self, key: &str) -> bool {
        let now = Instant::now();
        let period = Duration::from_secs(self.config.period_secs);
        
        let mut clients = self.clients.write().await;
        let client = clients.entry(key.to_string()).or_insert_with(|| ClientInfo {
            requests: 0,
            period_start: now,
        });
        
        // Проверка периода
        if now.duration_since(client.period_start) > period {
            client.requests = 0;
            client.period_start = now;
        }
        
        // Проверка лимита
        if client.requests >= self.config.max_requests {
            return false;
        }
        
        client.requests += 1;
        true
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
    req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_ip(&req);
    if !state.rate_limiter_api.is_allowed(&ip).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({"error": "Rate limit exceeded. Try again later."})),
        ).into_response();
    }
    next.run(req).await
}

/// Middleware для auth rate limiting через AppState
pub async fn app_auth_rate_limit(
    State(state): State<std::sync::Arc<crate::api::state::AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_ip(&req);
    if !state.rate_limiter_auth.is_allowed(&ip).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({"error": "Too many authentication attempts. Try again in 1 minute."})),
        ).into_response();
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 5,
            period_secs: 60,
        });
        
        for i in 0..5 {
            assert!(limiter.is_allowed("test").await, "Request {} should be allowed", i);
        }
        
        assert!(!limiter.is_allowed("test").await, "Request 6 should be blocked");
    }
}
