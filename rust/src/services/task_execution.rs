//! Сервис выполнения задач
//!
//! Предоставляет единую точку запуска задач, используемую как HTTP-хендлером,
//! так и планировщиком (scheduler).

use chrono::Utc;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

use crate::db::store::{PlanApprovalManager, Store};
use crate::db_lib::AccessKeyInstallerImpl;
use crate::models::{Environment, Inventory, Repository, Task, TaskOutput, TerraformPlan};
use crate::services::local_job::LocalJob;
use crate::services::task_logger::{BasicLogger, LogListener, TaskLogger, TaskStatus};

/// Запускает задачу в фоновом потоке.
///
/// Загружает шаблон, инвентарь, репозиторий и окружение, запускает LocalJob,
/// сохраняет вывод в БД и обновляет статус задачи.
pub async fn execute_task(store: Arc<dyn Store + Send + Sync>, mut task: Task) {
    info!(
        "[task_runner] Starting task {} (template {})",
        task.id, task.template_id
    );

    // Обновляем статус → Running и фиксируем время начала
    task.status = TaskStatus::Running;
    task.start = Some(Utc::now());
    match store.update_task(task.clone()).await {
        Ok(()) => info!("[task_runner] task {} status → Running", task.id),
        Err(e) => error!("[task_runner] task {} failed to set Running: {e}", task.id),
    }

    // Загружаем шаблон
    let template = match store.get_template(task.project_id, task.template_id).await {
        Ok(t) => t,
        Err(e) => {
            error!(
                "[task_runner] task {}: failed to get template: {e}",
                task.id
            );
            task.status = TaskStatus::Error;
            task.end = Some(Utc::now());
            let _ = store.update_task(task).await;
            return;
        }
    };

    // Phase 2: Plan Approval gate — if template requires approval, pause before executing
    if template.require_approval {
        // Check if there's already an approved plan for this task
        let existing_plan = store
            .get_plan_by_task(task.project_id, task.id)
            .await
            .unwrap_or(None);
        match existing_plan {
            Some(ref plan) if plan.status == "approved" => {
                // Plan was approved — proceed with execution
                info!(
                    "[task_runner] task {}: plan approved, proceeding with execution",
                    task.id
                );
            }
            Some(ref plan) if plan.status == "rejected" => {
                // Plan was rejected — stop task
                info!(
                    "[task_runner] task {}: plan rejected, stopping task",
                    task.id
                );
                let _ = store
                    .update_task_status(task.project_id, task.id, TaskStatus::Error)
                    .await;
                return;
            }
            None => {
                // No plan yet — create pending record and set WaitingConfirmation
                info!(
                    "[task_runner] task {}: require_approval=true, creating pending plan",
                    task.id
                );
                let pending_plan = TerraformPlan {
                    id: 0,
                    task_id: task.id,
                    project_id: task.project_id,
                    plan_output: String::new(),
                    plan_json: None,
                    resources_added: 0,
                    resources_changed: 0,
                    resources_removed: 0,
                    status: "pending".to_string(),
                    created_at: chrono::Utc::now(),
                    reviewed_at: None,
                    reviewed_by: None,
                    review_comment: None,
                };
                let _ = store.create_plan(pending_plan).await;
                let _ = store
                    .update_task_status(task.project_id, task.id, TaskStatus::WaitingConfirmation)
                    .await;
                return;
            }
            _ => {
                // Plan pending — still waiting for review
                info!("[task_runner] task {}: plan still pending review", task.id);
                let _ = store
                    .update_task_status(task.project_id, task.id, TaskStatus::WaitingConfirmation)
                    .await;
                return;
            }
        }
    }

    // Загружаем инвентарь, репозиторий, окружение
    let inventory_id = task.inventory_id.or(template.inventory_id);
    let inventory = match inventory_id {
        Some(id) => store
            .get_inventory(task.project_id, id)
            .await
            .unwrap_or_default(),
        None => Inventory::default(),
    };

    let repository_id = task.repository_id.or(template.repository_id);
    let repository = match repository_id {
        Some(id) => store
            .get_repository(task.project_id, id)
            .await
            .unwrap_or_default(),
        None => Repository::default(),
    };

    let environment_id = task.environment_id.or(template.environment_id);
    let environment = match environment_id {
        Some(id) => store
            .get_environment(task.project_id, id)
            .await
            .unwrap_or_default(),
        None => Environment::default(),
    };

    // Логгер с буфером для сохранения в БД
    let log_buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let buf_clone = log_buffer.clone();
    let logger = Arc::new(BasicLogger::new());
    logger.add_log_listener(Box::new(move |_time, msg| {
        let _ = buf_clone.lock().map(|mut v| v.push(msg));
    }));

    let work_dir =
        std::env::temp_dir().join(format!("semaphore_task_{}_{}", task.project_id, task.id));
    let tmp_dir = work_dir.join("tmp");

    if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
        error!(
            "[task_runner] task {}: failed to create workdir: {e}",
            task.id
        );
        task.status = TaskStatus::Error;
        task.end = Some(Utc::now());
        let _ = store.update_task(task).await;
        return;
    }

    let key_installer = AccessKeyInstallerImpl::new();
    let mut job = LocalJob::new(
        task.clone(),
        template,
        inventory,
        repository,
        environment,
        logger,
        key_installer,
        work_dir,
        tmp_dir,
    );

    job.store = Some(store.clone());
    let result = job.run("runner", None, "default").await;
    job.cleanup();

    // Сохраняем логи в БД
    let log_lines: Vec<String> = log_buffer.lock().map(|v| v.clone()).unwrap_or_default();
    for line in log_lines {
        let output = TaskOutput {
            id: 0,
            task_id: task.id,
            project_id: task.project_id,
            time: chrono::Utc::now(),
            output: line,
            stage_id: None,
        };
        let _ = store.create_task_output(output).await;
    }

    task.end = Some(Utc::now());
    match result {
        Ok(()) => {
            info!("[task_runner] task {} completed successfully", task.id);
            task.status = TaskStatus::Success;
            let _ = store.update_task(task).await;
        }
        Err(e) => {
            error!("[task_runner] task {} failed: {e}", task.id);
            task.status = TaskStatus::Error;
            let _ = store.update_task(task).await;
        }
    }
}

/// Отправляет Telegram уведомление о завершении задачи
pub async fn send_telegram_notification(
    telegram_bot: Option<&std::sync::Arc<crate::services::telegram_bot::TelegramBot>>,
    task: &Task,
    template_name: &str,
    project_name: &str,
    author: &str,
) {
    let Some(bot) = telegram_bot else {
        return;
    };

    let task_url = format!(
        "{}/project/{}/tasks/{}",
        crate::config::get_public_host(),
        task.project_id,
        task.id
    );

    let duration_secs = task
        .end
        .zip(task.start)
        .map(|(end, start)| (end - start).num_seconds() as u64)
        .unwrap_or(0);

    match task.status {
        TaskStatus::Success => {
            bot.notify_task_success(
                project_name,
                template_name,
                task.id,
                author,
                duration_secs,
                &task_url,
            )
            .await;
        }
        TaskStatus::Error => {
            bot.notify_task_failed(
                project_name,
                template_name,
                task.id,
                author,
                duration_secs,
                &task_url,
            )
            .await;
        }
        TaskStatus::Stopped => {
            bot.notify_task_stopped(project_name, template_name, task.id, &task_url)
                .await;
        }
        _ => {}
    }
}
