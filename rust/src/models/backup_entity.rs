//! Backup Entity Trait
//!
//! Трейт для сущностей бэкапа

use serde::{de::DeserializeOwned, Serialize};

/// Trait для сущностей, поддерживающих бэкап
pub trait BackupEntity: Serialize + DeserializeOwned + Clone {
    /// Получает название сущности
    fn get_name(&self) -> &str;

    /// Получает тип сущности
    fn get_type() -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(crate = "serde")]
    struct TestBackupEntity {
        name: String,
        entity_type: String,
    }

    impl TestBackupEntity {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                entity_type: "test".to_string(),
            }
        }
    }

    impl BackupEntity for TestBackupEntity {
        fn get_name(&self) -> &str {
            &self.name
        }

        fn get_type() -> &'static str {
            "test_entity"
        }
    }

    #[test]
    fn test_backup_entity_get_name() {
        let entity = TestBackupEntity::new("My Entity");
        assert_eq!(entity.get_name(), "My Entity");
    }

    #[test]
    fn test_backup_entity_get_type() {
        assert_eq!(<TestBackupEntity as BackupEntity>::get_type(), "test_entity");
    }

    #[test]
    fn test_backup_entity_serialization() {
        let entity = TestBackupEntity::new("Serializable Entity");
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"name\":\"Serializable Entity\""));
    }

    #[test]
    fn test_backup_entity_deserialization() {
        let json = r#"{"name":"Deserialized Entity","entity_type":"test"}"#;
        let entity: TestBackupEntity = serde_json::from_str(json).unwrap();
        assert_eq!(entity.get_name(), "Deserialized Entity");
    }

    #[test]
    fn test_backup_entity_clone() {
        let entity = TestBackupEntity::new("Clone Me");
        let cloned = entity.clone();
        assert_eq!(cloned.get_name(), entity.get_name());
    }
}
