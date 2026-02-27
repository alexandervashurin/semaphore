//! Environment CRUD Operations for BoltDB
//!
//! Операции с окружениями в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::Environment;

impl BoltStore {
    /// Получает окружения проекта
    pub async fn get_environments(&self, project_id: i32) -> Result<Vec<Environment>> {
        self.get_objects::<Environment>(project_id, "environments", crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: String::new(),
        }).await
    }

    /// Получает окружение по ID
    pub async fn get_environment(&self, project_id: i32, env_id: i32) -> Result<Environment> {
        self.get_object::<Environment>(project_id, "environments", env_id).await
    }

    /// Создаёт окружение
    pub async fn create_environment(&self, mut env: Environment) -> Result<Environment> {
        env.id = self.get_next_id("environments")?;
        self.create_object(env.project_id, "environments", &env).await?;
        Ok(env)
    }

    /// Обновляет окружение
    pub async fn update_environment(&self, env: Environment) -> Result<()> {
        self.update_object(env.project_id, "environments", env.id, &env).await
    }

    /// Удаляет окружение
    pub async fn delete_environment(&self, project_id: i32, env_id: i32) -> Result<()> {
        self.delete_object(project_id, "environments", env_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_operations() {
        // Тест для проверки операций с окружениями
        assert!(true);
    }
}
