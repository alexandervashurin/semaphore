//! Analytics API Handlers
//!
//! Обработчики для аналитики и дашбордов

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::analytics::*;
use crate::db::store::{TaskManager, TemplateManager, InventoryManager, RepositoryManager, EnvironmentManager, AccessKeyManager, ScheduleManager, UserManager, ProjectStore};
use crate::services::task_logger::TaskStatus;
use serde::Deserialize;

/// Параметры запроса аналитики проекта
#[derive(Debug, Deserialize)]
pub struct AnalyticsParams {
    #[serde(default)]
    pub period: Option<String>, // day, week, month, year
}

/// GET /api/project/{project_id}/analytics - Аналитика проекта
pub async fn get_project_analytics(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i64>,
    Query(params): Query<AnalyticsParams>,
) -> Result<Json<ProjectAnalytics>, StatusCode> {
    // Получаем базовую статистику из tasks
    let tasks = state.store
        .get_tasks(project_id as i32, None::<i32>)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get tasks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Считаем статистику
    let total_tasks = tasks.len() as i64;
    let successful_tasks = tasks.iter().filter(|t| t.task.status == TaskStatus::Success).count() as i64;
    let failed_tasks = tasks.iter().filter(|t| t.task.status == TaskStatus::Error).count() as i64;
    let stopped_tasks = tasks.iter().filter(|t| t.task.status == TaskStatus::Stopped).count() as i64;
    let pending_tasks = tasks.iter().filter(|t| t.task.status == TaskStatus::Waiting || t.task.status == TaskStatus::Starting).count() as i64;
    let running_tasks = tasks.iter().filter(|t| t.task.status == TaskStatus::Running).count() as i64;
    
    let success_rate = if total_tasks > 0 {
        (successful_tasks as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };

    // Вычисляем avg/max/min длительности задач (start..end)
    let durations_secs: Vec<f64> = tasks.iter()
        .filter_map(|t| match (t.task.start, t.task.end) {
            (Some(s), Some(e)) => {
                let d = (e - s).num_seconds();
                if d >= 0 { Some(d as f64) } else { None }
            }
            _ => None,
        })
        .collect();
    let avg_task_duration_secs = if durations_secs.is_empty() {
        0.0
    } else {
        durations_secs.iter().sum::<f64>() / durations_secs.len() as f64
    };
    let max_task_duration_secs = durations_secs.iter().cloned().fold(0.0f64, f64::max);
    let min_task_duration_secs = if durations_secs.is_empty() {
        0.0
    } else {
        durations_secs.iter().cloned().fold(f64::MAX, f64::min)
    };
    let total_duration_secs = durations_secs.iter().sum::<f64>() as i64;
    
    // Получаем шаблоны для статистики
    let templates = state.store
        .get_templates(project_id as i32)
        .await
        .unwrap_or_default();
    
    // Получаем инвентари
    let inventories = state.store
        .get_inventories(project_id as i32)
        .await
        .unwrap_or_default();
    
    // Получаем репозитории
    let repositories = state.store
        .get_repositories(project_id as i32)
        .await
        .unwrap_or_default();
    
    // Получаем окружения
    let environments = state.store
        .get_environments(project_id as i32)
        .await
        .unwrap_or_default();
    
    // Получаем ключи
    let keys = state.store
        .get_access_keys(project_id as i32)
        .await
        .unwrap_or_default();
    
    // Получаем расписания
    let schedules = state.store
        .get_schedules(project_id as i32)
        .await
        .unwrap_or_default();
    
    // Получаем пользователей проекта
    let users = state.store
        .get_users(Default::default())
        .await
        .unwrap_or_default();
    
    // Получаем проект для имени
    let project = state.store
        .get_project(project_id as i32)
        .await
        .ok();
    
    // Создаём статистику
    let stats = ProjectStats {
        project_id,
        project_name: project.map(|p| p.name).unwrap_or_else(|| format!("Project {}", project_id)),
        total_tasks,
        successful_tasks,
        failed_tasks,
        stopped_tasks,
        pending_tasks,
        running_tasks,
        total_templates: templates.len() as i64,
        total_users: users.len() as i64,
        total_inventories: inventories.len() as i64,
        total_repositories: repositories.len() as i64,
        total_environments: environments.len() as i64,
        total_keys: keys.len() as i64,
        total_schedules: schedules.len() as i64,
        success_rate,
        avg_task_duration_secs,
    };
    
    // Определяем период
    let period = params.period.as_deref().unwrap_or("week");
    
    // Создаём простую статистику задач
    let task_stats = TaskStats {
        period: period.to_string(),
        total: total_tasks,
        success: successful_tasks,
        failed: failed_tasks,
        stopped: stopped_tasks,
        avg_duration_secs: avg_task_duration_secs,
        max_duration_secs: max_task_duration_secs,
        min_duration_secs: min_task_duration_secs,
        total_duration_secs,
    };
    
    // Вычисляем tasks_per_day (за последние 30 дней)
    let thirty_days_ago = Utc::now() - Duration::days(30);
    let recent_tasks = tasks.iter()
        .filter(|t| t.task.created > thirty_days_ago)
        .count() as f64;
    let tasks_per_day = recent_tasks / 30.0;
    let tasks_per_hour = tasks_per_day / 24.0;

    let performance = PerformanceMetrics {
        avg_queue_time_secs: 0.0,
        avg_execution_time_secs: avg_task_duration_secs,
        tasks_per_hour,
        tasks_per_day,
        concurrent_tasks_avg: 0.0,
        concurrent_tasks_max: 0,
        resource_usage: ResourceUsage::default(),
    };
    
    // Топ пользователей (заглушка)
    let top_users = vec![];
    
    // Топ шаблонов
    let top_templates = templates.iter().take(5).map(|t| TopItem {
        id: t.id as i64,
        name: t.name.clone(),
        value: 0,
        r#type: "template".to_string(),
    }).collect();
    
    // Недавняя активность (заглушка)
    let recent_activity = vec![];
    
    Ok(Json(ProjectAnalytics {
        stats,
        task_stats,
        performance,
        top_users,
        top_templates,
        recent_activity,
    }))
}

/// GET /api/project/{project_id}/analytics/tasks-chart - Данные для графика задач
pub async fn get_tasks_chart(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i64>,
    Query(params): Query<AnalyticsParams>,
) -> Result<Json<Vec<ChartData>>, StatusCode> {
    let tasks = state.store
        .get_tasks(project_id as i32, None::<i32>)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get tasks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Определяем диапазон дней по периоду
    let days: i64 = match params.period.as_deref().unwrap_or("week") {
        "month" => 30,
        "year"  => 365,
        _       => 7,  // week (default)
    };

    let now = Utc::now();
    let start = now - Duration::days(days);

    // Строим карту дата → счётчик
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<String, f64> = BTreeMap::new();

    // Заполняем все дни нулями
    for d in 0..days {
        let day = (start + Duration::days(d + 1)).format("%Y-%m-%d").to_string();
        counts.insert(day, 0.0);
    }

    // Считаем задачи по датам
    for t in &tasks {
        if t.task.created >= start {
            let day = t.task.created.format("%Y-%m-%d").to_string();
            *counts.entry(day).or_insert(0.0) += 1.0;
        }
    }

    let chart_data: Vec<ChartData> = counts.into_iter().map(|(label, value)| ChartData {
        label,
        value,
        timestamp: None,
    }).collect();

    Ok(Json(chart_data))
}

/// GET /api/project/{project_id}/analytics/status-distribution - Распределение по статусам
pub async fn get_status_distribution(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<ChartData>>, StatusCode> {
    let tasks = state.store
        .get_tasks(project_id as i32, None::<i32>)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get tasks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Считаем распределение
    let mut distribution = Vec::new();
    for status in &[TaskStatus::Success, TaskStatus::Error, TaskStatus::Stopped, TaskStatus::Waiting, TaskStatus::Running] {
        let count = tasks.iter().filter(|t| &t.task.status == status).count();
        if count > 0 {
            distribution.push(ChartData {
                label: format!("{:?}", status),
                value: count as f64,
                timestamp: None,
            });
        }
    }
    
    Ok(Json(distribution))
}

/// GET /api/analytics/system - Системная аналитика (для админов)
pub async fn get_system_analytics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SystemMetrics>, StatusCode> {
    // Получаем все проекты
    let projects = state.store
        .get_projects(None)
        .await
        .unwrap_or_default();
    
    // Получаем всех пользователей
    let users = state.store
        .get_users(Default::default())
        .await
        .unwrap_or_default();
    
    // Получаем все шаблоны
    let mut total_templates = 0;
    for project in &projects {
        let templates = state.store
            .get_templates(project.id)
            .await
            .unwrap_or_default();
        total_templates += templates.len();
    }
    
    // Заглушка для остальных метрик
    Ok(Json(SystemMetrics {
        total_projects: projects.len() as i64,
        total_users: users.len() as i64,
        total_tasks: 0,
        total_templates: total_templates as i64,
        total_runners: 0,
        active_runners: 0,
        running_tasks: 0,
        queued_tasks: 0,
        success_rate_24h: 0.0,
        avg_task_duration_24h: 0.0,
        tasks_24h: 0,
        tasks_7d: 0,
        tasks_30d: 0,
    }))
}
