//! Сервис запуска Playbook
//!
//! Этот модуль предоставляет функциональность для запуска playbook
//! через создание задачи (Task) в Semaphore.

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
        request: PlaybookRunRequest,
        store: &S,
    ) -> Result<PlaybookRunResult>
    where
        S: PlaybookManager + TemplateManager + InventoryManager + EnvironmentManager + TaskManager + UserManager + PlaybookRunManager,
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

        // 5. Получаем пользователя (если указан)
        let user_id = request.user_id.unwrap_or(1); // TODO: получить из контекста аутентификации
        let _user = store.get_user(user_id).await?;

        // 6. Создаем template для playbook (если нет)
        let template = Self::get_or_create_template_for_playbook(
            &playbook,
            project_id,
            request.inventory_id,
            store,
        ).await?;

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
            created_task.id,
            playbook.name,
            _playbook_run.id
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
            task_params: None,
            survey_vars: None,
            vaults: None,
        };

        let created_template = store.create_template(template).await?;

        info!(
            "Создан template {} для playbook {}",
            created_template.id,
            playbook.name
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
        let request = PlaybookRunRequest::new()
            .with_extra_vars(json!(["invalid", "array"]));

        assert!(request.validate().is_err());
    }
}
