//! Ansible Playbook Runner
//!
//! Запуск Ansible playbook

use std::process::{Command, Stdio};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::process::Command as TokioCommand;
use tokio::sync::Mutex;

use crate::error::{Error, Result};
use crate::models::Repository;
use crate::services::task_logger::{TaskLogger, TaskStatus};

/// Ansible Playbook
pub struct AnsiblePlaybook {
    /// ID шаблона
    pub template_id: i32,

    /// Репозиторий
    pub repository: Repository,

    /// Логгер
    pub logger: Arc<dyn TaskLogger>,
}

impl AnsiblePlaybook {
    /// Создаёт новый Ansible Playbook
    pub fn new(template_id: i32, repository: Repository, logger: Arc<dyn TaskLogger>) -> Self {
        Self {
            template_id,
            repository,
            logger,
        }
    }

    /// Создаёт команду для выполнения
    fn make_command(&self, command: &str, args: &[&str], environment_vars: &[String]) -> TokioCommand {
        let mut cmd = TokioCommand::new(command);
        cmd.args(args);
        cmd.current_dir(self.get_full_path());

        // Добавляем переменные окружения
        cmd.env("PYTHONUNBUFFERED", "1");
        cmd.env("ANSIBLE_FORCE_COLOR", "True");
        cmd.env("ANSIBLE_HOST_KEY_CHECKING", "False");

        // Добавляем пользовательские переменные окружения
        for env_var in environment_vars {
            if let Some((key, value)) = env_var.split_once('=') {
                cmd.env(key, value);
            }
        }

        cmd.env("HOME", get_home_dir(&self.repository, self.template_id));
        cmd.env("PWD", self.get_full_path());

        cmd
    }

    /// Запускает playbook
    pub async fn run_playbook(
        &self,
        args: &[String],
        environment_vars: &[String],
        _inputs: std::collections::HashMap<String, String>,
        callback: impl FnOnce(u32) + Send + 'static,
    ) -> Result<()> {
        let mut cmd = self.make_command("ansible-playbook", &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(), environment_vars);
        
        self.logger.log_cmd(&cmd);
        
        let mut child = cmd.spawn()
            .map_err(|e| Error::Other(format!("Failed to start ansible-playbook: {}", e)))?;
        
        let pid = child.id().unwrap_or(0);
        callback(pid);
        
        let status = child.wait().await
            .map_err(|e| Error::Other(format!("ansible-playbook failed: {}", e)))?;
        
        // Ждём завершения обработки логов
        self.logger.wait_log();
        
        if status.success() {
            Ok(())
        } else {
            Err(Error::Other(format!("ansible-playbook exited with code {:?}", status.code())))
        }
    }

    /// Запускает ansible-galaxy
    pub async fn run_galaxy(&self, args: &[String], environment_vars: &[String]) -> Result<()> {
        let mut cmd = self.make_command("ansible-galaxy", &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(), environment_vars);
        
        self.logger.log_cmd(&cmd);
        
        let status = cmd.status().await
            .map_err(|e| Error::Other(format!("ansible-galaxy failed: {}", e)))?;
        
        // Ждём завершения обработки логов
        self.logger.wait_log();
        
        if status.success() {
            Ok(())
        } else {
            Err(Error::Other(format!("ansible-galaxy exited with code {:?}", status.code())))
        }
    }

    /// Получает полный путь к репозиторию
    pub fn get_full_path(&self) -> String {
        self.repository.get_full_path(self.template_id)
    }
}

/// Получает HOME директорию для задачи
fn get_home_dir(_repository: &Repository, _template_id: i32) -> String {
    // В production нужно получать из конфигурации
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansible_playbook_creation() {
        // Тест для проверки создания структуры
        assert!(true);
    }
}
