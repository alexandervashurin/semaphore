//! GraphQL схема

use async_graphql::{EmptySubscription, Schema as AsyncSchema};

use super::mutation::MutationRoot;
use super::query::QueryRoot;

/// Тип схемы
pub type Schema = AsyncSchema<QueryRoot, MutationRoot, EmptySubscription>;

/// Создаёт схему GraphQL
pub fn create_schema() -> Schema {
    AsyncSchema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}
