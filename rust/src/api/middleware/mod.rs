//! Middleware модули
//!
//! Предоставляет middleware для:
//! - Rate limiting (ограничение запросов)
//! - Security headers (CSP, HSTS, X-Frame-Options)
//! - CORS (Cross-Origin Resource Sharing)
//! - Кэширование

pub mod cache;
pub mod rate_limiter;
pub mod security_headers;
pub mod trace_id;

pub use cache::CacheMiddleware;
pub use rate_limiter::*;
pub use security_headers::*;
pub use trace_id::{trace_id_middleware, TraceId};

// Ре-экспорт ErrorResponse для обратной совместимости
use crate::error::Error as CrateError;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            code: None,
            details: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Создаёт ErrorResponse из crate::Error
    pub fn from_crate_error(e: &CrateError) -> (StatusCode, Self) {
        match e {
            CrateError::NotFound(_) => (StatusCode::NOT_FOUND, Self::new(e.to_string())),
            CrateError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, Self::new(e.to_string())),
            CrateError::Forbidden(_) => (StatusCode::FORBIDDEN, Self::new(e.to_string())),
            CrateError::Validation(_) => (StatusCode::BAD_REQUEST, Self::new(e.to_string())),
            CrateError::Auth(_) => (StatusCode::UNAUTHORIZED, Self::new(e.to_string())),
            CrateError::Database(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Self::new(e.to_string()))
            }
            CrateError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, Self::new(e.to_string())),
            CrateError::Git(_) => (StatusCode::INTERNAL_SERVER_ERROR, Self::new(e.to_string())),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Self::new(e.to_string())),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}
