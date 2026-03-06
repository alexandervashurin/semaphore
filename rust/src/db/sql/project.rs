//! Project CRUD Operations
//!
//! Адаптер для декомпозированных модулей
//!
//! Новые модули: sqlite::project, postgres::project, mysql::project

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Project;

impl SqlDb {
    /// Получает все проекты
    pub async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::project::get_projects(pool, user_id).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::project::get_projects(pool, user_id).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::project::get_projects(pool, user_id).await
            }
        }
    }

    /// Получает проект по ID
    pub async fn get_project(&self, project_id: i32) -> Result<Project> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::project::get_project(pool, project_id).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::project::get_project(pool, project_id).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::project::get_project(pool, project_id).await
            }
        }
    }

    /// Создаёт проект
    pub async fn create_project(&self, project: Project) -> Result<Project> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::project::create_project(pool, project).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::project::create_project(pool, project).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::project::create_project(pool, project).await
            }
        }
    }

    /// Обновляет проект
    pub async fn update_project(&self, project: Project) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::project::update_project(pool, project).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::project::update_project(pool, project).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::project::update_project(pool, project).await
            }
        }
    }

    /// Удаляет проект
    pub async fn delete_project(&self, project_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::project::delete_project(pool, project_id).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::project::delete_project(pool, project_id).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::project::delete_project(pool, project_id).await
            }
        }
    }
}
