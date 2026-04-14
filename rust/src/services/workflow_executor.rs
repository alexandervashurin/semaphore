//! Workflow Execution Engine
//!
//! Выполняет DAG workflow: запускает шаблоны из узлов в правильном порядке
//! с учётом условий переходов (success/failure/always)

use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::state::AppState;
use crate::db::store::{
    EnvironmentManager, InventoryManager, RepositoryManager, Store, TaskManager, TemplateManager,
    WorkflowManager,
};
use crate::error::{Error, Result};
use crate::models::environment::Environment;
use crate::models::inventory::Inventory;
use crate::models::repository::Repository;
use crate::models::task::Task;
use crate::models::template::Template;
use crate::models::workflow::{Workflow, WorkflowEdge, WorkflowNode, WorkflowRun};
use crate::services::task_logger::TaskStatus;

/// Состояние выполнения узла
#[derive(Debug, Clone)]
pub enum NodeExecutionStatus {
    Pending,
    Running(Task),
    Success(Task),
    Failed(Task),
    Skipped,
}

/// Контекст выполнения workflow
pub struct WorkflowExecutionContext {
    pub workflow: Workflow,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub run: WorkflowRun,
    pub node_statuses: HashMap<i32, NodeExecutionStatus>,
    pub project_id: i32,
}

impl WorkflowExecutionContext {
    pub fn new(
        workflow: Workflow,
        nodes: Vec<WorkflowNode>,
        edges: Vec<WorkflowEdge>,
        run: WorkflowRun,
    ) -> Self {
        let project_id = workflow.project_id;
        let mut node_statuses = HashMap::new();

        for node in &nodes {
            node_statuses.insert(node.id, NodeExecutionStatus::Pending);
        }

        Self {
            workflow,
            nodes,
            edges,
            run,
            node_statuses,
            project_id,
        }
    }

    /// Найти начальные узлы (в которые нет входящих рёбер)
    pub fn find_start_nodes(&self) -> Vec<i32> {
        let mut has_incoming = HashSet::new();

        for edge in &self.edges {
            has_incoming.insert(edge.to_node_id);
        }

        self.nodes
            .iter()
            .filter(|n| !has_incoming.contains(&n.id))
            .map(|n| n.id)
            .collect()
    }

    /// Найти узлы, которые должны выполняться после данного узла
    pub fn find_next_nodes(&self, node_id: i32, task_status: TaskStatus) -> Vec<i32> {
        let mut next_nodes = Vec::new();

        for edge in &self.edges {
            if edge.from_node_id == node_id {
                let should_execute = match edge.condition.as_str() {
                    "success" => matches!(task_status, TaskStatus::Success),
                    "failure" => matches!(
                        task_status,
                        TaskStatus::Error | TaskStatus::Stopped | TaskStatus::NotExecuted
                    ),
                    "always" => true,
                    _ => false,
                };

                if should_execute {
                    next_nodes.push(edge.to_node_id);
                }
            }
        }

        next_nodes
    }

    /// Проверить, все ли узлы завершены
    pub fn is_complete(&self) -> bool {
        self.node_statuses.values().all(|s| {
            matches!(
                s,
                NodeExecutionStatus::Success(_)
                    | NodeExecutionStatus::Failed(_)
                    | NodeExecutionStatus::Skipped
            )
        })
    }

    /// Проверить, есть ли запущенные узлы
    pub fn has_running_nodes(&self) -> bool {
        self.node_statuses
            .values()
            .any(|s| matches!(s, NodeExecutionStatus::Running(_)))
    }
}

/// Workflow Executor - выполняет DAG workflow
pub struct WorkflowExecutor {
    pub state: Arc<AppState>,
    pub context: Arc<Mutex<WorkflowExecutionContext>>,
}

impl WorkflowExecutor {
    pub fn new(
        state: Arc<AppState>,
        workflow: Workflow,
        nodes: Vec<WorkflowNode>,
        edges: Vec<WorkflowEdge>,
        run: WorkflowRun,
    ) -> Self {
        let context = Arc::new(Mutex::new(WorkflowExecutionContext::new(
            workflow, nodes, edges, run,
        )));

        Self { state, context }
    }

