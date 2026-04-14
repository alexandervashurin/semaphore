//! Модель TemplateVault

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// TemplateVault - хранилище секретов для шаблона
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateVault {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID шаблона
    pub template_id: i32,

    /// ID проекта
    pub project_id: i32,

    /// ID хранилища секретов
    pub vault_id: i32,

    /// ID ключа доступа к хранилищу
    pub vault_key_id: i32,

    /// Название хранилища
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_vault_serialization() {
        let vault = TemplateVault {
            id: 1,
            template_id: 10,
            project_id: 5,
            vault_id: 3,
            vault_key_id: 2,
            name: "Production Vault".to_string(),
        };
        let json = serde_json::to_string(&vault).unwrap();
        assert!(json.contains("\"name\":\"Production Vault\""));
        assert!(json.contains("\"vault_id\":3"));
    }

    #[test]
    fn test_template_vault_clone() {
        let vault = TemplateVault {
            id: 1,
            template_id: 10,
            project_id: 5,
            vault_id: 3,
            vault_key_id: 2,
            name: "Test Vault".to_string(),
        };
        let cloned = vault.clone();
        assert_eq!(cloned.id, vault.id);
        assert_eq!(cloned.name, vault.name);
    }

    #[test]
    fn test_template_vault_deserialization() {
        let json = r#"{"id":5,"template_id":20,"project_id":15,"vault_id":8,"vault_key_id":3,"name":"Dev Vault"}"#;
        let vault: TemplateVault = serde_json::from_str(json).unwrap();
        assert_eq!(vault.id, 5);
        assert_eq!(vault.name, "Dev Vault");
    }

    #[test]
    fn test_template_vault_debug() {
        let vault = TemplateVault {
            id: 1,
            template_id: 10,
            project_id: 5,
            vault_id: 3,
            vault_key_id: 2,
            name: "Debug Vault".to_string(),
        };
        let debug_str = format!("{:?}", vault);
        assert!(debug_str.contains("TemplateVault"));
        assert!(debug_str.contains("Debug Vault"));
    }

    #[test]
    fn test_template_vault_all_fields() {
        let vault = TemplateVault {
            id: 99,
            template_id: 200,
            project_id: 50,
            vault_id: 10,
            vault_key_id: 7,
            name: "Full Vault".to_string(),
        };
        let json = serde_json::to_string(&vault).unwrap();
        assert!(json.contains("\"id\":99"));
        assert!(json.contains("\"template_id\":200"));
        assert!(json.contains("\"project_id\":50"));
        assert!(json.contains("\"vault_key_id\":7"));
    }

    #[test]
    fn test_template_vault_empty_name() {
        let vault = TemplateVault {
            id: 1,
            template_id: 1,
            project_id: 1,
            vault_id: 1,
            vault_key_id: 1,
            name: String::new(),
        };
        let json = serde_json::to_string(&vault).unwrap();
        assert!(json.contains("\"name\":\"\""));
    }

    #[test]
    fn test_template_vault_deserialization_full() {
        let json = r#"{"id":10,"template_id":100,"project_id":50,"vault_id":20,"vault_key_id":15,"name":"Prod Vault"}"#;
        let vault: TemplateVault = serde_json::from_str(json).unwrap();
        assert_eq!(vault.id, 10);
        assert_eq!(vault.template_id, 100);
        assert_eq!(vault.project_id, 50);
        assert_eq!(vault.vault_id, 20);
        assert_eq!(vault.vault_key_id, 15);
    }

    #[test]
    fn test_template_vault_special_chars_name() {
        let vault = TemplateVault {
            id: 1,
            template_id: 1,
            project_id: 1,
            vault_id: 1,
            vault_key_id: 1,
            name: "Vault & <secrets> \"quoted\"".to_string(),
        };
        let json = serde_json::to_string(&vault).unwrap();
        let deserialized: TemplateVault = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Vault & <secrets> \"quoted\"");
    }

    #[test]
    fn test_template_vault_zero_ids() {
        let vault = TemplateVault {
            id: 0,
            template_id: 0,
            project_id: 0,
            vault_id: 0,
            vault_key_id: 0,
            name: "Zero".to_string(),
        };
        assert_eq!(vault.id, 0);
        assert_eq!(vault.template_id, 0);
    }

    #[test]
    fn test_template_vault_roundtrip() {
        let original = TemplateVault {
            id: 7,
            template_id: 14,
            project_id: 21,
            vault_id: 28,
            vault_key_id: 35,
            name: "Roundtrip Vault".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: TemplateVault = serde_json::from_str(&json).unwrap();
        assert_eq!(original.id, restored.id);
        assert_eq!(original.name, restored.name);
        assert_eq!(original.vault_key_id, restored.vault_key_id);
    }

    #[test]
    fn test_template_vault_unicode_name() {
        let vault = TemplateVault {
            id: 1,
            template_id: 1,
            project_id: 1,
            vault_id: 1,
            vault_key_id: 1,
            name: "Хранилище секретов".to_string(),
        };
        let json = serde_json::to_string(&vault).unwrap();
        let restored: TemplateVault = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "Хранилище секретов");
    }

    #[test]
    fn test_template_vault_clone_independence() {
        let mut vault = TemplateVault {
            id: 1,
            template_id: 1,
            project_id: 1,
            vault_id: 1,
            vault_key_id: 1,
            name: "Original".to_string(),
        };
        let cloned = vault.clone();
        vault.name = "Modified".to_string();
        assert_eq!(cloned.name, "Original");
    }

    #[test]
    fn test_template_vault_large_ids() {
        let vault = TemplateVault {
            id: i32::MAX,
            template_id: i32::MAX,
            project_id: i32::MAX,
            vault_id: i32::MAX,
            vault_key_id: i32::MAX,
            name: "Max IDs".to_string(),
        };
        let json = serde_json::to_string(&vault).unwrap();
        let restored: TemplateVault = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, i32::MAX);
        assert_eq!(restored.template_id, i32::MAX);
    }

    #[test]
    fn test_template_vault_newline_in_name() {
        let vault = TemplateVault {
            id: 1,
            template_id: 1,
            project_id: 1,
            vault_id: 1,
            vault_key_id: 1,
            name: "Line1\nLine2".to_string(),
        };
        let json = serde_json::to_string(&vault).unwrap();
        let restored: TemplateVault = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "Line1\nLine2");
    }

    #[test]
    fn test_template_vault_debug_format() {
        let vault = TemplateVault {
            id: 99,
            template_id: 100,
            project_id: 200,
            vault_id: 300,
            vault_key_id: 400,
            name: "DebugTest".to_string(),
        };
        let debug_str = format!("{:?}", vault);
        assert!(debug_str.contains("99"));
        assert!(debug_str.contains("DebugTest"));
        assert!(debug_str.contains("TemplateVault"));
    }
}
