//! IntegrationMatcherManager + IntegrationExtractValueManager + ProjectRoleManager

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::{IntegrationMatcher, IntegrationExtractValue, Role};
use async_trait::async_trait;

#[async_trait]
impl IntegrationMatcherManager for SqlStore {
    async fn get_integration_matchers(&self, _project_id: i32, _integration_id: i32) -> Result<Vec<IntegrationMatcher>> {
        Ok(vec![])
    }
    async fn create_integration_matcher(&self, matcher: IntegrationMatcher) -> Result<IntegrationMatcher> {
        Ok(matcher)
    }
    async fn update_integration_matcher(&self, _matcher: IntegrationMatcher) -> Result<()> {
        Ok(())
    }
    async fn delete_integration_matcher(&self, _project_id: i32, _integration_id: i32, _matcher_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl IntegrationExtractValueManager for SqlStore {
    async fn get_integration_extract_values(&self, _project_id: i32, _integration_id: i32) -> Result<Vec<IntegrationExtractValue>> {
        Ok(vec![])
    }
    async fn create_integration_extract_value(&self, value: IntegrationExtractValue) -> Result<IntegrationExtractValue> {
        Ok(value)
    }
    async fn update_integration_extract_value(&self, _value: IntegrationExtractValue) -> Result<()> {
        Ok(())
    }
    async fn delete_integration_extract_value(&self, _project_id: i32, _integration_id: i32, _value_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl ProjectRoleManager for SqlStore {
    async fn get_project_roles(&self, _project_id: i32) -> Result<Vec<Role>> {
        Ok(vec![])
    }
    async fn create_project_role(&self, role: Role) -> Result<Role> {
        Ok(role)
    }
    async fn update_project_role(&self, _role: Role) -> Result<()> {
        Ok(())
    }
    async fn delete_project_role(&self, _project_id: i32, _role_id: i32) -> Result<()> {
        Ok(())
    }
}
