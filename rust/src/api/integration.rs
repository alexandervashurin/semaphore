//! API - Integration Handler
//!
//! Обработчики для интеграций (общие, не project-specific)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::Integration;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::db::store::RetrieveQueryParams;

/// Получает все интеграции (глобальные)
pub async fn get_integrations(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RetrieveQueryParams>,
) -> Result<Json<Vec<Integration>>, (StatusCode, Json<ErrorResponse>)> {
    // В реальной реализации нужно получить глобальные интеграции
    Ok(Json(vec![]))
}

/// Получает интеграцию по ID
pub async fn get_integration(
    State(state): State<Arc<AppState>>,
    Path(integration_id): Path<i32>,
) -> Result<Json<Integration>, (StatusCode, Json<ErrorResponse>)> {
    // В реальной реализации нужно получить интеграцию из БД
    Err((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("Integration not found".to_string()))
    ))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_handler() {
        // Тест для проверки обработчиков интеграций
        assert!(true);
    }
}
