//! Модель шаблона (Template)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, database::Database, decode::Decode, encode::Encode};

/// Тип шаблона
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateType {
    Default,
    Build,
    Deploy,
    Task,
    Ansible,
    Terraform,
    Shell,
}

impl std::fmt::Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateType::Default => write!(f, "default"),
            TemplateType::Build => write!(f, "build"),
            TemplateType::Deploy => write!(f, "deploy"),
            TemplateType::Task => write!(f, "task"),
            TemplateType::Ansible => write!(f, "ansible"),
            TemplateType::Terraform => write!(f, "terraform"),
            TemplateType::Shell => write!(f, "shell"),
        }
    }
}

impl std::str::FromStr for TemplateType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(TemplateType::Default),
            "build" => Ok(TemplateType::Build),
            "deploy" => Ok(TemplateType::Deploy),
            "task" => Ok(TemplateType::Task),
            "ansible" => Ok(TemplateType::Ansible),
            "terraform" => Ok(TemplateType::Terraform),
            "shell" => Ok(TemplateType::Shell),
            _ => Ok(TemplateType::Default),
        }
    }
}

impl<DB: Database> Type<DB> for TemplateType
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for TemplateType
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "build" => TemplateType::Build,
            "deploy" => TemplateType::Deploy,
            "task" => TemplateType::Task,
            "ansible" => TemplateType::Ansible,
            "terraform" => TemplateType::Terraform,
            "shell" => TemplateType::Shell,
            _ => TemplateType::Default,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for TemplateType
where
    DB: 'q,
    String: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            TemplateType::Build => "build",
            TemplateType::Deploy => "deploy",
            TemplateType::Task => "task",
            TemplateType::Ansible => "ansible",
            TemplateType::Terraform => "terraform",
            TemplateType::Shell => "shell",
            TemplateType::Default => "default",
        }
        .to_string();
        <String as Encode<'q, DB>>::encode(s, buf)
    }
}

/// Приложение, используемое шаблоном
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateApp {
    Ansible,
    Terraform,
    Tofu,
    Terragrunt,
    Bash,
    PowerShell,
    Python,
    Pulumi,
    Default,
}

impl std::fmt::Display for TemplateApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateApp::Ansible => write!(f, "ansible"),
            TemplateApp::Terraform => write!(f, "terraform"),
            TemplateApp::Tofu => write!(f, "tofu"),
            TemplateApp::Terragrunt => write!(f, "terragrunt"),
            TemplateApp::Bash => write!(f, "bash"),
            TemplateApp::PowerShell => write!(f, "powershell"),
            TemplateApp::Python => write!(f, "python"),
            TemplateApp::Pulumi => write!(f, "pulumi"),
            TemplateApp::Default => write!(f, "default"),
        }
    }
}

impl std::str::FromStr for TemplateApp {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "ansible" => TemplateApp::Ansible,
            "terraform" => TemplateApp::Terraform,
            "tofu" => TemplateApp::Tofu,
            "terragrunt" => TemplateApp::Terragrunt,
            "bash" => TemplateApp::Bash,
            "powershell" => TemplateApp::PowerShell,
            "python" => TemplateApp::Python,
            "pulumi" => TemplateApp::Pulumi,
            _ => TemplateApp::Default,
        })
    }
}

impl<DB: Database> Type<DB> for TemplateApp
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for TemplateApp
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "ansible" => TemplateApp::Ansible,
            "terraform" => TemplateApp::Terraform,
            "tofu" => TemplateApp::Tofu,
            "terragrunt" => TemplateApp::Terragrunt,
            "bash" => TemplateApp::Bash,
            "powershell" => TemplateApp::PowerShell,
            "python" => TemplateApp::Python,
            "pulumi" => TemplateApp::Pulumi,
            _ => TemplateApp::Default,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for TemplateApp
where
    DB: 'q,
    String: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            TemplateApp::Ansible => "ansible",
            TemplateApp::Terraform => "terraform",
            TemplateApp::Tofu => "tofu",
            TemplateApp::Terragrunt => "terragrunt",
            TemplateApp::Bash => "bash",
            TemplateApp::PowerShell => "powershell",
            TemplateApp::Python => "python",
            TemplateApp::Pulumi => "pulumi",
            TemplateApp::Default => "default",
        }
        .to_string();
        <String as Encode<'q, DB>>::encode(s, buf)
    }
}

