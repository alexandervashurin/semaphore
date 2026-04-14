//! LocalJob Environment - переменные окружения и task details
//!
//! Аналог services/tasks/local_job_environment.go из Go версии

use serde_json::{Map, Value};
use std::collections::HashMap;

use crate::error::Result;
use crate::models::template::TemplateType;
use crate::services::local_job::LocalJob;

impl LocalJob {
    /// Получает детали задачи в виде карты
    pub fn get_task_details(
        &self,
        username: &str,
        incoming_version: Option<&str>,
    ) -> HashMap<String, Value> {
        let mut details = HashMap::new();

        details.insert("id".to_string(), Value::Number(self.task.id.into()));

        if let Some(ref message) = self.task.message {
            if !message.is_empty() {
                details.insert("message".to_string(), Value::String(message.clone()));
            }
        }

        details.insert("username".to_string(), Value::String(username.to_string()));
        details.insert("url".to_string(), Value::String(self.task.get_url()));

        if let Some(ref hash) = self.task.commit_hash {
            details.insert("commit_hash".to_string(), Value::String(hash.clone()));
        }

        if let Some(ref msg) = self.task.commit_message {
            details.insert("commit_message".to_string(), Value::String(msg.clone()));
        }

        details.insert(
            "inventory_name".to_string(),
            Value::String(self.inventory.name.clone()),
        );
        details.insert(
            "inventory_id".to_string(),
            Value::Number(self.inventory.id.into()),
        );
        details.insert(
            "repository_name".to_string(),
            Value::String(self.repository.name.clone()),
        );
        details.insert(
            "repository_id".to_string(),
            Value::Number(self.repository.id.into()),
        );

        if self.template.r#type != TemplateType::Task {
            details.insert(
                "type".to_string(),
                Value::String(self.template.r#type.to_string()),
            );

            if let Some(ver) = incoming_version {
                details.insert(
                    "incoming_version".to_string(),
                    Value::String(ver.to_string()),
                );
            }

            if self.template.r#type == TemplateType::Build {
                if let Some(ref ver) = self.task.version {
                    details.insert("target_version".to_string(), Value::String(ver.clone()));
                }
            }
        }

