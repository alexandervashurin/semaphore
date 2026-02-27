//! Ansible Models
//!
//! Ansible модели для Semaphore

use serde::{Deserialize, Serialize};

/// Ansible Playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsiblePlaybook {
    /// Название playbook
    pub name: String,

    /// Путь к playbook
    pub path: String,
}

/// Ansible Galaxy Requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsibleGalaxyRequirements {
    /// Роли
    #[serde(default)]
    pub roles: Vec<GalaxyRequirement>,

    /// Коллекции
    #[serde(default)]
    pub collections: Vec<GalaxyRequirement>,
}

/// Galaxy Requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyRequirement {
    /// Название
    pub name: String,

    /// Версия
    #[serde(default)]
    pub version: String,
}
