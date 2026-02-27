//! Backup Entity Trait
//!
//! Трейт для сущностей бэкапа

use serde::{Serialize, de::DeserializeOwned};

/// Trait для сущностей, поддерживающих бэкап
pub trait BackupEntity: Serialize + DeserializeOwned + Clone {
    /// Получает название сущности
    fn get_name(&self) -> &str;

    /// Получает тип сущности
    fn get_type() -> &'static str;
}
