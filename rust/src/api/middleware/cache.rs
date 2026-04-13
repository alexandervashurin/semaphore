//! Cache Middleware - Middleware для кэширования HTTP запросов
//!
//! Кэширует GET запросы для улучшения производительности

use crate::cache::RedisCache;
use axum::{
    body::Body,
    http::{header, Method, Request, Response, StatusCode},
    middleware::Next,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{debug, warn};

/// Middleware для кэширования ответов
pub struct CacheMiddleware {
    redis: Arc<RedisCache>,
    ttl_secs: u64,
    skip_paths: Vec<String>,
}

impl CacheMiddleware {
    /// Создаёт новый middleware
    pub fn new(redis: Arc<RedisCache>, ttl_secs: u64, skip_paths: Vec<String>) -> Self {
        Self {
            redis,
            ttl_secs,
            skip_paths,
        }
    }

    /// Проверяет нужно ли пропускать кэширование для пути
    fn should_skip(&self, path: &str) -> bool {
        self.skip_paths.iter().any(|p| path.starts_with(p))
    }

    /// Генерирует ключ кэша для запроса
    fn generate_cache_key(&self, method: &Method, uri: &str) -> String {
        // Создаём хэш из метода и URI
        let mut hasher = Sha256::new();
        hasher.update(method.as_str().as_bytes());
        hasher.update(uri.as_bytes());

        let hash = format!("{:x}", hasher.finalize());
        format!("http_cache:{}", hash)
    }

    /// Обрабатывает запрос
    pub async fn handle(
        &self,
        req: Request<Body>,
        next: Next,
    ) -> Result<Response<Body>, StatusCode> {
        // Кэшируем только GET запросы
        if req.method() != Method::GET {
            return Ok(next.run(req).await);
        }

        let path = req.uri().path().to_string();

        // Пропускаем указанные пути
        if self.should_skip(&path) {
            debug!("Skipping cache for path: {}", path);
            return Ok(next.run(req).await);
        }

        // Генерируем ключ кэша
        let cache_key = self.generate_cache_key(req.method(), &path);

        // Пробуем получить из кэша
        if let Some(cached_body) = self.redis.get::<String>(&cache_key).await.unwrap_or(None) {
            debug!("Cache hit for key: {}", cache_key);

            // Восстанавливаем ответ из кэша
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-Cache", "HIT")
                .body(Body::from(cached_body))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok(response);
        }

        // Кэш промах - выполняем запрос
        debug!("Cache miss for key: {}", cache_key);
        let response = next.run(req).await;

        // Кэшируем успешные ответы
        if response.status() == StatusCode::OK {
            // Разбираем response на части
            let (parts, body) = response.into_parts();

            // Читаем тело ответа
            let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
                Ok(bytes) => bytes,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };

            let body_str = String::from_utf8_lossy(&body_bytes).to_string();

            // Сохраняем в кэш
            if let Err(e) = self
                .redis
                .set_with_ttl(&cache_key, &body_str, self.ttl_secs)
                .await
            {
                warn!("Failed to cache response: {}", e);
            }

            // Восстанавливаем response
            return Ok(Response::from_parts(parts, Body::from(body_bytes)));
        }

        Ok(response)
    }
}

