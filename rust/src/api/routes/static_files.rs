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
