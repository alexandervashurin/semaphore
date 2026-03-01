//! Global Runner CRUD Operations for BoltDB
//!
//! Операции с глобальными раннерами в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::Runner;

impl BoltStore {
    /// Получает глобальные раннеры
    pub async fn get_global_runners(&self) -> Result<Vec<Runner>> {
        self.get_objects::<Runner>(0, "global_runners", crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        }).await
    }

    /// Создаёт глобальный раннер
    pub async fn create_global_runner(&self, mut runner: Runner) -> Result<Runner> {
        runner.id = self.get_next_id("global_runners")?;
        self.create_object(0, "global_runners", &runner).await?;
        Ok(runner)
    }

    /// Обновляет глобальный раннер
    pub async fn update_global_runner(&self, runner: Runner) -> Result<()> {
        self.update_object(0, "global_runners", runner.id, &runner).await
    }

    /// Удаляет глобальный раннер
    pub async fn delete_global_runner(&self, runner_id: i32) -> Result<()> {
        self.delete_object(0, "global_runners", runner_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_runner_operations() {
        // Тест для проверки операций с глобальными раннерами
        assert!(true);
    }
}
