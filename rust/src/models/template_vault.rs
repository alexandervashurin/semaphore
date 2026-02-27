//! Модель TemplateVault

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// TemplateVault - хранилище секретов для шаблона
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateVault {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID шаблона
    pub template_id: i32,

    /// ID проекта
    pub project_id: i32,

    /// ID хранилища секретов
    pub vault_id: i32,

    /// ID ключа доступа к хранилищу
    pub vault_key_id: i32,

    /// Название хранилища
    pub name: String,
}
