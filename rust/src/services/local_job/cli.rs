//! LocalJob CLI - работа с аргументами командной строки
//!
//! Аналог services/tasks/local_job_cli.go из Go версии

use serde_json::Value;
use std::collections::HashMap;

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
    #[allow(clippy::type_complexity)]
    pub fn get_cli_args_map(
        &self,
    ) -> Result<(HashMap<String, Vec<String>>, HashMap<String, Vec<String>>)> {
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

    /// Получает параметры шаблона (из задачи)
    pub fn get_template_params(&self) -> Result<Value> {
        self.task.params.clone().map(Ok).unwrap_or(Ok(Value::Null))
    }

    /// Получает параметры задачи
    pub fn get_params<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let params_str = self
            .task
            .params
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();
        let params: T = serde_json::from_str(&params_str)?;
        Ok(params)
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

    fn create_test_job_with_args() -> LocalJob {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some(r#"["--arg1", "--arg2"]"#.to_string());
        task.params = Some(serde_json::json!({"key": "value"}));

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test Template".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some(r#"["--template-arg"]"#.to_string());

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
