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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_type_alias_exists() {
        // Verify the Schema type alias is accessible and correct
        fn assert_schema_type<T>() {}
        assert_schema_type::<Schema>();
    }

    #[test]
    fn test_query_root_default() {
        let _query = QueryRoot;
    }

    #[test]
    fn test_mutation_root_default() {
        let _mutation = MutationRoot;
    }

    #[test]
    fn test_subscription_root_default() {
        let _subscription = SubscriptionRoot;
    }

    #[test]
    fn test_schema_type_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // Schema type should be Send + Sync for axum state
        assert_send_sync::<Schema>();
    }
}
