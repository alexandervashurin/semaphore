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
}
