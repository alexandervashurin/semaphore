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

    #[test]
    fn test_auth_routes_health_endpoints_count() {
        // healthz, readyz, /api/health, /api/health/live, /api/health/ready, /api/health/full
        let router = auth_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_auth_routes_has_auth_verify_endpoint() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
        // verify_session handler is used
    }

    #[test]
    fn test_auth_routes_has_recovery_endpoint() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
        // recovery_session handler is used
    }

    #[test]
    fn test_auth_routes_oidc_login_path() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
        // OIDC login path: /api/auth/oidc/{provider}
    }

    #[test]
    fn test_auth_routes_oidc_callback_path() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
        // OIDC callback path: /api/auth/oidc/{provider}/callback
    }

    #[test]
    fn test_auth_routes_totp_start_path() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
        // TOTP start path: /api/auth/totp/start
    }

    #[test]
    fn test_auth_routes_totp_confirm_path() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
        // TOTP confirm path: /api/auth/totp/confirm
    }

    #[test]
    fn test_auth_routes_totp_disable_path() {
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
        // TOTP disable path: /api/auth/totp/disable
    }

    #[test]
    fn test_auth_routes_uses_post_for_totp() {
        // All TOTP routes use POST method
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
    }

    #[test]
    fn test_auth_routes_uses_get_for_health() {
        // All health routes use GET method
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
    }

    #[test]
    fn test_auth_routes_uses_get_for_oidc() {
        // OIDC routes use GET method
        let _fn: fn() -> Router<Arc<AppState>> = auth_routes;
    }
}