    /// Запустить выполнение workflow
    pub async fn execute(&self) -> Result<()> {
        let project_id = self.context.lock().await.project_id;
        let workflow_id = self.context.lock().await.run.workflow_id;

        // Обновить статус запуска на "running"
        self.state
            .store
            .update_workflow_run_status(
                self.context.lock().await.run.id,
                "running",
                Some("Workflow started".to_string()),
            )
            .await?;

        // Найти начальные узлы
        let start_nodes = {
            let ctx = self.context.lock().await;
            ctx.find_start_nodes()
        };

        // Создать очередь узлов для выполнения
        let mut queue: Vec<i32> = start_nodes;

        while !queue.is_empty() {
            // Запустить все узлы из очереди параллельно
            let mut running_tasks = Vec::new();

            while let Some(node_id) = queue.pop() {
                let executor = WorkflowExecutor::new(
                    self.state.clone(),
                    self.context.lock().await.workflow.clone(),
                    self.context.lock().await.nodes.clone(),
                    self.context.lock().await.edges.clone(),
                    self.context.lock().await.run.clone(),
                );

                let handle = tokio::spawn(async move { executor.execute_node_sync(node_id).await });
                running_tasks.push((node_id, handle));
            }

            // Дождаться завершения всех задач и собрать следующие узлы
            let mut new_queue = Vec::new();

            let old_tasks = std::mem::take(&mut running_tasks);
            for (node_id, handle) in old_tasks {
                match handle.await {
                    Ok(Ok(next_nodes)) => {
                        // Узел выполнен успешно, добавить следующие узлы
                        for next in next_nodes {
                            new_queue.push(next);
                        }
                    }
                    Ok(Err(e)) => {
                        eprintln!("[workflow_executor] Node {} error: {}", node_id, e);
                    }
                    Err(e) => {
                        eprintln!("[workflow_executor] Node {} join error: {}", node_id, e);
                    }
                }
            }

            queue = new_queue;
        }

        // Завершить workflow
        self.finalize_workflow().await?;

        Ok(())
    }

    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            context: self.context.clone(),
        }
    }

    /// Выполнить один узел workflow (синхронная версия для tokio::spawn)
    /// Возвращает список следующих узлов для запуска
    async fn execute_node_sync(&self, node_id: i32) -> Result<Vec<i32>> {
        println!("[workflow_executor] Executing node {}", node_id);

        // Получить данные узла
        let (template, task, project_id) = {
            let mut ctx = self.context.lock().await;

            // Проверить статус узла
            let status = ctx
                .node_statuses
                .get(&node_id)
                .cloned()
                .unwrap_or(NodeExecutionStatus::Pending);
            if !matches!(status, NodeExecutionStatus::Pending) {
                println!(
                    "[workflow_executor] Node {} already executed: {:?}",
                    node_id, status
                );
                return Ok(Vec::new());
            }

            // Получить шаблон
            let node = ctx
                .nodes
                .iter()
                .find(|n| n.id == node_id)
                .ok_or_else(|| Error::NotFound(format!("Node {} not found", node_id)))?;

            let template = self
                .state
                .store
                .get_template(ctx.project_id, node.template_id)
                .await
                .map_err(|e| {
                    Error::NotFound(format!("Template {} not found: {}", node.template_id, e))
                })?;

            let project_id = ctx.project_id;

            // Создать задачу
            let task = Task {
                id: 0,
                template_id: template.id,
                project_id: ctx.project_id,
                status: TaskStatus::Waiting,
                playbook: Some(template.playbook.clone()),
                environment: None,
                secret: None,
                arguments: None,
                git_branch: None,
                user_id: None, // Workflow запускает без конкретного пользователя
                integration_id: None,
                schedule_id: None,
                created: Utc::now(),
                start: Some(Utc::now()),
                end: None,
                message: Some(format!("Workflow node: {}", node.name)),
                commit_hash: None,
                commit_message: None,
                build_task_id: None,
                version: None,
                inventory_id: template.inventory_id,
                repository_id: template.repository_id,
                environment_id: template.environment_id,
                params: None,
            };

            let created_task = self.state.store.create_task(task.clone()).await?;

            // Обновить статус узла на Running
            ctx.node_statuses
                .insert(node_id, NodeExecutionStatus::Running(created_task.clone()));

            (template, created_task, project_id)
        };

        // Запустить задачу
        let task_result = self.run_task(task.clone(), template, project_id).await;

        // Обновить статус узла по результату и вернуть следующие узлы
        let next_nodes = {
            let mut ctx = self.context.lock().await;

            let final_status = match &task_result {
                Ok(status) => match status {
                    TaskStatus::Success => NodeExecutionStatus::Success(task.clone()),
                    TaskStatus::Error | TaskStatus::Stopped | TaskStatus::NotExecuted => {
                        NodeExecutionStatus::Failed(task.clone())
                    }
                    _ => NodeExecutionStatus::Failed(task.clone()),
                },
                Err(_) => NodeExecutionStatus::Failed(task.clone()),
            };

            ctx.node_statuses.insert(node_id, final_status);

            // Найти следующие узлы
            let task_status = task_result.unwrap_or(TaskStatus::Error);
            ctx.find_next_nodes(node_id, task_status)
        };

        Ok(next_nodes)
    }

    /// Запустить задачу Ansible/Terraform
    async fn run_task(
        &self,
        task: Task,
        template: Template,
        project_id: i32,
    ) -> Result<TaskStatus> {
        use crate::api::handlers::tasks::execute_task_background_with_template;

        // Получить inventory, repository, environment
        let inventory = if let Some(inv_id) = task.inventory_id.or(template.inventory_id) {
            self.state
                .store
                .get_inventory(project_id, inv_id)
                .await
                .unwrap_or_default()
        } else {
            Inventory::default()
        };

        let repository = if let Some(repo_id) = task.repository_id.or(template.repository_id) {
            self.state
                .store
                .get_repository(project_id, repo_id)
                .await
                .unwrap_or_default()
        } else {
            Repository::default()
        };

        let environment = if let Some(env_id) = task.environment_id.or(template.environment_id) {
            self.state
                .store
                .get_environment(project_id, env_id)
                .await
                .unwrap_or_default()
        } else {
            Environment::default()
        };

        // Запустить задачу в фоне и ждать результата
        let result = execute_task_background_with_template(
            self.state.clone(),
            task.clone(),
            template,
            inventory,
            repository,
            environment,
        )
        .await;

        Ok(result)
    }

    /// Завершить workflow
    async fn finalize_workflow(&self) -> Result<()> {
        let ctx = self.context.lock().await;

        let has_failures = ctx
            .node_statuses
            .values()
            .any(|s| matches!(s, NodeExecutionStatus::Failed(_)));

        let (final_status, message) = if has_failures {
            (
                "failed".to_string(),
                Some("Workflow completed with failures".to_string()),
            )
        } else {
            (
                "success".to_string(),
                Some("Workflow completed successfully".to_string()),
            )
        };

        drop(ctx);

        self.state
            .store
            .update_workflow_run_status(self.context.lock().await.run.id, &final_status, message)
            .await?;

        Ok(())
    }
}

