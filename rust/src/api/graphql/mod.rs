//! GraphQL API модуль
//!
//! Предоставляет GraphQL альтернативу REST API

pub mod mutation;
pub mod query;
pub mod schema;
pub mod subscription;
pub mod types;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQLRequest;
use async_graphql_axum::GraphQLResponse;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use crate::api::state::AppState;

use std::sync::Arc;

/// Создаёт маршруты GraphQL
pub fn graphql_routes() -> Router<Arc<AppState>> {
    let schema = schema::create_schema();

    Router::new()
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .with_state(schema)
}

/// Обработчик GraphQL запросов
pub async fn graphql_handler(
    State(schema): State<schema::Schema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// GraphiQL playground
pub async fn graphql_playground() -> Html<String> {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
