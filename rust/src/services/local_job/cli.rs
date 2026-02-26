//! LocalJob CLI - работа с аргументами командной строки
//!
//! Аналог services/tasks/local_job_cli.go из Go версии

use std::collections::HashMap;
use serde_json::Value;

use crate::error::Result;
use crate::services::local_job::LocalJob;

impl LocalJob {
    /// Получает аргументы CLI из шаблона и задачи
    pub fn get_cli_args(&self) -> Result<(Vec<String>, Vec<String>)> {
        let mut template_args = Vec::new();
        let mut task_args = Vec::new();

        // Аргументы из шаблона
        if let Some(ref args) = self.template.arguments {
            if let Ok(args_vec) = serde_json::from_str::<Vec<String>>(args) {
                template_args = args_vec;
            }
        }

        // Аргументы из задачи
        if let Some(ref args) = self.task.arguments {
            if let Ok(args_vec) = serde_json::from_str::<Vec<String>>(args) {
                task_args = args_vec;
            }
        }

        Ok((template_args, task_args))
    }

    /// Получает аргументы CLI в виде карты (для Terraform стадий)
    pub fn get_cli_args_map(&self) -> Result<(HashMap<String, Vec<String>>, HashMap<String, Vec<String>>)> {
        let mut template_args_map = HashMap::new();
        let mut task_args_map = HashMap::new();

        // Аргументы из шаблона
        if let Some(ref args) = self.template.arguments {
            // Пробуем распарсить как HashMap
            if let Ok(map) = serde_json::from_str::<HashMap<String, Vec<String>>>(args) {
                template_args_map = map;
            } else {
                // Если не удалось, пробуем как Vec<String>
                if let Ok(args_vec) = serde_json::from_str::<Vec<String>>(args) {
                    template_args_map.insert("default".to_string(), args_vec);
                }
            }
        }

        // Аргументы из задачи
        if let Some(ref args) = self.task.arguments {
            // Пробуем распарсить как HashMap
            if let Ok(map) = serde_json::from_str::<HashMap<String, Vec<String>>>(args) {
                task_args_map = map;
            } else {
                // Если не удалось, пробуем как Vec<String>
                if let Ok(args_vec) = serde_json::from_str::<Vec<String>>(args) {
                    task_args_map.insert("default".to_string(), args_vec);
                }
            }
        }

        Ok((template_args_map, task_args_map))
    }

    /// Получает параметры шаблона
    pub fn get_template_params(&self) -> Result<Value> {
        let params: Value = serde_json::from_str(&self.template.params)?;
        Ok(params)
    }

    /// Получает параметры задачи
    pub fn get_params<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let params: T = serde_json::from_str(&self.task.params)?;
        Ok(params)
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

    fn create_test_job_with_args() -> LocalJob {
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
            arguments: Some(r#"["--arg1", "--arg2"]"#.to_string()),
            params: r#"{"key": "value"}"#.to_string(),
            ..Default::default()
        };

        let template = crate::models::Template {
            id: 1,
            name: String::from("Test Template"),
            project_id: 1,
            playbook: String::from("test.yml"),
            template_type: crate::models::TemplateType::Task,
            arguments: Some(r#"["--template-arg"]"#.to_string()),
            params: r#"{"template_key": "template_value"}"#.to_string(),
            ..Default::default()
        };

        LocalJob::new(
            task,
            template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        )
    }

    #[test]
    fn test_get_cli_args() {
        let job = create_test_job_with_args();
        let (template_args, task_args) = job.get_cli_args().unwrap();

        assert_eq!(template_args.len(), 1);
        assert_eq!(template_args[0], "--template-arg");
        assert_eq!(task_args.len(), 2);
        assert_eq!(task_args[0], "--arg1");
        assert_eq!(task_args[1], "--arg2");
    }

    #[test]
    fn test_get_cli_args_map() {
        let job = create_test_job_with_args();
        let (template_map, task_map) = job.get_cli_args_map().unwrap();

        assert!(template_map.contains_key("default"));
        assert!(task_map.contains_key("default"));
    }

    #[test]
    fn test_get_template_params() {
        let job = create_test_job_with_args();
        let params = job.get_template_params().unwrap();

        assert!(params.is_object());
    }
}
