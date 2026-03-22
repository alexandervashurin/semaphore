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
    pub async fn get_template_roles(&self, project_id: i32, template_id: i32) -> Result<Vec<TemplateRolePerm>> {
        let rows = sqlx::query(
            "SELECT * FROM template_role WHERE template_id = $1 AND project_id = $2"
        )
        .bind(template_id)
        .bind(project_id)
        .fetch_all(self.pg_pool_template_roles()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| TemplateRolePerm {
            id: row.get("id"),
            project_id: row.get("project_id"),
            template_id: row.get("template_id"),
            role_id: row.get("role_id"),
            role_slug: row.try_get("role_slug").ok().unwrap_or_default(),
        }).collect())
    }

    /// Создаёт TemplateRole
    pub async fn create_template_role(&self, mut role: TemplateRolePerm) -> Result<TemplateRolePerm> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO template_role (template_id, project_id, role_id, role_slug) \
             VALUES ($1, $2, $3, $4) RETURNING id"
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
             WHERE id = $3 AND template_id = $4 AND project_id = $5"
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
    pub async fn delete_template_role(&self, project_id: i32, template_id: i32, role_id: i32) -> Result<()> {
        sqlx::query(
            "DELETE FROM template_role WHERE id = $1 AND template_id = $2 AND project_id = $3"
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

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDb {
        db: SqlDb,
        _temp: tempfile::NamedTempFile,
    }

    async fn create_test_db() -> TestDb {
        let (db_path, temp) = crate::db::sql::init::test_sqlite_url();

        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();

        // Создаём таблицу template_role
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS template_role (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                template_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                role_id INTEGER NOT NULL,
                role_slug TEXT NOT NULL DEFAULT ''
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();

        TestDb { db, _temp: temp }
    }

    #[tokio::test]
    async fn test_create_and_get_template_role() {
        let TestDb { db, _temp } = create_test_db().await;

        let role = TemplateRolePerm {
            id: 0,
            template_id: 1,
            project_id: 1,
            role_id: 2,
            role_slug: "admin".to_string(),
        };

        let created = db.create_template_role(role.clone()).await.unwrap();
        assert!(created.id > 0);

        let roles = db.get_template_roles(1, 1).await.unwrap();
        assert!(roles.len() >= 1);
        assert_eq!(roles[0].role_id, 2);

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_template_role() {
        let TestDb { db, _temp } = create_test_db().await;

        let role = TemplateRolePerm {
            id: 0,
            template_id: 1,
            project_id: 1,
            role_id: 2,
            role_slug: "admin".to_string(),
        };

        let created = db.create_template_role(role).await.unwrap();

        let mut updated = created.clone();
        updated.role_id = 3;

        db.update_template_role(updated).await.unwrap();

        let roles = db.get_template_roles(1, 1).await.unwrap();
        assert_eq!(roles[0].role_id, 3);

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_template_role() {
        let TestDb { db, _temp } = create_test_db().await;

        let role = TemplateRolePerm {
            id: 0,
            template_id: 1,
            project_id: 1,
            role_id: 2,
            role_slug: "admin".to_string(),
        };

        let created = db.create_template_role(role).await.unwrap();

        db.delete_template_role(1, 1, created.id).await.unwrap();

        let roles = db.get_template_roles(1, 1).await.unwrap();
        assert!(roles.is_empty());

        // Cleanup
        let _ = db.close().await;
    }
}
