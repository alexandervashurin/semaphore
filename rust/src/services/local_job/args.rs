//! LocalJob Args - генерация аргументов для различных типов задач
//!
//! Аналог services/tasks/local_job_args.go из Go версии

use crate::error::Result;
use crate::services::local_job::LocalJob;
use std::collections::HashMap;

impl LocalJob {
    /// Получает аргументы для shell скрипта
    pub fn get_shell_args(
        &self,
        username: &str,
        incoming_version: Option<&str>,
    ) -> Result<Vec<String>> {
        let extra_vars = self.get_environment_extra_vars(username, incoming_version)?;
        let (template_args, task_args) = self.get_cli_args()?;

        let mut args = Vec::new();

        // Скрипт для выполнения
        args.push(self.template.playbook.clone());

        // Секретные переменные из environment.secrets (JSON)
        if let Some(ref secrets_json) = self.environment.secrets {
            if let Ok(secrets) =
                serde_json::from_str::<Vec<crate::models::EnvironmentSecretValue>>(secrets_json)
            {
                for secret in secrets {
                    if secret.secret_type == crate::models::EnvironmentSecretType::Var {
                        args.push(format!("{}={}", secret.name, secret.secret));
                    }
                }
            }
        }

        // Аргументы шаблона
        args.extend(template_args);

        // Extra vars и Survey vars
        for (name, value) in extra_vars {
            if name != "semaphore_vars" {
                args.push(format!("{}={}", name, value));
            }
        }

        // Аргументы задачи
        args.extend(task_args);

        Ok(args)
    }

