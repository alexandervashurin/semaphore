//! LDAP Group Mapping Model
//!
//! Маппинг LDAP-групп на проекты Velum с ролями.
//! При логине через LDAP — автоматически добавляет пользователя в проект.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Маппинг LDAP-группы → проект/роль
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LdapGroupMapping {
    pub id: i32,
    /// DN LDAP-группы, например: CN=devops,OU=Groups,DC=company,DC=com
    pub ldap_group_dn: String,
    pub project_id: i32,
    /// Роль в проекте: owner / manager / task:runner
    pub role: String,
    pub created_at: String,
    /// Название проекта (joined, optional)
    #[sqlx(default)]
    pub project_name: String,
}

/// Создание нового маппинга
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapGroupMappingCreate {
    pub ldap_group_dn: String,
    pub project_id: i32,
    pub role: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ldap_group_mapping_serialization() {
        let mapping = LdapGroupMapping {
            id: 1,
            ldap_group_dn: "CN=devops,OU=Groups,DC=company,DC=com".to_string(),
            project_id: 10,
            role: "manager".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            project_name: "My Project".to_string(),
        };
        let json = serde_json::to_string(&mapping).unwrap();
        assert!(json.contains("\"ldap_group_dn\":\"CN=devops"));
        assert!(json.contains("\"role\":\"manager\""));
    }

    #[test]
    fn test_ldap_group_mapping_create_serialization() {
        let create = LdapGroupMappingCreate {
            ldap_group_dn: "CN=admin,DC=example,DC=com".to_string(),
            project_id: 5,
            role: "owner".to_string(),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"ldap_group_dn\":\"CN=admin"));
        assert!(json.contains("\"project_id\":5"));
    }

    #[test]
    fn test_ldap_group_mapping_clone() {
        let mapping = LdapGroupMapping {
            id: 1,
            ldap_group_dn: "CN=test,DC=example,DC=com".to_string(),
            project_id: 10,
            role: "manager".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            project_name: "Test Project".to_string(),
        };
        let cloned = mapping.clone();
        assert_eq!(cloned.ldap_group_dn, mapping.ldap_group_dn);
        assert_eq!(cloned.role, mapping.role);
    }

    #[test]
    fn test_ldap_group_mapping_create_clone() {
        let create = LdapGroupMappingCreate {
            ldap_group_dn: "CN=users,DC=example,DC=com".to_string(),
            project_id: 1,
            role: "task_runner".to_string(),
        };
        let cloned = create.clone();
        assert_eq!(cloned.ldap_group_dn, create.ldap_group_dn);
        assert_eq!(cloned.project_id, create.project_id);
    }

    #[test]
    fn test_ldap_group_mapping_debug() {
        let mapping = LdapGroupMapping {
            id: 1,
            ldap_group_dn: "CN=test".to_string(),
            project_id: 1,
            role: "manager".to_string(),
            created_at: "2024-01-01".to_string(),
            project_name: "Debug".to_string(),
        };
        let debug_str = format!("{:?}", mapping);
        assert!(debug_str.contains("LdapGroupMapping"));
    }

    #[test]
    fn test_ldap_group_mapping_create_debug() {
        let create = LdapGroupMappingCreate {
            ldap_group_dn: "CN=debug".to_string(),
            project_id: 1,
            role: "owner".to_string(),
        };
        let debug_str = format!("{:?}", create);
        assert!(debug_str.contains("LdapGroupMappingCreate"));
    }

    #[test]
    fn test_ldap_group_mapping_deserialization() {
        let json = r#"{"id":10,"ldap_group_dn":"CN=admins,DC=test,DC=com","project_id":50,"role":"owner","created_at":"2024-06-01T00:00:00Z","project_name":"Admin Project"}"#;
        let mapping: LdapGroupMapping = serde_json::from_str(json).unwrap();
        assert_eq!(mapping.id, 10);
        assert_eq!(mapping.project_id, 50);
        assert_eq!(mapping.role, "owner");
    }

    #[test]
    fn test_ldap_group_mapping_create_deserialization() {
        let json =
            r#"{"ldap_group_dn":"CN=newgroup,DC=test,DC=com","project_id":5,"role":"manager"}"#;
        let create: LdapGroupMappingCreate = serde_json::from_str(json).unwrap();
        assert_eq!(create.ldap_group_dn, "CN=newgroup,DC=test,DC=com");
        assert_eq!(create.role, "manager");
    }

    #[test]
    fn test_ldap_group_mapping_all_roles() {
        let roles = ["owner", "manager", "task_runner"];
        for role in roles {
            let mapping = LdapGroupMapping {
                id: 1,
                ldap_group_dn: "CN=test".to_string(),
                project_id: 1,
                role: role.to_string(),
                created_at: "2024-01-01".to_string(),
                project_name: "Test".to_string(),
            };
            let json = serde_json::to_string(&mapping).unwrap();
            assert!(json.contains(&format!("\"role\":\"{}\"", role)));
        }
    }

    #[test]
    fn test_ldap_group_mapping_empty_project_name() {
        let mapping = LdapGroupMapping {
            id: 1,
            ldap_group_dn: "CN=test".to_string(),
            project_id: 1,
            role: "manager".to_string(),
            created_at: "2024-01-01".to_string(),
            project_name: String::new(),
        };
        let json = serde_json::to_string(&mapping).unwrap();
        assert!(json.contains("\"project_name\":\"\""));
    }

    #[test]
    fn test_ldap_group_mapping_create_equality() {
        let a = LdapGroupMappingCreate {
            ldap_group_dn: "CN=same".to_string(),
            project_id: 1,
            role: "owner".to_string(),
        };
        let b = a.clone();
        assert_eq!(a.ldap_group_dn, b.ldap_group_dn);
        assert_eq!(a.project_id, b.project_id);
        assert_eq!(a.role, b.role);
    }

    #[test]
    fn test_ldap_group_mapping_unicode_dn() {
        let mapping = LdapGroupMapping {
            id: 1,
            ldap_group_dn: "CN=Группа,OU=Пользователи,DC=example,DC=com".to_string(),
            project_id: 1,
            role: "manager".to_string(),
            created_at: "2024-01-01".to_string(),
            project_name: "Тест".to_string(),
        };
        let json = serde_json::to_string(&mapping).unwrap();
        let restored: LdapGroupMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(
            restored.ldap_group_dn,
            "CN=Группа,OU=Пользователи,DC=example,DC=com"
        );
    }

    #[test]
    fn test_ldap_group_mapping_clone_independence() {
        let mut mapping = LdapGroupMapping {
            id: 1,
            ldap_group_dn: "CN=original".to_string(),
            project_id: 1,
            role: "owner".to_string(),
            created_at: "2024-01-01".to_string(),
            project_name: "Test".to_string(),
        };
        let cloned = mapping.clone();
        mapping.ldap_group_dn = "CN=modified".to_string();
        assert_eq!(cloned.ldap_group_dn, "CN=original");
    }

    #[test]
    fn test_ldap_group_mapping_create_roundtrip() {
        let original = LdapGroupMappingCreate {
            ldap_group_dn: "CN=roundtrip,DC=test,DC=com".to_string(),
            project_id: 42,
            role: "task_runner".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: LdapGroupMappingCreate = serde_json::from_str(&json).unwrap();
        assert_eq!(original.ldap_group_dn, restored.ldap_group_dn);
        assert_eq!(original.project_id, restored.project_id);
        assert_eq!(original.role, restored.role);
    }

    #[test]
    fn test_ldap_group_mapping_debug_contains_fields() {
        let mapping = LdapGroupMapping {
            id: 99,
            ldap_group_dn: "CN=debug".to_string(),
            project_id: 50,
            role: "owner".to_string(),
            created_at: "2024-01-01".to_string(),
            project_name: "Debug Project".to_string(),
        };
        let debug_str = format!("{:?}", mapping);
        assert!(debug_str.contains("99"));
        assert!(debug_str.contains("Debug Project"));
        assert!(debug_str.contains("LdapGroupMapping"));
    }
}
