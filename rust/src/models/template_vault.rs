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
}
