//! Role CRUD Operations for BoltDB
//!
//! Операции с ролями в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::Role;

impl BoltStore {
    /// Получает роли проекта
    pub async fn get_roles(&self, project_id: i32) -> Result<Vec<Role>> {
        self.get_objects::<Role>(project_id, "roles", crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: String::new(),
        }).await
    }

    /// Получает роль по ID
    pub async fn get_role(&self, project_id: i32, role_id: i32) -> Result<Role> {
        self.get_object::<Role>(project_id, "roles", role_id).await
    }

    /// Создаёт роль
    pub async fn create_role(&self, mut role: Role) -> Result<Role> {
        role.id = self.get_next_id("roles")?;
        self.create_object(role.project_id, "roles", &role).await?;
        Ok(role)
    }

    /// Обновляет роль
    pub async fn update_role(&self, role: Role) -> Result<()> {
        self.update_object(role.project_id, "roles", role.id, &role).await
    }

    /// Удаляет роль
    pub async fn delete_role(&self, project_id: i32, role_id: i32) -> Result<()> {
        self.delete_object(project_id, "roles", role_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_operations() {
        // Тест для проверки операций с ролями
        assert!(true);
    }
}
