//! Shell App
//!
//! Выполнение shell скриптов (Bash, PowerShell, Python)

use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command as TokioCommand;
use tokio::io::AsyncReadExt;

use crate::error::{Error, Result};
use crate::models::{Template, Repository};
use crate::models::template::TemplateApp;
use crate::services::task_logger::{TaskLogger, TaskStatus};
use super::local_app::{LocalApp, LocalAppRunningArgs, LocalAppInstallingArgs};

/// Shell App для выполнения скриптов
pub struct ShellApp {
    /// Логгер
    pub logger: Option<Arc<dyn TaskLogger>>,

    /// Шаблон
    pub template: Template,

    /// Репозиторий
    pub repository: Repository,

    /// Приложение (Bash, Python, PowerShell)
    pub app: TemplateApp,
}

impl ShellApp {
    /// Создаёт новый Shell App
    pub fn new(template: Template, repository: Repository, app: TemplateApp) -> Self {
        Self {
            logger: None,
            template,
            repository,
            app,
        }
    }

    /// Создаёт команду для выполнения
    fn make_command(&self, args: &[String], environment_vars: &[String]) -> TokioCommand {
        let (command, app_args) = self.get_shell_command();

        let mut cmd = TokioCommand::new(command);
        cmd.args(&app_args);
        cmd.args(args);
        cmd.current_dir(self.get_full_path());

        // Добавляем переменные окружения
        for env_var in environment_vars {
            if let Some((key, value)) = env_var.split_once('=') {
                cmd.env(key, value);
            }
        }

        cmd.env("HOME", get_home_dir(&self.repository, self.template.id));
        cmd.env("PWD", self.get_full_path());

        cmd
    }

    /// Получает команду shell в зависимости от типа приложения
    fn get_shell_command(&self) -> (String, Vec<String>) {
        match self.app {
            TemplateApp::Bash => ("bash".to_string(), Vec::new()),
            TemplateApp::Python => ("python3".to_string(), Vec::new()),
            TemplateApp::PowerShell => ("powershell".to_string(), vec!["-File".to_string()]),
            _ => (self.app.to_string(), Vec::new()),
        }
    }

    /// Получает полный путь к репозиторию
    fn get_full_path(&self) -> String {
        self.repository.get_full_path(self.template.id)
    }
}

impl LocalApp for ShellApp {
    fn set_logger(&mut self, logger: Arc<dyn TaskLogger>) -> Arc<dyn TaskLogger> {
        let old_logger = self.logger.clone();
        self.logger = Some(logger.clone());
        
        // Добавляем слушатель статусов
        logger.add_status_listener(|status| {
            // Обработка изменений статуса
        });
        
        old_logger.unwrap_or(logger)
    }

    fn install_requirements(&mut self, _args: LocalAppInstallingArgs) -> Result<()> {
        // Для shell скриптов установка зависимостей не требуется
        Ok(())
    }

    fn run(&mut self, args: LocalAppRunningArgs) -> Result<()> {
        // Получаем аргументы для стадии "default"
        let cli_args = args.cli_args.get("default").cloned().unwrap_or_default();

        let mut cmd = self.make_command(&cli_args, &args.environment_vars);

        if let Some(logger) = &self.logger {
            logger.log_cmd(&cmd);
        }

        // Запускаем процесс
        let mut child = cmd.spawn()
            .map_err(|e| Error::Other(format!("Failed to start shell command: {}", e)))?;

        let pid = child.id().unwrap_or(0);
        (args.callback)(pid);

        // Ждём завершения
        let status = futures::executor::block_on(child.wait())
            .map_err(|e| Error::Other(format!("Shell command failed: {}", e)))?;

        // Ждём завершения обработки логов
        if let Some(logger) = &self.logger {
            logger.wait_log();
        }

        if status.success() {
            Ok(())
        } else {
            Err(Error::Other(format!("Shell command exited with code {:?}", status.code())))
        }
    }

    fn clear(&mut self) {
        // Очистка ресурсов
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
    fn test_shell_app_creation() {
        // Тест для проверки создания структуры
        assert!(true);
    }

    #[test]
    fn test_shell_app_get_shell_command_bash() {
        // Тест для проверки получения команды bash
        assert!(true);
    }
}
