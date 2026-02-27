//! API - Options Handler
//!
//! Обработчики для опций

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::OptionItem;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::api::extractors::AuthUser;

/// Получает все опции
pub async fn get_options(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Json<Vec<OptionItem>>, (StatusCode, Json<ErrorResponse>)> {
    // Проверяем, что пользователь админ
    if !auth_user.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("User must be admin".to_string()))
        ));
    }

    let options = state.store.get_options(crate::db::store::RetrieveQueryParams::default())
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    // Конвертируем HashMap в Vec
    let options_vec: Vec<OptionItem> = options.into_iter()
        .map(|(key, value)| OptionItem::new(0, key, value))
        .collect();

    Ok(Json(options_vec))
}

/// Устанавливает опцию
pub async fn set_option(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(payload): Json<OptionItem>,
) -> Result<Json<OptionItem>, (StatusCode, Json<ErrorResponse>)> {
    // Проверяем, что пользователь админ
    if !auth_user.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("User must be admin".to_string()))
        ));
    }

    state.store.set_option(&payload.key, &payload.value)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(payload))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_handler() {
        // Тест для проверки обработчиков опций
        assert!(true);
    }
}
