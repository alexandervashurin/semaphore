//! TerraformInventoryManager - управление Terraform inventory

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::{TerraformInventoryAlias, TerraformInventoryState};
use async_trait::async_trait;

#[async_trait]
impl TerraformInventoryManager for SqlStore {
    async fn create_terraform_inventory_alias(
        &self,
        alias: TerraformInventoryAlias,
    ) -> Result<TerraformInventoryAlias> {
        self.create_terraform_inventory_alias(alias).await
    }

    async fn update_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<()> {
        self.db.update_terraform_inventory_alias(alias).await
    }

    async fn get_terraform_inventory_alias_by_alias(
        &self,
        alias: &str,
    ) -> Result<TerraformInventoryAlias> {
        self.get_terraform_inventory_alias_by_alias(alias).await
    }

    async fn get_terraform_inventory_alias(
        &self,
        project_id: i32,
        inventory_id: i32,
        alias_id: &str,
    ) -> Result<TerraformInventoryAlias> {
        self.get_terraform_inventory_alias(project_id, inventory_id, alias_id)
            .await
    }

    async fn get_terraform_inventory_aliases(
        &self,
        project_id: i32,
        inventory_id: i32,
    ) -> Result<Vec<TerraformInventoryAlias>> {
        self.get_terraform_inventory_aliases(project_id, inventory_id)
            .await
    }

    async fn delete_terraform_inventory_alias(
        &self,
        project_id: i32,
        inventory_id: i32,
        alias_id: &str,
    ) -> Result<()> {
        self.delete_terraform_inventory_alias(project_id, inventory_id, alias_id)
            .await
    }

    async fn get_terraform_inventory_states(
        &self,
        project_id: i32,
        inventory_id: i32,
        params: RetrieveQueryParams,
    ) -> Result<Vec<TerraformInventoryState>> {
        self.get_terraform_inventory_states(project_id, inventory_id, params)
            .await
    }

    async fn create_terraform_inventory_state(
        &self,
        state: TerraformInventoryState,
    ) -> Result<TerraformInventoryState> {
        self.create_terraform_inventory_state(state).await
    }

    async fn delete_terraform_inventory_state(
        &self,
        project_id: i32,
        inventory_id: i32,
        state_id: i32,
    ) -> Result<()> {
        self.delete_terraform_inventory_state(project_id, inventory_id, state_id)
            .await
    }

    async fn get_terraform_inventory_state(
        &self,
        project_id: i32,
        inventory_id: i32,
        state_id: i32,
    ) -> Result<TerraformInventoryState> {
        self.get_terraform_inventory_state(project_id, inventory_id, state_id)
            .await
    }

    async fn get_terraform_state_count(&self) -> Result<i32> {
        self.db.get_terraform_state_count().await
    }
}

#[cfg(test)]
mod tests {
    use crate::models::terraform_inventory::{
        Alias, TerraformInventoryAlias, TerraformInventoryState,
    };
    use chrono::Utc;

    #[test]
    fn test_terraform_inventory_alias_new() {
        let alias = TerraformInventoryAlias::new(10, 5, 3, "prod-inventory".to_string());
        assert_eq!(alias.project_id, 10);
        assert_eq!(alias.inventory_id, 5);
        assert_eq!(alias.auth_key_id, 3);
        assert_eq!(alias.alias, "prod-inventory");
        assert!(alias.task_id.is_none());
    }

    #[test]
    fn test_terraform_inventory_alias_to_alias() {
        let tf = TerraformInventoryAlias::new(10, 5, 3, "test".to_string());
        let base = tf.to_alias();
        assert_eq!(base.alias, "test");
        assert_eq!(base.project_id, 10);
    }

    #[test]
    fn test_terraform_inventory_alias_serialization() {
        let alias = TerraformInventoryAlias::new(10, 5, 3, "prod".to_string());
        let json = serde_json::to_string(&alias).unwrap();
        assert!(json.contains("\"alias\":\"prod\""));
        assert!(json.contains("\"project_id\":10"));
        assert!(json.contains("\"inventory_id\":5"));
        assert!(json.contains("\"auth_key_id\":3"));
    }

    #[test]
    fn test_terraform_inventory_state_new() {
        let state = TerraformInventoryState::new(10, 5, "{\"resources\":[]}".to_string());
        assert_eq!(state.id, 0);
        assert_eq!(state.project_id, 10);
        assert_eq!(state.inventory_id, 5);
        assert_eq!(state.state, Some("{\"resources\":[]}".to_string()));
        assert!(state.task_id.is_none());
    }

    #[test]
    fn test_terraform_inventory_state_serialization() {
        let state = TerraformInventoryState {
            id: 1,
            created: Utc::now(),
            task_id: Some(100),
            project_id: 10,
            inventory_id: 5,
            state: Some("{\"outputs\":{}}".to_string()),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"project_id\":10"));
        assert!(json.contains("\"task_id\":100"));
    }

    #[test]
    fn test_alias_struct() {
        let alias = Alias {
            alias: "my-alias".to_string(),
            project_id: 42,
        };
        assert_eq!(alias.alias, "my-alias");
        assert_eq!(alias.project_id, 42);
    }

    #[test]
    fn test_terraform_inventory_state_no_task_id() {
        let state = TerraformInventoryState::new(10, 5, "{}".to_string());
        assert!(state.task_id.is_none());
    }

    #[test]
    fn test_terraform_inventory_alias_clone() {
        let alias = TerraformInventoryAlias::new(1, 1, 1, "clone-test".to_string());
        let cloned = alias.clone();
        assert_eq!(cloned.alias, alias.alias);
        assert_eq!(cloned.auth_key_id, alias.auth_key_id);
    }

    #[test]
    fn test_terraform_inventory_state_skip_nulls() {
        let state = TerraformInventoryState {
            id: 1,
            created: Utc::now(),
            task_id: None,
            project_id: 1,
            inventory_id: 1,
            state: None,
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(!json.contains("task_id"));
        assert!(!json.contains("state"));
    }

    #[test]
    fn test_terraform_inventory_state_clone() {
        let state = TerraformInventoryState::new(5, 3, "{\"a\":1}".to_string());
        let cloned = state.clone();
        assert_eq!(cloned.project_id, state.project_id);
        assert_eq!(cloned.inventory_id, state.inventory_id);
        assert_eq!(cloned.state, state.state);
    }
}
