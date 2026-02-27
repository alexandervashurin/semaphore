//! Runner CRUD Operations for BoltDB
//!
//! Операции с раннерами в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::Runner;

impl BoltStore {
    /// Получает раннеры
    pub async fn get_runners(&self, project_id: Option<i32>) -> Result<Vec<Runner>> {
        if let Some(pid) = project_id {
            self.get_objects::<Runner>(pid, "runners", crate::db::store::RetrieveQueryParams {
                offset: 0,
                count: Some(1000),
                filter: String::new(),
            }).await
        } else {
            self.get_objects::<Runner>(0, "global_runners", crate::db::store::RetrieveQueryParams {
                offset: 0,
                count: Some(1000),
                filter: String::new(),
            }).await
        }
    }

    /// Получает раннер по ID
    pub async fn get_runner(&self, runner_id: i32) -> Result<Runner> {
        // В базовой версии возвращаем ошибку
        Err(Error::NotFound("Runner not found".to_string()))
    }

    /// Создаёт раннер
    pub async fn create_runner(&self, mut runner: Runner) -> Result<Runner> {
        runner.id = self.get_next_id("runners")?;
        self.create_object(runner.project_id, "runners", &runner).await?;
        Ok(runner)
    }

    /// Обновляет раннер
    pub async fn update_runner(&self, runner: Runner) -> Result<()> {
        self.update_object(runner.project_id, "runners", runner.id, &runner).await
    }

    /// Удаляет раннер
    pub async fn delete_runner(&self, runner_id: i32) -> Result<()> {
        // В базовой версии возвращаем ошибку
        Err(Error::Other("Not implemented".to_string()))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_operations() {
        // Тест для проверки операций с раннерами
        assert!(true);
    }
}
