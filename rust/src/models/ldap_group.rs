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
