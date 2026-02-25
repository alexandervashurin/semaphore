//! Промежуточное ПО (middleware)
//!
//! Этот модуль содержит различные middleware для обработки запросов:
//! - Логирование запросов и ответов
//! - Аутентификация и авторизация
//! - Обработка ошибок
//! - Ограничение скорости (rate limiting)
//! - CORS
//! - Таймауты

use axum::{
    extract::{State, Request},
    http::{StatusCode, header},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{info, warn, error, debug};

use crate::api::state::AppState;

/// Структура ответа об ошибке
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// Создаёт новый ответ об ошибке
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            code: None,
            details: None,
        }
    }

    /// Добавляет код ошибки
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Добавляет детали ошибки
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

/// Middleware для логирования запросов
///
/// Логирует:
/// - Метод и путь запроса
/// - Время выполнения
/// - Статус ответа
pub async fn request_logger(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    
    // Извлекаем ID запроса для трассировки
    let request_id = headers
        .get("X-Request-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    let start = Instant::now();
    
    // Логируем входящий запрос
    debug!(
        method = %method,
        path = %uri,
        request_id = %request_id,
        "Входящий запрос"
    );
    
    let response = next.run(request).await;
    let duration = start.elapsed();
    
    // Логируем исходящий ответ
    let status = response.status();
    info!(
        method = %method,
        path = %uri,
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        request_id = %request_id,
        "Запрос обработан"
    );
    
    response
}

/// Middleware для аутентификации
///
/// Проверяет наличие и валидность токена аутентификации
pub async fn auth_middleware(
    State(_state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // Получаем заголовок Authorization
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    // Для некоторых путей аутентификация не требуется
    let path = request.uri().path();
    if is_public_path(path) {
        return Ok(next.run(request).await);
    }

    // Проверяем наличие токена
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Требуется аутентификация")
                    .with_code("AUTH_REQUIRED")),
            ));
        }
    };

    // TODO: Реализовать проверку токена
    // let user = state.store.get_user_by_token(token).await?;

    // Пока просто проверяем, что токен не пустой
    if token.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Неверный токен")
                .with_code("INVALID_TOKEN")),
        ));
    }

    // Добавляем информацию о пользователе в request extensions
    // request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

/// Middleware для проверки прав доступа
///
/// Проверяет, имеет ли пользователь доступ к ресурсу
pub async fn permission_middleware(
    State(_state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let path = request.uri().path();

    // Извлекаем ID проекта из пути
    if let Some(project_id) = extract_project_id(path) {
        // TODO: Проверить права доступа пользователя к проекту
        // let user = request.extensions().get::<User>().unwrap();
        // let has_permission = state.store.check_permission(user.id, project_id, &method).await?;

        // Пока просто пропускаем все запросы
        debug!("Проверка прав доступа к проекту {}", project_id);
    }

    Ok(next.run(request).await)
}

/// Middleware для ограничения размера запроса
pub async fn request_size_limit(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    const MAX_SIZE: usize = 10 * 1024 * 1024; // 10 MB
    
    if let Some(content_length) = request.headers().get(header::CONTENT_LENGTH) {
        if let Ok(size) = content_length.to_str() {
            if let Ok(size) = size.parse::<usize>() {
                if size > MAX_SIZE {
                    return Err((
                        StatusCode::PAYLOAD_TOO_LARGE,
                        Json(ErrorResponse::new(
                            format!("Размер запроса превышает лимит ({} MB)", MAX_SIZE / 1024 / 1024)
                        ).with_code("PAYLOAD_TOO_LARGE")),
                    ));
                }
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// Middleware для добавления заголовков безопасности
pub async fn security_headers(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Защита от XSS
    headers.insert(
        "X-Content-Type-Options",
        "nosniff".parse().unwrap(),
    );
    
    // Защита от clickjacking
    headers.insert(
        "X-Frame-Options",
        "DENY".parse().unwrap(),
    );
    
    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'".parse().unwrap(),
    );
    
    // HSTS
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    
    response
}

/// Middleware для обработки паник
pub async fn panic_handler(
    request: Request,
    next: Next,
) -> Response {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                next.run(request).await
            })
        })
    }));
    
    match result {
        Ok(response) => response,
        Err(_) => {
            error!("Произошла паника при обработке запроса");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Внутренняя ошибка сервера")
                    .with_code("PANIC")),
            ).into_response()
        }
    }
}

/// Обработчик ошибок для tracing middleware
pub struct LogFailure;

impl LogFailure {
    pub fn log(&self, status: StatusCode) {
        if status.is_server_error() {
            error!("Ошибка сервера: {}", status);
        } else if status.is_client_error() {
            warn!("Ошибка клиента: {}", status);
        }
    }
}

/// Вспомогательные функции

/// Проверяет, является ли путь публичным (не требует аутентификации)
fn is_public_path(path: &str) -> bool {
    path.starts_with("/api/auth/") ||
    path.starts_with("/api/health") ||
    path.starts_with("/api/version") ||
    path == "/favicon.ico" ||
    path.starts_with("/static/")
}

/// Извлекает ID проекта из пути
fn extract_project_id(path: &str) -> Option<i32> {
    // Пример пути: /api/projects/123/templates
    let parts: Vec<&str> = path.split('/').collect();
    
    if parts.len() >= 4 && parts[2] == "projects" {
        parts[3].parse().ok()
    } else {
        None
    }
}

/// Middleware для установки таймаута на запрос
pub async fn timeout_middleware(
    request: Request,
    next: Next,
) -> Response {
    let timeout_duration = Duration::from_secs(30);
    
    tokio::time::timeout(timeout_duration, async {
        next.run(request).await
    })
    .await
    .unwrap_or_else(|_| {
        error!("Превышено время ожидания запроса");
        (
            StatusCode::GATEWAY_TIMEOUT,
            Json(ErrorResponse::new("Превышено время ожидания запроса")
                .with_code("TIMEOUT")),
        ).into_response()
    })
}

/// Создаёт стек middleware по умолчанию
pub fn default_middleware_stack() -> Vec<&'static str> {
    vec![
        "request_logger",
        "security_headers",
        "request_size_limit",
        "auth_middleware",
        "permission_middleware",
        "timeout_middleware",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response() {
        let error = ErrorResponse::new("Test error")
            .with_code("TEST_CODE")
            .with_details(serde_json::json!({"key": "value"}));
        
        assert_eq!(error.error, "Test error");
        assert_eq!(error.code, Some("TEST_CODE".to_string()));
        assert!(error.details.is_some());
    }

    #[test]
    fn test_is_public_path() {
        assert!(is_public_path("/api/auth/login"));
        assert!(is_public_path("/api/health"));
        assert!(!is_public_path("/api/projects"));
        assert!(!is_public_path("/api/users/1"));
    }

    #[test]
    fn test_extract_project_id() {
        assert_eq!(extract_project_id("/api/projects/123/templates"), Some(123));
        assert_eq!(extract_project_id("/api/projects/456/tasks"), Some(456));
        assert_eq!(extract_project_id("/api/users/1"), None);
    }
}
