//! LocalJob Environment - переменные окружения и task details
//!
//! Аналог services/tasks/local_job_environment.go из Go версии

use std::collections::HashMap;
use serde_json::{Map, Value};

use crate::error::Result;
use crate::models::template::TemplateType;
use crate::services::local_job::LocalJob;

impl LocalJob {
    /// Получает детали задачи в виде карты
    pub fn get_task_details(&self, username: &str, incoming_version: Option<&str>) -> HashMap<String, Value> {
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

        details.insert("inventory_name".to_string(), Value::String(self.inventory.name.clone()));
        details.insert("inventory_id".to_string(), Value::Number(self.inventory.id.into()));
        details.insert("repository_name".to_string(), Value::String(self.repository.name.clone()));
        details.insert("repository_id".to_string(), Value::Number(self.repository.id.into()));

        if self.template.r#type != TemplateType::Task {
            details.insert("type".to_string(), Value::String(self.template.r#type.to_string()));

            if let Some(ver) = incoming_version {
                details.insert("incoming_version".to_string(), Value::String(ver.to_string()));
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
        semaphore_vars.insert("task_details".to_string(), serde_json::to_value(task_details)?);

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
        semaphore_vars.insert("task_details".to_string(), serde_json::to_value(task_details)?);
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
        // secrets - это JSON строка, нужно распарсить
        if let Some(ref _secrets_json) = self.environment.secrets {
            // TODO: Распарсить secrets_json и получить секреты
            // for secret in secrets {
            //     if secret.secret_type == crate::models::EnvironmentSecretType::Env {
            //         res.push(format!("{}={}", secret.name, secret.secret));
            //     }
            // }
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
                        crate::utils::shell::shell_quote(&crate::utils::shell::shell_strip_unsafe(&detail_str))
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
    use chrono::Utc;
    use std::sync::Arc;
    use crate::services::task_logger::BasicLogger;
    use crate::db_lib::AccessKeyInstallerImpl;
    use std::path::PathBuf;

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
        assert_eq!(details.get("username").unwrap().as_str().unwrap(), "testuser");
        assert_eq!(details.get("inventory_name").unwrap().as_str().unwrap(), "Test Inventory");
        assert_eq!(details.get("repository_name").unwrap().as_str().unwrap(), "Test Repo");
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
}
