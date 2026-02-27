//! Local App Types
//!
//! Типы для локальных приложений

use std::sync::Arc;
use crate::services::task_logger::TaskLogger;
use crate::error::Result;

/// Аргументы для запуска локального приложения
#[derive(Debug, Clone)]
pub struct LocalAppRunningArgs {
    /// Аргументы командной строки по стадиям
    pub cli_args: std::collections::HashMap<String, Vec<String>>,

    /// Переменные окружения
    pub environment_vars: Vec<String>,

    /// Входные данные для интерактивных команд
    pub inputs: std::collections::HashMap<String, String>,

    /// Параметры задачи
    pub task_params: Box<dyn std::any::Any + Send + Sync>,

    /// Параметры шаблона
    pub template_params: Box<dyn std::any::Any + Send + Sync>,

    /// Callback для получения PID процесса
    pub callback: Box<dyn FnOnce(u32) + Send + 'static>,
}

impl Default for LocalAppRunningArgs {
    fn default() -> Self {
        Self {
            cli_args: std::collections::HashMap::new(),
            environment_vars: Vec::new(),
            inputs: std::collections::HashMap::new(),
            task_params: Box::new(()),
            template_params: Box::new(()),
            callback: Box::new(|_| {}),
        }
    }
}

/// Аргументы для установки зависимостей локального приложения
#[derive(Debug, Clone)]
pub struct LocalAppInstallingArgs {
    /// Переменные окружения
    pub environment_vars: Vec<String>,

    /// Параметры шаблона
    pub template_params: Box<dyn std::any::Any + Send + Sync>,

    /// Параметры
    pub params: Box<dyn std::any::Any + Send + Sync>,

    /// Установщик ключей доступа
    pub installer: Option<Arc<dyn AccessKeyInstaller>>,
}

impl Default for LocalAppInstallingArgs {
    fn default() -> Self {
        Self {
            environment_vars: Vec::new(),
            template_params: Box::new(()),
            params: Box::new(()),
            installer: None,
        }
    }
}

/// Трейт для локального приложения
pub trait LocalApp: Send + Sync {
    /// Устанавливает логгер
    fn set_logger(&mut self, logger: Arc<dyn TaskLogger>) -> Arc<dyn TaskLogger>;

    /// Устанавливает зависимости
    fn install_requirements(&mut self, args: LocalAppInstallingArgs) -> Result<()>;

    /// Запускает приложение
    fn run(&mut self, args: LocalAppRunningArgs) -> Result<()>;

    /// Очищает ресурсы
    fn clear(&mut self);
}

/// Трейт для установщика ключей доступа
pub trait AccessKeyInstaller: Send + Sync {
    /// Устанавливает ключ доступа
    fn install(&self) -> Result<()>;

    /// Удаляет ключ доступа
    fn cleanup(&self) -> Result<()>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_app_running_args_default() {
        let args = LocalAppRunningArgs::default();
        assert!(args.cli_args.is_empty());
        assert!(args.environment_vars.is_empty());
        assert!(args.inputs.is_empty());
    }

    #[test]
    fn test_local_app_installing_args_default() {
        let args = LocalAppInstallingArgs::default();
        assert!(args.environment_vars.is_empty());
        assert!(args.installer.is_none());
    }
}
