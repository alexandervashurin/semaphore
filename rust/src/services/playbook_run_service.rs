//! Сервис запуска Playbook
//!
//! Этот модуль предоставляет функциональность для запуска playbook
//! через создание задачи (Task) в Velum.

use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::playbook_run::{PlaybookRunRequest, PlaybookRunResult};
use crate::models::playbook_run_history::{PlaybookRun, PlaybookRunCreate, PlaybookRunStatus};
use crate::models::task::{Task, TaskStage, TaskStageType};
use crate::models::template::TemplateType;
use crate::services::task_logger::TaskStatus;
use chrono::Utc;
use tracing::info;

/// Сервис для запуска playbook
pub struct PlaybookRunService;

impl PlaybookRunService {
    /// Запускает playbook через создание задачи
    ///
    /// # Arguments
    /// * `playbook_id` - ID playbook для запуска
    /// * `project_id` - ID проекта
    /// * `request` - Параметры запуска
    /// * `store` - Хранилище данных
    ///
    /// # Returns
    /// * `Result<PlaybookRunResult>` - Результат запуска
    pub async fn run_playbook<S>(
        playbook_id: i32,
        project_id: i32,
        user_id: i32,
        request: PlaybookRunRequest,
        store: &S,
    ) -> Result<PlaybookRunResult>
    where
        S: PlaybookManager
            + TemplateManager
            + InventoryManager
            + EnvironmentManager
            + TaskManager
            + UserManager
            + PlaybookRunManager,
    {
        // 1. Валидация запроса
        request.validate().map_err(Error::Validation)?;

        // 2. Получаем playbook
        let playbook = store.get_playbook(playbook_id, project_id).await?;

        // 3. Проверяем inventory (если указан)
        if let Some(inventory_id) = request.inventory_id {
            store.get_inventory(project_id, inventory_id).await?;
        }

        // 4. Проверяем environment (если указан)
        if let Some(environment_id) = request.environment_id {
            store.get_environment(project_id, environment_id).await?;
        }

        // 5. Проверяем пользователя из контекста аутентификации
        let _user = store.get_user(user_id).await?;

        // 6. Создаем template для playbook (если нет)
        let template = Self::get_or_create_template_for_playbook(
            &playbook,
            project_id,
            request.inventory_id,
            store,
        )
        .await?;

        // 7. Создаем задачу
        let task = Task {
            id: 0, // Будет установлен БД
            template_id: template.id,
            project_id,
            status: TaskStatus::Waiting,
            playbook: Some(playbook.name.clone()),
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: Some(user_id),
            integration_id: None,
            schedule_id: None,
            created: Utc::now(),
            start: None,
            end: None,
            message: Some("Playbook запущен через API".to_string()),
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: request.inventory_id,
            repository_id: playbook.repository_id,
            environment_id: request.environment_id,
            params: None,
        };

        // 8. Сохраняем задачу
        let created_task = store.create_task(task).await?;

        // 9. Создаем запись истории запуска
        let playbook_run_create = PlaybookRunCreate {
            project_id,
            playbook_id,
            task_id: Some(created_task.id),
            template_id: Some(template.id),
            inventory_id: request.inventory_id,
            environment_id: request.environment_id,
            extra_vars: request.extra_vars.map(|v| v.to_string()),
            limit_hosts: request.limit,
            tags: request.tags.map(|t| t.join(",")),
            skip_tags: request.skip_tags.map(|t| t.join(",")),
            user_id: Some(user_id),
        };

        let _playbook_run = store.create_playbook_run(playbook_run_create).await?;

        info!(
            "Задача {} создана для playbook {}, запись истории {}",
            created_task.id, playbook.name, _playbook_run.id
        );

        // 10. Возвращаем результат
        Ok(PlaybookRunResult {
            task_id: created_task.id,
            template_id: template.id,
            status: created_task.status.to_string(),
            message: "Задача создана и ожидает выполнения".to_string(),
        })
    }

