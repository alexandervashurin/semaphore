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
}

/// Параметры задачи по умолчанию
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultTaskParams {}
