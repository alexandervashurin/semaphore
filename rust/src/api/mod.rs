//! HTTP API на базе Axum
//!
//! Предоставляет REST API для управления Semaphore

pub mod apps;
pub mod auth;
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

use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use std::sync::Arc;

use state::AppState;

// Ре-экспорт middleware
pub use middleware::{rate_limiter, security_headers};
use websocket::WebSocketManager;

/// Создаёт приложение Axum
pub fn create_app(store: Box<dyn crate::db::Store + Send + Sync>) -> Router {
    let ws_manager = Arc::new(WebSocketManager::new());

    let state = Arc::new(AppState::new(
        store,
        crate::config::Config::default(),
    ));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // GraphQL API
        .merge(graphql::graphql_routes())
        // API routes (должны быть перед static для корректной обработки)
        .merge(routes::api_routes())
        // Static files с fallback
        .merge(routes::static_routes())
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