/// Шаблон задачи
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Template {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Название шаблона
    pub name: String,

    /// Псевдоним шаблона
    pub playbook: String,

    /// Описание
    pub description: String,

    /// ID инвентаря
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_id: Option<i32>,

    /// ID репозитория
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<i32>,

    /// ID окружения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<i32>,

    /// Тип шаблона
    pub r#type: TemplateType,

    /// Приложение
    pub app: TemplateApp,

    /// Ветка Git по умолчанию
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Аргументы командной строки
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,

    /// ID ключа vault
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_key_id: Option<i32>,

    /// ID View (группа шаблонов)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_id: Option<i32>,

    /// ID шаблона сборки (для type=deploy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_template_id: Option<i32>,

    /// Автозапуск при успешном build
    #[serde(default)]
    pub autorun: bool,

    /// Разрешить переопределение аргументов при запуске
    #[serde(default)]
    pub allow_override_args_in_task: bool,

    /// Разрешить переопределение ветки при запуске
    #[serde(default)]
    pub allow_override_branch_in_task: bool,

    /// Разрешить смену инвентаря при запуске
    #[serde(default)]
    pub allow_inventory_in_task: bool,

    /// Разрешить параллельный запуск
    #[serde(default)]
    pub allow_parallel_tasks: bool,

    /// Подавлять уведомления при успехе
    #[serde(default)]
    pub suppress_success_alerts: bool,

    /// Требовать подтверждения плана перед apply (Phase 2: Plan Approval)
    #[serde(default)]
    pub require_approval: bool,

    /// Параметры задачи (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_params: Option<serde_json::Value>,

    /// Переменные опроса (survey variables) - JSON массив
    #[serde(skip_serializing_if = "Option::is_none")]
    pub survey_vars: Option<serde_json::Value>,

    /// Vault ключи - JSON массив
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaults: Option<serde_json::Value>,

    // ── Template Inheritance (Jenkins) ──────────────────────────────────────
    /// Родительский шаблон — наследовать его extra_vars как базу
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_template_id: Option<i32>,

    // ── Execution Environments (AWX) ────────────────────────────────────────
    /// Docker-образ для запуска задачи (Execution Environment).
    /// Если задан — задача запускается внутри `docker run --rm <image>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_image: Option<String>,

    // ── Sync Hooks (Argo CD) ────────────────────────────────────────────────
    /// Шаблон, запускаемый ДО основной задачи (pre-hook)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_template_id: Option<i32>,
    /// Шаблон, запускаемый ПОСЛЕ успеха (post-hook)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_template_id: Option<i32>,
    /// Шаблон, запускаемый при ОШИБКЕ (fail-hook / rollback)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_template_id: Option<i32>,

    // ── Deployment Environment (GitLab Environments) ────────────────────────
    /// ID deployment environment (production/staging/dev) — обновляется при запуске
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deploy_environment_id: Option<i32>,
}

/// Шаблон с правами доступа
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateWithPerms {
    #[serde(flatten)]
    pub template: Template,
    pub user_id: i32,
    pub role: String,
}

