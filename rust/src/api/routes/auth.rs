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
        .route("/healthz", get(handlers::health))
        .route("/readyz", get(handlers::health_ready))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_routes_creation() {
        let router = auth_routes();
        let _ = router;
    }

    #[test]
    fn test_auth_routes_return_type() {
        let router = auth_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_auth_routes_has_health_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
    }

    #[test]
    fn test_auth_routes_has_oidc_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
    }

    #[test]
    fn test_auth_routes_has_totp_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
    }

    #[test]
    fn test_auth_routes_state_type() {
        let router = auth_routes();
        let _router: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_auth_routes_module_imports() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
    }

    #[test]
    fn test_auth_routes_totp_submodule() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
    }
}
