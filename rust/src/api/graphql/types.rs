//! GraphQL типы — полный набор для публичного API

use async_graphql::SimpleObject;

// ── Core domain ──────────────────────────────────────────────────────────────

/// Пользователь
#[derive(SimpleObject, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub name: String,
    pub email: String,
    pub admin: bool,
}

/// Проект
#[derive(SimpleObject, Debug, Clone)]
pub struct Project {
    pub id: i32,
    pub name: String,
}

/// Шаблон (Template)
#[derive(SimpleObject, Debug, Clone)]
pub struct Template {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub playbook: String,
}

/// Задача (Task)
#[derive(SimpleObject, Debug, Clone)]
pub struct Task {
    pub id: i32,
    pub template_id: i32,
    pub project_id: i32,
    pub status: String,
    pub created: String,
}

/// Инвентарь
#[derive(SimpleObject, Debug, Clone)]
pub struct Inventory {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub r#type: String,
}

/// Репозиторий
#[derive(SimpleObject, Debug, Clone)]
pub struct Repository {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub git_url: String,
    pub git_branch: String,
}

/// Переменные окружения
#[derive(SimpleObject, Debug, Clone)]
pub struct Environment {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
}

/// Расписание (Schedule)
#[derive(SimpleObject, Debug, Clone)]
pub struct Schedule {
    pub id: i32,
    pub project_id: i32,
    pub template_id: i32,
    pub name: String,
    pub cron_format: String,
    pub active: bool,
}

/// Раннер (Runner)
#[derive(SimpleObject, Debug, Clone)]
pub struct Runner {
    pub id: i32,
    pub name: String,
    pub active: bool,
    pub webhook: String,
}

/// Событие аудит-лога
#[derive(SimpleObject, Debug, Clone)]
pub struct AuditEvent {
    pub id: i32,
    pub project_id: Option<i32>,
    pub object_type: String,
    pub object_id: i32,
    pub description: String,
    pub created: String,
}

// ── Kubernetes ────────────────────────────────────────────────────────────────

/// Kubernetes namespace
#[derive(SimpleObject, Debug, Clone)]
pub struct KubernetesNamespace {
    pub name: String,
    pub status: String,
    pub labels: Vec<String>,
}

/// Kubernetes node
#[derive(SimpleObject, Debug, Clone)]
pub struct KubernetesNode {
    pub name: String,
    pub status: String,
    pub roles: Vec<String>,
    pub version: String,
    pub os_image: String,
}

/// Kubernetes cluster info
#[derive(SimpleObject, Debug, Clone)]
pub struct KubernetesClusterInfo {
    pub server_url: String,
    pub version: String,
    pub namespace: String,
}

// ── Subscription event types ──────────────────────────────────────────────────

/// Строка лога задачи — для subscription taskOutput
#[derive(SimpleObject, Debug, Clone)]
pub struct TaskOutputLine {
    pub task_id: i32,
    pub line: String,
    pub timestamp: String,
    /// Уровень: "info" | "warning" | "error" | "debug"
    pub level: String,
}

