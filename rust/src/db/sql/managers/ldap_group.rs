//! LDAP Group Mapping SQL Manager

use crate::db::sql::SqlStore;
use crate::db::store::LdapGroupMappingManager;
use crate::error::{Error, Result};
use crate::models::ldap_group::{LdapGroupMapping, LdapGroupMappingCreate};
use async_trait::async_trait;

#[async_trait]
impl LdapGroupMappingManager for SqlStore {
    async fn get_ldap_group_mappings(&self) -> Result<Vec<LdapGroupMapping>> {
        let rows = sqlx::query_as::<_, LdapGroupMapping>(
            r#"SELECT lgm.id, lgm.ldap_group_dn, lgm.project_id, lgm.role, lgm.created_at::text,
                          COALESCE(p.name,'') AS project_name
                   FROM ldap_group_mapping lgm
                   LEFT JOIN project p ON p.id = lgm.project_id
                   ORDER BY lgm.id"#,
        )
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(rows)
    }

    async fn create_ldap_group_mapping(
        &self,
        payload: LdapGroupMappingCreate,
    ) -> Result<LdapGroupMapping> {
        let row = sqlx::query_as::<_, LdapGroupMapping>(
                r#"INSERT INTO ldap_group_mapping (ldap_group_dn, project_id, role)
                   VALUES ($1, $2, $3)
                   RETURNING id, ldap_group_dn, project_id, role, created_at::text, '' AS project_name"#
            )
            .bind(&payload.ldap_group_dn)
            .bind(payload.project_id)
            .bind(&payload.role)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(row)
    }

    async fn delete_ldap_group_mapping(&self, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM ldap_group_mapping WHERE id = $1")
            .bind(id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_mappings_for_groups(&self, group_dns: &[String]) -> Result<Vec<LdapGroupMapping>> {
        if group_dns.is_empty() {
            return Ok(vec![]);
        }
        let all = self.get_ldap_group_mappings().await?;
        Ok(all
            .into_iter()
            .filter(|m| group_dns.contains(&m.ldap_group_dn))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::ldap_group::{LdapGroupMapping, LdapGroupMappingCreate};

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
    fn test_ldap_group_mapping_deserialize() {
        let json = r#"{"id":3,"ldap_group_dn":"CN=admins,DC=test,DC=com","project_id":2,"role":"owner","created_at":"2024-06-01T00:00:00Z","project_name":"Admin Project"}"#;
        let mapping: LdapGroupMapping = serde_json::from_str(json).unwrap();
        assert_eq!(mapping.id, 3);
        assert_eq!(mapping.role, "owner");
        assert_eq!(mapping.project_name, "Admin Project");
    }

    #[test]
    fn test_ldap_group_mapping_create_deserialize() {
        let json =
            r#"{"ldap_group_dn":"CN=dev,DC=test,DC=com","project_id":7,"role":"task_runner"}"#;
        let create: LdapGroupMappingCreate = serde_json::from_str(json).unwrap();
        assert_eq!(create.project_id, 7);
        assert_eq!(create.role, "task_runner");
    }

    #[test]
    fn test_ldap_group_mapping_different_roles() {
        let roles = ["owner", "manager", "task_runner", "viewer"];
        for role in roles {
            let create = LdapGroupMappingCreate {
                ldap_group_dn: "CN=test,DC=com".to_string(),
                project_id: 1,
                role: role.to_string(),
            };
            let json = serde_json::to_string(&create).unwrap();
            assert!(json.contains(&format!("\"role\":\"{}\"", role)));
        }
    }

    #[test]
    fn test_ldap_group_mapping_empty_project_name() {
        let mapping = LdapGroupMapping {
            id: 1,
            ldap_group_dn: "CN=test,DC=com".to_string(),
            project_id: 1,
            role: "owner".to_string(),
            created_at: "2024-01-01".to_string(),
            project_name: String::new(),
        };
        assert!(mapping.project_name.is_empty());
    }
}
