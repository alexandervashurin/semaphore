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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_role_perm_new() {
        let role = TemplateRolePerm {
            id: 0,
            project_id: 1,
            template_id: 10,
            role_id: 5,
            role_slug: "admin".to_string(),
        };
        assert_eq!(role.project_id, 1);
        assert_eq!(role.template_id, 10);
        assert_eq!(role.role_id, 5);
        assert_eq!(role.role_slug, "admin");
    }

    #[test]
    fn test_template_role_perm_serialization() {
        let role = TemplateRolePerm {
            id: 1,
            project_id: 2,
            template_id: 3,
            role_id: 4,
            role_slug: "manager".to_string(),
        };
        let json = serde_json::to_string(&role).unwrap();
        assert!(json.contains("\"project_id\":2"));
        assert!(json.contains("\"template_id\":3"));
        assert!(json.contains("\"role_id\":4"));
        assert!(json.contains("\"role_slug\":\"manager\""));
    }

    #[test]
    fn test_template_role_perm_deserialization() {
        let json =
            r#"{"id":10,"project_id":100,"template_id":200,"role_id":300,"role_slug":"viewer"}"#;
        let role: TemplateRolePerm = serde_json::from_str(json).unwrap();
        assert_eq!(role.id, 10);
        assert_eq!(role.project_id, 100);
        assert_eq!(role.template_id, 200);
        assert_eq!(role.role_id, 300);
        assert_eq!(role.role_slug, "viewer");
    }

    #[test]
    fn test_template_role_perm_clone() {
        let role = TemplateRolePerm {
            id: 5,
            project_id: 1,
            template_id: 2,
            role_id: 3,
            role_slug: "editor".to_string(),
        };
        let cloned = role.clone();
        assert_eq!(cloned.role_slug, role.role_slug);
        assert_eq!(cloned.template_id, role.template_id);
    }

    #[test]
    fn test_template_role_perm_debug_format() {
        let role = TemplateRolePerm {
            id: 1,
            project_id: 1,
            template_id: 1,
            role_id: 1,
            role_slug: "debug".to_string(),
        };
        let debug_str = format!("{:?}", role);
        assert!(debug_str.contains("TemplateRolePerm"));
        assert!(debug_str.contains("debug"));
    }

    #[test]
    fn test_template_role_perm_empty_slug() {
        let role = TemplateRolePerm {
            id: 1,
            project_id: 1,
            template_id: 1,
            role_id: 1,
            role_slug: "".to_string(),
        };
        assert!(role.role_slug.is_empty());
    }

    #[test]
    fn test_template_role_perm_id_zero() {
        let role = TemplateRolePerm {
            id: 0,
            project_id: 0,
            template_id: 0,
            role_id: 0,
            role_slug: "none".to_string(),
        };
        assert_eq!(role.id, 0);
        assert_eq!(role.project_id, 0);
    }

    #[test]
    fn test_template_role_perm_multiple_slugs() {
        let slugs = vec!["admin", "manager", "viewer", "editor", "runner"];
        for slug in slugs {
            let role = TemplateRolePerm {
                id: 1,
                project_id: 1,
                template_id: 1,
                role_id: 1,
                role_slug: slug.to_string(),
            };
            let json = serde_json::to_string(&role).unwrap();
            assert!(json.contains(slug));
        }
    }

    #[test]
    fn test_template_role_perm_equality_by_fields() {
        let role1 = TemplateRolePerm {
            id: 1,
            project_id: 10,
            template_id: 20,
            role_id: 30,
            role_slug: "test".to_string(),
        };
        let role2 = TemplateRolePerm {
            id: 1,
            project_id: 10,
            template_id: 20,
            role_id: 30,
            role_slug: "test".to_string(),
        };
        assert_eq!(role1.id, role2.id);
        assert_eq!(role1.project_id, role2.project_id);
        assert_eq!(role1.template_id, role2.template_id);
        assert_eq!(role1.role_id, role2.role_id);
        assert_eq!(role1.role_slug, role2.role_slug);
    }

    #[test]
    fn test_template_role_perm_serialization_roundtrip() {
        let original = TemplateRolePerm {
            id: 42,
            project_id: 7,
            template_id: 13,
            role_id: 99,
            role_slug: "superadmin".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let decoded: TemplateRolePerm = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, original.id);
        assert_eq!(decoded.project_id, original.project_id);
        assert_eq!(decoded.template_id, original.template_id);
        assert_eq!(decoded.role_id, original.role_id);
        assert_eq!(decoded.role_slug, original.role_slug);
    }
}
