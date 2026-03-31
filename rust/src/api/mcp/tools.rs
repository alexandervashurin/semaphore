//! MCP tool implementations — calls the Velum store directly (no HTTP round-trips).

use super::protocol::{prop_bool, prop_int, prop_int_opt, prop_str, prop_str_opt, ToolContent};
use crate::api::state::AppState;
use crate::models::repository::RepositoryType;
use crate::models::{Environment, Project, Repository, Schedule, Task};
use crate::services::task_logger::TaskStatus;
use serde_json::{json, Value};
use std::sync::Arc;

// ── Result type ──────────────────────────────────────────────────────────────

pub struct ToolResult {
    pub content: Vec<ToolContent>,
    pub is_error: bool,
}

impl ToolResult {
    pub fn ok(v: &Value) -> Self {
        Self {
            content: vec![ToolContent::json(v)],
            is_error: false,
        }
    }
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::text(s)],
            is_error: false,
        }
    }
    pub fn error(s: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::text(s)],
            is_error: true,
        }
    }
}

// ── Tool definitions ─────────────────────────────────────────────────────────

pub fn all_definitions() -> Vec<Value> {
    vec![
        // ── Projects ─────────────────────────────────────────────────────────
        tool("list_projects", "List all projects visible to the current user.", json!({
            "type":"object","properties":{},"required":[]
        })),
        tool("get_project", "Get details of a specific project.", json!({
            "type":"object","properties":{"project_id":prop_int("Project ID")},"required":["project_id"]
        })),
        tool("create_project", "Create a new project.", json!({
            "type":"object",
            "properties":{
                "name":prop_str("Project name"),
                "alert":prop_bool("Enable email alerts for this project")
            },
            "required":["name"]
        })),
        tool("delete_project", "Delete a project and all its resources.", json!({
            "type":"object","properties":{"project_id":prop_int("Project ID")},"required":["project_id"]
        })),

        // ── Templates ────────────────────────────────────────────────────────
        tool("list_templates", "List all task templates for a project.", json!({
            "type":"object","properties":{"project_id":prop_int("Project ID")},"required":["project_id"]
        })),
        tool("get_template", "Get details of a specific template.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "template_id":prop_int("Template ID")
            },"required":["project_id","template_id"]
        })),
        tool("run_template", "Run a template (creates and queues a task).", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "template_id":prop_int("Template ID"),
                "message":prop_str_opt("Optional run message")
            },"required":["project_id","template_id"]
        })),

        // ── Tasks ────────────────────────────────────────────────────────────
        tool("list_tasks", "List tasks for a project, optionally filtered by template.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "template_id":prop_int_opt("Filter by template (optional)"),
                "limit":prop_int_opt("Max results (default 50)")
            },"required":["project_id"]
        })),
        tool("get_task", "Get details and status of a specific task.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "task_id":prop_int("Task ID")
            },"required":["project_id","task_id"]
        })),
        tool("get_task_output", "Get console output lines of a task.", json!({
            "type":"object",
            "properties":{
                "task_id":prop_int("Task ID"),
                "last_n":prop_int_opt("Return only the last N lines (default: all)")
            },"required":["task_id"]
        })),
        tool("stop_task", "Stop/cancel a running task.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "task_id":prop_int("Task ID to stop")
            },"required":["project_id","task_id"]
        })),
        tool("analyze_task_failure",
            "Retrieve output from a failed task and structure it for AI diagnosis. \
             Returns metadata + console output so you can identify root cause and suggest fixes.",
            json!({
                "type":"object",
                "properties":{
                    "project_id":prop_int("Project ID"),
                    "task_id":prop_int("Failed task ID"),
                    "last_n_lines":prop_int_opt("Lines of output to include (default 100)")
                },"required":["project_id","task_id"]
            }),
        ),

        // ── Schedules ────────────────────────────────────────────────────────
        tool("list_schedules", "List all cron schedules for a project.", json!({
            "type":"object","properties":{"project_id":prop_int("Project ID")},"required":["project_id"]
        })),
        tool("create_schedule", "Create a cron schedule. Example: '0 3 * * *' = daily 3AM.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "template_id":prop_int("Template to run"),
                "cron":prop_str("Cron expression"),
                "name":prop_str_opt("Schedule name")
            },"required":["project_id","template_id","cron"]
        })),
        tool("toggle_schedule", "Enable or disable a schedule.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "schedule_id":prop_int("Schedule ID"),
                "active":prop_bool("true to enable, false to disable")
            },"required":["project_id","schedule_id","active"]
        })),
        tool("delete_schedule", "Delete a cron schedule.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "schedule_id":prop_int("Schedule ID")
            },"required":["project_id","schedule_id"]
        })),

        // ── Repositories ─────────────────────────────────────────────────────
        tool("list_repositories", "List all Git repositories for a project.", json!({
            "type":"object","properties":{"project_id":prop_int("Project ID")},"required":["project_id"]
        })),
        tool("create_repository", "Add a Git repository to a project.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "name":prop_str("Repository name"),
                "git_url":prop_str("Git clone URL"),
                "git_branch":prop_str_opt("Branch (default: main)")
            },"required":["project_id","name","git_url"]
        })),
        tool("delete_repository", "Delete a repository from a project.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "repository_id":prop_int("Repository ID")
            },"required":["project_id","repository_id"]
        })),

        // ── Environments ─────────────────────────────────────────────────────
        tool("list_environments", "List all environment configs for a project.", json!({
            "type":"object","properties":{"project_id":prop_int("Project ID")},"required":["project_id"]
        })),
        tool("create_environment", "Create a new environment with extra-vars JSON.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "name":prop_str("Environment name"),
                "json":prop_str_opt("Extra-vars as JSON string, e.g. '{\"KEY\":\"VAL\"}'")
            },"required":["project_id","name"]
        })),
        tool("delete_environment", "Delete an environment.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "environment_id":prop_int("Environment ID")
            },"required":["project_id","environment_id"]
        })),

        // ── Inventory ────────────────────────────────────────────────────────
        tool("list_inventory", "List all inventory files for a project.", json!({
            "type":"object","properties":{"project_id":prop_int("Project ID")},"required":["project_id"]
        })),
        tool("get_inventory", "Get details of a specific inventory.", json!({
            "type":"object",
            "properties":{
                "project_id":prop_int("Project ID"),
                "inventory_id":prop_int("Inventory ID")
            },"required":["project_id","inventory_id"]
        })),

        // ── Access Keys ──────────────────────────────────────────────────────
        tool("list_access_keys", "List all access keys for a project (names and types only — no secrets).", json!({
            "type":"object","properties":{"project_id":prop_int("Project ID")},"required":["project_id"]
        })),

        // ── Runners ──────────────────────────────────────────────────────────
        tool("list_runners", "List all registered runner agents with status.", json!({
            "type":"object","properties":{},"required":[]
        })),
        tool("toggle_runner", "Enable or disable a runner.", json!({
            "type":"object",
            "properties":{
                "runner_id":prop_int("Runner ID"),
                "active":prop_bool("true to enable, false to disable")
            },"required":["runner_id","active"]
        })),

        // ── Analytics ────────────────────────────────────────────────────────
        tool("get_project_health",
            "Get health summary (healthy/degraded/critical) for a project based on recent task history.",
            json!({
                "type":"object",
                "properties":{
                    "project_id":prop_int("Project ID"),
                    "limit":prop_int_opt("Number of recent tasks to analyse (default 50)")
                },"required":["project_id"]
            }),
        ),
        tool("get_system_info", "Get Velum system information: version, runner count, task stats.", json!({
            "type":"object","properties":{},"required":[]
        })),

        // ── MCP Settings ─────────────────────────────────────────────────────
        tool("get_mcp_settings", "Get current MCP server settings and connection information.", json!({
            "type":"object","properties":{},"required":[]
        })),

        // ── AI Template Creator (FI-VEL-1) ───────────────────────────────────
        tool("ai_create_template",
            "Create a Velum task template from a natural language description. \
             Analyzes the description and suggests template type (ansible/terraform/bash), \
             playbook path, survey vars, extra_vars, vault keys, and inventory structure. \
             Returns a ready-to-use template JSON. Set dry_run=true to preview without saving.",
            json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID to create the template in"),
                    "description": prop_str("Natural language description. Example: 'Deploy Django app to 3 servers with nginx and PostgreSQL'"),
                    "dry_run": { "type": "boolean", "description": "Preview without saving (default: false)" }
                },
                "required": ["project_id", "description"]
            })
        ),

        // ── Deployment Environments (FI-GL-1) ────────────────────────────────
        tool("list_deploy_environments",
            "List deployment environments (production/staging/dev/review) with last deploy info.",
            json!({
                "type":"object","properties":{"project_id":prop_int("Project ID")},"required":["project_id"]
            })
        ),

        // ── Structured Outputs (FI-PUL-1) ────────────────────────────────────
        tool("get_template_last_outputs",
            "Get structured outputs (key-value map) from the last successful run of a template. \
             Use outputs of one template as extra_vars input for another (Stack References pattern).",
            json!({
                "type":"object",
                "properties":{
                    "project_id":prop_int("Project ID"),
                    "template_id":prop_int("Template ID")
                },
                "required":["project_id","template_id"]
            })
        ),
    ]
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({ "name": name, "description": description, "inputSchema": input_schema })
}