/// Helper функции для инвалидации HTTP кэша
pub async fn invalidate_http_cache(
    redis: &RedisCache,
    path_pattern: &str,
) -> crate::error::Result<()> {
    // В полной реализации нужно удалять ключи по паттерну
    debug!("Invalidating HTTP cache for pattern: {}", path_pattern);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        let key1 = middleware.generate_cache_key(&Method::GET, "/api/projects");
        let key2 = middleware.generate_cache_key(&Method::GET, "/api/projects");

        assert_eq!(key1, key2);

        let key3 = middleware.generate_cache_key(&Method::GET, "/api/tasks");
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_should_skip() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec!["/api/auth".to_string(), "/api/admin".to_string()],
        );

        assert!(middleware.should_skip("/api/auth/login"));
        assert!(middleware.should_skip("/api/admin/users"));
        assert!(!middleware.should_skip("/api/projects"));
    }

    #[test]
    fn test_cache_key_different_methods() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        let key_get = middleware.generate_cache_key(&Method::GET, "/api/data");
        let key_post = middleware.generate_cache_key(&Method::POST, "/api/data");

        assert_ne!(key_get, key_post);
    }

    #[test]
    fn test_cache_key_starts_with_prefix() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        let key = middleware.generate_cache_key(&Method::GET, "/api/test");
        assert!(key.starts_with("http_cache:"));
    }

    #[test]
    fn test_cache_key_deterministic() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        let key1 = middleware.generate_cache_key(&Method::GET, "/api/projects");
        let key2 = middleware.generate_cache_key(&Method::GET, "/api/projects");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_uris() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        let key1 = middleware.generate_cache_key(&Method::GET, "/api/projects/1");
        let key2 = middleware.generate_cache_key(&Method::GET, "/api/projects/2");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_should_skip_empty_paths() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        assert!(!middleware.should_skip("/"));
        assert!(!middleware.should_skip("/api"));
    }

    #[test]
    fn test_should_skip_prefix_match() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec!["/api/admin".to_string()],
        );

        assert!(middleware.should_skip("/api/admin"));
        assert!(middleware.should_skip("/api/admin/"));
        assert!(middleware.should_skip("/api/admin/settings"));
        // "starts_with" also matches "/api/administer" since it starts with "/api/admin"
        assert!(middleware.should_skip("/api/administer"));
    }

    #[test]
    fn test_should_skip_multiple_patterns() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![
                "/api/auth".to_string(),
                "/api/admin".to_string(),
                "/api/health".to_string(),
            ],
        );

        assert!(middleware.should_skip("/api/auth/token"));
        assert!(middleware.should_skip("/api/admin/users"));
        assert!(middleware.should_skip("/api/health"));
        assert!(!middleware.should_skip("/api/projects"));
        assert!(!middleware.should_skip("/api/tasks"));
    }

    #[test]
    fn test_cache_key_hash_length() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        let key = middleware.generate_cache_key(&Method::GET, "/api/test");
        // "http_cache:" + 64 hex chars (SHA-256)
        assert_eq!(key.len(), "http_cache:".len() + 64);
    }

    #[test]
    fn test_cache_key_hex_format() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        let key = middleware.generate_cache_key(&Method::DELETE, "/api/items/1");
        let hex_part = key.strip_prefix("http_cache:").unwrap();
        // All characters should be hex (0-9, a-f)
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_cache_key_case_sensitive_uri() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        let key1 = middleware.generate_cache_key(&Method::GET, "/api/Projects");
        let key2 = middleware.generate_cache_key(&Method::GET, "/api/projects");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_with_query_string_omitted() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        // The middleware uses req.uri().path() which omits query string
        let key1 = middleware.generate_cache_key(&Method::GET, "/api/data");
        assert!(key1.starts_with("http_cache:"));
    }

    #[test]
    fn test_ttl_secs_stored() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            600,
            vec![],
        );

        assert_eq!(middleware.ttl_secs, 600);
    }

    #[test]
    fn test_skip_paths_stored() {
        let paths = vec!["/api/auth".to_string(), "/api/admin".to_string()];
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            paths.clone(),
        );

        assert_eq!(middleware.skip_paths, paths);
    }

    #[test]
    fn test_cache_key_empty_uri() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        let key = middleware.generate_cache_key(&Method::GET, "");
        assert!(key.starts_with("http_cache:"));
    }

    #[test]
    fn test_cache_key_complex_uri() {
        let middleware = CacheMiddleware::new(
            Arc::new(RedisCache::new(crate::cache::RedisConfig::default())),
            300,
            vec![],
        );

        let key = middleware.generate_cache_key(
            &Method::GET,
            "/api/projects/123/tasks/456/details",
        );
        assert!(key.starts_with("http_cache:"));
        assert_ne!(
            key,
            middleware.generate_cache_key(&Method::GET, "/api/projects/123/tasks")
        );
    }
}
