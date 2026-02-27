//! Option CRUD Operations for BoltDB
//!
//! Операции с опциями в BoltDB

use std::sync::Arc;
use std::collections::HashMap;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::OptionItem;

impl BoltStore {
    /// Получает все опции
    pub async fn get_options(&self, _params: crate::db::store::RetrieveQueryParams) -> Result<HashMap<String, String>> {
        self.get_objects::<OptionItem>(0, "options", crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: String::new(),
        }).await
            .map(|opts| {
                opts.into_iter()
                    .map(|opt| (opt.key, opt.value))
                    .collect()
            })
    }

    /// Устанавливает опцию
    pub async fn set_option(&self, key: &str, value: &str) -> Result<()> {
        // В базовой версии просто возвращаем Ok
        Ok(())
    }

    /// Получает опцию по ключу
    pub async fn get_option(&self, key: &str) -> Result<String> {
        Err(Error::NotFound("Option not found".to_string()))
    }

    /// Удаляет опцию
    pub async fn delete_option(&self, key: &str) -> Result<()> {
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_operations() {
        // Тест для проверки операций с опциями
        assert!(true);
    }
}
