//! Маршруты пользователей
//!
//! Пользователи, текущий пользователь

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

/// Создаёт маршруты пользователей
pub fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Текущий пользователь
        .route("/api/user", get(handlers::get_current_user))
        // Пользователи
        .route("/api/users", get(handlers::get_users))
        .route("/api/users", post(handlers::create_user))
        .route("/api/users/{id}", get(handlers::get_user))
        .route("/api/users/{id}", put(handlers::update_user))
        .route("/api/users/{id}", delete(handlers::delete_user))
        .route(
            "/api/users/{id}/password",
            post(handlers::update_user_password),
        )
}
