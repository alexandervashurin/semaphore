//! HTTP API на базе Axum
//!
//! Предоставляет REST API для управления Semaphore

pub mod auth;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod state;
pub mod user;
pub mod users;
pub mod websocket;

use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use std::sync::Arc;

use state::AppState;
use websocket::WebSocketManager;

/// Создаёт приложение Axum
pub fn create_app(store: Box<dyn crate::db::Store + Send + Sync>) -> Router {
    let ws_manager = Arc::new(WebSocketManager::new());
    
    let state = Arc::new(AppState {
        store,
        config: crate::config::Config::default(),
        ws_manager,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // API routes
        .merge(routes::api_routes())
        // Static files
        .merge(routes::static_routes())
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
