//! Модель ObjectReferrers - ссылки на объекты

use serde::{Deserialize, Serialize};

/// Ссылки на объект
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectReferrers {
    /// Ссылки из шаблонов
    #[serde(default)]
    pub templates: Vec<i32>,

    /// Ссылки из задач
    #[serde(default)]
    pub tasks: Vec<i32>,

    /// Ссылки из расписаний
    #[serde(default)]
    pub schedules: Vec<i32>,

    /// Ссылки из интеграций
    #[serde(default)]
    pub integrations: Vec<i32>,
}

impl ObjectReferrers {
    /// Создаёт новые пустые ссылки
    pub fn new() -> Self {
        Self::default()
    }

    /// Проверяет, есть ли ссылки
    pub fn is_empty(&self) -> bool {
        self.templates.is_empty() &&
        self.tasks.is_empty() &&
        self.schedules.is_empty() &&
        self.integrations.is_empty()
    }
}
