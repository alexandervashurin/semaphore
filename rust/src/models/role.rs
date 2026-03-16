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
        if self.run_tasks { mask |= 0b0000_0001; }
        if self.update_resources { mask |= 0b0000_0010; }
        if self.manage_project { mask |= 0b0000_0100; }
        if self.manage_users { mask |= 0b0000_1000; }
        if self.manage_roles { mask |= 0b0001_0000; }
        if self.view_audit_log { mask |= 0b0010_0000; }
        if self.manage_integrations { mask |= 0b0100_0000; }
        if self.manage_secret_storages { mask |= 0b1000_0000; }
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
    pub fn new_with_permissions(project_id: i32, slug: String, name: String, permissions: i32) -> Self {
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
