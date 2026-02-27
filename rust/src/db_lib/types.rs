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
    pub fn new(project_id: i32, template_id: i32, task_id: i32, command: String, working_dir: String) -> Self {
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
