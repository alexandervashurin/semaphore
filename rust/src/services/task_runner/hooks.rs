//! TaskRunner Hooks - hooks для задач
//!
//! Аналог services/tasks/task_runner_hooks.go из Go версии

use crate::error::Result;
use crate::models::{Hook, HookType};
use crate::services::task_runner::TaskRunner;

impl TaskRunner {
    /// run_hooks запускает hooks для задачи
    pub async fn run_hooks(&self, event_type: &str) -> Result<()> {
        // Получение hooks из хранилища по template_id
        let hooks = self
            .pool
            .store()
            .get_hooks_by_template(self.task.template_id)
            .await
            .unwrap_or_default();

        if hooks.is_empty() {
            return Ok(());
        }

        // Запуск hooks для указанного события
        for hook in hooks {
            // Проверяем, подходит ли hook для этого события
            if self.hook_matches_event(&hook, event_type) {
                if let Err(e) = self.execute_hook(&hook).await {
                    self.log(&format!("Hook '{}' failed: {}", hook.name, e));
                    // Продолжаем выполнение даже если hook не удался
                }
            }
        }

        Ok(())
    }

    /// Проверяет, соответствует ли hook событию
    fn hook_matches_event(&self, hook: &Hook, event_type: &str) -> bool {
        // В простой реализации все hooks выполняются для всех событий
        // В полной версии можно добавить поле event в Hook
        hook.name.contains(event_type) || event_type == "all"
    }

    /// execute_hook выполняет один hook
    async fn execute_hook(&self, hook: &Hook) -> Result<()> {
        self.log(&format!(
            "Executing hook: {} (type: {:?})",
            hook.name, hook.r#type
        ));

        match hook.r#type {
            HookType::Http => {
                // HTTP запрос
                if let Some(url) = &hook.url {
                    let client = reqwest::Client::new();
                    let method = hook.http_method.as_deref().unwrap_or("GET");

                    let request = match method.to_uppercase().as_str() {
                        "GET" => client.get(url),
                        "POST" => client.post(url),
                        "PUT" => client.put(url),
                        "DELETE" => client.delete(url),
                        _ => client.get(url),
                    };

                    // Добавляем тело запроса если есть
                    let request = if let Some(body) = &hook.http_body {
                        request.body(body.clone())
                    } else {
                        request
                    };

                    // Устанавливаем таймаут если указан
                    let request = if let Some(timeout) = hook.timeout_secs {
                        request.timeout(std::time::Duration::from_secs(timeout as u64))
                    } else {
                        request
                    };

                    let response = request.send().await;
                    match response {
                        Ok(resp) => {
                            self.log(&format!(
                                "Hook '{}' completed with status: {}",
                                hook.name,
                                resp.status()
                            ));
                        }
                        Err(e) => {
                            return Err(crate::error::Error::Other(format!(
                                "HTTP hook failed: {}",
                                e
                            )));
                        }
                    }
                }
            }
            HookType::Bash | HookType::Python => {
                // Выполнение скрипта
                if let Some(script) = &hook.script {
                    let (shell, arg) = match hook.r#type {
                        HookType::Bash => ("bash", "-c"),
                        HookType::Python => ("python3", "-c"),
                        _ => ("bash", "-c"),
                    };

                    let output = tokio::process::Command::new(shell)
                        .arg(arg)
                        .arg(script)
                        .output()
                        .await;

                    match output {
                        Ok(out) => {
                            let stdout = String::from_utf8_lossy(&out.stdout);
                            let stderr = String::from_utf8_lossy(&out.stderr);

                            if !stdout.is_empty() {
                                self.log(&format!("Hook '{}' stdout: {}", hook.name, stdout));
                            }
                            if !stderr.is_empty() {
                                self.log(&format!("Hook '{}' stderr: {}", hook.name, stderr));
                            }

                            if !out.status.success() {
                                return Err(crate::error::Error::Other(format!(
                                    "Script hook failed with exit code: {:?}",
                                    out.status.code()
                                )));
                            }
                        }
                        Err(e) => {
                            return Err(crate::error::Error::Other(format!(
                                "Script hook failed: {}",
                                e
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// run_before_hooks запускает hooks перед задачей
    pub async fn run_before_hooks(&self) -> Result<()> {
        self.run_hooks("before_task").await
    }

    /// run_after_hooks запускает hooks после задачи
    pub async fn run_after_hooks(&self) -> Result<()> {
        self.run_hooks("after_task").await
    }

    /// run_on_success_hooks запускает hooks при успешном завершении
    pub async fn run_on_success_hooks(&self) -> Result<()> {
        self.run_hooks("on_success").await
    }

    /// run_on_failure_hooks запускает hooks при ошибке
    pub async fn run_on_failure_hooks(&self) -> Result<()> {
        self.run_hooks("on_failure").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::MockStore;
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::models::{Project, Task};
    use crate::services::task_logger::TaskStatus;
    use crate::services::task_pool::TaskPool;
    use chrono::Utc;
    use std::sync::Arc;

    fn create_test_task_runner() -> TaskRunner {
        let task = Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: TaskStatus::Waiting,
            message: None,
            commit_hash: None,
            commit_message: None,
            version: None,
            project_id: 1,
            ..Default::default()
        };

        let pool = Arc::new(TaskPool::new(Arc::new(MockStore::new()), 5));

        TaskRunner::new(
            task,
            pool,
            "testuser".to_string(),
            AccessKeyInstallerImpl::new(),
        )
    }

    #[tokio::test]
    async fn test_run_hooks_empty() {
        let runner = create_test_task_runner();
        let result = runner.run_hooks("before_task").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_before_hooks() {
        let runner = create_test_task_runner();
        let result = runner.run_before_hooks().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_after_hooks() {
        let runner = create_test_task_runner();
        let result = runner.run_after_hooks().await;
        assert!(result.is_ok());
    }
}
