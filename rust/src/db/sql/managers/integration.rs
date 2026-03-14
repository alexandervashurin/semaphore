//! IntegrationManager - управление интеграциями

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Integration;
use async_trait::async_trait;

#[async_trait]
impl IntegrationManager for SqlStore {
    async fn get_integrations(&self, _project_id: i32) -> Result<Vec<Integration>> { Ok(vec![]) }
    async fn get_integration(&self, _project_id: i32, _integration_id: i32) -> Result<Integration> { Err(Error::NotFound("Интеграция не найдена".to_string())) }
    async fn create_integration(&self, _integration: Integration) -> Result<Integration> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_integration(&self, _integration: Integration) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_integration(&self, _project_id: i32, _integration_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

