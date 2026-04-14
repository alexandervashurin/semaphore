//! GraphQL API модуль
//!
//! Предоставляет GraphQL альтернативу REST API.
//!
//! ## Endpoints
//! - `GET  /graphql`    — GraphiQL playground
//! - `POST /graphql`    — HTTP запросы (query / mutation)
//! - `GET  /graphql/ws` — WebSocket подписки (Subscription)

pub mod mutation;
pub mod query;
pub mod schema;
pub mod subscription;
pub mod types;

use std::sync::Arc;

use async_graphql::http::{ALL_WEBSOCKET_PROTOCOLS, GraphiQLSource};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    Router,
    extract::{State, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
};

use crate::api::state::AppState;

/// Создаёт маршруты GraphQL, получая AppState для инъекции в схему.
pub fn graphql_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    let schema = schema::create_schema(app_state);

    Router::new()
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .route("/graphql/ws", get(graphql_ws_handler))
        .with_state(schema)
}

/// Обработчик HTTP GraphQL запросов (query + mutation)
pub async fn graphql_handler(
    State(schema): State<schema::Schema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// WebSocket обработчик для GraphQL Subscription
pub async fn graphql_ws_handler(
    State(schema): State<schema::Schema>,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| async move {
            GraphQLWebSocket::new(stream, schema, protocol)
                .serve()
                .await;
        })
}

/// GraphiQL playground с поддержкой подписок через WebSocket
pub async fn graphql_playground() -> Html<String> {
    Html(
        GraphiQLSource::build()
            .endpoint("/graphql")
            .subscription_endpoint("/graphql/ws")
            .finish(),
    )
}
