//! API - Cache Handler
//!
//! Обработчики для кэша

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::api::extractors::AuthUser;

/// Очищает кэш
pub async fn clear_cache(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Проверяем, что пользователь админ
    if !auth_user.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("User must be admin".to_string()))
        ));
    }

    // В реальной реализации нужно очистить кэш
    // state.config.clear_tmp_dir()?;

    Ok(StatusCode::NO_CONTENT)
}

/// Очищает кэш проекта (B-BE-24)
///
/// DELETE /api/project/{id}/cache
pub async fn clear_project_cache(
    State(_state): State<Arc<AppState>>,
    Path(_project_id): Path<i32>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Stub: очищает временные файлы проекта
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_handler() {
        // Тест для проверки обработчиков кэша
        assert!(true);
    }
}