/// Изменение статуса задачи — для subscription taskStatus
#[derive(SimpleObject, Debug, Clone)]
pub struct TaskStatusEvent {
    pub task_id: i32,
    pub project_id: i32,
    pub status: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_fields() {
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            admin: false,
        };
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.name, "Test User");
        assert!(!user.admin);
    }

    #[test]
    fn test_user_clone_and_debug() {
        let user = User {
            id: 1,
            username: "u".to_string(),
            name: "n".to_string(),
            email: "e@t.com".to_string(),
            admin: true,
        };
        let cloned = user.clone();
        assert_eq!(format!("{:?}", user), format!("{:?}", cloned));
    }

    #[test]
    fn test_project_fields() {
        let project = Project {
            id: 100,
            name: "Test Project".to_string(),
        };
        assert_eq!(project.id, 100);
        assert_eq!(project.name, "Test Project");
    }

    #[test]
    fn test_template_fields() {
        let template = Template {
            id: 5,
            project_id: 10,
            name: "deploy-template".to_string(),
            playbook: "deploy.yml".to_string(),
        };
        assert_eq!(template.id, 5);
        assert_eq!(template.project_id, 10);
        assert_eq!(template.playbook, "deploy.yml");
    }

    #[test]
    fn test_task_fields() {
        let task = Task {
            id: 42,
            template_id: 1,
            project_id: 10,
            status: "success".to_string(),
            created: "2024-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(task.id, 42);
        assert_eq!(task.status, "success");
        assert_eq!(task.template_id, 1);
    }

    #[test]
    fn test_inventory_fields() {
        let inventory = Inventory {
            id: 1,
            project_id: 10,
            name: "hosts".to_string(),
            r#type: "file".to_string(),
        };
        assert_eq!(inventory.name, "hosts");
        assert_eq!(inventory.r#type, "file");
    }

    #[test]
    fn test_repository_fields() {
        let repo = Repository {
            id: 3,
            project_id: 10,
            name: "infra".to_string(),
            git_url: "git@github.com:org/infra.git".to_string(),
            git_branch: "main".to_string(),
        };
        assert_eq!(repo.git_url, "git@github.com:org/infra.git");
        assert_eq!(repo.git_branch, "main");
    }

    #[test]
    fn test_environment_fields() {
        let env = Environment {
            id: 2,
            project_id: 10,
            name: "staging".to_string(),
        };
        assert_eq!(env.name, "staging");
    }

    #[test]
    fn test_schedule_fields() {
        let schedule = Schedule {
            id: 1,
            project_id: 10,
            template_id: 5,
            name: "daily-backup".to_string(),
            cron_format: "0 2 * * *".to_string(),
            active: true,
        };
        assert_eq!(schedule.active, true);
        assert_eq!(schedule.cron_format, "0 2 * * *");
    }

    #[test]
    fn test_runner_fields() {
        let runner = Runner {
            id: 1,
            name: "runner-alpha".to_string(),
            active: true,
            webhook: "https://hooks.example.com/runner".to_string(),
        };
        assert_eq!(runner.name, "runner-alpha");
        assert!(runner.active);
    }

    #[test]
    fn test_audit_event_fields() {
        let event = AuditEvent {
            id: 1,
            project_id: Some(10),
            object_type: "task".to_string(),
            object_id: 50,
            description: "Task completed".to_string(),
            created: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(event.object_type, "task");
        assert_eq!(event.object_id, 50);
    }

    #[test]
    fn test_audit_event_null_project_id() {
        let event = AuditEvent {
            id: 1,
            project_id: None,
            object_type: "user".to_string(),
            object_id: 1,
            description: "User created".to_string(),
            created: "2024-01-01T00:00:00Z".to_string(),
        };
        assert!(event.project_id.is_none());
    }

    #[test]
    fn test_kubernetes_namespace_fields() {
        let ns = KubernetesNamespace {
            name: "velum-system".to_string(),
            status: "Active".to_string(),
            labels: vec!["app=velum".to_string(), "env=prod".to_string()],
        };
        assert_eq!(ns.name, "velum-system");
        assert_eq!(ns.labels.len(), 2);
    }

    #[test]
    fn test_kubernetes_namespace_empty_labels() {
        let ns = KubernetesNamespace {
            name: "default".to_string(),
            status: "Active".to_string(),
            labels: vec![],
        };
        assert!(ns.labels.is_empty());
    }

    #[test]
    fn test_kubernetes_node_fields() {
        let node = KubernetesNode {
            name: "worker-1".to_string(),
            status: "Ready".to_string(),
            roles: vec!["worker".to_string()],
            version: "v1.30.0".to_string(),
            os_image: "Ubuntu 22.04".to_string(),
        };
        assert_eq!(node.version, "v1.30.0");
        assert_eq!(node.os_image, "Ubuntu 22.04");
        assert_eq!(node.roles.len(), 1);
    }

    #[test]
    fn test_kubernetes_cluster_info_fields() {
        let info = KubernetesClusterInfo {
            server_url: "https://k8s.cluster.local:6443".to_string(),
            version: "v1.30.0".to_string(),
            namespace: "production".to_string(),
        };
        assert_eq!(info.namespace, "production");
        assert_eq!(info.server_url, "https://k8s.cluster.local:6443");
    }

    #[test]
    fn test_task_output_line_fields() {
        let line = TaskOutputLine {
            task_id: 100,
            line: "ok: [host1]".to_string(),
            timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            level: "info".to_string(),
        };
        assert_eq!(line.task_id, 100);
        assert_eq!(line.level, "info");
    }

    #[test]
    fn test_task_status_event_fields() {
        let event = TaskStatusEvent {
            task_id: 50,
            project_id: 5,
            status: "waiting".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(event.project_id, 5);
        assert_eq!(event.status, "waiting");
    }

    #[test]
    fn test_all_types_implement_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<User>();
        assert_clone::<Project>();
        assert_clone::<Template>();
        assert_clone::<Task>();
        assert_clone::<Inventory>();
        assert_clone::<Repository>();
        assert_clone::<Environment>();
        assert_clone::<Schedule>();
        assert_clone::<Runner>();
        assert_clone::<AuditEvent>();
        assert_clone::<KubernetesNamespace>();
        assert_clone::<KubernetesNode>();
        assert_clone::<KubernetesClusterInfo>();
        assert_clone::<TaskOutputLine>();
        assert_clone::<TaskStatusEvent>();
    }

    #[test]
    fn test_all_types_implement_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<User>();
        assert_debug::<Project>();
        assert_debug::<Template>();
        assert_debug::<Task>();
        assert_debug::<Inventory>();
        assert_debug::<Repository>();
        assert_debug::<Environment>();
        assert_debug::<Schedule>();
        assert_debug::<Runner>();
        assert_debug::<AuditEvent>();
        assert_debug::<KubernetesNamespace>();
        assert_debug::<KubernetesNode>();
        assert_debug::<KubernetesClusterInfo>();
        assert_debug::<TaskOutputLine>();
        assert_debug::<TaskStatusEvent>();
    }

    #[test]
    fn test_user_equality() {
        let u1 = User {
            id: 1,
            username: "a".to_string(),
            name: "a".to_string(),
            email: "a@b.com".to_string(),
            admin: true,
        };
        let u2 = u1.clone();
        assert_eq!(u1.id, u2.id);
        assert_eq!(u1.username, u2.username);
    }
}
