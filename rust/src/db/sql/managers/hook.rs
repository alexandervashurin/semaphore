//! HookManager - управление хуками

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Hook;
use async_trait::async_trait;

#[async_trait]
impl HookManager for SqlStore {
    async fn get_hooks_by_template(&self, template_id: i32) -> Result<Vec<Hook>> {
        // Заглушка - возвращаем пустой список
        Ok(Vec::new())
    }
}