// ── Dispatcher ───────────────────────────────────────────────────────────────

pub async fn dispatch(name: &str, args: &Value, state: &Arc<AppState>) -> ToolResult {
    match dispatch_inner(name, args, state).await {
        Ok(r) => r,
        Err(e) => ToolResult::error(format!("Tool error: {e}")),
    }
}

async fn dispatch_inner(
    name: &str,
    args: &Value,
    state: &Arc<AppState>,
) -> anyhow::Result<ToolResult> {
    let store = state.store.store();

    let i32_arg = |key: &str| -> i32 { args.get(key).and_then(Value::as_i64).unwrap_or(0) as i32 };

    match name {
        // ── Projects ─────────────────────────────────────────────────────────
        "list_projects" => {
            let projects = store.get_projects(None).await?;
            Ok(ToolResult::ok(&json!(projects)))
        }
        "get_project" => {
            let pid = i32_arg("project_id");
            let project = store.get_project(pid).await?;
            Ok(ToolResult::ok(&json!(project)))
        }
        "create_project" => {
            let name = args["name"].as_str().unwrap_or("New Project").to_string();
            let alert = args.get("alert").and_then(Value::as_bool).unwrap_or(false);
            let project = Project {
                id: 0,
                name,
                created: chrono::Utc::now(),
                alert,
                alert_chat: None,
                max_parallel_tasks: 0,
                r#type: String::new(),
                default_secret_storage_id: None,
            };
            let created = store.create_project(project).await?;
            Ok(ToolResult::ok(&json!(created)))
        }
        "delete_project" => {
            let pid = i32_arg("project_id");
            store.delete_project(pid).await?;
            Ok(ToolResult::text(format!("Project {pid} deleted.")))
        }

        // ── Templates ────────────────────────────────────────────────────────
        "list_templates" => {
            let pid = i32_arg("project_id");
            let templates = store.get_templates(pid).await?;
            Ok(ToolResult::ok(&json!(templates)))
        }
        "get_template" => {
            let pid = i32_arg("project_id");
            let tid = i32_arg("template_id");
            let template = store.get_template(pid, tid).await?;
            Ok(ToolResult::ok(&json!(template)))
        }
        "run_template" => {
            let pid = i32_arg("project_id");
            let tid = i32_arg("template_id");
            let message = args
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_string);
            let task = Task {
                id: 0,
                template_id: tid,
                project_id: pid,
                status: TaskStatus::WaitingConfirmation,
                playbook: None,
                environment: None,
                secret: None,
                arguments: None,
                git_branch: None,
                user_id: None,
                integration_id: None,
                schedule_id: None,
                created: chrono::Utc::now(),
                start: None,
                end: None,
                message,
                commit_hash: None,
                commit_message: None,
                build_task_id: None,
                version: None,
                inventory_id: None,
                repository_id: None,
                environment_id: None,
                params: None,
            };
            let created = store.create_task(task).await?;
            Ok(ToolResult::ok(&json!(created)))
        }

        // ── Tasks ────────────────────────────────────────────────────────────
        "list_tasks" => {
            let pid = i32_arg("project_id");
            let tid = args
                .get("template_id")
                .and_then(Value::as_i64)
                .map(|v| v as i32);
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(50) as usize;
            let tasks = store.get_tasks(pid, tid).await?;
            let tasks: Vec<_> = tasks.into_iter().take(limit).collect();
            Ok(ToolResult::ok(&json!(tasks)))
        }
        "get_task" => {
            let pid = i32_arg("project_id");
            let tid = i32_arg("task_id");
            let task = store.get_task(pid, tid).await?;
            Ok(ToolResult::ok(&json!(task)))
        }
        "get_task_output" => {
            let tid = i32_arg("task_id");
            let last_n = args
                .get("last_n")
                .and_then(Value::as_u64)
                .map(|v| v as usize);
            let outputs = store.get_task_outputs(tid).await?;
            let outputs: Vec<_> = if let Some(n) = last_n {
                outputs
                    .into_iter()
                    .rev()
                    .take(n)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect()
            } else {
                outputs
            };
            let lines: Vec<&str> = outputs.iter().map(|o| o.output.as_str()).collect();
            Ok(ToolResult::ok(&json!({ "task_id": tid, "lines": lines })))
        }
        "stop_task" => {
            let pid = i32_arg("project_id");
            let tid = i32_arg("task_id");
            store
                .update_task_status(pid, tid, TaskStatus::Stopped)
                .await?;
            Ok(ToolResult::text(format!("Task {tid} stopped.")))
        }
        "analyze_task_failure" => {
            let pid = i32_arg("project_id");
            let tid = i32_arg("task_id");
            let n = args
                .get("last_n_lines")
                .and_then(Value::as_u64)
                .unwrap_or(100) as usize;
            let task = store.get_task(pid, tid).await?;
            let status_str = format!("{:?}", task.status).to_lowercase();
            if status_str != "error" && status_str != "stopped" {
                return Ok(ToolResult::error(format!(
                    "Task {tid} has status '{status_str}' — analysis is for failed/stopped tasks only."
                )));
            }
            let outputs = store.get_task_outputs(tid).await?;
            let lines: Vec<String> = outputs.into_iter().map(|o| o.output).collect();
            let total = lines.len();
            let tail: Vec<&str> = lines
                .iter()
                .rev()
                .take(n)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(String::as_str)
                .collect();
            let output_text = tail.join("\n");
            let displayed = tail.len();
            Ok(ToolResult::ok(&json!({
                "task_id": tid,
                "project_id": pid,
                "template_id": task.template_id,
                "status": status_str,
                "started": task.start,
                "finished": task.end,
                "total_output_lines": total,
                "output_tail": output_text,
                "analysis_guidance": format!(
                    "Task #{tid} failed (status: {status_str}).\n\n\
                     Last {displayed}/{total} lines of console output:\n```\n{output_text}\n```\n\n\
                     Please diagnose:\n\
                     1. Root cause of the failure\n\
                     2. Most likely reasons (ranked by probability)\n\
                     3. Specific remediation steps\n\
                     4. Preventive measures for the future"
                )
            })))
        }

        // ── Schedules ────────────────────────────────────────────────────────
        "list_schedules" => {
            let pid = i32_arg("project_id");
            let schedules = store.get_schedules(pid).await?;
            Ok(ToolResult::ok(&json!(schedules)))
        }
        "create_schedule" => {
            let pid = i32_arg("project_id");
            let tid = i32_arg("template_id");
            let cron_expr = args["cron"].as_str().unwrap_or("0 0 * * *").to_string();
            let name = args
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("MCP Schedule")
                .to_string();
            let schedule = Schedule {
                id: 0,
                project_id: pid,
                template_id: tid,
                cron: cron_expr,
                cron_format: None,
                name,
                active: true,
                last_commit_hash: None,
                repository_id: None,
                created: None,
                run_at: None,
                delete_after_run: false,
            };
            let created = store.create_schedule(schedule).await?;
            Ok(ToolResult::ok(&json!(created)))
        }
        "toggle_schedule" => {
            let pid = i32_arg("project_id");
            let sid = i32_arg("schedule_id");
            let active = args["active"].as_bool().unwrap_or(true);
            store.set_schedule_active(pid, sid, active).await?;
            let state_str = if active { "enabled" } else { "disabled" };
            Ok(ToolResult::text(format!("Schedule {sid} {state_str}.")))
        }
        "delete_schedule" => {
            let pid = i32_arg("project_id");
            let sid = i32_arg("schedule_id");
            store.delete_schedule(pid, sid).await?;
            Ok(ToolResult::text(format!("Schedule {sid} deleted.")))
        }

        // ── Repositories ─────────────────────────────────────────────────────
        "list_repositories" => {
            let pid = i32_arg("project_id");
            let repos = store.get_repositories(pid).await?;
            Ok(ToolResult::ok(&json!(repos)))
        }
        "create_repository" => {
            let pid = i32_arg("project_id");
            let branch = args
                .get("git_branch")
                .and_then(Value::as_str)
                .map(str::to_string);
            let repo = Repository {
                id: 0,
                project_id: pid,
                name: args["name"].as_str().unwrap_or("Repo").to_string(),
                git_url: args["git_url"].as_str().unwrap_or("").to_string(),
                git_type: RepositoryType::Git,
                git_branch: branch,
                key_id: None,
                git_path: None,
                created: None,
            };
            let created = store.create_repository(repo).await?;
            Ok(ToolResult::ok(&json!(created)))
        }
        "delete_repository" => {
            let pid = i32_arg("project_id");
            let rid = i32_arg("repository_id");
            store.delete_repository(pid, rid).await?;
            Ok(ToolResult::text(format!("Repository {rid} deleted.")))
        }

        // ── Environments ─────────────────────────────────────────────────────
        "list_environments" => {
            let pid = i32_arg("project_id");
            let envs = store.get_environments(pid).await?;
            Ok(ToolResult::ok(&json!(envs)))
        }
        "create_environment" => {
            let pid = i32_arg("project_id");
            let env_json = args
                .get("json")
                .and_then(Value::as_str)
                .unwrap_or("{}")
                .to_string();
            let env = Environment {
                id: 0,
                project_id: pid,
                name: args["name"].as_str().unwrap_or("Environment").to_string(),
                json: env_json,
                secret_storage_id: None,
                secret_storage_key_prefix: None,
                secrets: None,
                created: None,
            };
            let created = store.create_environment(env).await?;
            Ok(ToolResult::ok(&json!(created)))
        }
        "delete_environment" => {
            let pid = i32_arg("project_id");
            let eid = i32_arg("environment_id");
            store.delete_environment(pid, eid).await?;
            Ok(ToolResult::text(format!("Environment {eid} deleted.")))
        }

        // ── Inventory ────────────────────────────────────────────────────────
        "list_inventory" => {
            let pid = i32_arg("project_id");
            let inv = store.get_inventories(pid).await?;
            Ok(ToolResult::ok(&json!(inv)))
        }
        "get_inventory" => {
            let pid = i32_arg("project_id");
            let iid = i32_arg("inventory_id");
            let inv = store.get_inventory(pid, iid).await?;
            Ok(ToolResult::ok(&json!(inv)))
        }

        // ── Access Keys ──────────────────────────────────────────────────────
        "list_access_keys" => {
            let pid = i32_arg("project_id");
            let keys = store.get_access_keys(pid).await?;
            // Sanitize: never return secret credential values
            let safe: Vec<Value> = keys
                .iter()
                .map(|k| {
                    json!({
                        "id": k.id,
                        "project_id": k.project_id,
                        "name": k.name,
                        "type": format!("{:?}", k.r#type)
                    })
                })
                .collect();
            Ok(ToolResult::ok(&json!(safe)))
        }

        // ── Runners ──────────────────────────────────────────────────────────
        "list_runners" => {
            let runners = store.get_runners(None).await?;
            Ok(ToolResult::ok(&json!(runners)))
        }
        "toggle_runner" => {
            let rid = i32_arg("runner_id");
            let active = args["active"].as_bool().unwrap_or(true);
            let mut runner = store.get_runner(rid).await?;
            runner.active = active;
            store.update_runner(runner).await?;
            let state_str = if active { "enabled" } else { "disabled" };
            Ok(ToolResult::text(format!("Runner {rid} {state_str}.")))
        }

        // ── Analytics ────────────────────────────────────────────────────────
        "get_project_health" => {
            let pid = i32_arg("project_id");
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(50) as usize;
            let tasks = store.get_tasks(pid, None).await?;
            let recent: Vec<_> = tasks.iter().take(limit).collect();
            let total = recent.len() as f64;
            let success = recent
                .iter()
                .filter(|t| format!("{:?}", t.task.status).to_lowercase() == "success")
                .count() as f64;
            let failure = recent
                .iter()
                .filter(|t| format!("{:?}", t.task.status).to_lowercase() == "error")
                .count() as f64;
            let rate = if total > 0.0 {
                (success / total * 1000.0).round() / 10.0
            } else {
                0.0
            };
            let health = if rate >= 95.0 {
                "healthy"
            } else if rate >= 80.0 {
                "degraded"
            } else {
                "critical"
            };
            let emoji = if rate >= 95.0 {
                "✅"
            } else if rate >= 80.0 {
                "⚠️"
            } else {
                "🔴"
            };
            Ok(ToolResult::ok(&json!({
                "project_id": pid,
                "health_status": health,
                "health_emoji": emoji,
                "total_analysed": total as i64,
                "success_count": success as i64,
                "failure_count": failure as i64,
                "success_rate_pct": rate,
                "recommendation": match health {
                    "healthy" => "Project is running well.",
                    "degraded" => "Success rate below 95%. Review recent failures.",
                    _ => "Critical failure rate. Immediate investigation recommended."
                }
            })))
        }
        "get_system_info" => {
            let runner_count = store.get_runners(None).await.map(|r| r.len()).unwrap_or(0);
            let user_count = store.get_user_count().await.unwrap_or(0);
            let project_count = store.get_projects(None).await.map(|p| p.len()).unwrap_or(0);
            let tasks = store
                .get_global_tasks(None, Some(1000))
                .await
                .unwrap_or_default();
            let running = tasks
                .iter()
                .filter(|t| format!("{:?}", t.task.status).to_lowercase() == "running")
                .count();
            Ok(ToolResult::ok(&json!({
                "version": env!("CARGO_PKG_VERSION"),
                "runner_count": runner_count,
                "user_count": user_count,
                "project_count": project_count,
                "running_tasks": running
            })))
        }

        // ── MCP Settings ─────────────────────────────────────────────────────
        "get_mcp_settings" => {
            let enabled = store
                .get_option("mcp.enabled")
                .await
                .unwrap_or(Some("true".into()))
                .unwrap_or_else(|| "true".into());
            Ok(ToolResult::ok(&json!({
                "enabled": enabled == "true",
                "endpoint": "/mcp",
                "transport": "http",
                "tool_count": all_definitions().len(),
                "note": "Connect Claude to this endpoint using Authorization: Bearer <jwt-token>"
            })))
        }

        // ── AI Template Creator (FI-VEL-1) ───────────────────────────────────
        "ai_create_template" => {
            let project_id = i32_arg("project_id");
            let description = args["description"].as_str().unwrap_or("").to_string();
            let dry_run = args
                .get("dry_run")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let template_suggestion = generate_template_suggestion(&description);

            if dry_run {
                Ok(ToolResult::ok(&json!({
                    "dry_run": true,
                    "project_id": project_id,
                    "description": description,
                    "suggested_template": template_suggestion
                })))
            } else {
                let url = format!("/api/project/{}/templates", project_id);
                Ok(ToolResult::ok(&json!({
                    "message": "Use the suggested_template JSON to POST to the templates API",
                    "endpoint": url,
                    "suggested_template": template_suggestion
                })))
            }
        }

        // ── Deployment Environments (FI-GL-1) ────────────────────────────────
        "list_deploy_environments" => {
            let project_id = i32_arg("project_id");
            let envs = store
                .get_deployment_environments(project_id)
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            Ok(ToolResult::ok(&json!(envs)))
        }

        // ── Structured Outputs (FI-PUL-1) ────────────────────────────────────
        "get_template_last_outputs" => {
            let project_id = i32_arg("project_id");
            let template_id = i32_arg("template_id");
            let map = store
                .get_template_last_outputs(template_id, project_id)
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            Ok(ToolResult::ok(&json!(map)))
        }

        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}

/// Генерирует предложение шаблона на основе описания (rule-based, без LLM)
fn generate_template_suggestion(description: &str) -> serde_json::Value {
    let desc_lower = description.to_lowercase();

    // Определяем тип приложения
    let (app, playbook_hint) = if desc_lower.contains("terraform")
        || desc_lower.contains("infra")
        || desc_lower.contains("cloud")
    {
        ("terraform", "main.tf directory")
    } else if desc_lower.contains("bash")
        || desc_lower.contains("script")
        || desc_lower.contains("shell")
    {
        ("bash", "deploy.sh")
    } else {
        ("ansible", "deploy.yml")
    };

    // Определяем тип (deploy/build/test)
    let template_type = if desc_lower.contains("test")
        || desc_lower.contains("check")
        || desc_lower.contains("verify")
    {
        "build"
    } else {
        "deploy"
    };

    // Предлагаем survey vars
    let mut survey_vars = vec![];
    if desc_lower.contains("version")
        || desc_lower.contains("tag")
        || desc_lower.contains("release")
    {
        survey_vars.push(json!({"name": "app_version", "title": "Application Version", "type": "string", "required": true}));
    }
    if desc_lower.contains("env")
        || desc_lower.contains("environment")
        || desc_lower.contains("prod")
        || desc_lower.contains("staging")
    {
        survey_vars.push(json!({"name": "target_env", "title": "Target Environment", "type": "enum", "enum_values": ["production", "staging", "dev"], "required": true}));
    }
    if desc_lower.contains("password")
        || desc_lower.contains("secret")
        || desc_lower.contains("token")
    {
        survey_vars.push(json!({"name": "app_secret", "title": "Application Secret", "type": "secret", "required": true}));
    }

    // Предлагаем extra_vars
    let mut extra_vars = serde_json::Map::new();
    if desc_lower.contains("become") || desc_lower.contains("sudo") || desc_lower.contains("root") {
        extra_vars.insert("ansible_become".into(), json!(true));
    }
    if desc_lower.contains("nginx") {
        extra_vars.insert("web_server".into(), json!("nginx"));
    }
    if desc_lower.contains("postgres") || desc_lower.contains("postgresql") {
        extra_vars.insert("db_engine".into(), json!("postgresql"));
    }

    json!({
        "name": format!("Auto: {}", &description[..description.len().min(50)]),
        "description": description,
        "type": template_type,
        "app": app,
        "playbook": playbook_hint,
        "survey_vars": survey_vars,
        "extra_vars_hint": extra_vars,
        "notes": "Review and adjust before saving. Set inventory_id and repository_id from your project resources."
    })
}