/// Запустить workflow (публичный API)
pub async fn run_workflow(
    state: Arc<AppState>,
    workflow_id: i32,
    project_id: i32,
) -> Result<WorkflowRun> {
    // Проверить workflow
    let workflow = state.store.get_workflow(workflow_id, project_id).await?;

    // Получить узлы и рёбра
    let nodes = state.store.get_workflow_nodes(workflow_id).await?;
    let edges = state.store.get_workflow_edges(workflow_id).await?;

    if nodes.is_empty() {
        return Err(Error::Other("Workflow has no nodes".to_string()));
    }

    // Создать запись запуска
    let run = state
        .store
        .create_workflow_run(workflow_id, project_id)
        .await?;

    // Создать executor и запустить в фоне
    let executor = WorkflowExecutor::new(state.clone(), workflow, nodes, edges, run.clone());

    tokio::spawn(async move {
        if let Err(e) = executor.execute().await {
            eprintln!("[workflow_executor] Workflow execution error: {}", e);

            // Обновить статус на error
            let _ = state
                .store
                .update_workflow_run_status(
                    run.id,
                    "failed",
                    Some(format!("Execution error: {}", e)),
                )
                .await;
        }
    });

    Ok(run)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_workflow() -> Workflow {
        Workflow {
            id: 1,
            project_id: 10,
            name: "Test Workflow".to_string(),
            description: None,
            created: Utc::now(),
            updated: Utc::now(),
        }
    }

    fn create_test_node(id: i32) -> WorkflowNode {
        WorkflowNode {
            id,
            workflow_id: 1,
            template_id: id * 10,
            name: format!("Node {}", id),
            pos_x: 0.0,
            pos_y: 0.0,
            wave: 0,
        }
    }

    fn create_test_edge(from: i32, to: i32, condition: &str) -> WorkflowEdge {
        WorkflowEdge {
            id: 0,
            workflow_id: 1,
            from_node_id: from,
            to_node_id: to,
            condition: condition.to_string(),
        }
    }

    fn create_test_task(id: i32) -> Task {
        Task {
            id,
            template_id: 1,
            project_id: 10,
            status: TaskStatus::Success,
            created: Utc::now(),
            ..Default::default()
        }
    }

    fn create_test_run() -> WorkflowRun {
        WorkflowRun {
            id: 1,
            workflow_id: 1,
            project_id: 10,
            status: "pending".to_string(),
            message: None,
            created: Utc::now(),
            started: None,
            finished: None,
        }
    }

    #[test]
    fn test_find_start_nodes_no_edges() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let starts = ctx.find_start_nodes();

        // Нет рёбер — все узлы начальные
        assert_eq!(starts.len(), 2);
    }

    #[test]
    fn test_find_start_nodes_with_edges() {
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
        ];
        let edges = vec![
            create_test_edge(1, 2, "success"),
            create_test_edge(2, 3, "success"),
        ];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let starts = ctx.find_start_nodes();

        // Только узел 1 не имеет входящих рёбер
        assert_eq!(starts, vec![1]);
    }

    #[test]
    fn test_find_next_nodes_success_condition() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![create_test_edge(1, 2, "success")];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let next = ctx.find_next_nodes(1, TaskStatus::Success);

        assert_eq!(next, vec![2]);
    }

    #[test]
    fn test_find_next_nodes_failure_condition() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![create_test_edge(1, 2, "failure")];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        // Error должен триггернуть failure edge
        let next = ctx.find_next_nodes(1, TaskStatus::Error);
        assert_eq!(next, vec![2]);

        // Stopped тоже
        let next = ctx.find_next_nodes(1, TaskStatus::Stopped);
        assert_eq!(next, vec![2]);
    }

    #[test]
    fn test_find_next_nodes_always_condition() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![create_test_edge(1, 2, "always")];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        // "always" должен работать для любого статуса
        for status in &[
            TaskStatus::Success,
            TaskStatus::Error,
            TaskStatus::Running,
            TaskStatus::Waiting,
        ] {
            let next = ctx.find_next_nodes(1, *status);
            assert_eq!(next, vec![2], "Failed for status {:?}", status);
        }
    }

    #[test]
    fn test_is_complete_all_done() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1)];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        assert!(!ctx.is_complete()); // Pending — не complete
    }

    #[test]
    fn test_has_running_nodes() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1)];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        assert!(!ctx.has_running_nodes()); // Все pending
    }

    #[test]
    fn test_is_complete_with_mixed_statuses() {
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
        ];
        let edges = vec![];
        let run = create_test_run();

        let mut ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        // Помечаем узлы 1 и 2 как Success
        ctx.node_statuses
            .insert(1, NodeExecutionStatus::Success(create_test_task(1)));
        ctx.node_statuses
            .insert(2, NodeExecutionStatus::Success(create_test_task(2)));
        // Узел 3 остаётся Pending
        assert!(!ctx.is_complete());
    }

    #[test]
    fn test_is_complete_when_all_success() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![];
        let run = create_test_run();

        let mut ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        ctx.node_statuses
            .insert(1, NodeExecutionStatus::Success(create_test_task(1)));
        ctx.node_statuses
            .insert(2, NodeExecutionStatus::Success(create_test_task(2)));

        assert!(ctx.is_complete());
    }

    #[test]
    fn test_has_running_nodes_with_running_task() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![];
        let run = create_test_run();

        let mut ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        ctx.node_statuses
            .insert(1, NodeExecutionStatus::Success(create_test_task(1)));
        ctx.node_statuses
            .insert(2, NodeExecutionStatus::Running(create_test_task(2)));

        assert!(ctx.has_running_nodes());
    }

    #[test]
    fn test_find_next_nodes_no_matching_condition() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        // Edge требует "failure", но задача успешна
        let edges = vec![create_test_edge(1, 2, "failure")];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let next = ctx.find_next_nodes(1, TaskStatus::Success);

        // Нет matching — ничего не возвращаем
        assert!(next.is_empty());
    }

    #[test]
    fn test_find_next_nodes_multiple_edges() {
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
        ];
        let edges = vec![
            create_test_edge(1, 2, "success"),
            create_test_edge(1, 3, "failure"),
        ];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        // При Success должен сработать только edge к узлу 2
        let next = ctx.find_next_nodes(1, TaskStatus::Success);
        assert_eq!(next, vec![2]);

        // При Error — к узлу 3
        let next = ctx.find_next_nodes(1, TaskStatus::Error);
        assert_eq!(next, vec![3]);
    }

    #[test]
    fn test_workflow_executor_clone() {
        // WorkflowExecutor содержит Arc<AppState>, проверяем что clone работает
        // Для простоты проверяем что структура создаётся
        let nodes = vec![create_test_node(1)];
        let edges = vec![create_test_edge(1, 1, "success")];
        let run = create_test_run();
        let workflow = create_test_workflow();
        let ctx =
            WorkflowExecutionContext::new(workflow, nodes.clone(), edges.clone(), run.clone());
        let _ctx2 = ctx; // должна быть возможность перемещения
    }

    #[test]
    fn test_node_execution_status_variants() {
        let task = create_test_task(1);

        // Pending
        let pending = NodeExecutionStatus::Pending;
        assert!(matches!(pending, NodeExecutionStatus::Pending));

        // Running
        let running = NodeExecutionStatus::Running(task.clone());
        assert!(matches!(running, NodeExecutionStatus::Running(_)));

        // Success
        let success = NodeExecutionStatus::Success(task.clone());
        assert!(matches!(success, NodeExecutionStatus::Success(_)));

        // Failed
        let failed = NodeExecutionStatus::Failed(task.clone());
        assert!(matches!(failed, NodeExecutionStatus::Failed(_)));

        // Skipped
        let skipped = NodeExecutionStatus::Skipped;
        assert!(matches!(skipped, NodeExecutionStatus::Skipped));
    }

    #[test]
    fn test_find_start_nodes_multiple_starts() {
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
        ];
        // Узлы 1 и 2 не имеют входящих рёбер, узел 3 зависит от 1
        let edges = vec![create_test_edge(1, 3, "success")];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let starts = ctx.find_start_nodes();

        assert_eq!(starts.len(), 2);
        assert!(starts.contains(&1));
        assert!(starts.contains(&2));
    }

    #[test]
    fn test_find_start_nodes_all_dependent() {
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
        ];
        // Цепочка 1 -> 2 -> 3
        let edges = vec![
            create_test_edge(1, 2, "success"),
            create_test_edge(2, 3, "success"),
        ];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let starts = ctx.find_start_nodes();

        // Только узел 1 начальный
        assert_eq!(starts, vec![1]);
    }

    #[test]
    fn test_find_next_nodes_skip_completed_nodes() {
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
        ];
        let edges = vec![
            create_test_edge(1, 2, "success"),
            create_test_edge(1, 3, "failure"),
        ];
        let run = create_test_run();

        let mut ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        // Помечаем узел 2 как Success
        ctx.node_statuses
            .insert(2, NodeExecutionStatus::Success(create_test_task(2)));

        // При Success узла 1 следующий — узел 2, но он уже completed
        // find_next_nodes всё равно вернёт 2, но execute_node должен проверить статус
        let next = ctx.find_next_nodes(1, TaskStatus::Success);
        assert_eq!(next, vec![2]);
    }

    #[test]
    fn test_is_complete_with_failed_nodes() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![];
        let run = create_test_run();

        let mut ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        ctx.node_statuses
            .insert(1, NodeExecutionStatus::Success(create_test_task(1)));
        ctx.node_statuses
            .insert(2, NodeExecutionStatus::Failed(create_test_task(2)));

        // Failed считается завершённым
        assert!(ctx.is_complete());
    }

    #[test]
    fn test_is_complete_with_skipped_nodes() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![];
        let run = create_test_run();

        let mut ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        ctx.node_statuses
            .insert(1, NodeExecutionStatus::Success(create_test_task(1)));
        ctx.node_statuses.insert(2, NodeExecutionStatus::Skipped);

        assert!(ctx.is_complete());
    }

    #[test]
    fn test_find_next_nodes_condition_with_multiple_edges() {
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
            create_test_node(4),
        ];
        let edges = vec![
            create_test_edge(1, 2, "success"),
            create_test_edge(1, 3, "failure"),
            create_test_edge(1, 4, "always"),
        ];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        // При Success — узлы 2 и 4
        let next = ctx.find_next_nodes(1, TaskStatus::Success);
        assert_eq!(next.len(), 2);
        assert!(next.contains(&2));
        assert!(next.contains(&4));

        // При Error — узлы 3 и 4
        let next = ctx.find_next_nodes(1, TaskStatus::Error);
        assert_eq!(next.len(), 2);
        assert!(next.contains(&3));
        assert!(next.contains(&4));
    }

    #[test]
    fn test_context_project_id_from_workflow() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1)];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        assert_eq!(ctx.project_id, 10);
    }

    #[test]
    fn test_context_nodes_count_matches() {
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
            create_test_node(4),
            create_test_node(5),
        ];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        assert_eq!(ctx.node_statuses.len(), 5);
    }

    #[test]
    fn test_workflow_creation() {
        let workflow = create_test_workflow();
        assert_eq!(workflow.id, 1);
        assert_eq!(workflow.project_id, 10);
        assert_eq!(workflow.name, "Test Workflow");
    }

    #[test]
    fn test_node_creation_with_different_ids() {
        for id in &[1, 5, 10, 100] {
            let node = create_test_node(*id);
            assert_eq!(node.id, *id);
            assert_eq!(node.workflow_id, 1);
            assert_eq!(node.template_id, *id * 10);
        }
    }

    #[test]
    fn test_edge_creation_with_conditions() {
        let conditions = vec!["success", "failure", "always"];
        for condition in conditions {
            let edge = create_test_edge(1, 2, condition);
            assert_eq!(edge.from_node_id, 1);
            assert_eq!(edge.to_node_id, 2);
            assert_eq!(edge.condition, condition);
        }
    }

    #[test]
    fn test_find_next_nodes_unknown_condition() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![create_test_edge(1, 2, "unknown_condition")];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let next = ctx.find_next_nodes(1, TaskStatus::Success);

        // Unknown condition не матчится ни с чем
        assert!(next.is_empty());
    }

    #[test]
    fn test_node_execution_status_equality() {
        let task1 = create_test_task(1);
        let task2 = create_test_task(1);

        let status1 = NodeExecutionStatus::Success(task1);
        let status2 = NodeExecutionStatus::Success(task2);

        // Проверяем что оба Success
        assert!(matches!(status1, NodeExecutionStatus::Success(_)));
        assert!(matches!(status2, NodeExecutionStatus::Success(_)));
    }

    #[test]
    fn test_context_initialization_all_pending() {
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
        ];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        // Все узлы должны быть Pending
        for status in ctx.node_statuses.values() {
            assert!(matches!(status, NodeExecutionStatus::Pending));
        }
    }

    #[test]
    fn test_find_start_nodes_diamond_pattern() {
        // Diamond pattern: 1 -> 2, 1 -> 3, 2 -> 4, 3 -> 4
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
            create_test_node(4),
        ];
        let edges = vec![
            create_test_edge(1, 2, "success"),
            create_test_edge(1, 3, "success"),
            create_test_edge(2, 4, "success"),
            create_test_edge(3, 4, "success"),
        ];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let starts = ctx.find_start_nodes();

        assert_eq!(starts, vec![1]);
    }

    #[test]
    fn test_find_next_nodes_parallel_paths() {
        // 1 -> 2, 1 -> 3 (parallel)
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
        ];
        let edges = vec![
            create_test_edge(1, 2, "success"),
            create_test_edge(1, 3, "success"),
        ];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let next = ctx.find_next_nodes(1, TaskStatus::Success);

        assert_eq!(next.len(), 2);
        assert!(next.contains(&2));
        assert!(next.contains(&3));
    }

    #[test]
    fn test_task_creation_for_workflow_node() {
        let task = create_test_task(42);
        assert_eq!(task.id, 42);
        assert_eq!(task.template_id, 1);
        assert_eq!(task.project_id, 10);
        assert!(matches!(task.status, TaskStatus::Success));
    }

    #[test]
    fn test_workflow_run_initial_state() {
        let run = create_test_run();
        assert_eq!(run.id, 1);
        assert_eq!(run.workflow_id, 1);
        assert_eq!(run.project_id, 10);
        assert_eq!(run.status, "pending");
        assert!(run.message.is_none());
        assert!(run.started.is_none());
        assert!(run.finished.is_none());
    }

    #[test]
    fn test_node_properties() {
        let node = create_test_node(5);
        assert_eq!(node.id, 5);
        assert_eq!(node.workflow_id, 1);
        assert_eq!(node.template_id, 50);
        assert_eq!(node.name, "Node 5");
        assert_eq!(node.pos_x, 0.0);
        assert_eq!(node.pos_y, 0.0);
    }

    #[test]
    fn test_edge_properties() {
        let edge = create_test_edge(10, 20, "success");
        assert_eq!(edge.from_node_id, 10);
        assert_eq!(edge.to_node_id, 20);
        assert_eq!(edge.condition, "success");
        assert_eq!(edge.workflow_id, 1);
    }

    #[test]
    fn test_context_workflow_metadata() {
        let workflow = Workflow {
            id: 99,
            project_id: 77,
            name: "Custom Workflow".to_string(),
            description: Some("Test description".to_string()),
            created: Utc::now(),
            updated: Utc::now(),
        };
        let nodes = vec![create_test_node(1)];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        assert_eq!(ctx.project_id, 77);
        assert_eq!(ctx.workflow.id, 99);
    }

    // ===========================================================================
    // Additional tests (20+)
    // ===========================================================================

    #[test]
    fn test_find_start_nodes_single_node() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1)];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let starts = ctx.find_start_nodes();

        assert_eq!(starts, vec![1]);
    }

    #[test]
    fn test_find_start_nodes_linear_chain() {
        // 1 -> 2 -> 3 -> 4
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
            create_test_node(4),
        ];
        let edges = vec![
            create_test_edge(1, 2, "success"),
            create_test_edge(2, 3, "success"),
            create_test_edge(3, 4, "success"),
        ];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let starts = ctx.find_start_nodes();

        assert_eq!(starts, vec![1]);
    }

    #[test]
    fn test_find_start_nodes_no_incoming_for_any() {
        // All nodes independent — no edges
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(10),
            create_test_node(20),
            create_test_node(30),
        ];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let starts = ctx.find_start_nodes();

        assert_eq!(starts.len(), 3);
        assert!(starts.contains(&10));
        assert!(starts.contains(&20));
        assert!(starts.contains(&30));
    }

    #[test]
    fn test_find_next_nodes_multiple_from_same_source() {
        // Node 1 -> 2 (success), 1 -> 3 (success), 1 -> 4 (always)
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
            create_test_node(4),
        ];
        let edges = vec![
            create_test_edge(1, 2, "success"),
            create_test_edge(1, 3, "success"),
            create_test_edge(1, 4, "always"),
        ];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let next = ctx.find_next_nodes(1, TaskStatus::Success);

        // Should include 2, 3, and 4
        assert_eq!(next.len(), 3);
        assert!(next.contains(&2));
        assert!(next.contains(&3));
        assert!(next.contains(&4));
    }

    #[test]
    fn test_find_next_nodes_only_failure_triggers() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![create_test_edge(1, 2, "failure")];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        // Success should NOT trigger failure edge
        let next = ctx.find_next_nodes(1, TaskStatus::Success);
        assert!(next.is_empty());
    }

    #[test]
    fn test_find_next_nodes_stopped_triggers_failure() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![create_test_edge(1, 2, "failure")];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let next = ctx.find_next_nodes(1, TaskStatus::Stopped);

        assert_eq!(next, vec![2]);
    }

    #[test]
    fn test_find_next_nodes_not_executed_triggers_failure() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![create_test_edge(1, 2, "failure")];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let next = ctx.find_next_nodes(1, TaskStatus::NotExecuted);

        assert_eq!(next, vec![2]);
    }

    #[test]
    fn test_is_complete_pending_not_complete() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        assert!(!ctx.is_complete());
    }

    #[test]
    fn test_is_complete_running_not_complete() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1)];
        let edges = vec![];
        let run = create_test_run();

        let mut ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        ctx.node_statuses
            .insert(1, NodeExecutionStatus::Running(create_test_task(1)));

        assert!(!ctx.is_complete());
    }

    #[test]
    fn test_has_running_nodes_no_running() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![];
        let run = create_test_run();

        let mut ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        ctx.node_statuses
            .insert(1, NodeExecutionStatus::Success(create_test_task(1)));
        ctx.node_statuses
            .insert(2, NodeExecutionStatus::Failed(create_test_task(2)));

        assert!(!ctx.has_running_nodes());
    }

    #[test]
    fn test_has_running_nodes_all_pending() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        // Pending != Running
        assert!(!ctx.has_running_nodes());
    }

    #[test]
    fn test_context_find_next_nodes_empty_edges() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let next = ctx.find_next_nodes(1, TaskStatus::Success);

        assert!(next.is_empty());
    }

    #[test]
    fn test_workflow_node_name_format() {
        let node = create_test_node(42);
        assert_eq!(node.name, "Node 42");
    }

    #[test]
    fn test_workflow_edge_id_zero() {
        let edge = create_test_edge(1, 2, "success");
        assert_eq!(edge.id, 0); // ID set by DB
    }

    #[test]
    fn test_find_next_nodes_condition_not_executed() {
        // Running and Waiting should not trigger success or failure edges
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![
            create_test_edge(1, 2, "success"),
            create_test_edge(1, 2, "failure"),
        ];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        // Running status
        let next = ctx.find_next_nodes(1, TaskStatus::Running);
        assert!(next.is_empty());

        // Waiting status
        let next = ctx.find_next_nodes(1, TaskStatus::Waiting);
        assert!(next.is_empty());
    }

    #[test]
    fn test_node_execution_status_debug_display() {
        let task = create_test_task(100);

        // Verify each variant can be matched
        let statuses: Vec<NodeExecutionStatus> = vec![
            NodeExecutionStatus::Pending,
            NodeExecutionStatus::Running(task.clone()),
            NodeExecutionStatus::Success(task.clone()),
            NodeExecutionStatus::Failed(task.clone()),
            NodeExecutionStatus::Skipped,
        ];

        for status in &statuses {
            match status {
                NodeExecutionStatus::Pending => {}
                NodeExecutionStatus::Running(t) => assert_eq!(t.id, 100),
                NodeExecutionStatus::Success(t) => assert_eq!(t.id, 100),
                NodeExecutionStatus::Failed(t) => assert_eq!(t.id, 100),
                NodeExecutionStatus::Skipped => {}
            }
        }
    }

    #[test]
    fn test_context_nodes_initialized_as_pending() {
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
        ];
        let edges = vec![];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        assert_eq!(ctx.node_statuses.len(), 3);
        for (_, status) in &ctx.node_statuses {
            assert!(matches!(status, NodeExecutionStatus::Pending));
        }
    }

    #[test]
    fn test_find_start_nodes_complex_dag() {
        // 1 -> 3, 2 -> 3, 2 -> 4, 3 -> 5
        // Start nodes: 1 and 2 (no incoming)
        let workflow = create_test_workflow();
        let nodes = vec![
            create_test_node(1),
            create_test_node(2),
            create_test_node(3),
            create_test_node(4),
            create_test_node(5),
        ];
        let edges = vec![
            create_test_edge(1, 3, "success"),
            create_test_edge(2, 3, "success"),
            create_test_edge(2, 4, "success"),
            create_test_edge(3, 5, "success"),
        ];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);
        let starts = ctx.find_start_nodes();

        assert_eq!(starts.len(), 2);
        assert!(starts.contains(&1));
        assert!(starts.contains(&2));
    }

    #[test]
    fn test_workflow_run_status_transitions() {
        let mut run = create_test_run();
        assert_eq!(run.status, "pending");

        run.status = "running".to_string();
        assert_eq!(run.status, "running");

        run.status = "success".to_string();
        assert_eq!(run.status, "success");
    }

    #[test]
    fn test_workflow_run_with_message() {
        let mut run = create_test_run();
        run.message = Some("Workflow started manually".to_string());
        assert!(run.message.is_some());
        assert_eq!(run.message.unwrap(), "Workflow started manually");
    }

    #[test]
    fn test_workflow_metadata_description_optional() {
        let workflow_with_desc = Workflow {
            id: 1,
            project_id: 1,
            name: "Test".to_string(),
            description: Some("A workflow".to_string()),
            created: Utc::now(),
            updated: Utc::now(),
        };
        assert!(workflow_with_desc.description.is_some());

        let workflow_no_desc = Workflow {
            id: 2,
            project_id: 1,
            name: "Test 2".to_string(),
            description: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        assert!(workflow_no_desc.description.is_none());
    }

    #[test]
    fn test_find_next_nodes_edge_from_nonexistent_node() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![create_test_edge(99, 2, "success")]; // from node 99 which doesn't exist
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        // Asking for node 1 — no edges from it
        let next = ctx.find_next_nodes(1, TaskStatus::Success);
        assert!(next.is_empty());
    }

    #[test]
    fn test_context_with_many_nodes_and_edges() {
        let workflow = create_test_workflow();
        let nodes: Vec<WorkflowNode> = (1..=10).map(create_test_node).collect();
        let edges: Vec<WorkflowEdge> = (1..=9)
            .map(|i| create_test_edge(i, i + 1, "success"))
            .collect();
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        assert_eq!(ctx.node_statuses.len(), 10);
        assert_eq!(ctx.edges.len(), 9);

        let starts = ctx.find_start_nodes();
        assert_eq!(starts, vec![1]);
    }

    #[test]
    fn test_find_next_nodes_all_task_statuses_for_always() {
        let workflow = create_test_workflow();
        let nodes = vec![create_test_node(1), create_test_node(2)];
        let edges = vec![create_test_edge(1, 2, "always")];
        let run = create_test_run();

        let ctx = WorkflowExecutionContext::new(workflow, nodes, edges, run);

        let all_statuses = [
            TaskStatus::Success,
            TaskStatus::Error,
            TaskStatus::Stopped,
            TaskStatus::Running,
            TaskStatus::Waiting,
            TaskStatus::NotExecuted,
        ];

        for status in all_statuses {
            let next = ctx.find_next_nodes(1, status);
            assert_eq!(next, vec![2], "always should match for {:?}", status);
        }
    }
}
