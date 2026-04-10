//! Общие типы для db_lib

use std::collections::HashMap;

/// Аргументы для установки зависимостей приложения
#[derive(Debug, Clone)]
pub struct LocalAppInstallingArgs {
    /// ID проекта
    pub project_id: i32,

    /// ID шаблона
    pub template_id: i32,

    /// ID задачи
    pub task_id: i32,

    /// Путь к репозиторию
    pub repo_path: String,

    /// Переменные окружения
    pub environment: HashMap<String, String>,

    /// Дополнительные аргументы
    pub extra_args: Vec<String>,
}

impl LocalAppInstallingArgs {
    /// Создаёт новые аргументы установки
    pub fn new(project_id: i32, template_id: i32, task_id: i32, repo_path: String) -> Self {
        Self {
            project_id,
            template_id,
            task_id,
            repo_path,
            environment: HashMap::new(),
            extra_args: Vec::new(),
        }
    }

    /// Добавляет переменную окружения
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }

    /// Добавляет дополнительный аргумент
    pub fn with_extra_arg(mut self, arg: String) -> Self {
        self.extra_args.push(arg);
        self
    }
}

/// Аргументы для запуска приложения
#[derive(Debug, Clone)]
pub struct LocalAppRunningArgs {
    /// ID проекта
    pub project_id: i32,

    /// ID шаблона
    pub template_id: i32,

    /// ID задачи
    pub task_id: i32,

    /// Команда для запуска
    pub command: String,

    /// Аргументы команды
    pub args: Vec<String>,

    /// Переменные окружения
    pub environment: HashMap<String, String>,

    /// Рабочая директория
    pub working_dir: String,

    /// Таймаут в секундах
    pub timeout_secs: Option<u64>,
}

impl LocalAppRunningArgs {
    /// Создаёт новые аргументы запуска
    pub fn new(
        project_id: i32,
        template_id: i32,
        task_id: i32,
        command: String,
        working_dir: String,
    ) -> Self {
        Self {
            project_id,
            template_id,
            task_id,
            command,
            args: Vec::new(),
            environment: HashMap::new(),
            working_dir,
            timeout_secs: None,
        }
    }

    /// Добавляет аргумент команды
    pub fn with_arg(mut self, arg: String) -> Self {
        self.args.push(arg);
        self
    }

    /// Добавляет переменную окружения
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }

    /// Устанавливает таймаут
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- LocalAppInstallingArgs tests ---

    #[test]
    fn test_installing_args_new() {
        let args = LocalAppInstallingArgs::new(1, 2, 3, "/repo".to_string());
        assert_eq!(args.project_id, 1);
        assert_eq!(args.template_id, 2);
        assert_eq!(args.task_id, 3);
        assert_eq!(args.repo_path, "/repo");
        assert!(args.environment.is_empty());
        assert!(args.extra_args.is_empty());
    }

    #[test]
    fn test_installing_args_with_env() {
        let args = LocalAppInstallingArgs::new(1, 2, 3, "/repo".to_string())
            .with_env("KEY".to_string(), "VALUE".to_string())
            .with_env("FOO".to_string(), "BAR".to_string());
        assert_eq!(args.environment.len(), 2);
        assert_eq!(args.environment.get("KEY"), Some(&"VALUE".to_string()));
        assert_eq!(args.environment.get("FOO"), Some(&"BAR".to_string()));
    }

    #[test]
    fn test_installing_args_with_extra_args() {
        let args = LocalAppInstallingArgs::new(1, 2, 3, "/repo".to_string())
            .with_extra_arg("--verbose".to_string())
            .with_extra_arg("--dry-run".to_string());
        assert_eq!(args.extra_args.len(), 2);
        assert_eq!(args.extra_args[0], "--verbose");
        assert_eq!(args.extra_args[1], "--dry-run");
    }

    #[test]
    fn test_installing_args_chained() {
        let args = LocalAppInstallingArgs::new(10, 20, 30, "/path".to_string())
            .with_env("A".to_string(), "1".to_string())
            .with_extra_arg("x".to_string());
        assert_eq!(args.project_id, 10);
        assert_eq!(args.environment.len(), 1);
        assert_eq!(args.extra_args.len(), 1);
    }

    #[test]
    fn test_installing_args_debug_impl() {
        let args = LocalAppInstallingArgs::new(1, 2, 3, "/repo".to_string());
        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("LocalAppInstallingArgs"));
        assert!(debug_str.contains("project_id"));
    }

    #[test]
    fn test_installing_args_clone() {
        let args = LocalAppInstallingArgs::new(1, 2, 3, "/repo".to_string())
            .with_env("K".to_string(), "V".to_string());
        let cloned = args.clone();
        assert_eq!(cloned.project_id, args.project_id);
        assert_eq!(cloned.environment, args.environment);
    }

    // --- LocalAppRunningArgs tests ---

    #[test]
    fn test_running_args_new() {
        let args = LocalAppRunningArgs::new(1, 2, 3, "python main.py".to_string(), "/work".to_string());
        assert_eq!(args.project_id, 1);
        assert_eq!(args.template_id, 2);
        assert_eq!(args.task_id, 3);
        assert_eq!(args.command, "python main.py");
        assert_eq!(args.working_dir, "/work");
        assert!(args.args.is_empty());
        assert!(args.environment.is_empty());
        assert!(args.timeout_secs.is_none());
    }

    #[test]
    fn test_running_args_with_arg() {
        let args = LocalAppRunningArgs::new(1, 2, 3, "cmd".to_string(), "/dir".to_string())
            .with_arg("--help".to_string())
            .with_arg("-v".to_string());
        assert_eq!(args.args.len(), 2);
        assert_eq!(args.args[0], "--help");
    }

    #[test]
    fn test_running_args_with_env() {
        let args = LocalAppRunningArgs::new(1, 2, 3, "cmd".to_string(), "/dir".to_string())
            .with_env("DEBUG".to_string(), "1".to_string());
        assert_eq!(args.environment.len(), 1);
        assert_eq!(args.environment.get("DEBUG"), Some(&"1".to_string()));
    }

    #[test]
    fn test_running_args_with_timeout() {
        let args = LocalAppRunningArgs::new(1, 2, 3, "cmd".to_string(), "/dir".to_string())
            .with_timeout(300);
        assert_eq!(args.timeout_secs, Some(300));
    }

    #[test]
    fn test_running_args_chained() {
        let args = LocalAppRunningArgs::new(5, 6, 7, "run".to_string(), "/tmp".to_string())
            .with_arg("--config=prod.yml".to_string())
            .with_env("NODE_ENV".to_string(), "production".to_string())
            .with_timeout(60);
        assert_eq!(args.args.len(), 1);
        assert_eq!(args.environment.len(), 1);
        assert_eq!(args.timeout_secs, Some(60));
    }

    #[test]
    fn test_running_args_debug_impl() {
        let args = LocalAppRunningArgs::new(1, 2, 3, "cmd".to_string(), "/dir".to_string());
        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("LocalAppRunningArgs"));
        assert!(debug_str.contains("command"));
    }

    #[test]
    fn test_running_args_clone() {
        let args = LocalAppRunningArgs::new(1, 2, 3, "cmd".to_string(), "/dir".to_string())
            .with_timeout(100);
        let cloned = args.clone();
        assert_eq!(cloned.timeout_secs, args.timeout_secs);
        assert_eq!(cloned.working_dir, args.working_dir);
    }
}
