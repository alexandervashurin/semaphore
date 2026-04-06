//! Ansible Models
//!
//! Ansible модели для Velum

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

impl Default for AnsibleGalaxyRequirements {
    fn default() -> Self {
        Self {
            roles: Vec::new(),
            collections: Vec::new(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansible_playbook_serialization() {
        let playbook = AnsiblePlaybook {
            name: "site.yml".to_string(),
            path: "/playbooks/site.yml".to_string(),
        };
        let json = serde_json::to_string(&playbook).unwrap();
        assert!(json.contains("\"name\":\"site.yml\""));
        assert!(json.contains("\"path\":\"/playbooks/site.yml\""));
    }

    #[test]
    fn test_ansible_playbook_deserialization() {
        let json = r#"{"name":"deploy.yml","path":"/deploy.yml"}"#;
        let playbook: AnsiblePlaybook = serde_json::from_str(json).unwrap();
        assert_eq!(playbook.name, "deploy.yml");
        assert_eq!(playbook.path, "/deploy.yml");
    }

    #[test]
    fn test_galaxy_requirements_defaults() {
        let req = AnsibleGalaxyRequirements::default();
        assert!(req.roles.is_empty());
        assert!(req.collections.is_empty());
    }

    #[test]
    fn test_galaxy_requirements_serialization() {
        let req = AnsibleGalaxyRequirements {
            roles: vec![GalaxyRequirement {
                name: "geerlingguy.nginx".to_string(),
                version: "1.0.0".to_string(),
            }],
            collections: vec![],
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"roles\":["));
        assert!(json.contains("\"name\":\"geerlingguy.nginx\""));
        assert!(json.contains("\"collections\":[]"));
    }

    #[test]
    fn test_galaxy_requirements_deserialization() {
        let json = r#"{"roles":[{"name":"geerlingguy.docker","version":"2.0.0"}],"collections":[{"name":"community.general","version":"3.0.0"}]}"#;
        let req: AnsibleGalaxyRequirements = serde_json::from_str(json).unwrap();
        assert_eq!(req.roles.len(), 1);
        assert_eq!(req.roles[0].name, "geerlingguy.docker");
        assert_eq!(req.collections.len(), 1);
        assert_eq!(req.collections[0].name, "community.general");
    }

    #[test]
    fn test_galaxy_requirement_default_version() {
        let req: GalaxyRequirement = serde_json::from_str(r#"{"name":"test.role"}"#).unwrap();
        assert_eq!(req.name, "test.role");
        assert_eq!(req.version, "");
    }

    #[test]
    fn test_ansible_playbook_clone() {
        let playbook = AnsiblePlaybook {
            name: "test.yml".to_string(),
            path: "/test.yml".to_string(),
        };
        let cloned = playbook.clone();
        assert_eq!(cloned.name, playbook.name);
        assert_eq!(cloned.path, playbook.path);
    }
}