/// Разрешение для шаблона
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplatePermission {
    pub id: i32,
    pub template_id: i32,
    pub user_id: i32,
    pub role: String,
    pub created: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_type_display() {
        assert_eq!(TemplateType::Ansible.to_string(), "ansible");
        assert_eq!(TemplateType::Terraform.to_string(), "terraform");
        assert_eq!(TemplateType::Shell.to_string(), "shell");
    }

    #[test]
    fn test_template_type_from_str() {
        assert_eq!(
            "ansible".parse::<TemplateType>().unwrap(),
            TemplateType::Ansible
        );
        assert_eq!(
            "unknown".parse::<TemplateType>().unwrap(),
            TemplateType::Default
        );
    }

    #[test]
    fn test_template_type_serialization() {
        assert_eq!(
            serde_json::to_string(&TemplateType::Deploy).unwrap(),
            "\"deploy\""
        );
    }

    #[test]
    fn test_template_app_display() {
        assert_eq!(TemplateApp::Ansible.to_string(), "ansible");
        assert_eq!(TemplateApp::Terraform.to_string(), "terraform");
        assert_eq!(TemplateApp::Bash.to_string(), "bash");
    }

    #[test]
    fn test_template_app_from_str() {
        assert_eq!(
            "ansible".parse::<TemplateApp>().unwrap(),
            TemplateApp::Ansible
        );
        assert_eq!(
            "unknown".parse::<TemplateApp>().unwrap(),
            TemplateApp::Default
        );
    }

    #[test]
    fn test_template_serialization() {
        let template = Template {
            id: 1,
            project_id: 10,
            name: "Deploy to Prod".to_string(),
            playbook: "deploy.yml".to_string(),
            description: "Production deploy template".to_string(),
            inventory_id: Some(5),
            repository_id: Some(3),
            environment_id: Some(2),
            r#type: TemplateType::Deploy,
            app: TemplateApp::Ansible,
            git_branch: Some("main".to_string()),
            created: Utc::now(),
            arguments: None,
            vault_key_id: None,
            view_id: None,
            build_template_id: None,
            autorun: false,
            allow_override_args_in_task: true,
            allow_override_branch_in_task: true,
            allow_inventory_in_task: false,
            allow_parallel_tasks: false,
            suppress_success_alerts: false,
            require_approval: true,
            task_params: None,
            survey_vars: None,
            vaults: None,
            parent_template_id: None,
            execution_image: None,
            pre_template_id: None,
            post_template_id: None,
            fail_template_id: None,
            deploy_environment_id: None,
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("\"name\":\"Deploy to Prod\""));
        assert!(json.contains("\"type\":\"deploy\""));
        assert!(json.contains("\"app\":\"ansible\""));
    }

    #[test]
    fn test_template_skip_nulls() {
        let template = Template {
            id: 1,
            project_id: 10,
            name: "Simple".to_string(),
            playbook: "simple.yml".to_string(),
            description: "".to_string(),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            r#type: TemplateType::Default,
            app: TemplateApp::Default,
            git_branch: None,
            created: Utc::now(),
            arguments: None,
            vault_key_id: None,
            view_id: None,
            build_template_id: None,
            autorun: false,
            allow_override_args_in_task: false,
            allow_override_branch_in_task: false,
            allow_inventory_in_task: false,
            allow_parallel_tasks: false,
            suppress_success_alerts: false,
            require_approval: false,
            task_params: None,
            survey_vars: None,
            vaults: None,
            parent_template_id: None,
            execution_image: None,
            pre_template_id: None,
            post_template_id: None,
            fail_template_id: None,
            deploy_environment_id: None,
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(!json.contains("\"inventory_id\":"));
        assert!(!json.contains("\"git_branch\":"));
        assert!(!json.contains("\"execution_image\":"));
    }

    #[test]
    fn test_template_type_display_all_variants() {
        assert_eq!(TemplateType::Default.to_string(), "default");
        assert_eq!(TemplateType::Build.to_string(), "build");
        assert_eq!(TemplateType::Deploy.to_string(), "deploy");
        assert_eq!(TemplateType::Task.to_string(), "task");
        assert_eq!(TemplateType::Ansible.to_string(), "ansible");
        assert_eq!(TemplateType::Terraform.to_string(), "terraform");
        assert_eq!(TemplateType::Shell.to_string(), "shell");
    }

    #[test]
    fn test_template_type_from_str_all_variants() {
        assert_eq!(
            "default".parse::<TemplateType>().unwrap(),
            TemplateType::Default
        );
        assert_eq!(
            "build".parse::<TemplateType>().unwrap(),
            TemplateType::Build
        );
        assert_eq!(
            "deploy".parse::<TemplateType>().unwrap(),
            TemplateType::Deploy
        );
        assert_eq!("task".parse::<TemplateType>().unwrap(), TemplateType::Task);
        assert_eq!(
            "shell".parse::<TemplateType>().unwrap(),
            TemplateType::Shell
        );
    }

    #[test]
    fn test_template_type_serialize_all_variants() {
        let types = [
            TemplateType::Default,
            TemplateType::Build,
            TemplateType::Deploy,
            TemplateType::Task,
            TemplateType::Ansible,
            TemplateType::Terraform,
            TemplateType::Shell,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            assert!(json.starts_with('"') && json.ends_with('"'));
        }
    }

    #[test]
    fn test_template_app_display_all_variants() {
        assert_eq!(TemplateApp::Default.to_string(), "default");
        assert_eq!(TemplateApp::Tofu.to_string(), "tofu");
        assert_eq!(TemplateApp::Terragrunt.to_string(), "terragrunt");
        assert_eq!(TemplateApp::PowerShell.to_string(), "powershell");
        assert_eq!(TemplateApp::Python.to_string(), "python");
        assert_eq!(TemplateApp::Pulumi.to_string(), "pulumi");
    }

    #[test]
    fn test_template_app_serialize_all_variants() {
        let apps = [
            TemplateApp::Default,
            TemplateApp::Ansible,
            TemplateApp::Terraform,
            TemplateApp::Tofu,
            TemplateApp::Terragrunt,
            TemplateApp::Bash,
            TemplateApp::PowerShell,
            TemplateApp::Python,
            TemplateApp::Pulumi,
        ];
        for app in &apps {
            let json = serde_json::to_string(app).unwrap();
            assert!(json.starts_with('"') && json.ends_with('"'));
        }
    }

    #[test]
    fn test_template_default() {
        let template = Template::default();
        assert_eq!(template.id, 0);
        assert_eq!(template.name, "");
        assert_eq!(template.playbook, "");
        assert_eq!(template.r#type, TemplateType::Default);
        assert_eq!(template.app, TemplateApp::Default);
    }

    #[test]
    fn test_template_default_template() {
        let template = Template::default_template(10, "Test".to_string(), "test.yml".to_string());
        assert_eq!(template.project_id, 10);
        assert_eq!(template.name, "Test");
        assert_eq!(template.playbook, "test.yml");
        assert!(!template.autorun);
        assert!(!template.require_approval);
    }

    #[test]
    fn test_survey_var_serialization() {
        let survey_var = SurveyVar {
            name: "environment".to_string(),
            title: "Target Environment".to_string(),
            description: "Choose target env".to_string(),
            r#type: "enum".to_string(),
            enum_values: Some(vec!["dev".to_string(), "prod".to_string()]),
            required: true,
        };
        let json = serde_json::to_string(&survey_var).unwrap();
        assert!(json.contains("\"name\":\"environment\""));
        assert!(json.contains("\"required\":true"));
        assert!(json.contains("\"enum_values\":["));
    }

    #[test]
    fn test_template_vault_ref_serialization() {
        let vault_ref = TemplateVaultRef {
            vault_key_id: 42,
            r#type: "hashicorp".to_string(),
        };
        let json = serde_json::to_string(&vault_ref).unwrap();
        assert!(json.contains("\"vault_key_id\":42"));
        assert!(json.contains("\"type\":\"hashicorp\""));
    }

    #[test]
    fn test_template_filter_default() {
        let filter = TemplateFilter::default();
        assert!(filter.project_id.is_none());
        assert!(filter.r#type.is_none());
        assert!(filter.app.is_none());
        assert!(filter.view_id.is_none());
    }

    #[test]
    fn test_template_clone() {
        let template = Template::default_template(1, "Clone".to_string(), "clone.yml".to_string());
        let cloned = template.clone();
        assert_eq!(cloned.name, template.name);
        assert_eq!(cloned.playbook, template.playbook);
        assert_eq!(cloned.autorun, template.autorun);
    }
}

/// Разрешения шаблона для ролей
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateRolePerm {
    pub id: i32,
    pub project_id: i32,
    pub template_id: i32,
    pub role_id: i32,
    pub role_slug: String,
}

/// Фильтр для шаблонов
#[derive(Debug, Clone, Default)]
pub struct TemplateFilter {
    pub project_id: Option<i32>,
    pub r#type: Option<TemplateType>,
    pub app: Option<TemplateApp>,
    pub view_id: Option<i32>,
}

/// Переменная опроса (Survey Variable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyVar {
    /// Имя переменной (key)
    pub name: String,
    /// Заголовок (label для UI)
    #[serde(default)]
    pub title: String,
    /// Описание (подсказка)
    #[serde(default)]
    pub description: String,
    /// Тип: string / int / enum / secret
    #[serde(default)]
    pub r#type: String,
    /// Значения для enum (только для type=enum)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    /// Обязательная переменная
    #[serde(default)]
    pub required: bool,
}

/// Vault ключ в шаблоне
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVaultRef {
    /// ID ключа vault
    pub vault_key_id: i32,
    /// Тип vault
    #[serde(default)]
    pub r#type: String,
}

impl Template {
    /// Создаёт новый шаблон с значениями по умолчанию
    pub fn default_template(project_id: i32, name: String, playbook: String) -> Self {
        Self {
            id: 0,
            project_id,
            name,
            playbook,
            description: String::new(),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            r#type: TemplateType::Default,
            app: TemplateApp::Default,
            git_branch: None,
            created: Utc::now(),
            arguments: None,
            vault_key_id: None,
            view_id: None,
            build_template_id: None,
            autorun: false,
            allow_override_args_in_task: false,
            allow_override_branch_in_task: false,
            allow_inventory_in_task: false,
            allow_parallel_tasks: false,
            suppress_success_alerts: false,
            require_approval: false,
            task_params: None,
            survey_vars: None,
            vaults: None,
            parent_template_id: None,
            execution_image: None,
            pre_template_id: None,
            post_template_id: None,
            fail_template_id: None,
            deploy_environment_id: None,
        }
    }
}

impl Default for Template {
    fn default() -> Self {
        Self::default_template(0, String::new(), String::new())
    }
}
