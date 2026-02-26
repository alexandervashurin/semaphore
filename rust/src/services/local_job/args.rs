//! LocalJob Args - генерация аргументов для различных типов задач
//!
//! Аналог services/tasks/local_job_args.go из Go версии

use std::collections::HashMap;
use crate::error::Result;
use crate::services::local_job::LocalJob;

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

        // Секретные переменные
        for secret in &self.environment.secrets {
            if secret.secret_type == crate::models::EnvironmentSecretType::Var {
                args.push(format!("{}={}", secret.name, secret.secret));
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

        // Аргументы для секретов
        let mut secret_args = Vec::new();
        for secret in &self.environment.secrets {
            if secret.secret_type != crate::models::EnvironmentSecretType::Var {
                continue;
            }
            secret_args.push("-var".to_string());
            secret_args.push(format!("{}={}", secret.name, secret.secret));
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
    use chrono::Utc;
    use std::sync::Arc;
    use crate::services::task_logger::BasicLogger;
    use crate::db_lib::AccessKeyInstallerImpl;
    use std::path::PathBuf;

    fn create_test_shell_job() -> LocalJob {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::models::TaskStatus::Waiting,
            message: String::new(),
            commit_hash: None,
            commit_message: None,
            version: None,
            project_id: 1,
            arguments: None,
            params: r#"{}"#.to_string(),
            ..Default::default()
        };

        let template = crate::models::Template {
            id: 1,
            name: String::from("Test Shell"),
            project_id: 1,
            playbook: String::from("test.sh"),
            template_type: crate::models::TemplateType::Shell,
            arguments: None,
            params: String::new(),
            ..Default::default()
        };

        let environment = crate::models::Environment {
            id: 1,
            name: String::from("Test Env"),
            project_id: 1,
            json: String::from(r#"{"var1": "value1"}"#),
            env: None,
            secrets: vec![],
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
}
