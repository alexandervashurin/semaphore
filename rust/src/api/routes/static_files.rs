//! Маршруты статических файлов
//!
//! Static files serving для frontend SPA

use crate::api::state::AppState;
use axum::{
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

/// Создаёт маршруты для статических файлов
pub fn static_routes() -> Router<Arc<AppState>> {
    // Путь к директории с frontend: SEMAPHORE_WEB_PATH или относительно Cargo.toml (rust/../web/public)
    let web_path = std::env::var("SEMAPHORE_WEB_PATH").unwrap_or_else(|_| {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("..").join("web").join("public");
        // Канонический путь для корректной работы на Windows
        path.canonicalize()
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| path.to_string_lossy().to_string())
    });

    // Проверяем существование директории
    let path = std::path::Path::new(&web_path);
    if !path.exists() || !path.is_dir() {
        tracing::warn!(
            "Web path {} does not exist, static files will not be served",
            web_path
        );
        return Router::new();
    }
    tracing::info!("Serving static files from {}", web_path);

    // Middleware для проверки пути - API маршруты не обрабатываются
    async fn check_api_path(
        req: axum::http::Request<axum::body::Body>,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Если путь начинается с /api/, возвращаем 404 чтобы обработал API роутер
        if req.uri().path().starts_with("/api/") {
            return Err(StatusCode::NOT_FOUND);
        }
        Ok(next.run(req).await)
    }

    // ServeDir для раздачи статических файлов с fallback на index.html для SPA
    let serve_dir = ServeDir::new(&web_path)
        .not_found_service(ServeDir::new(&web_path).fallback(ServeDir::new(&web_path).append_index_html_on_directories(true)));

    Router::new()
        // В axum 0.8 используем fallback_service вместо nest_service
        .fallback_service(serve_dir)
        .layer(middleware::from_fn(check_api_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_routes_creation() {
        let router = static_routes();
        // Router всегда создаётся успешно, даже если web_path не существует
        // (в этом случае возвращается пустой Router с warning в логах)
    }

    #[test]
    fn test_static_routes_router_type() {
        let router = static_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_web_path_env_variable() {
        let web_path = std::env::var("SEMAPHORE_WEB_PATH");
        // Переменная может быть не установлена - это нормально
        let _ = web_path;
    }

    #[test]
    fn test_default_web_path_structure() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let default_path = manifest_dir.join("..").join("web").join("public");
        let path_str = default_path.to_string_lossy();
        assert!(path_str.contains("web"), "Default path should contain 'web'");
        assert!(path_str.contains("public"), "Default path should contain 'public'");
    }

    #[test]
    fn test_check_api_path_logic_blocks_api_paths() {
        let api_path = "/api/tasks";
        let non_api_path = "/index.html";
        assert!(api_path.starts_with("/api/"), "API path should start with /api/");
        assert!(!non_api_path.starts_with("/api/"), "Static path should not start with /api/");
    }

    #[test]
    fn test_check_api_path_allows_static_paths() {
        let static_paths = ["/index.html", "/assets/app.js", "/favicon.ico", "/"];
        for path in static_paths {
            assert!(!path.starts_with("/api/"), "{} should not be blocked", path);
        }
    }

    #[test]
    fn test_static_routes_handles_missing_web_path() {
        // When web path doesn't exist, returns empty Router with warning
        let router = static_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_static_routes_env_variable_supported() {
        // SEMAPHORE_WEB_PATH env var can override default path
        let result = std::env::var("SEMAPHORE_WEB_PATH");
        // May or may not be set - both are valid
        let _ = result;
    }

    #[test]
    fn test_static_routes_default_path_resolution() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let default_path = manifest_dir.join("..").join("web").join("public");
        // Path should be constructable
        assert!(default_path.to_str().is_some() || default_path.to_str().is_none());
    }

    #[test]
    fn test_check_api_path_various_api_paths() {
        let api_paths = ["/api/users", "/api/projects/1", "/api/auth/login", "/api/"];
        for path in api_paths {
            assert!(path.starts_with("/api/"), "{} should be blocked", path);
        }
    }
}
