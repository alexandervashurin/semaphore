//! Модель роли

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Роль - набор разрешений
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: i32,
    pub project_id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    /// Bitmask разрешений (i32)
    pub permissions: Option<i32>,
}

/// Разрешения роли (bitmask flags)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RolePermissions {
    pub run_tasks: bool,
    pub update_resources: bool,
    pub manage_project: bool,
    pub manage_users: bool,
    pub manage_roles: bool,
    pub view_audit_log: bool,
    pub manage_integrations: bool,
    pub manage_secret_storages: bool,
}

impl RolePermissions {
    /// Создаёт разрешения из bitmask
    pub fn from_bitmask(bitmask: i32) -> Self {
        Self {
            run_tasks: (bitmask & 0b0000_0001) != 0,
            update_resources: (bitmask & 0b0000_0010) != 0,
            manage_project: (bitmask & 0b0000_0100) != 0,
            manage_users: (bitmask & 0b0000_1000) != 0,
            manage_roles: (bitmask & 0b0001_0000) != 0,
            view_audit_log: (bitmask & 0b0010_0000) != 0,
            manage_integrations: (bitmask & 0b0100_0000) != 0,
            manage_secret_storages: (bitmask & 0b1000_0000) != 0,
        }
    }

    /// Преобразует разрешения в bitmask
    pub fn to_bitmask(&self) -> i32 {
        let mut mask = 0;
        if self.run_tasks {
            mask |= 0b0000_0001;
        }
        if self.update_resources {
            mask |= 0b0000_0010;
        }
        if self.manage_project {
            mask |= 0b0000_0100;
        }
        if self.manage_users {
            mask |= 0b0000_1000;
        }
        if self.manage_roles {
            mask |= 0b0001_0000;
        }
        if self.view_audit_log {
            mask |= 0b0010_0000;
        }
        if self.manage_integrations {
            mask |= 0b0100_0000;
        }
        if self.manage_secret_storages {
            mask |= 0b1000_0000;
        }
        mask
    }

    /// Полные права (admin)
    pub fn admin() -> Self {
        Self {
            run_tasks: true,
            update_resources: true,
            manage_project: true,
            manage_users: true,
            manage_roles: true,
            view_audit_log: true,
            manage_integrations: true,
            manage_secret_storages: true,
        }
    }
}

impl Default for RolePermissions {
    fn default() -> Self {
        Self {
            run_tasks: true,
            update_resources: false,
            manage_project: false,
            manage_users: false,
            manage_roles: false,
            view_audit_log: false,
            manage_integrations: false,
            manage_secret_storages: false,
        }
    }
}

impl Role {
    /// Создаёт новую роль
    pub fn new(project_id: i32, slug: String, name: String) -> Self {
        Self {
            id: 0,
            project_id,
            slug,
            name,
            description: None,
            permissions: Some(0),
        }
    }

    /// Создаёт новую роль с разрешениями
    pub fn new_with_permissions(
        project_id: i32,
        slug: String,
        name: String,
        permissions: i32,
    ) -> Self {
        Self {
            id: 0,
            project_id,
            slug,
            name,
            description: None,
            permissions: Some(permissions),
        }
    }

    /// Получает разрешения из bitmask
    pub fn get_permissions(&self) -> RolePermissions {
        RolePermissions::from_bitmask(self.permissions.unwrap_or(0))
    }

    /// Устанавливает разрешения
    pub fn set_permissions(&mut self, perms: RolePermissions) {
        self.permissions = Some(perms.to_bitmask());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_permissions_from_bitmask() {
        let perms = RolePermissions::from_bitmask(0b0000_0111);
        assert!(perms.run_tasks);
        assert!(perms.update_resources);
        assert!(perms.manage_project);
        assert!(!perms.manage_users);
    }

    #[test]
    fn test_role_permissions_to_bitmask() {
        let perms = RolePermissions {
            run_tasks: true,
            update_resources: true,
            manage_project: false,
            manage_users: false,
            manage_roles: false,
            view_audit_log: false,
            manage_integrations: false,
            manage_secret_storages: false,
        };
        assert_eq!(perms.to_bitmask(), 0b0000_0011);
    }

    #[test]
    fn test_role_permissions_admin() {
        let admin = RolePermissions::admin();
        assert_eq!(admin.to_bitmask(), 0b1111_1111);
    }

    #[test]
    fn test_role_permissions_default() {
        let default = RolePermissions::default();
        assert!(default.run_tasks);
        assert!(!default.update_resources);
        assert!(!default.manage_project);
        assert_eq!(default.to_bitmask(), 0b0000_0001);
    }

    #[test]
    fn test_role_new() {
        let role = Role::new(10, "developer".to_string(), "Developer".to_string());
        assert_eq!(role.id, 0);
        assert_eq!(role.project_id, 10);
        assert_eq!(role.slug, "developer");
        assert!(role.description.is_none());
    }

    #[test]
    fn test_role_with_permissions() {
        let role = Role::new_with_permissions(5, "admin".to_string(), "Admin".to_string(), 0b1111_1111);
        let perms = role.get_permissions();
        assert!(perms.run_tasks);
        assert!(perms.manage_secret_storages);
    }

    #[test]
    fn test_role_get_set_permissions() {
        let mut role = Role::new(1, "custom".to_string(), "Custom".to_string());
        let perms = RolePermissions::admin();
        role.set_permissions(perms);
        assert_eq!(role.permissions, Some(0b1111_1111));
        let retrieved = role.get_permissions();
        assert!(retrieved.manage_users);
    }

    #[test]
    fn test_role_serialization() {
        let role = Role {
            id: 1,
            project_id: 10,
            slug: "manager".to_string(),
            name: "Manager".to_string(),
            description: Some("Can manage resources".to_string()),
            permissions: Some(0b0000_0010),
        };
        let json = serde_json::to_string(&role).unwrap();
        assert!(json.contains("\"slug\":\"manager\""));
        assert!(json.contains("\"permissions\":2"));
    }

    #[test]
    fn test_role_permissions_from_zero_bitmask() {
        let perms = RolePermissions::from_bitmask(0);
        assert!(!perms.run_tasks);
        assert!(!perms.update_resources);
        assert!(!perms.manage_project);
        assert!(!perms.manage_users);
        assert!(!perms.manage_roles);
        assert!(!perms.view_audit_log);
        assert!(!perms.manage_integrations);
        assert!(!perms.manage_secret_storages);
    }

    #[test]
    fn test_role_permissions_serialization() {
        let perms = RolePermissions::admin();
        let json = serde_json::to_string(&perms).unwrap();
        assert!(json.contains("\"run_tasks\":true"));
        assert!(json.contains("\"manage_secret_storages\":true"));
    }

    #[test]
    fn test_role_permissions_deserialization() {
        let json = r#"{"run_tasks":true,"update_resources":false,"manage_project":true,"manage_users":false,"manage_roles":false,"view_audit_log":false,"manage_integrations":false,"manage_secret_storages":false}"#;
        let perms: RolePermissions = serde_json::from_str(json).unwrap();
        assert!(perms.run_tasks);
        assert!(!perms.update_resources);
        assert!(perms.manage_project);
        assert_eq!(perms.to_bitmask(), 0b0000_0101);
    }

    #[test]
    fn test_role_clone() {
        let role = Role::new_with_permissions(1, "clone".to_string(), "Clone".to_string(), 0b1111_1111);
        let cloned = role.clone();
        assert_eq!(cloned.slug, role.slug);
        assert_eq!(cloned.permissions, role.permissions);
    }
}
