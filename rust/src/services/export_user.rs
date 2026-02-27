//! User Exporter
//!
//! Экспорт пользователей

use crate::error::{Error, Result};
use crate::models::User;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр пользователей
pub struct UserExporter {
    /// Карта значений
    pub value_map: ValueMap<User>,
}

impl UserExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for UserExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for UserExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, _exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        let users = store.get_users(crate::db::store::RetrieveQueryParams::default())
            .map_err(|e| Error::Other(format!("Failed to load users: {}", e)))?;

        for user in users {
            self.value_map.append_value(user, "global".to_string())?;
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        for val in &self.value_map.values {
            let old = val.value.clone();

            // Создаём пользователя с паролем
            let user_with_pwd = crate::models::user::UserWithPwd {
                pwd: old.password.clone(),
                user: old,
            };

            let obj = store.import_user(user_with_pwd)
                .map_err(|e| Error::Other(format!("Failed to import user: {}", e)))?;

            exporter.map_int_keys(self.get_name(), "global", val.value.id, obj.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "User"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec![]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec![]
    }

    /// Получает ошибки
    fn get_errors(&self) -> Vec<String> {
        self.value_map.errors.clone()
    }

    /// Очищает
    fn clear(&mut self) {
        self.value_map.clear();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_exporter_creation() {
        let exporter = UserExporter::new();
        assert_eq!(exporter.get_name(), "User");
    }

    #[test]
    fn test_user_exporter_depends_on() {
        let exporter = UserExporter::new();
        assert!(exporter.export_depends_on().is_empty());
    }
}