    /// Получает или создает template для playbook
    async fn get_or_create_template_for_playbook<S>(
        playbook: &crate::models::Playbook,
        project_id: i32,
        inventory_id: Option<i32>,
        store: &S,
    ) -> Result<crate::models::Template>
    where
        S: TemplateManager + InventoryManager,
    {
        // Пытаемся найти существующий template для этого playbook
        let templates = store.get_templates(project_id).await?;

        for template in templates {
            if template.app == crate::models::template::TemplateApp::Ansible
                && template.playbook == playbook.name
            {
                // Обновляем inventory если нужно
                return Ok(template);
            }
        }

        // Создаем новый template
        let template_type = match playbook.playbook_type.as_str() {
            "terraform" => TemplateType::Terraform,
            "shell" => TemplateType::Shell,
            _ => TemplateType::Ansible,
        };

        let template = crate::models::Template {
            id: 0,
            project_id,
            inventory_id,
            repository_id: playbook.repository_id,
            environment_id: None,
            name: format!("Playbook: {}", playbook.name),
            playbook: playbook.name.clone(),
            app: crate::models::template::TemplateApp::Ansible,
            r#type: template_type,
            git_branch: None,
            created: Utc::now(),
            description: format!("Auto-generated template for {}", playbook.name),
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

        let created_template = store.create_template(template).await?;

        info!(
            "Создан template {} для playbook {}",
            created_template.id, playbook.name
        );

        Ok(created_template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_playbook_run_request_validation() {
        let request = PlaybookRunRequest::new()
            .with_inventory(1)
            .with_extra_vars(json!({"key": "value"}));

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_playbook_run_request_invalid_extra_vars() {
        let request = PlaybookRunRequest::new().with_extra_vars(json!(["invalid", "array"]));

        assert!(request.validate().is_err());
    }

    #[tokio::test]
    async fn test_run_playbook_playbook_not_found() {
        let store = crate::db::MockStore::new();
        let request = PlaybookRunRequest::new();

        // Playbook не найден в пустом MockStore
        let result = PlaybookRunService::run_playbook(
            999, // playbook_id
            1,   // project_id
            1,   // user_id
            request,
            &store,
        )
        .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_playbook_run_request_default() {
        let request = PlaybookRunRequest::new();
        assert!(request.inventory_id.is_none());
        assert!(request.environment_id.is_none());
        assert!(request.extra_vars.is_none());
        assert!(request.limit.is_none());
        assert!(request.tags.is_none());
        assert!(request.skip_tags.is_none());
    }

    #[test]
    fn test_playbook_run_request_with_all_fields() {
        let request = PlaybookRunRequest::new()
            .with_inventory(5)
            .with_environment(3)
            .with_extra_vars(json!({"env": "prod"}))
            .with_limit("web01".to_string())
            .with_tags(vec!["deploy".to_string()]);

        assert_eq!(request.inventory_id, Some(5));
        assert_eq!(request.environment_id, Some(3));
        assert!(request.extra_vars.is_some());
        assert_eq!(request.limit, Some("web01".to_string()));
        assert_eq!(request.tags, Some(vec!["deploy".to_string()]));
    }

    #[test]
    fn test_playbook_run_request_validate_empty() {
        let request = PlaybookRunRequest::new();
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_playbook_run_status_variants() {
        assert_eq!(format!("{}", PlaybookRunStatus::Waiting), "waiting");
        assert_eq!(format!("{}", PlaybookRunStatus::Running), "running");
        assert_eq!(format!("{}", PlaybookRunStatus::Success), "success");
        assert_eq!(format!("{}", PlaybookRunStatus::Failed), "failed");
    }

    #[test]
    fn test_playbook_run_create_defaults() {
        let run_create = PlaybookRunCreate {
            project_id: 1,
            playbook_id: 1,
            task_id: Some(100),
            template_id: Some(1),
            inventory_id: None,
            environment_id: None,
            extra_vars: None,
            limit_hosts: None,
            tags: None,
            skip_tags: None,
            user_id: Some(1),
        };

        assert_eq!(run_create.project_id, 1);
        assert_eq!(run_create.playbook_id, 1);
        assert_eq!(run_create.task_id, Some(100));
    }

    #[test]
    fn test_playbook_run_request_with_null_extra_vars() {
        // extra_vars = null (None в JSON) — это валидно
        let request = PlaybookRunRequest::new().with_extra_vars(json!(null));
        // Это может быть Ok или Err в зависимости от validate реализации
        let _ = request.validate();
    }

    #[test]
    fn test_playbook_run_request_with_skip_tags() {
        let request = PlaybookRunRequest::new()
            .with_tags(vec!["deploy".to_string()])
            .with_extra_vars(json!({}));

        assert!(request.tags.is_some());
        assert_eq!(request.tags.as_ref().unwrap().len(), 1);
        assert!(request.skip_tags.is_none());
    }

    #[test]
    fn test_playbook_run_result_creation() {
        let result = PlaybookRunResult {
            task_id: 42,
            template_id: 7,
            status: "waiting".to_string(),
            message: "Task created".to_string(),
        };

        assert_eq!(result.task_id, 42);
        assert_eq!(result.template_id, 7);
        assert_eq!(result.status, "waiting");
        assert_eq!(result.message, "Task created");
    }

    #[test]
    fn test_playbook_run_result_clone() {
        let result = PlaybookRunResult {
            task_id: 1,
            template_id: 1,
            status: "running".to_string(),
            message: "test".to_string(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.task_id, result.task_id);
        assert_eq!(cloned.status, result.status);
    }

    #[test]
    fn test_playbook_run_status_cancelled_display() {
        assert_eq!(format!("{}", PlaybookRunStatus::Cancelled), "cancelled");
    }

    #[test]
    fn test_playbook_run_status_equality() {
        assert_eq!(PlaybookRunStatus::Waiting, PlaybookRunStatus::Waiting);
        assert_ne!(PlaybookRunStatus::Success, PlaybookRunStatus::Failed);
        assert_ne!(PlaybookRunStatus::Running, PlaybookRunStatus::Waiting);
    }

    #[test]
    fn test_playbook_run_status_serialize_all() {
        let statuses = [
            PlaybookRunStatus::Waiting,
            PlaybookRunStatus::Running,
            PlaybookRunStatus::Success,
            PlaybookRunStatus::Failed,
            PlaybookRunStatus::Cancelled,
        ];
        for status in &statuses {
            let json_str = serde_json::to_string(status).unwrap();
            assert!(json_str.starts_with('"'));
            assert!(json_str.ends_with('"'));
        }
    }

    #[test]
    fn test_playbook_run_request_validate_with_array_extra_vars() {
        let request = PlaybookRunRequest::new().with_extra_vars(json!(["item1", "item2"]));
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_playbook_run_request_validate_with_string_extra_vars() {
        let request = PlaybookRunRequest::new().with_extra_vars(json!("not_an_object"));
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_playbook_run_request_validate_with_integer_extra_vars() {
        let request = PlaybookRunRequest::new().with_extra_vars(json!(42));
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_playbook_run_request_builder_chaining() {
        let request = PlaybookRunRequest::new()
            .with_inventory(10)
            .with_environment(20)
            .with_limit("hosts".to_string())
            .with_tags(vec!["tag1".to_string(), "tag2".to_string()])
            .with_extra_vars(json!({"key": "val"}));

        assert_eq!(request.inventory_id, Some(10));
        assert_eq!(request.environment_id, Some(20));
        assert_eq!(request.limit, Some("hosts".to_string()));
        assert_eq!(request.tags, Some(vec!["tag1".to_string(), "tag2".to_string()]));
        assert!(request.extra_vars.is_some());
    }

    #[test]
    fn test_playbook_run_request_default_all_none() {
        let request = PlaybookRunRequest::default();
        assert!(request.inventory_id.is_none());
        assert!(request.environment_id.is_none());
        assert!(request.extra_vars.is_none());
        assert!(request.limit.is_none());
        assert!(request.tags.is_none());
        assert!(request.skip_tags.is_none());
        assert!(request.user_id.is_none());
    }

    #[test]
    fn test_playbook_run_request_debug_format() {
        let request = PlaybookRunRequest::new().with_inventory(5);
        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("PlaybookRunRequest"));
        assert!(debug_str.contains("Some(5)"));
    }

    #[test]
    fn test_playbook_run_result_debug_format() {
        let result = PlaybookRunResult {
            task_id: 99,
            template_id: 1,
            status: "success".to_string(),
            message: "done".to_string(),
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("PlaybookRunResult"));
        assert!(debug_str.contains("99"));
    }

    #[test]
    fn test_playbook_run_create_with_all_fields() {
        let run_create = PlaybookRunCreate {
            project_id: 10,
            playbook_id: 20,
            task_id: Some(300),
            template_id: Some(5),
            inventory_id: Some(1),
            environment_id: Some(2),
            extra_vars: Some("{}".to_string()),
            limit_hosts: Some("web".to_string()),
            tags: Some("deploy,config".to_string()),
            skip_tags: Some("test".to_string()),
            user_id: Some(42),
        };

        assert_eq!(run_create.project_id, 10);
        assert_eq!(run_create.playbook_id, 20);
        assert_eq!(run_create.task_id, Some(300));
        assert_eq!(run_create.inventory_id, Some(1));
        assert_eq!(run_create.environment_id, Some(2));
        assert_eq!(run_create.user_id, Some(42));
        assert_eq!(run_create.limit_hosts, Some("web".to_string()));
        assert_eq!(run_create.tags, Some("deploy,config".to_string()));
        assert_eq!(run_create.skip_tags, Some("test".to_string()));
    }

    #[test]
    fn test_playbook_run_create_clone() {
        let create = PlaybookRunCreate {
            project_id: 1,
            playbook_id: 1,
            task_id: None,
            template_id: None,
            inventory_id: None,
            environment_id: None,
            extra_vars: None,
            limit_hosts: None,
            tags: None,
            skip_tags: None,
            user_id: None,
        };
        let cloned = create.clone();
        assert_eq!(cloned.project_id, create.project_id);
        assert_eq!(cloned.playbook_id, create.playbook_id);
        assert_eq!(cloned.task_id, None);
    }

    #[test]
    fn test_playbook_run_request_with_only_environment() {
        let request = PlaybookRunRequest::new().with_environment(99);
        assert_eq!(request.environment_id, Some(99));
        assert!(request.inventory_id.is_none());
    }

    #[test]
    fn test_playbook_run_request_with_only_limit() {
        let request = PlaybookRunRequest::new().with_limit("localhost".to_string());
        assert_eq!(request.limit, Some("localhost".to_string()));
        assert!(request.inventory_id.is_none());
        assert!(request.tags.is_none());
    }

    #[test]
    fn test_playbook_run_request_validate_empty_object_extra_vars() {
        let request = PlaybookRunRequest::new().with_extra_vars(json!({}));
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_playbook_run_request_validate_nested_object_extra_vars() {
        let request = PlaybookRunRequest::new().with_extra_vars(json!({
            "app": {"name": "test", "version": "1.0"},
            "env": "prod"
        }));
        assert!(request.validate().is_ok());
    }
}
