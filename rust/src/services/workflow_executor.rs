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
