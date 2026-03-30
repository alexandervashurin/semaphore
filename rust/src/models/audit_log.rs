//! Модель Audit Log для отслеживания действий пользователей
//!
//! Расширенное логирование всех значимых действий в системе

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Тип действия в audit log
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Аутентификация
    Login,
    Logout,
    LoginFailed,
    PasswordChanged,
    PasswordResetRequested,
    TwoFactorEnabled,
    TwoFactorDisabled,

    // Пользователи
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserJoinedProject,
    UserLeftProject,
    UserRoleChanged,

    // Проекты
    ProjectCreated,
    ProjectUpdated,
    ProjectDeleted,

    // Задачи (Tasks)
    TaskCreated,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskStopped,
    TaskDeleted,

    // Шаблоны (Templates)
    TemplateCreated,
    TemplateUpdated,
    TemplateDeleted,
    TemplateRun,

    // Инвентарь
    InventoryCreated,
    InventoryUpdated,
    InventoryDeleted,

    // Репозитории
    RepositoryCreated,
    RepositoryUpdated,
    RepositoryDeleted,

    // Окружения (Environments)
    EnvironmentCreated,
    EnvironmentUpdated,
    EnvironmentDeleted,

    // Ключи доступа (Access Keys)
    AccessKeyCreated,
    AccessKeyUpdated,
    AccessKeyDeleted,

    // Интеграции
    IntegrationCreated,
    IntegrationUpdated,
    IntegrationDeleted,
    WebhookTriggered,

    // Расписания (Schedules)
    ScheduleCreated,
    ScheduleUpdated,
    ScheduleDeleted,
    ScheduleTriggered,

    // Раннеры
    RunnerCreated,
    RunnerUpdated,
    RunnerDeleted,
    RunnerConnected,
    RunnerDisconnected,

    // Системные
    ConfigChanged,
    BackupCreated,
    RestorePerformed,
    MigrationApplied,

    // Kubernetes операции
    KubernetesResourceCreated,
    KubernetesResourceUpdated,
    KubernetesResourceDeleted,
    KubernetesResourceScaled,
    KubernetesHelmReleaseInstalled,
    KubernetesHelmReleaseUpgraded,
    KubernetesHelmReleaseRolledBack,
    KubernetesHelmReleaseUninstalled,

    // Другое
    Other,
}

/// Объект аудита (тип сущности)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditObjectType {
    User,
    Project,
    Task,
    Template,
    Inventory,
    Repository,
    Environment,
    AccessKey,
    Integration,
    Schedule,
    Runner,
    View,
    Secret,
    System,
    Kubernetes,
    Other,
}

/// Уровень важности записи audit log
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum AuditLevel {
    Info,     // Информационное
    Warning,  // Предупреждение
    Error,    // Ошибка
    Critical, // Критическое
}

/// Детали действия audit log
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Запись audit log
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    /// Уникальный ID записи
    pub id: i64,

    /// ID проекта (если применимо)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,

    /// ID пользователя, выполнившего действие
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,

    /// Имя пользователя (денормализация для быстрого поиска)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Тип действия
    pub action: AuditAction,

    /// Тип объекта, над которым выполнено действие
    pub object_type: AuditObjectType,

    /// ID объекта (если применимо)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_id: Option<i64>,

    /// Название объекта (денормализация)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_name: Option<String>,

    /// Описание действия
    pub description: String,

    /// Уровень важности
    pub level: AuditLevel,

    /// IP адрес
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// User agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Дополнительные данные в JSON формате
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,

    /// Время создания записи
    pub created: DateTime<Utc>,
}

/// Параметры поиска для audit log
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditLogFilter {
    /// Фильтр по project_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,

    /// Фильтр по user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,

    /// Фильтр по username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Фильтр по действию
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<AuditAction>,

    /// Фильтр по типу объекта
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<AuditObjectType>,

    /// Фильтр по object_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_id: Option<i64>,

    /// Фильтр по уровню важности
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<AuditLevel>,

    /// Поиск по описанию (LIKE)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    /// Дата начала периода
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from: Option<DateTime<Utc>>,

    /// Дата окончания периода
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to: Option<DateTime<Utc>>,

    /// Лимит записей
    #[serde(default = "default_limit")]
    pub limit: i64,

    /// Смещение
    #[serde(default)]
    pub offset: i64,

    /// Сортировка по полю
    #[serde(default = "default_sort")]
    pub sort: String,

    /// Порядок сортировки (asc/desc)
    #[serde(default = "default_order")]
    pub order: String,
}

fn default_limit() -> i64 {
    50
}
fn default_sort() -> String {
    "created".to_string()
}
fn default_order() -> String {
    "desc".to_string()
}

