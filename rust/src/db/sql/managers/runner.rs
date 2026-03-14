//! RunnerManager - управление раннерами

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Runner;
use async_trait::async_trait;

#[async_trait]
impl RunnerManager for SqlStore {
    async fn get_runners(&self, _project_id: Option<i32>) -> Result<Vec<Runner>> { Ok(vec![]) }
    async fn get_runner(&self, _runner_id: i32) -> Result<Runner> { Err(Error::NotFound("Раннер не найден".to_string())) }
    async fn create_runner(&self, _runner: Runner) -> Result<Runner> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_runner(&self, _runner: Runner) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_runner(&self, _runner_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

