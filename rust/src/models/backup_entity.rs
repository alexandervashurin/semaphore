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
        assert_eq!(
            <TestBackupEntity as BackupEntity>::get_type(),
            "test_entity"
        );
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

    #[test]
    fn test_backup_entity_empty_name() {
        let entity = TestBackupEntity::new("");
        assert_eq!(entity.get_name(), "");
        assert_eq!(TestBackupEntity::get_type(), "test_entity");
    }

    #[test]
    fn test_backup_entity_special_chars() {
        let entity = TestBackupEntity::new("Entity with 'quotes' & <special> chars");
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("quotes"));
    }

    #[test]
    fn test_backup_entity_roundtrip() {
        let original = TestBackupEntity {
            name: "Roundtrip Entity".to_string(),
            entity_type: "test_roundtrip".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: TestBackupEntity = serde_json::from_str(&json).unwrap();
        assert_eq!(original.name, restored.name);
        assert_eq!(original.entity_type, restored.entity_type);
    }

    #[test]
    fn test_backup_entity_long_name() {
        let long_name = "A".repeat(1000);
        let entity = TestBackupEntity::new(&long_name);
        assert_eq!(entity.get_name().len(), 1000);
    }

    #[test]
    fn test_backup_entity_debug() {
        let entity = TestBackupEntity::new("Debug Entity");
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("TestBackupEntity"));
        assert!(debug_str.contains("Debug Entity"));
    }

    #[test]
    fn test_backup_entity_unicode_name() {
        let entity = TestBackupEntity::new("Тестовая сущность 🚀");
        let json = serde_json::to_string(&entity).unwrap();
        let restored: TestBackupEntity = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.get_name(), "Тестовая сущность 🚀");
    }

    #[test]
    fn test_backup_entity_clone_independence() {
        let mut entity = TestBackupEntity::new("Original");
        let cloned = entity.clone();
        entity.name = "Modified".to_string();
        assert_eq!(cloned.get_name(), "Original");
    }

    #[test]
    fn test_backup_entity_newline_in_name() {
        let entity = TestBackupEntity::new("Line1\nLine2\tTab");
        let json = serde_json::to_string(&entity).unwrap();
        let restored: TestBackupEntity = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.get_name(), "Line1\nLine2\tTab");
    }

    #[test]
    fn test_backup_entity_debug_format() {
        let entity = TestBackupEntity {
            name: "DebugTest".to_string(),
            entity_type: "debug_type".to_string(),
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("DebugTest"));
        assert!(debug_str.contains("TestBackupEntity"));
    }

    #[test]
    fn test_backup_entity_type_field_serialization() {
        let entity = TestBackupEntity::new("Test");
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"entity_type\":\"test\""));
    }
}
