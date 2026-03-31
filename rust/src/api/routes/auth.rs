//! Маршруты аутентификации
//!
//! Health checks, аутентификация, TOTP, OIDC

use crate::api::handlers;
use crate::api::handlers::totp;
use crate::api::state::AppState;
use axum::{routing::get, routing::post, Router};
use std::sync::Arc;

/// Создаёт маршруты аутентификации
pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Health checks
        .route("/api/health", get(handlers::health))
        .route("/api/health/live", get(handlers::health_live))
        .route("/api/health/ready", get(handlers::health_ready))
        .route("/api/health/full", get(handlers::health_full))
        // Аутентификация (login/logout/refresh определены в auth_routes с rate limiter)
        .route("/api/auth/verify", post(handlers::verify_session))
        .route("/api/auth/recovery", post(handlers::recovery_session))
        // OIDC
        .route("/api/auth/oidc/{provider}", get(handlers::oidc_login))
        .route(
            "/api/auth/oidc/{provider}/callback",
            get(handlers::oidc_callback),
        )
        // TOTP
        .route("/api/auth/totp/start", post(totp::start_totp_setup))
        .route("/api/auth/totp/confirm", post(totp::confirm_totp_setup))
        .route("/api/auth/totp/disable", post(totp::disable_totp))
}
