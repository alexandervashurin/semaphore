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

    #[test]
    fn test_get_cli_args_with_no_arguments() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = None;

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = None;

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (template_args, task_args) = job.get_cli_args().unwrap();
        assert!(template_args.is_empty());
        assert!(task_args.is_empty());
    }

    #[test]
    fn test_get_cli_args_with_invalid_json() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some("not valid json".to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some("{broken".to_string());

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        // Должно вернуть пустые вектора при невалидном JSON
        let (template_args, task_args) = job.get_cli_args().unwrap();
        assert!(template_args.is_empty());
        assert!(task_args.is_empty());
    }

    #[test]
    fn test_get_cli_args_map_with_hashmap_format() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some(r#"{"apply": ["-auto-approve"], "plan": ["-detailed-exitcode"]}"#.to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some(r#"{"init": ["-upgrade"]}"#.to_string());

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (template_map, task_map) = job.get_cli_args_map().unwrap();
        assert!(template_map.contains_key("init"));
        assert_eq!(template_map["init"], vec!["-upgrade"]);
        assert!(task_map.contains_key("apply"));
        assert_eq!(task_map["apply"], vec!["-auto-approve"]);
        assert!(task_map.contains_key("plan"));
        assert_eq!(task_map["plan"], vec!["-detailed-exitcode"]);
    }

    #[test]
    fn test_get_cli_args_empty_strings() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some(r#"[""]"#.to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some(r#"[""]"#.to_string());

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (template_args, task_args) = job.get_cli_args().unwrap();
        assert_eq!(template_args.len(), 1);
        assert_eq!(template_args[0], "");
        assert_eq!(task_args.len(), 1);
        assert_eq!(task_args[0], "");
    }

    #[test]
    fn test_get_cli_args_with_many_args() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some(r#"["-a", "-b", "-c", "-d", "-e", "-f", "-g"]"#.to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some(r#"["-x", "-y", "-z"]"#.to_string());

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (template_args, task_args) = job.get_cli_args().unwrap();
        assert_eq!(template_args.len(), 3);
        assert_eq!(task_args.len(), 7);
    }

    #[test]
    fn test_get_cli_args_map_fallback_vec_to_default() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some(r#"["--flag1", "--flag2"]"#.to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some(r#"["--tmpl-flag"]"#.to_string());

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (template_map, task_map) = job.get_cli_args_map().unwrap();
        assert!(template_map.contains_key("default"));
        assert_eq!(template_map["default"], vec!["--tmpl-flag"]);
        assert!(task_map.contains_key("default"));
        assert_eq!(task_map["default"], vec!["--flag1", "--flag2"]);
    }

    #[test]
    fn test_get_template_params_with_none() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.params = None;

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let params = job.get_template_params().unwrap();
        assert!(params.is_null());
    }

    #[test]
    fn test_get_params_deserialize_struct() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct TestParams {
            key: String,
        }

        let job = create_test_job_with_args();
        let params: TestParams = job.get_params().unwrap();
        assert_eq!(params.key, "value");
    }

    #[test]
    fn test_get_params_with_none_returns_error() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct TestParams {
            key: String,
        }

        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.params = None;

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let result: std::result::Result<TestParams, _> = job.get_params();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_cli_args_map_with_empty_json_object() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some("{}".to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some("{}".to_string());

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (template_map, task_map) = job.get_cli_args_map().unwrap();
        assert!(template_map.is_empty() || template_map.contains_key("default"));
        assert!(task_map.is_empty() || task_map.contains_key("default"));
    }

    #[test]
    fn test_get_cli_args_preserves_arg_order() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some(r#"["--third", "--fourth", "--fifth"]"#.to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some(r#"["--first", "--second"]"#.to_string());

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (template_args, task_args) = job.get_cli_args().unwrap();
        assert_eq!(template_args[0], "--first");
        assert_eq!(template_args[1], "--second");
        assert_eq!(task_args[0], "--third");
        assert_eq!(task_args[1], "--fourth");
        assert_eq!(task_args[2], "--fifth");
    }

    #[test]
    fn test_get_template_params_with_complex_object() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.params = Some(serde_json::json!({
            "nested": {"a": 1, "b": [1, 2, 3]},
            "flag": true,
            "count": 42
        }));

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let params = job.get_template_params().unwrap();
        assert!(params.get("nested").is_some());
        assert!(params.get("flag").unwrap().as_bool().unwrap());
        assert_eq!(params.get("count").unwrap().as_i64().unwrap(), 42);
    }

    #[test]
    fn test_get_cli_args_map_only_task_args() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some(r#"{"deploy": ["--force"]}"#.to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = None;

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (template_map, task_map) = job.get_cli_args_map().unwrap();
        assert!(template_map.is_empty());
        assert!(task_map.contains_key("deploy"));
        assert_eq!(task_map["deploy"], vec!["--force"]);
    }

    #[test]
    fn test_get_cli_args_with_single_argument() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some(r#"["--only"]"#.to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = None;

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (template_args, task_args) = job.get_cli_args().unwrap();
        assert!(template_args.is_empty());
        assert_eq!(task_args.len(), 1);
        assert_eq!(task_args[0], "--only");
    }

    #[test]
    fn test_get_cli_args_map_with_both_hashmap_format() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some(r#"{"apply": ["-auto-approve"]}"#.to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.name = "Test".to_string();
        template.project_id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some(r#"{"init": ["-backend=false"]}"#.to_string());

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (tmpl_map, task_map) = job.get_cli_args_map().unwrap();
        assert_eq!(tmpl_map["init"], vec!["-backend=false"]);
        assert_eq!(task_map["apply"], vec!["-auto-approve"]);
    }

    #[test]
    fn test_get_template_params_with_empty_object() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.params = Some(serde_json::json!({}));

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let params = job.get_template_params().unwrap();
        assert!(params.is_object());
        assert!(params.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_get_params_with_empty_string_returns_error() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct TestParams {
            key: String,
        }

        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.params = Some(serde_json::json!(""));

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let result: std::result::Result<TestParams, _> = job.get_params();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_cli_args_with_invalid_task_json_valid_template() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some("invalid json".to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some(r#"["--valid"]"#.to_string());

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (tmpl, task_args) = job.get_cli_args().unwrap();
        assert_eq!(tmpl, vec!["--valid"]);
        assert!(task_args.is_empty());
    }

    #[test]
    fn test_get_cli_args_map_with_invalid_json_falls_back() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some("not json".to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some("also not json".to_string());

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (tmpl_map, task_map) = job.get_cli_args_map().unwrap();
        assert!(tmpl_map.is_empty());
        assert!(task_map.is_empty());
    }

    #[test]
    fn test_get_cli_args_with_args_array_containing_empty_string() {
        let job = create_test_job_with_args();
        let (template_args, task_args) = job.get_cli_args().unwrap();
        assert_eq!(template_args, vec!["--template-arg"]);
        assert_eq!(task_args, vec!["--arg1", "--arg2"]);
    }

    #[test]
    fn test_get_template_params_returns_json_value() {
        let job = create_test_job_with_args();
        let params = job.get_template_params().unwrap();
        assert!(params.is_object());
        assert_eq!(params.get("key").unwrap().as_str().unwrap(), "value");
    }

    #[test]
    fn test_get_params_deserializes_integer_field() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct NumParams {
            key: serde_json::Value,
        }

        let job = create_test_job_with_args();
        let params: NumParams = job.get_params().unwrap();
        assert_eq!(params.key, "value");
    }

    #[test]
    fn test_get_cli_args_map_with_only_template_args() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = None;

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = Some(r#"{"plan": ["-out=tfplan"]}"#.to_string());

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (tmpl_map, task_map) = job.get_cli_args_map().unwrap();
        assert!(tmpl_map.contains_key("plan"));
        assert!(task_map.is_empty());
    }

    #[test]
    fn test_get_cli_args_with_long_argument_values() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let long_val = "a".repeat(1000);
        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some(serde_json::json!([format!("--val={}", long_val)]).to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = None;

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (_, task_args) = job.get_cli_args().unwrap();
        assert_eq!(task_args.len(), 1);
        assert!(task_args[0].starts_with("--val="));
    }

    #[test]
    fn test_get_cli_args_map_multiple_keys() {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let mut task = crate::models::Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();
        task.arguments = Some(r#"{"plan": ["-out=p"], "apply": ["-auto-approve"]}"#.to_string());

        let mut template = crate::models::Template::default();
        template.id = 1;
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.arguments = None;

        let job = LocalJob::new(
            task, template,
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger, key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        );

        let (_, task_map) = job.get_cli_args_map().unwrap();
        assert_eq!(task_map.len(), 2);
        assert!(task_map.contains_key("plan"));
        assert!(task_map.contains_key("apply"));
    }
}
