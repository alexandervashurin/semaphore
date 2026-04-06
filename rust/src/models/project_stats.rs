//! Project Stats Model
//!
//! Статистика проекта

use serde::{Deserialize, Serialize};

/// Статистика проекта
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectStats {
    /// Количество задач
    pub task_count: i32,

    /// Количество успешных задач
    pub success_count: i32,

    /// Количество неудачных задач
    pub fail_count: i32,

    /// Количество остановленных задач
    pub stopped_count: i32,

    /// Количество активных пользователей
    pub active_user_count: i32,

    /// Количество шаблонов
    pub template_count: i32,

    /// Количество инвентарей
    pub inventory_count: i32,

    /// Количество репозиториев
    pub repository_count: i32,
}

impl ProjectStats {
    /// Создаёт новую статистику
    pub fn new() -> Self {
        Self::default()
    }

    /// Процент успешных задач
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.fail_count + self.stopped_count;
        if total == 0 {
            0.0
        } else {
            (self.success_count as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_stats_default() {
        let stats = ProjectStats::default();
        assert_eq!(stats.task_count, 0);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_project_stats_success_rate() {
        let stats = ProjectStats {
            task_count: 100,
            success_count: 80,
            fail_count: 15,
            stopped_count: 5,
            active_user_count: 10,
            template_count: 5,
            inventory_count: 3,
            repository_count: 2,
        };
        // 80 / (80 + 15 + 5) * 100 = 80%
        assert!((stats.success_rate() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_project_stats_zero_total() {
        let stats = ProjectStats::new();
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_project_stats_serialization() {
        let stats = ProjectStats {
            task_count: 50,
            success_count: 40,
            fail_count: 8,
            stopped_count: 2,
            active_user_count: 5,
            template_count: 3,
            inventory_count: 2,
            repository_count: 1,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"task_count\":50"));
        assert!(json.contains("\"success_count\":40"));
    }
}
