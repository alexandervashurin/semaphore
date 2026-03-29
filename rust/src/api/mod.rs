//! HTTP API на базе Axum
//!
//! Предоставляет REST API для управления Velum

pub mod apps;
pub mod auth;
pub mod mcp;
pub mod auth_ldap;
pub mod auth_local;
pub mod cache;
pub mod events;
pub mod extractors;
pub mod graphql;
pub mod handlers;
pub mod integration;
pub mod login;
pub mod middleware;
pub mod options;
pub mod routes;
pub mod runners;
pub mod state;
pub mod store_wrapper;
pub mod system_info;
pub mod user;
pub mod users;
pub mod websocket;

use axum::{Router, middleware as axum_middleware};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use std::sync::Arc;
use tracing::{info, warn};

use state::AppState;

// Ре-экспорт middleware
pub use middleware::{rate_limiter, security_headers};

/// Создаёт приложение Axum
pub fn create_app(store: Arc<dyn crate::db::Store + Send + Sync>) -> Router {
    let config = crate::config::Config::default();
    
    // Инициализация Redis cache для HA режима
    let cache = if config.ha.enable && !config.ha.redis.host.is_empty() {
        let redis_url = config.ha.redis_url();
        info!("HA mode enabled, connecting to Redis: {}", redis_url);
        
        match crate::cache::RedisCache::new(crate::cache::RedisConfig {
            url: redis_url,
            key_prefix: "velum:".to_string(),
            default_ttl_secs: 300,
            max_retries: 3,
            connection_timeout_secs: 5,
            enabled: true,
        }).initialize_sync() {
            Ok(cache) => {
                info!("Redis cache initialized successfully");
                Some(Arc::new(cache))
            }
            Err(e) => {
                warn!("Failed to initialize Redis cache: {}. HA features may not work.", e);
                None
            }
        }
    } else {
        info!("HA mode disabled or Redis not configured, running in single-node mode");
        None
    };

    let state = Arc::new(AppState::new(
        store,
        config,
        cache,
    ));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Auth роуты с жёстким rate limiting (5 попыток/мин per IP)
    let auth_routes = Router::new()
        .route("/api/auth/login", axum::routing::get(handlers::get_login_metadata).post(handlers::login))
        .route("/api/auth/logout", axum::routing::post(handlers::logout))
        .route("/api/auth/refresh", axum::routing::post(handlers::refresh_token))
        .layer(axum_middleware::from_fn_with_state(
            Arc::clone(&state),
            middleware::rate_limiter::app_auth_rate_limit,
        ));

    Router::new()
        // GraphQL API
        .merge(graphql::graphql_routes())
        // Auth с отдельным строгим rate limiter
        .merge(auth_routes)
        // Остальные API с мягким rate limiting (100 req/min per IP)
        .merge(routes::api_routes().layer(axum_middleware::from_fn_with_state(
            Arc::clone(&state),
            middleware::rate_limiter::app_api_rate_limit,
        )))
        // Static files с fallback
        .merge(routes::static_routes())
        // Middleware (порядок: последний layer применяется первым)
        .layer(axum_middleware::from_fn(middleware::security_headers))
        .layer(axum_middleware::from_fn(middleware::trace_id_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