        details
    }

    /// Получает дополнительные переменные из окружения
    pub fn get_environment_extra_vars(
        &self,
        username: &str,
        incoming_version: Option<&str>,
    ) -> Result<HashMap<String, Value>> {
        let mut extra_vars: HashMap<String, Value> =
            serde_json::from_str(&self.environment.json).unwrap_or_default();

        let task_details = self.get_task_details(username, incoming_version);
        let mut semaphore_vars = Map::new();
        semaphore_vars.insert(
            "task_details".to_string(),
            serde_json::to_value(task_details)?,
        );

        extra_vars.insert("semaphore_vars".to_string(), Value::Object(semaphore_vars));

        Ok(extra_vars)
    }

    /// Получает JSON дополнительных переменных
    pub fn get_environment_extra_vars_json(
        &mut self,
        username: &str,
        incoming_version: Option<&str>,
    ) -> Result<String> {
        let mut extra_vars: HashMap<String, Value> =
            serde_json::from_str(&self.environment.json).unwrap_or_default();

        if !self.secret.is_empty() {
            let secret_vars: HashMap<String, Value> =
                serde_json::from_str(&self.secret).unwrap_or_default();
            extra_vars.extend(secret_vars);
        }

        // Очищаем секреты после использования
        self.secret = String::new();

        let task_details = self.get_task_details(username, incoming_version);
        let mut semaphore_vars = Map::new();
        semaphore_vars.insert(
            "task_details".to_string(),
            serde_json::to_value(task_details)?,
        );
        extra_vars.insert("semaphore_vars".to_string(), Value::Object(semaphore_vars));

        Ok(serde_json::to_string(&extra_vars)?)
    }

    /// Получает переменные окружения ENV
    pub fn get_environment_env(&self) -> Result<Vec<String>> {
        let mut res = Vec::new();

        // ENV переменные из окружения
        if !self.environment.json.is_empty() {
            let env_vars: HashMap<String, String> = serde_json::from_str(&self.environment.json)?;
            for (key, val) in env_vars {
                res.push(format!("{}={}", key, val));
            }
        }

        // Секретные ENV переменные
        // secrets - это JSON строка со списком EnvironmentSecretValue
        if let Some(ref secrets_json) = self.environment.secrets {
            if let Ok(secrets) =
                serde_json::from_str::<Vec<crate::models::EnvironmentSecretValue>>(secrets_json)
            {
                for secret in secrets {
                    if secret.secret_type == crate::models::EnvironmentSecretType::Env {
                        res.push(format!("{}={}", secret.name, secret.secret));
                    }
                }
            }
        }

        Ok(res)
    }

    /// Получает дополнительные shell переменные окружения
    pub fn get_shell_environment_extra_env(
        &self,
        username: &str,
        incoming_version: Option<&str>,
    ) -> Vec<String> {
        let mut extra_shell_vars = Vec::new();
        let task_details = self.get_task_details(username, incoming_version);

        for (task_detail, task_detail_value) in task_details {
            let env_var_name = format!("SEMAPHORE_TASK_DETAILS_{}", task_detail.to_uppercase());

            let detail_as_str = match task_detail_value {
                Value::String(s) => Some(s),
                Value::Number(n) => Some(n.to_string()),
                Value::Bool(b) => Some(b.to_string()),
                _ => None,
            };

            if let Some(detail_str) = detail_as_str {
                if !detail_str.is_empty() {
                    extra_shell_vars.push(format!(
                        "{}={}",
                        env_var_name,
                        crate::utils::shell::shell_quote(&crate::utils::shell::shell_strip_unsafe(
                            &detail_str
                        ))
                    ));
                }
            }
        }

        extra_shell_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::services::task_logger::BasicLogger;
    use chrono::Utc;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn create_test_job() -> LocalJob {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;
        task.message = Some("Test task".to_string());
        task.commit_hash = Some(String::from("abc123"));
        task.commit_message = Some(String::from("Test commit"));

        let mut inventory = crate::models::Inventory::default();
        inventory.id = 1;
        inventory.name = "Test Inventory".to_string();
        inventory.project_id = 1;
        inventory.inventory_type = crate::models::InventoryType::Static;
        inventory.inventory_data = "localhost".to_string();

        let mut repository = crate::models::Repository::default();
        repository.id = 1;
        repository.name = "Test Repo".to_string();
        repository.project_id = 1;
        repository.git_url = "https://github.com/test/test.git".to_string();
        repository.git_branch = Some("main".to_string());

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: String::from("Test Env"),
            json: String::from(r#"{"key": "value"}"#),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test Template".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;

        LocalJob::new(
            task,
            template,
            inventory,
            repository,
            environment,
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        )
    }

    #[test]
    fn test_get_task_details() {
        let job = create_test_job();
        let details = job.get_task_details("testuser", None);

        assert_eq!(details.get("id").unwrap().as_i64().unwrap(), 1);
        assert_eq!(
            details.get("username").unwrap().as_str().unwrap(),
            "testuser"
        );
        assert_eq!(
            details.get("inventory_name").unwrap().as_str().unwrap(),
            "Test Inventory"
        );
        assert_eq!(
            details.get("repository_name").unwrap().as_str().unwrap(),
            "Test Repo"
        );
    }

    #[test]
    fn test_get_environment_extra_vars() {
        let job = create_test_job();
        let extra_vars = job.get_environment_extra_vars("testuser", None).unwrap();

        assert!(extra_vars.contains_key("key"));
        assert!(extra_vars.contains_key("semaphore_vars"));
    }

    #[test]
    fn test_get_environment_env() {
        // Создаём job с пустым environment.json для проверки пустого env
        let mut job = create_test_job();
        job.environment.json = "{}".to_string();
        let env = job.get_environment_env().unwrap();
        assert!(env.is_empty());
    }

    #[test]
    fn test_get_shell_environment_extra_env() {
        let job = create_test_job();
        let shell_env = job.get_shell_environment_extra_env("testuser", None);
        assert!(!shell_env.is_empty());
    }

    #[test]
    fn test_get_task_details_with_incoming_version() {
        let job = create_test_job();
        let details = job.get_task_details("deployer", Some("v2.0.0"));

        assert_eq!(
            details.get("username").unwrap().as_str().unwrap(),
            "deployer"
        );
        assert_eq!(
            details.get("commit_hash").unwrap().as_str().unwrap(),
            "abc123"
        );
        assert_eq!(
            details.get("commit_message").unwrap().as_str().unwrap(),
            "Test commit"
        );
    }

    #[test]
    fn test_get_task_details_without_commit() {
        let mut job = create_test_job();
        job.task.commit_hash = None;
        job.task.commit_message = None;

        let details = job.get_task_details("testuser", None);

        assert!(!details.contains_key("commit_hash"));
        assert!(!details.contains_key("commit_message"));
    }

    #[test]
    fn test_get_environment_extra_vars_with_empty_json() {
        let mut job = create_test_job();
        job.environment.json = "{}".to_string();

        let extra_vars = job.get_environment_extra_vars("testuser", None).unwrap();

        // Должен содержать только semaphore_vars
        assert!(extra_vars.contains_key("semaphore_vars"));
        assert_eq!(extra_vars.len(), 1);
    }

    #[test]
    fn test_get_environment_extra_vars_with_invalid_json() {
        let mut job = create_test_job();
        job.environment.json = "not valid json".to_string();

        let extra_vars = job.get_environment_extra_vars("testuser", None).unwrap();

        // Должен вернуть только semaphore_vars при невалидном JSON
        assert!(extra_vars.contains_key("semaphore_vars"));
    }

    #[test]
    fn test_get_task_details_url_present() {
        let job = create_test_job();
        let details = job.get_task_details("testuser", None);

        // URL должен быть сформирован
        let url = details.get("url").unwrap().as_str().unwrap();
        assert!(!url.is_empty());
    }

    #[test]
    fn test_get_environment_extra_vars_contains_task_details() {
        let job = create_test_job();
        let extra_vars = job.get_environment_extra_vars("testuser", None).unwrap();

        let semaphore_vars = extra_vars.get("semaphore_vars").unwrap();
        let task_details = semaphore_vars.get("task_details").unwrap();
        assert!(task_details.get("id").is_some());
        assert!(task_details.get("username").is_some());
    }

    #[test]
    fn test_get_environment_extra_vars_json_includes_secrets() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        let mut inventory = crate::models::Inventory::default();
        inventory.id = 1;
        inventory.name = "Inv".to_string();

        let mut repository = crate::models::Repository::default();
        repository.id = 1;
        repository.name = "Repo".to_string();

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Env".to_string(),
            json: r#"{"VAR1": "val1"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;

        let mut job = LocalJob::new(
            task,
            template,
            inventory,
            repository,
            environment,
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        // secret должен быть включён в результат
        job.secret = r#"{"SECRET_VAR": "top_secret"}"#.to_string();
        let result = job.get_environment_extra_vars_json("user", None).unwrap();

        assert!(result.contains("SECRET_VAR"));
        assert!(result.contains("top_secret"));
    }

    #[test]
    fn test_get_environment_extra_vars_json_clears_secret() {
        let mut job = create_test_job();
        job.secret = r#"{"X": "Y"}"#.to_string();

        let _ = job.get_environment_extra_vars_json("user", None).unwrap();
        // После вызова secret должен быть очищен
        assert!(job.secret.is_empty());
    }

    #[test]
    fn test_get_environment_env_with_secrets() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        // Только valid secret_type: "env" (file не десериализуется в EnvironmentSecretType)
        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Env".to_string(),
            json: "{}".to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: Some(
                r#"[
                {"name": "API_KEY", "secret": "abc123", "secret_type": "env"},
                {"name": "DB_HOST", "secret": "localhost", "secret_type": "env"}
            ]"#
                .to_string(),
            ),
            created: None,
        };

        let job = LocalJob::new(
            task,
            crate::models::Template::default(),
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let env = job.get_environment_env().unwrap();
        assert!(env.iter().any(|e| e == "API_KEY=abc123"));
        assert!(env.iter().any(|e| e == "DB_HOST=localhost"));
        assert_eq!(env.len(), 2);
    }

    #[test]
    fn test_get_environment_env_with_invalid_secrets_json() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Env".to_string(),
            json: r#"{"KEY": "val"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: Some("not json".to_string()),
            created: None,
        };

        let job = LocalJob::new(
            task,
            crate::models::Template::default(),
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        // Не должен паниковать — просто вернёт env vars без секретов
        let env = job.get_environment_env().unwrap();
        assert!(env.iter().any(|e| e == "KEY=val"));
    }

    #[test]
    fn test_get_shell_environment_extra_env_shell_strips_unsafe() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;
        task.message = Some("test; rm -rf /".to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;

        let job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let shell_env = job.get_shell_environment_extra_env("user", None);
        // Должен содержать SEMAPHORE_TASK_DETAILS_MESSAGE
        let msg_var = shell_env
            .iter()
            .find(|e| e.starts_with("SEMAPHORE_TASK_DETAILS_MESSAGE="));
        assert!(msg_var.is_some());
        // Опасные символы должны быть экранированы
        let msg_val = msg_var.unwrap();
        assert!(msg_val.contains("test"));
    }

    #[test]
    fn test_get_task_details_with_build_type_and_version() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;
        task.version = Some("v1.0.0".to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Build".to_string();
        template.project_id = 1;
        template.playbook = "build.sh".to_string();
        template.r#type = TemplateType::Build;

        let job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let details = job.get_task_details("builder", Some("v0.9.0"));
        assert_eq!(details.get("type").unwrap().as_str().unwrap(), "build");
        assert_eq!(
            details.get("target_version").unwrap().as_str().unwrap(),
            "v1.0.0"
        );
        assert_eq!(
            details.get("incoming_version").unwrap().as_str().unwrap(),
            "v0.9.0"
        );
    }

    #[test]
    fn test_get_environment_env_empty_json() {
        let mut job = create_test_job();
        job.environment.json = "".to_string();
        let env = job.get_environment_env().unwrap();
        assert!(env.is_empty());
    }

    #[test]
    fn test_get_environment_env_with_multiple_var_types() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Env".to_string(),
            json: r#"{"ENV_VAR1": "a", "ENV_VAR2": "b"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: Some(r#"[]"#.to_string()),
            created: None,
        };

        let job = LocalJob::new(
            task,
            crate::models::Template::default(),
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let env = job.get_environment_env().unwrap();
        assert_eq!(env.len(), 2);
    }

    #[test]
    fn test_get_task_details_contains_id_as_number() {
        let job = create_test_job();
        let details = job.get_task_details("user", None);
        let id_val = details.get("id").unwrap();
        assert!(id_val.is_number());
        assert_eq!(id_val.as_i64().unwrap(), 1);
    }

    #[test]
    fn test_get_task_details_with_non_type_task() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Shell;

        let job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let details = job.get_task_details("user", None);
        assert!(details.contains_key("type"));
        assert_eq!(details.get("type").unwrap().as_str().unwrap(), "shell");
    }

    #[test]
    fn test_get_task_details_no_incoming_version_for_task_type() {
        let job = create_test_job();
        // TemplateType::Task не добавляет incoming_version
        let details = job.get_task_details("user", Some("v1.0"));
        assert!(!details.contains_key("incoming_version"));
    }

    #[test]
    fn test_get_environment_extra_vars_json_is_valid() {
        let mut job = create_test_job();
        let result = job.get_environment_extra_vars_json("user", None).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.get("semaphore_vars").is_some());
    }

    #[test]
    fn test_get_environment_extra_vars_json_contains_key_from_env() {
        let mut job = create_test_job();
        // environment.json = {"key": "value"}
        let result = job.get_environment_extra_vars_json("user", None).unwrap();
        assert!(result.contains("key"));
    }

    #[test]
    fn test_get_shell_environment_extra_env_contains_username() {
        let job = create_test_job();
        let vars = job.get_shell_environment_extra_env("myuser", None);
        // Должен содержать SEMAPHORE_TASK_DETAILS_USERNAME
        let user_var = vars
            .iter()
            .find(|v| v.starts_with("SEMAPHORE_TASK_DETAILS_USERNAME="));
        assert!(user_var.is_some());
        assert!(user_var.unwrap().contains("myuser"));
    }

    #[test]
    fn test_get_environment_env_with_empty_secrets_array() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Env".to_string(),
            json: "{}".to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: Some("[]".to_string()),
            created: None,
        };

        let job = LocalJob::new(
            task,
            crate::models::Template::default(),
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let env = job.get_environment_env().unwrap();
        assert!(env.is_empty());
    }

    #[test]
    fn test_get_task_details_with_build_type_no_version() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;
        task.version = None;

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Build".to_string();
        template.project_id = 1;
        template.playbook = "build.sh".to_string();
        template.r#type = TemplateType::Build;

        let job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let details = job.get_task_details("builder", Some("v1.0"));
        // incoming_version должен быть
        assert!(details.contains_key("incoming_version"));
        // target_version не должен быть (task.version = None)
        assert!(!details.contains_key("target_version"));
    }

    #[test]
    fn test_get_environment_extra_vars_with_multiple_env_values() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Env".to_string(),
            json: r#"{"VAR_A": "1", "VAR_B": "2", "VAR_C": "3"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        let job = LocalJob::new(
            task,
            crate::models::Template::default(),
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let extra_vars = job.get_environment_extra_vars("user", None).unwrap();
        assert!(extra_vars.contains_key("VAR_A"));
        assert!(extra_vars.contains_key("VAR_B"));
        assert!(extra_vars.contains_key("VAR_C"));
    }

    #[test]
    fn test_get_shell_environment_extra_env_empty_message() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;
        task.message = Some("".to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;

        let job = LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let shell_env = job.get_shell_environment_extra_env("user", None);
        // Пустое message не должно попасть в vars
        let msg_var = shell_env
            .iter()
            .find(|e| e.starts_with("SEMAPHORE_TASK_DETAILS_MESSAGE="));
        assert!(msg_var.is_none());
    }

    #[test]
    fn test_get_environment_extra_vars_json_clears_secret_even_with_invalid_json() {
        let mut job = create_test_job();
        job.secret = "not valid json".to_string();

        let result = job.get_environment_extra_vars_json("user", None);
        assert!(result.is_ok());
        assert!(job.secret.is_empty());
    }

    #[test]
    fn test_get_task_details_inventory_and_repo_ids() {
        let job = create_test_job();
        let details = job.get_task_details("user", None);
        assert!(details.contains_key("inventory_id"));
        assert!(details.contains_key("repository_id"));
        assert!(details.contains_key("inventory_name"));
        assert!(details.contains_key("repository_name"));
    }

    #[test]
    fn test_get_environment_env_parses_json_env_vars() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.template_id = 1;
        task.project_id = 1;

        let environment = crate::models::Environment {
            id: 1,
            project_id: 1,
            name: "Env".to_string(),
            json: r#"{"MY_VAR": "my_val"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        let job = LocalJob::new(
            task,
            crate::models::Template::default(),
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            environment,
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let env = job.get_environment_env().unwrap();
        assert!(env.iter().any(|e| e == "MY_VAR=my_val"));
    }
}
