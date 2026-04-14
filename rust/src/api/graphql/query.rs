//! GraphQL Query корень — полный набор запросов

use crate::api::state::AppState;
use crate::db::store::{
    EnvironmentManager, EventManager, InventoryManager, ProjectStore, RepositoryManager,
    RunnerManager, ScheduleManager, TaskManager, TemplateManager, UserManager,
};
use async_graphql::{Context, Object, Result};

use super::types::{
    AuditEvent, Environment, Inventory, KubernetesClusterInfo, KubernetesNamespace, KubernetesNode,
    Project, Repository, Runner, Schedule, Task, Template, User,
};

/// Корневой тип для Query
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    // ── Core queries ──────────────────────────────────────────────────────────

    /// Ping для проверки работоспособности
    async fn ping(&self) -> Result<String> {
        Ok("pong".to_string())
    }

    /// Получить всех пользователей
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let state = ctx.data::<AppState>()?;
        let users = state.store.get_users(Default::default()).await?;
        Ok(users
            .into_iter()
            .map(|u| User {
                id: u.id,
                username: u.username,
                name: u.name,
                email: u.email,
                admin: u.admin,
            })
            .collect())
    }

    /// Получить пользователя по ID
    async fn user(&self, ctx: &Context<'_>, id: i32) -> Result<User> {
        let state = ctx.data::<AppState>()?;
        let u = state.store.get_user(id).await?;
        Ok(User {
            id: u.id,
            username: u.username,
            name: u.name,
            email: u.email,
            admin: u.admin,
        })
    }

    /// Получить все проекты
    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let state = ctx.data::<AppState>()?;
        let projects = state.store.get_projects(None).await?;
        Ok(projects
            .into_iter()
            .map(|p| Project {
                id: p.id,
                name: p.name,
            })
            .collect())
    }

    /// Получить проект по ID
    async fn project(&self, ctx: &Context<'_>, id: i32) -> Result<Project> {
        let state = ctx.data::<AppState>()?;
        let p = state.store.get_project(id).await?;
        Ok(Project {
            id: p.id,
            name: p.name,
        })
    }

    /// Получить шаблоны проекта
    async fn templates(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Template>> {
        let state = ctx.data::<AppState>()?;
        let templates = state.store.get_templates(project_id).await?;
        Ok(templates
            .into_iter()
            .map(|t| Template {
                id: t.id,
                project_id: t.project_id,
                name: t.name,
                playbook: t.playbook,
            })
            .collect())
    }

    /// Получить задачи проекта
    async fn tasks(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Task>> {
        let state = ctx.data::<AppState>()?;
        let tasks = state.store.get_tasks(project_id, None).await?;
        Ok(tasks
            .into_iter()
            .map(|t| Task {
                id: t.task.id,
                template_id: t.task.template_id,
                project_id: t.task.project_id,
                status: t.task.status.to_string(),
                created: t.task.created.to_rfc3339(),
            })
            .collect())
    }

    /// Получить одну задачу по ID
    async fn task(&self, ctx: &Context<'_>, project_id: i32, task_id: i32) -> Result<Task> {
        let state = ctx.data::<AppState>()?;
        let t = state.store.get_task(project_id, task_id).await?;
        Ok(Task {
            id: t.id,
            template_id: t.template_id,
            project_id: t.project_id,
            status: t.status.to_string(),
            created: t.created.to_rfc3339(),
        })
    }

    // ── Extended queries ──────────────────────────────────────────────────────

    /// Получить инвентари проекта
    async fn inventories(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Inventory>> {
        let state = ctx.data::<AppState>()?;
        let items = state.store.get_inventories(project_id).await?;
        Ok(items
            .into_iter()
            .map(|inv| Inventory {
                id: inv.id,
                project_id: inv.project_id,
                name: inv.name,
                r#type: inv.inventory_type.to_string(),
            })
            .collect())
    }

    /// Получить репозитории проекта
    async fn repositories(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Repository>> {
        let state = ctx.data::<AppState>()?;
        let items = state.store.get_repositories(project_id).await?;
        Ok(items
            .into_iter()
            .map(|r| Repository {
                id: r.id,
                project_id: r.project_id,
                name: r.name,
                git_url: r.git_url,
                git_branch: r.git_branch.unwrap_or_default(),
            })
            .collect())
    }

    /// Получить окружения проекта
    async fn environments(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Environment>> {
        let state = ctx.data::<AppState>()?;
        let items = state.store.get_environments(project_id).await?;
        Ok(items
            .into_iter()
            .map(|e| Environment {
                id: e.id,
                project_id: e.project_id,
                name: e.name,
            })
            .collect())
    }

    /// Получить расписания проекта
    async fn schedules(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Schedule>> {
        let state = ctx.data::<AppState>()?;
        let items = state.store.get_schedules(project_id).await?;
        Ok(items
            .into_iter()
            .map(|s| Schedule {
                id: s.id,
                project_id: s.project_id,
                template_id: s.template_id,
                name: s.name,
                cron_format: s.cron_format.unwrap_or_default(),
                active: s.active,
            })
            .collect())
    }

    /// Получить раннеры (опционально — для конкретного проекта)
    async fn runners(&self, ctx: &Context<'_>, project_id: Option<i32>) -> Result<Vec<Runner>> {
        let state = ctx.data::<AppState>()?;
        let items = state.store.get_runners(project_id).await?;
        Ok(items
            .into_iter()
            .map(|r| Runner {
                id: r.id,
                name: r.name,
                active: r.active,
                webhook: r.webhook.unwrap_or_default(),
            })
            .collect())
    }

    /// Получить события аудит-лога (опционально — для конкретного проекта)
    ///
    /// `limit` по умолчанию 100, максимум 1000.
    async fn audit_events(
        &self,
        ctx: &Context<'_>,
        project_id: Option<i32>,
        limit: Option<usize>,
    ) -> Result<Vec<AuditEvent>> {
        let state = ctx.data::<AppState>()?;
        let limit = limit.unwrap_or(100).min(1000);
        let items = state.store.get_events(project_id, limit).await?;
        Ok(items
            .into_iter()
            .map(|e| AuditEvent {
                id: e.id,
                project_id: e.project_id,
                object_type: e.object_type,
                object_id: e.object_id.unwrap_or(0),
                description: e.description,
                created: e.created.to_rfc3339(),
            })
            .collect())
    }

    // ── Kubernetes queries ────────────────────────────────────────────────────

    /// Список Kubernetes namespaces.
    ///
    /// Возвращает пустой список если Kubernetes не сконфигурирован.
    async fn kubernetes_namespaces(&self, ctx: &Context<'_>) -> Result<Vec<KubernetesNamespace>> {
        let state = ctx.data::<AppState>()?;

        let client = match state.kubernetes_client() {
            Ok(c) => c,
            Err(_) => return Ok(vec![]),
        };

        use crate::api::handlers::kubernetes::client::KubernetesClusterService;
        let svc = KubernetesClusterService::new(client);
        let raw = svc.list_namespaces().await.unwrap_or_default();

        Ok(raw
            .into_iter()
            .map(|v| {
                let name = v["name"].as_str().unwrap_or("unknown").to_string();
                let status = v["status"].as_str().unwrap_or("Unknown").to_string();
                let labels = v["labels"]
                    .as_object()
                    .map(|m| {
                        m.iter()
                            .map(|(k, val)| format!("{}={}", k, val.as_str().unwrap_or("")))
                            .collect()
                    })
                    .unwrap_or_default();
                KubernetesNamespace {
                    name,
                    status,
                    labels,
                }
            })
            .collect())
    }

    /// Список Kubernetes nodes.
    ///
    /// Возвращает пустой список если Kubernetes не сконфигурирован.
    async fn kubernetes_nodes(&self, ctx: &Context<'_>) -> Result<Vec<KubernetesNode>> {
        let state = ctx.data::<AppState>()?;

        let client = match state.kubernetes_client() {
            Ok(c) => c,
            Err(_) => return Ok(vec![]),
        };

        use crate::api::handlers::kubernetes::client::KubernetesClusterService;
        let svc = KubernetesClusterService::new(client);
        let raw = svc.list_nodes().await.unwrap_or_default();

        Ok(raw
            .into_iter()
            .map(|v| {
                let name = v["name"].as_str().unwrap_or("unknown").to_string();

                let status = v["status"].as_str().unwrap_or("Unknown").to_string();

                let roles: Vec<String> = v["roles"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|r| r.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();

                let version = v["version"].as_str().unwrap_or("").to_string();
                let os_image = v["os_image"].as_str().unwrap_or("").to_string();

                KubernetesNode {
                    name,
                    status,
                    roles,
                    version,
                    os_image,
                }
            })
            .collect())
    }

    /// Информация о Kubernetes кластере.
    ///
    /// Возвращает ошибку если Kubernetes не сконфигурирован.
    async fn kubernetes_cluster_info(&self, ctx: &Context<'_>) -> Result<KubernetesClusterInfo> {
        let state = ctx.data::<AppState>()?;

        let client = state
            .kubernetes_client()
            .map_err(|e| async_graphql::Error::new(format!("Kubernetes not configured: {e}")))?;

        let config = client.config();

        Ok(KubernetesClusterInfo {
            server_url: config.kubeconfig_path.clone().unwrap_or_default(),
            version: "unknown".to_string(),
            namespace: client.default_namespace().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::graphql::types::{
        AuditEvent, Environment, Inventory, KubernetesClusterInfo, KubernetesNamespace,
        KubernetesNode, Project, Repository, Runner, Schedule, Task, Template, User,
    };

    #[test]
    fn test_query_root_exists() {
        let root = QueryRoot;
        drop(root);
    }

    #[test]
    fn test_user_type_fields() {
        let user = User {
            id: 1,
            username: "admin".to_string(),
            name: "Admin".to_string(),
            email: "admin@test.com".to_string(),
            admin: true,
        };
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "admin");
        assert!(user.admin);
    }

    #[test]
    fn test_project_type_fields() {
        let project = Project {
            id: 42,
            name: "My Project".to_string(),
        };
        assert_eq!(project.id, 42);
        assert_eq!(project.name, "My Project");
    }

    #[test]
    fn test_template_type_fields() {
        let template = Template {
            id: 1,
            project_id: 10,
            name: "deploy".to_string(),
            playbook: "deploy.yml".to_string(),
        };
        assert_eq!(template.project_id, 10);
        assert_eq!(template.playbook, "deploy.yml");
    }

    #[test]
    fn test_task_type_fields() {
        let task = Task {
            id: 100,
            template_id: 5,
            project_id: 10,
            status: "running".to_string(),
            created: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(task.id, 100);
        assert_eq!(task.status, "running");
    }

    #[test]
    fn test_inventory_type_fields() {
        let inventory = Inventory {
            id: 1,
            project_id: 10,
            name: "hosts".to_string(),
            r#type: "static".to_string(),
        };
        assert_eq!(inventory.r#type, "static");
        assert_eq!(inventory.name, "hosts");
    }

    #[test]
    fn test_repository_type_fields() {
        let repo = Repository {
            id: 1,
            project_id: 10,
            name: "my-repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_branch: "main".to_string(),
        };
        assert_eq!(repo.git_url, "https://github.com/test/repo.git");
        assert_eq!(repo.git_branch, "main");
    }

    #[test]
    fn test_environment_type_fields() {
        let env = Environment {
            id: 1,
            project_id: 10,
            name: "production".to_string(),
        };
        assert_eq!(env.name, "production");
    }

    #[test]
    fn test_schedule_type_fields() {
        let schedule = Schedule {
            id: 1,
            project_id: 10,
            template_id: 5,
            name: "nightly".to_string(),
            cron_format: "0 0 * * *".to_string(),
            active: true,
        };
        assert_eq!(schedule.cron_format, "0 0 * * *");
        assert!(schedule.active);
    }

    #[test]
    fn test_runner_type_fields() {
        let runner = Runner {
            id: 1,
            name: "runner-1".to_string(),
            active: true,
            webhook: "https://runner.example.com/hook".to_string(),
        };
        assert_eq!(runner.name, "runner-1");
        assert!(runner.active);
    }

    #[test]
    fn test_audit_event_type_fields() {
        let event = AuditEvent {
            id: 1,
            project_id: Some(10),
            object_type: "template".to_string(),
            object_id: 5,
            description: "Template created".to_string(),
            created: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(event.object_type, "template");
        assert_eq!(event.description, "Template created");
    }

    #[test]
    fn test_kubernetes_namespace_fields() {
        let ns = KubernetesNamespace {
            name: "default".to_string(),
            status: "Active".to_string(),
            labels: vec!["kubernetes.io/metadata.name=default".to_string()],
        };
        assert_eq!(ns.name, "default");
        assert_eq!(ns.status, "Active");
    }

    #[test]
    fn test_kubernetes_node_fields() {
        let node = KubernetesNode {
            name: "node-1".to_string(),
            status: "Ready".to_string(),
            roles: vec!["master".to_string(), "control-plane".to_string()],
            version: "v1.30.0".to_string(),
            os_image: "Ubuntu 22.04.3 LTS".to_string(),
        };
        assert_eq!(node.version, "v1.30.0");
        assert_eq!(node.os_image, "Ubuntu 22.04.3 LTS");
    }

    #[test]
    fn test_kubernetes_cluster_info_fields() {
        let info = KubernetesClusterInfo {
            server_url: "https://k8s.example.com:6443".to_string(),
            version: "v1.30.0".to_string(),
            namespace: "velum".to_string(),
        };
        assert_eq!(info.server_url, "https://k8s.example.com:6443");
        assert_eq!(info.namespace, "velum");
    }

    #[test]
    fn test_query_root_multiple_instances() {
        let _root1 = QueryRoot;
        let _root2 = QueryRoot;
        // Multiple instances should be possible
    }
}
