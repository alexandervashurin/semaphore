//! Template Roles - операции с TemplateRole
//!
//! Аналог db/sql/template.go из Go версии (часть 3: TemplateRole)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_template_roles(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает все роли для шаблона
    pub async fn get_template_roles(
        &self,
        project_id: i32,
        template_id: i32,
    ) -> Result<Vec<TemplateRolePerm>> {
        let rows =
            sqlx::query("SELECT * FROM template_role WHERE template_id = $1 AND project_id = $2")
                .bind(template_id)
                .bind(project_id)
                .fetch_all(self.pg_pool_template_roles()?)
                .await
                .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| TemplateRolePerm {
                id: row.get("id"),
                project_id: row.get("project_id"),
                template_id: row.get("template_id"),
                role_id: row.get("role_id"),
                role_slug: row.try_get("role_slug").ok().unwrap_or_default(),
            })
            .collect())
    }

    /// Создаёт TemplateRole
    pub async fn create_template_role(
        &self,
        mut role: TemplateRolePerm,
    ) -> Result<TemplateRolePerm> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO template_role (template_id, project_id, role_id, role_slug) \
             VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(role.template_id)
        .bind(role.project_id)
        .bind(role.role_id)
        .bind(&role.role_slug)
        .fetch_one(self.pg_pool_template_roles()?)
        .await
        .map_err(Error::Database)?;

        role.id = id;
        Ok(role)
    }

    /// Обновляет TemplateRole
    pub async fn update_template_role(&self, role: TemplateRolePerm) -> Result<()> {
        sqlx::query(
            "UPDATE template_role SET role_id = $1, role_slug = $2 \
             WHERE id = $3 AND template_id = $4 AND project_id = $5",
        )
        .bind(role.role_id)
        .bind(&role.role_slug)
        .bind(role.id)
        .bind(role.template_id)
        .bind(role.project_id)
        .execute(self.pg_pool_template_roles()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет TemplateRole
    pub async fn delete_template_role(
        &self,
        project_id: i32,
        template_id: i32,
        role_id: i32,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM template_role WHERE id = $1 AND template_id = $2 AND project_id = $3",
        )
        .bind(role_id)
        .bind(template_id)
        .bind(project_id)
        .execute(self.pg_pool_template_roles()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }
}