    /// Получает аргументы для Terraform (карта по стадиям)
    pub fn get_terraform_args(
        &self,
        username: &str,
        incoming_version: Option<&str>,
    ) -> Result<HashMap<String, Vec<String>>> {
        let mut args_map = HashMap::new();

        let extra_vars = self.get_environment_extra_vars(username, incoming_version)?;

        // Параметры задачи
        let params: crate::models::TerraformTaskParams = self.get_params()?;

        // Аргументы для destroy
        let destroy_args = if params.destroy {
            vec!["-destroy".to_string()]
        } else {
            vec![]
        };

        // Аргументы для переменных
        let mut var_args = Vec::new();
        for (name, value) in &extra_vars {
            if name == "semaphore_vars" {
                continue;
            }
            var_args.push("-var".to_string());
            var_args.push(format!("{}={}", name, value));
        }

        // Аргументы для секретов из environment.secrets (JSON)
        let mut secret_args = Vec::new();
        if let Some(ref secrets_json) = self.environment.secrets {
            if let Ok(secrets) =
                serde_json::from_str::<Vec<crate::models::EnvironmentSecretValue>>(secrets_json)
            {
                for secret in secrets {
                    if secret.secret_type != crate::models::EnvironmentSecretType::Var {
                        continue;
                    }
                    secret_args.push("-var".to_string());
                    secret_args.push(format!("{}={}", secret.name, secret.secret));
                }
            }
        }

        // Базовые аргументы
        args_map.insert("default".to_string(), Vec::new());

        // Добавляем аргументы к стадиям
        for stage in args_map.keys().cloned().collect::<Vec<_>>() {
            if stage == "init" {
                continue;
            }

            let mut combined = destroy_args.clone();
            combined.extend(args_map.get(&stage).cloned().unwrap_or_default());
            combined.extend(var_args.clone());
            combined.extend(secret_args.clone());
            args_map.insert(stage, combined);
        }

        Ok(args_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::models::TemplateType;
    use crate::services::task_logger::BasicLogger;
    use chrono::Utc;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn create_test_shell_job() -> LocalJob {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            message: None,
            commit_hash: None,
            commit_message: None,
            version: None,
            project_id: 1,
            arguments: None,
            params: Some(serde_json::json!({})),
            ..Default::default()
        };

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test Shell".to_string();
        template.project_id = 1;
        template.playbook = "test.sh".to_string();
        template.r#type = TemplateType::Shell;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: String::from("Test Env"),
            json: String::from(r#"{"var1": "value1"}"#),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        )
    }

    #[test]
    fn test_get_shell_args() {
        let job = create_test_shell_job();
        let args = job.get_shell_args("testuser", None).unwrap();

        assert!(!args.is_empty());
        assert_eq!(args[0], "test.sh");
    }

    #[test]
    fn test_get_terraform_args() {
        let job = create_test_shell_job();
        let args = job.get_terraform_args("testuser", None).unwrap();

        assert!(args.contains_key("default"));
    }

    #[test]
    fn test_get_shell_args_with_secrets() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            project_id: 1,
            ..Default::default()
        };

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "deploy.sh".to_string();
        template.r#type = TemplateType::Shell;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Secret Env".to_string(),
            json: r#"{"DB_HOST": "localhost"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: Some(r#"[{"name": "DB_PASSWORD", "secret": "secret123", "secret_type": "var"}]"#.to_string()),
            created: None,
        };

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let args = job.get_shell_args("testuser", None).unwrap();
        // DB_PASSWORD=secret123 должен быть (secret_type == "var")
        assert!(args.iter().any(|a| a == "DB_PASSWORD=secret123"));
        // playbook name должен быть первым
        assert_eq!(args[0], "deploy.sh");
    }

    #[test]
    fn test_get_shell_args_filters_semaphore_vars() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            project_id: 1,
            ..Default::default()
        };

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "run.sh".to_string();
        template.r#type = TemplateType::Shell;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Env".to_string(),
            json: r#"{"semaphore_vars": {"key": "val"}, "real_var": "real_value"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let args = job.get_shell_args("testuser", None).unwrap();
        // semaphore_vars не должен попасть в аргументы
        assert!(!args.iter().any(|a| a.starts_with("semaphore_vars=")));
    }

    #[test]
    fn test_get_terraform_args_with_destroy() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            project_id: 1,
            params: Some(serde_json::json!({"destroy": true})),
            ..Default::default()
        };
        task.params = Some(serde_json::json!({"destroy": true}));

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Terraform".to_string();
        template.project_id = 1;
        template.playbook = "main.tf".to_string();
        template.r#type = TemplateType::Terraform;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Env".to_string(),
            json: r#"{"region": "us-east-1"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let args_map = job.get_terraform_args("testuser", None).unwrap();
        // Все стадии должны содержать "-destroy" кроме init
        for (stage, args) in &args_map {
            if stage != "init" {
                assert!(args.contains(&"-destroy".to_string()),
                    "Stage '{}' should contain -destroy", stage);
            }
        }
    }

    #[test]
    fn test_get_terraform_args_includes_vars() {
        let job = create_test_shell_job();
        // environment.json = {"var1": "value1"} уже установлен
        let args_map = job.get_terraform_args("testuser", None).unwrap();

        // Проверяем что var=value попал в аргументы стадий (кроме init)
        for (stage, args) in &args_map {
            if stage != "init" {
                assert!(args.contains(&"-var".to_string()), "Stage '{}' should contain -var", stage);
                assert!(args.iter().any(|a| a.starts_with("var1=")),
                    "Stage '{}' should contain var1=", stage);
            }
        }
    }

    #[test]
    fn test_get_shell_args_multiple_secrets() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            project_id: 1,
            ..Default::default()
        };

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.playbook = "script.sh".to_string();
        template.r#type = TemplateType::Shell;

        // Только valid secret_type: "env" или "var" (file не десериализуется)
        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Multi Secret Env".to_string(),
            json: "{}".to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: Some(r#"[
                {"name": "API_KEY", "secret": "key123", "secret_type": "var"},
                {"name": "DB_PASS", "secret": "pass456", "secret_type": "var"},
                {"name": "ENV_ONLY", "secret": "env_val", "secret_type": "env"}
            ]"#.to_string()),
            created: None,
        };

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let args = job.get_shell_args("testuser", None).unwrap();
        assert!(args.iter().any(|a| a == "API_KEY=key123"));
        assert!(args.iter().any(|a| a == "DB_PASS=pass456"));
        // env type не попадает в shell args (только var)
        assert!(!args.iter().any(|a| a.starts_with("ENV_ONLY=")));
    }

    #[test]
    fn test_get_shell_args_playbook_is_first_arg() {
        let job = create_test_shell_job();
        let args = job.get_shell_args("testuser", None).unwrap();
        assert!(!args.is_empty());
        assert_eq!(args[0], "test.sh");
    }

    #[test]
    fn test_get_terraform_args_empty_environment() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            project_id: 1,
            params: Some(serde_json::json!({})),
            ..Default::default()
        };

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.playbook = "main.tf".to_string();
        template.r#type = TemplateType::Terraform;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Empty".to_string(),
            json: "{}".to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let args_map = job.get_terraform_args("testuser", None).unwrap();
        assert!(args_map.contains_key("default"));
        let default_args = args_map.get("default").unwrap();
        assert!(default_args.is_empty());
    }

    #[test]
    fn test_get_shell_args_with_invalid_secrets_json() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            project_id: 1,
            ..Default::default()
        };

        let mut template = crate::models::Template::default();
        template.playbook = "run.sh".to_string();
        template.r#type = TemplateType::Shell;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Bad Secrets".to_string(),
            json: "{}".to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: Some("not valid json".to_string()),
            created: None,
        };

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        // Не должен паниковать при невалидном JSON секретов
        let result = job.get_shell_args("testuser", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_shell_args_no_secrets() {
        let job = create_test_shell_job();
        let args = job.get_shell_args("testuser", None).unwrap();
        // playbook должен быть первым
        assert!(!args.is_empty());
        assert_eq!(args[0], "test.sh");
    }

    #[test]
    fn test_get_shell_args_with_template_args() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            project_id: 1,
            arguments: Some(r#"["--task-flag"]"#.to_string()),
            ..Default::default()
        };

        let mut template = crate::models::Template::default();
        template.playbook = "deploy.sh".to_string();
        template.r#type = TemplateType::Shell;
        template.arguments = Some(r#"["--verbose", "--dry-run"]"#.to_string());

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Env".to_string(),
            json: "{}".to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let args = job.get_shell_args("testuser", None).unwrap();
        assert!(args.contains(&"--verbose".to_string()));
        assert!(args.contains(&"--dry-run".to_string()));
        assert!(args.contains(&"--task-flag".to_string()));
    }

    #[test]
    fn test_get_terraform_args_with_multiple_vars() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            project_id: 1,
            params: Some(serde_json::json!({})),
            ..Default::default()
        };

        let mut template = crate::models::Template::default();
        template.playbook = "infra.tf".to_string();
        template.r#type = TemplateType::Terraform;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "TF Env".to_string(),
            json: r#"{"region": "eu-west-1", "env": "staging", "size": "large"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let args_map = job.get_terraform_args("testuser", None).unwrap();
        let apply_args = args_map.get("apply").cloned()
            .or_else(|| args_map.values().next().cloned())
            .unwrap_or_default();

        // Каждая переменная должна иметь -var и value
        let var_count = apply_args.iter().filter(|a| *a == "-var").count();
        assert!(var_count >= 3, "Expected at least 3 -var flags, got {}", var_count);
    }

    #[test]
    fn test_get_shell_args_with_incoming_version() {
        let job = create_test_shell_job();
        let args = job.get_shell_args("testuser", Some("v2.0.0")).unwrap();
        assert!(!args.is_empty());
        // playbook всегда первый
        assert_eq!(args[0], "test.sh");
    }

    #[test]
    fn test_get_terraform_args_with_secrets_and_vars() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            project_id: 1,
            params: Some(serde_json::json!({})),
            ..Default::default()
        };

        let mut template = crate::models::Template::default();
        template.playbook = "main.tf".to_string();
        template.r#type = TemplateType::Terraform;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Mixed".to_string(),
            json: r#"{"region": "us-east-1"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: Some(r#"[{"name": "TF_VAR_secret", "secret": "hidden", "secret_type": "var"}]"#.to_string()),
            created: None,
        };

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let args_map = job.get_terraform_args("testuser", None).unwrap();
        // default стадия содержит vars и secrets (combined args)
        let default_args = args_map.get("default").unwrap();
        assert!(default_args.iter().any(|a| a.starts_with("region=")),
            "default should contain region var");
        assert!(default_args.iter().any(|a| a.starts_with("TF_VAR_secret=")),
            "default should contain secret var");

        // Все стадии содержат объединённые args
        for (stage, args) in &args_map {
            if stage != "init" {
                assert!(args.iter().any(|a| a.starts_with("region=")),
                    "Stage '{}' should contain region var", stage);
            }
        }
    }

    #[test]
    fn test_get_shell_args_empty_template_args() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            project_id: 1,
            arguments: None,
            ..Default::default()
        };

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.playbook = "build.sh".to_string();
        template.r#type = TemplateType::Shell;
        template.arguments = Some(r#"[]"#.to_string());

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Env".to_string(),
            json: r#"{"BUILD_TYPE": "release"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let args = job.get_shell_args("builder", None).unwrap();
        // playbook первый
        assert_eq!(args[0], "build.sh");
        // extra vars должны быть в args (значения в формате KEY=Value)
        assert!(args.iter().any(|a| a.contains("BUILD_TYPE")),
            "Expected BUILD_TYPE in args, got: {:?}", args);
    }
}
