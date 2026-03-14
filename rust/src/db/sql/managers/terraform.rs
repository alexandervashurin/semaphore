//! TerraformInventoryManager - управление Terraform inventory

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::{TerraformInventoryAlias, TerraformInventoryState};
use async_trait::async_trait;

#[async_trait]
impl TerraformInventoryManager for SqlStore {
    async fn create_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<TerraformInventoryAlias> {
        self.create_terraform_inventory_alias(alias).await
    }

    async fn update_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<()> {
        self.update_terraform_inventory_alias(alias).await
    }

    async fn get_terraform_inventory_alias_by_alias(&self, alias: &str) -> Result<TerraformInventoryAlias> {
        self.get_terraform_inventory_alias_by_alias(alias).await
    }

    async fn get_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<TerraformInventoryAlias> {
        self.get_terraform_inventory_alias(project_id, inventory_id, alias_id).await
    }

    async fn get_terraform_inventory_aliases(&self, project_id: i32, inventory_id: i32) -> Result<Vec<TerraformInventoryAlias>> {
        self.get_terraform_inventory_aliases(project_id, inventory_id).await
    }

    async fn delete_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<()> {
        self.delete_terraform_inventory_alias(project_id, inventory_id, alias_id).await
    }

    async fn get_terraform_inventory_states(&self, project_id: i32, inventory_id: i32, params: RetrieveQueryParams) -> Result<Vec<TerraformInventoryState>> {
        self.get_terraform_inventory_states(project_id, inventory_id, params).await
    }

    async fn create_terraform_inventory_state(&self, state: TerraformInventoryState) -> Result<TerraformInventoryState> {
        self.create_terraform_inventory_state(state).await
    }

    async fn delete_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<()> {
        self.delete_terraform_inventory_state(project_id, inventory_id, state_id).await
    }

    async fn get_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<TerraformInventoryState> {
        self.get_terraform_inventory_state(project_id, inventory_id, state_id).await
    }

    async fn get_terraform_state_count(&self) -> Result<i32> {
        self.get_terraform_state_count().await
    }
}

