//! GraphQL схема — сборка из Query, Mutation, Subscription с инъекцией AppState

use std::sync::Arc;

use async_graphql::Schema as AsyncSchema;

use crate::api::state::AppState;

use super::mutation::MutationRoot;
use super::query::QueryRoot;
use super::subscription::SubscriptionRoot;

/// Тип схемы с активными Subscription
pub type Schema = AsyncSchema<QueryRoot, MutationRoot, SubscriptionRoot>;

/// Создаёт GraphQL схему с инъецированным AppState.
///
/// AppState доступен в resolvers через `ctx.data::<AppState>()`.
pub fn create_schema(state: Arc<AppState>) -> Schema {
    AsyncSchema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(state)
        .finish()
}