/// Результат поиска с пагинацией
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogResult {
    pub total: i64,
    pub records: Vec<AuditLog>,
    pub limit: i64,
    pub offset: i64,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::Login => write!(f, "login"),
            AuditAction::Logout => write!(f, "logout"),
            AuditAction::LoginFailed => write!(f, "login_failed"),
            AuditAction::PasswordChanged => write!(f, "password_changed"),
            AuditAction::PasswordResetRequested => write!(f, "password_reset_requested"),
            AuditAction::TwoFactorEnabled => write!(f, "two_factor_enabled"),
            AuditAction::TwoFactorDisabled => write!(f, "two_factor_disabled"),
            AuditAction::UserCreated => write!(f, "user_created"),
            AuditAction::UserUpdated => write!(f, "user_updated"),
            AuditAction::UserDeleted => write!(f, "user_deleted"),
            AuditAction::UserJoinedProject => write!(f, "user_joined_project"),
            AuditAction::UserLeftProject => write!(f, "user_left_project"),
            AuditAction::UserRoleChanged => write!(f, "user_role_changed"),
            AuditAction::ProjectCreated => write!(f, "project_created"),
            AuditAction::ProjectUpdated => write!(f, "project_updated"),
            AuditAction::ProjectDeleted => write!(f, "project_deleted"),
            AuditAction::TaskCreated => write!(f, "task_created"),
            AuditAction::TaskStarted => write!(f, "task_started"),
            AuditAction::TaskCompleted => write!(f, "task_completed"),
            AuditAction::TaskFailed => write!(f, "task_failed"),
            AuditAction::TaskStopped => write!(f, "task_stopped"),
            AuditAction::TaskDeleted => write!(f, "task_deleted"),
            AuditAction::TemplateCreated => write!(f, "template_created"),
            AuditAction::TemplateUpdated => write!(f, "template_updated"),
            AuditAction::TemplateDeleted => write!(f, "template_deleted"),
            AuditAction::TemplateRun => write!(f, "template_run"),
            AuditAction::InventoryCreated => write!(f, "inventory_created"),
            AuditAction::InventoryUpdated => write!(f, "inventory_updated"),
            AuditAction::InventoryDeleted => write!(f, "inventory_deleted"),
            AuditAction::RepositoryCreated => write!(f, "repository_created"),
            AuditAction::RepositoryUpdated => write!(f, "repository_updated"),
            AuditAction::RepositoryDeleted => write!(f, "repository_deleted"),
            AuditAction::EnvironmentCreated => write!(f, "environment_created"),
            AuditAction::EnvironmentUpdated => write!(f, "environment_updated"),
            AuditAction::EnvironmentDeleted => write!(f, "environment_deleted"),
            AuditAction::AccessKeyCreated => write!(f, "access_key_created"),
            AuditAction::AccessKeyUpdated => write!(f, "access_key_updated"),
            AuditAction::AccessKeyDeleted => write!(f, "access_key_deleted"),
            AuditAction::IntegrationCreated => write!(f, "integration_created"),
            AuditAction::IntegrationUpdated => write!(f, "integration_updated"),
            AuditAction::IntegrationDeleted => write!(f, "integration_deleted"),
            AuditAction::WebhookTriggered => write!(f, "webhook_triggered"),
            AuditAction::ScheduleCreated => write!(f, "schedule_created"),
            AuditAction::ScheduleUpdated => write!(f, "schedule_updated"),
            AuditAction::ScheduleDeleted => write!(f, "schedule_deleted"),
            AuditAction::ScheduleTriggered => write!(f, "schedule_triggered"),
            AuditAction::RunnerCreated => write!(f, "runner_created"),
            AuditAction::RunnerUpdated => write!(f, "runner_updated"),
            AuditAction::RunnerDeleted => write!(f, "runner_deleted"),
            AuditAction::RunnerConnected => write!(f, "runner_connected"),
            AuditAction::RunnerDisconnected => write!(f, "runner_disconnected"),
            AuditAction::ConfigChanged => write!(f, "config_changed"),
            AuditAction::BackupCreated => write!(f, "backup_created"),
            AuditAction::RestorePerformed => write!(f, "restore_performed"),
            AuditAction::MigrationApplied => write!(f, "migration_applied"),
            AuditAction::KubernetesResourceCreated => write!(f, "kubernetes_resource_created"),
            AuditAction::KubernetesResourceUpdated => write!(f, "kubernetes_resource_updated"),
            AuditAction::KubernetesResourceDeleted => write!(f, "kubernetes_resource_deleted"),
            AuditAction::KubernetesResourceScaled => write!(f, "kubernetes_resource_scaled"),
            AuditAction::KubernetesHelmReleaseInstalled => write!(f, "kubernetes_helm_release_installed"),
            AuditAction::KubernetesHelmReleaseUpgraded => write!(f, "kubernetes_helm_release_upgraded"),
            AuditAction::KubernetesHelmReleaseRolledBack => write!(f, "kubernetes_helm_release_rolled_back"),
            AuditAction::KubernetesHelmReleaseUninstalled => write!(f, "kubernetes_helm_release_uninstalled"),
            AuditAction::Other => write!(f, "other"),
        }
    }
}

impl std::fmt::Display for AuditObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditObjectType::User => write!(f, "user"),
            AuditObjectType::Project => write!(f, "project"),
            AuditObjectType::Task => write!(f, "task"),
            AuditObjectType::Template => write!(f, "template"),
            AuditObjectType::Inventory => write!(f, "inventory"),
            AuditObjectType::Repository => write!(f, "repository"),
            AuditObjectType::Environment => write!(f, "environment"),
            AuditObjectType::AccessKey => write!(f, "access_key"),
            AuditObjectType::Integration => write!(f, "integration"),
            AuditObjectType::Schedule => write!(f, "schedule"),
            AuditObjectType::Runner => write!(f, "runner"),
            AuditObjectType::View => write!(f, "view"),
            AuditObjectType::Secret => write!(f, "secret"),
            AuditObjectType::System => write!(f, "system"),
            AuditObjectType::Kubernetes => write!(f, "kubernetes"),
            AuditObjectType::Other => write!(f, "other"),
        }
    }
}

impl std::fmt::Display for AuditLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditLevel::Info => write!(f, "info"),
            AuditLevel::Warning => write!(f, "warning"),
            AuditLevel::Error => write!(f, "error"),
            AuditLevel::Critical => write!(f, "critical"),
        }
    }
}
