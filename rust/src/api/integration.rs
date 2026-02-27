//! Integration API - управление интеграциями
//!
//! Аналог api/projects/integration.go из Go версии

use axum::{
    extract::{Path, Query, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::api::extractors::AuthUser;
use crate::error::{Error, Result};
use crate::models::Integration;

/// Middleware для интеграций
pub async fn integration_middleware<B>(
    State(state): State<Arc<AppState>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<axum::response::Response> {
    // Получаем project_id из пути
    let project_id = request
        .extensions()
        .get::<i32>()
        .copied()
        .ok_or(Error::NotFound("Project ID not found".to_string()))?;

    // Получаем integration_id из пути
    let integration_id = request
        .extensions()
        .get::<i32>()
        .copied()
        .ok_or(Error::NotFound("Integration ID not found".to_string()))?;

    // Получаем интеграцию из БД
    let integration = state.store.get_integration(project_id, integration_id).await?;

    // Добавляем в request context
    request.extensions_mut().insert(integration);

    Ok(next.run(request).await)
}

/// Получает интеграцию по ID
pub async fn get_integration(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, .. }: AuthUser,
    Path(integration_id): Path<i32>,
) -> Result<Json<Integration>> {
    let project_id = get_project_id_from_context(&state)?;

    let integration = state.store.get_integration(project_id, integration_id).await?;
    Ok(Json(integration))
}

/// Получает все интеграции проекта
pub async fn get_integrations(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, .. }: AuthUser,
    Query(params): Query<RetrieveQueryParams>,
) -> Result<Json<Vec<Integration>>> {
    let project_id = get_project_id_from_context(&state)?;

    let integrations = state.store.get_integrations(project_id, params, false).await?;
    Ok(Json(integrations))
}

/// Получает рефереры интеграции
pub async fn get_integration_refs(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, .. }: AuthUser,
    Path(integration_id): Path<i32>,
) -> Result<Json<IntegrationRefs>> {
    let project_id = get_project_id_from_context(&state)?;

    let refs = state.store.get_integration_refs(project_id, integration_id).await?;
    Ok(Json(refs))
}

/// Добавляет новую интеграцию
pub async fn add_integration(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, .. }: AuthUser,
    Json(integration): Json<Integration>,
) -> Result<(StatusCode, Json<Integration>)> {
    let project_id = get_project_id_from_context(&state)?;

    // Проверяем что project_id совпадает
    if integration.project_id != project_id {
        return Err(Error::Other("Project ID in body and URL must be the same".to_string()));
    }

    // Валидация интеграции
    integration.validate()?;

    let new_integration = state.store.create_integration(integration).await?;
    Ok((StatusCode::CREATED, Json(new_integration)))
}

/// Обновляет интеграцию
pub async fn update_integration(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, .. }: AuthUser,
    Path(integration_id): Path<i32>,
    Json(integration): Json<Integration>,
) -> Result<StatusCode> {
    let project_id = get_project_id_from_context(&state)?;

    // Проверяем что ID совпадает
    if integration.id != integration_id {
        return Err(Error::Other("Integration ID in body and URL must be the same".to_string()));
    }

    state.store.update_integration(integration).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Удаляет интеграцию
pub async fn delete_integration(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, .. }: AuthUser,
    Path(integration_id): Path<i32>,
) -> Result<StatusCode> {
    let project_id = get_project_id_from_context(&state)?;
    
    state.store.delete_integration(project_id, integration_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Вспомогательная функция для получения project_id
fn get_project_id_from_context(state: &AppState) -> Result<i32> {
    // TODO: Получить project_id из контекста запроса
    // Пока используем заглушку
    Err(Error::NotFound("Project ID not found in context".to_string()))
}

// ============================================================================
// Типы данных
// ============================================================================

/// Параметры запроса
#[derive(Debug, Default, Deserialize)]
pub struct RetrieveQueryParams {
    pub offset: usize,
    pub count: usize,
    pub filter: Option<String>,
}

/// Рефереры интеграции
#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrationRefs {
    pub schedules: Vec<i32>,
    pub tasks: Vec<i32>,
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieve_query_params_default() {
        let params = RetrieveQueryParams::default();
        assert_eq!(params.offset, 0);
        assert_eq!(params.count, 0);
        assert!(params.filter.is_none());
    }

    #[test]
    fn test_integration_refs_serialization() {
        let refs = IntegrationRefs {
            schedules: vec![1, 2, 3],
            tasks: vec![4, 5, 6],
        };

        let json = serde_json::to_string(&refs).unwrap();
        assert!(json.contains("schedules"));
        assert!(json.contains("tasks"));
    }
}
