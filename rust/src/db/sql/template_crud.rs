//! Template CRUD - операции с шаблонами
//!
//! Аналог db/sql/template.go из Go версии (часть 1: CRUD)
//!
//! DEPRECATED: Используйте модули sqlite::template, postgres::template, mysql::template

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает все шаблоны проекта
    pub async fn get_templates(&self, project_id: i32) -> Result<Vec<Template>> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::template::get_templates(pool, project_id).await
    }

    /// Получает шаблон по ID
    pub async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::template::get_template(pool, project_id, template_id).await
    }

    /// Создаёт новый шаблон
    pub async fn create_template(&self, mut template: Template) -> Result<Template> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::template::create_template(pool, template).await
    }

    /// Обновляет шаблон
    pub async fn update_template(&self, template: Template) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::template::update_template(pool, template).await
    }

    /// Удаляет шаблон
    pub async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::template::delete_template(pool, project_id, template_id).await
    }
}

// Legacy code removed — now uses decomposed modules (postgres::template).
