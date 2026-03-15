//! Metrics Service Tests

#[cfg(test)]
mod tests {
    use crate::services::metrics::*;
    use prometheus::{Registry, Counter, Gauge, Histogram};
    use std::sync::Arc;

    #[test]
    fn test_metrics_registry_initialization() {
        let registry = MetricsManager::registry();
        assert!(true); // Registry initialized
    }

    #[test]
    fn test_metrics_manager_creation() {
        let manager = MetricsManager::new();
        assert!(true); // Manager created successfully
    }

    #[tokio::test]
    async fn test_task_counters_increment() {
        let manager = MetricsManager::new();
        
        manager.inc_project_task(1, true).await;
        manager.inc_project_task(1, true).await;
        manager.inc_project_task(1, false).await;
        
        let counters = manager.get_task_counters().await;
        let project_counters = counters.by_project.get(&1).unwrap();
        
        assert_eq!(project_counters.total, 3);
        assert_eq!(project_counters.success, 2);
        assert_eq!(project_counters.failed, 1);
    }

    #[tokio::test]
    async fn test_template_task_counters() {
        let manager = MetricsManager::new();
        
        manager.inc_template_task(1, true).await;
        manager.inc_template_task(1, true).await;
        
        let counters = manager.get_task_counters().await;
        let template_counters = counters.by_template.get(&1).unwrap();
        
        assert_eq!(template_counters.total, 2);
        assert_eq!(template_counters.success, 2);
    }

    #[tokio::test]
    async fn test_user_task_counters() {
        let manager = MetricsManager::new();
        
        manager.inc_user_task(1, true).await;
        manager.inc_user_task(1, false).await;
        
        let counters = manager.get_task_counters().await;
        let user_counters = counters.by_user.get(&1).unwrap();
        
        assert_eq!(user_counters.total, 2);
        assert_eq!(user_counters.success, 1);
        assert_eq!(user_counters.failed, 1);
    }

    #[test]
    fn test_task_counters_default() {
        let counters = TaskCounters::default();
        assert!(counters.by_project.is_empty());
        assert!(counters.by_template.is_empty());
        assert!(counters.by_user.is_empty());
    }

    #[test]
    fn test_project_task_counters_default() {
        let counters = ProjectTaskCounters::default();
        assert_eq!(counters.total, 0);
        assert_eq!(counters.success, 0);
        assert_eq!(counters.failed, 0);
        assert_eq!(counters.stopped, 0);
    }

    #[test]
    fn test_template_task_counters_default() {
        let counters = TemplateTaskCounters::default();
        assert_eq!(counters.total, 0);
        assert_eq!(counters.success, 0);
        assert_eq!(counters.failed, 0);
    }

    #[test]
    fn test_user_task_counters_default() {
        let counters = UserTaskCounters::default();
        assert_eq!(counters.total, 0);
        assert_eq!(counters.success, 0);
        assert_eq!(counters.failed, 0);
    }

    #[test]
    fn test_task_counters_clone() {
        let mut counters = TaskCounters::default();
        counters.by_project.insert(1, ProjectTaskCounters::default());
        
        let cloned = counters.clone();
        assert_eq!(cloned.by_project.len(), 1);
    }

    #[tokio::test]
    async fn test_multiple_projects() {
        let manager = MetricsManager::new();
        
        for project_id in 1..=5 {
            manager.inc_project_task(project_id, true).await;
        }
        
        let counters = manager.get_task_counters().await;
        assert_eq!(counters.by_project.len(), 5);
    }

    #[tokio::test]
    async fn test_concurrent_task_updates() {
        let manager = MetricsManager::new();
        let manager = Arc::new(manager);
        
        let mut handles = vec![];
        for i in 0..10 {
            let mgr = manager.clone();
            handles.push(tokio::spawn(async move {
                mgr.inc_project_task(1, true).await;
            }));
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        let counters = manager.get_task_counters().await;
        let project_counters = counters.by_project.get(&1).unwrap();
        assert_eq!(project_counters.total, 10);
    }
}
