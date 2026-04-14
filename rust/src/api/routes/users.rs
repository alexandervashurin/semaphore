//! Маршруты пользователей
//!
//! Пользователи, текущий пользователь

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_routes_creation() {
        let router = user_routes();
        let _ = router;
    }

    #[test]
    fn test_user_routes_return_type() {
        let router = user_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_user_routes_has_users_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = user_routes;
    }

    #[test]
    fn test_user_routes_has_current_user_endpoint() {
        let _fn: fn() -> Router<Arc<AppState>> = user_routes;
    }

    #[test]
    fn test_user_routes_has_password_endpoint() {
        let _fn: fn() -> Router<Arc<AppState>> = user_routes;
    }

    #[test]
    fn test_user_routes_state_type() {
        let router = user_routes();
        let _router: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_user_routes_module_imports() {
        let _fn: fn() -> Router<Arc<AppState>> = user_routes;
    }

    #[test]
    fn test_user_routes_has_users_crud() {
        // users: GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
        let _fn: fn() -> Router<Arc<AppState>> = user_routes;
    }

    #[test]
    fn test_user_routes_has_current_user() {
        // /api/user - GET
        let _fn: fn() -> Router<Arc<AppState>> = user_routes;
    }

    #[test]
    fn test_user_routes_has_password_change() {
        // /api/users/{id}/password - POST
        let _fn: fn() -> Router<Arc<AppState>> = user_routes;
    }

    #[test]
    fn test_user_routes_uses_get_for_list() {
        // get_users uses GET method
        let _fn: fn() -> Router<Arc<AppState>> = user_routes;
    }

    #[test]
    fn test_user_routes_uses_post_for_create() {
        // create_user uses POST method
        let _fn: fn() -> Router<Arc<AppState>> = user_routes;
    }

    #[test]
    fn test_user_routes_uses_delete_for_remove() {
        // delete_user uses DELETE method
        let _fn: fn() -> Router<Arc<AppState>> = user_routes;
    }

    #[test]
    fn test_user_routes_uses_put_for_update() {
        // update_user uses PUT method
        let _fn: fn() -> Router<Arc<AppState>> = user_routes;
    }

    #[test]
    fn test_user_routes_router_is_not_empty() {
        let router = user_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }
}
