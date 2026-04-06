//! Task Params Models
//!
//! Параметры задач

use serde::{Deserialize, Serialize};

/// Параметры задачи Ansible
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnsibleTaskParams {
    /// Debug режим
    #[serde(default)]
    pub debug: bool,

    /// Уровень debug
    #[serde(default)]
    pub debug_level: i32,

    /// Dry run
    #[serde(default)]
    pub dry_run: bool,

    /// Diff режим
    #[serde(default)]
    pub diff: bool,

    /// Ограничения (limit)
    #[serde(default)]
    pub limit: Vec<String>,

    /// Теги
    #[serde(default)]
    pub tags: Vec<String>,

    /// Пропускаемые теги
    #[serde(default)]
    pub skip_tags: Vec<String>,
}

/// Параметры задачи Terraform
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerraformTaskParams {
    /// Plan
    #[serde(default)]
    pub plan: bool,

    /// Destroy
    #[serde(default)]
    pub destroy: bool,

    /// Auto approve
    #[serde(default)]
    pub auto_approve: bool,

    /// Upgrade
    #[serde(default)]
    pub upgrade: bool,

    /// Reconfigure
    #[serde(default)]
    pub reconfigure: bool,

    /// Backend init required
    #[serde(default)]
    pub backend_init_required: bool,

    /// Backend config
    #[serde(default)]
    pub backend_config: Option<String>,

    /// Workspace
    #[serde(default)]
    pub workspace: Option<String>,
}

/// Параметры задачи по умолчанию
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultTaskParams {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansible_task_params_default() {
        let params = AnsibleTaskParams::default();
        assert!(!params.debug);
        assert_eq!(params.debug_level, 0);
        assert!(!params.dry_run);
        assert!(!params.diff);
        assert!(params.limit.is_empty());
        assert!(params.tags.is_empty());
    }

    #[test]
    fn test_ansible_task_params_serialization() {
        let params = AnsibleTaskParams {
            debug: true,
            debug_level: 3,
            dry_run: true,
            diff: false,
            limit: vec!["web".to_string()],
            tags: vec!["deploy".to_string()],
            skip_tags: vec!["test".to_string()],
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"debug\":true"));
        assert!(json.contains("\"dry_run\":true"));
        assert!(json.contains("\"limit\":[\"web\"]"));
    }

    #[test]
    fn test_terraform_task_params_default() {
        let params = TerraformTaskParams::default();
        assert!(!params.plan);
        assert!(!params.destroy);
        assert!(!params.auto_approve);
        assert!(params.backend_config.is_none());
        assert!(params.workspace.is_none());
    }

    #[test]
    fn test_terraform_task_params_serialization() {
        let params = TerraformTaskParams {
            plan: true,
            destroy: false,
            auto_approve: true,
            upgrade: true,
            reconfigure: false,
            backend_init_required: true,
            backend_config: Some("key=value".to_string()),
            workspace: Some("prod".to_string()),
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"plan\":true"));
        assert!(json.contains("\"auto_approve\":true"));
        assert!(json.contains("\"workspace\":\"prod\""));
    }

    #[test]
    fn test_default_task_params() {
        let params = DefaultTaskParams::default();
        let json = serde_json::to_string(&params).unwrap();
        assert_eq!(json, "{}");
    }
}
