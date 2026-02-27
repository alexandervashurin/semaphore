//! Модель события

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Тип события
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    TaskCreated,
    TaskUpdated,
    TaskDeleted,
    TemplateCreated,
    TemplateUpdated,
    TemplateDeleted,
    InventoryCreated,
    InventoryUpdated,
    InventoryDeleted,
    RepositoryCreated,
    RepositoryUpdated,
    RepositoryDeleted,
    EnvironmentCreated,
    EnvironmentUpdated,
    EnvironmentDeleted,
    AccessKeyCreated,
    AccessKeyUpdated,
    AccessKeyDeleted,
    IntegrationCreated,
    IntegrationUpdated,
    IntegrationDeleted,
    ScheduleCreated,
    ScheduleUpdated,
    ScheduleDeleted,
    UserJoined,
    UserLeft,
    UserUpdated,
    ProjectUpdated,
    Other,
}

/// Событие системы
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: i32,
    pub project_id: Option<i32>,
    pub user_id: Option<i32>,
    pub object_id: Option<i32>,
    pub object_type: String,
    pub description: String,
    pub created: DateTime<Utc>,
}
