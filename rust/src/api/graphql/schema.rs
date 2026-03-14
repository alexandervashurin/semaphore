//! GraphQL схема

use async_graphql::{EmptySubscription, Schema as AsyncSchema};

use super::query::QueryRoot;
use super::mutation::MutationRoot;

/// Тип схемы
pub type Schema = AsyncSchema<QueryRoot, MutationRoot, EmptySubscription>;

/// Создаёт схему GraphQL
pub fn create_schema() -> Schema {
    AsyncSchema::build(
        QueryRoot,
        MutationRoot,
        EmptySubscription,
    )
    .finish()
}
