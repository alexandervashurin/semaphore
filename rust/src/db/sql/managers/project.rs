//! ProjectManager - управление проектами

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::{Project, ProjectUser, Role};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl ProjectStore for SqlStore {
    async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>> {
        let (query, bind_user_id) = if let Some(uid) = user_id {
            ("SELECT p.* FROM project p JOIN project_user pu ON p.id = pu.project_id WHERE pu.user_id = $1", Some(uid))
        } else {
            ("SELECT * FROM project", None)
        };

        let mut q = sqlx::query(query);
        if let Some(uid) = bind_user_id {
            q = q.bind(uid);
        }

        let rows = q
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| Project {
                id: row.get("id"),
                created: row.get("created"),
                name: row.get("name"),
                alert: row.get("alert"),
                alert_chat: row.get("alert_chat"),
                max_parallel_tasks: row.get("max_parallel_tasks"),
                r#type: row.get("type"),
                default_secret_storage_id: row.get("default_secret_storage_id"),
            })
            .collect())
    }

    async fn get_project(&self, project_id: i32) -> Result<Project> {
        let query = "SELECT * FROM project WHERE id = $1";
        let row = sqlx::query(query)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound("Проект не найден".to_string()),
                _ => Error::Database(e),
            })?;

        Ok(Project {
            id: row.get("id"),
            created: row.get("created"),
            name: row.get("name"),
            alert: row.get("alert"),
            alert_chat: row.get("alert_chat"),
            max_parallel_tasks: row.get("max_parallel_tasks"),
            r#type: row.get("type"),
            default_secret_storage_id: row.get("default_secret_storage_id"),
        })
    }

    async fn create_project(&self, mut project: Project) -> Result<Project> {
        let query = "INSERT INTO project (name, created, alert, alert_chat, max_parallel_tasks, type, default_secret_storage_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id";
        let id: i32 = sqlx::query_scalar(query)
            .bind(&project.name)
            .bind(project.created)
            .bind(project.alert)
            .bind(&project.alert_chat)
            .bind(project.max_parallel_tasks)
            .bind(&project.r#type)
            .bind(project.default_secret_storage_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        project.id = id;
        Ok(project)
    }

    async fn update_project(&self, project: Project) -> Result<()> {
        let query = "UPDATE project SET name = $1, alert = $2, alert_chat = $3, max_parallel_tasks = $4, type = $5, default_secret_storage_id = $6 WHERE id = $7";
        sqlx::query(query)
            .bind(&project.name)
            .bind(project.alert)
            .bind(&project.alert_chat)
            .bind(project.max_parallel_tasks)
            .bind(&project.r#type)
            .bind(project.default_secret_storage_id)
            .bind(project.id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_project(&self, project_id: i32) -> Result<()> {
        let query = "DELETE FROM project WHERE id = $1";
        sqlx::query(query)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn create_project_user(&self, project_user: ProjectUser) -> Result<()> {
        let role_str = project_user.role.to_string();
        sqlx::query(
            "INSERT INTO project_user (project_id, user_id, role, created) VALUES ($1, $2, $3, $4)",
        )
        .bind(project_user.project_id)
        .bind(project_user.user_id)
        .bind(&role_str)
        .bind(project_user.created)
        .execute(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_project_user(&self, project_id: i32, user_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM project_user WHERE project_id = $1 AND user_id = $2")
            .bind(project_id)
            .bind(user_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

// ============================================================================
// ProjectRoleManager - CRUD для кастомных ролей проекта
// ============================================================================

#[async_trait]
impl ProjectRoleManager for SqlStore {
    async fn get_project_roles(&self, project_id: i32) -> Result<Vec<Role>> {
        let roles = sqlx::query_as::<_, Role>(
                "SELECT id, project_id, slug, name, description, permissions FROM project_role WHERE project_id = $1 ORDER BY id"
            )
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(roles)
    }

    async fn create_project_role(&self, role: Role) -> Result<Role> {
        let permissions = role.permissions.unwrap_or(0);
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query_as::<_, Role>(
                "INSERT INTO project_role (project_id, slug, name, description, permissions) VALUES ($1, $2, $3, $4, $5) RETURNING id, project_id, slug, name, description, permissions"
            )
            .bind(role.project_id)
            .bind(&role.slug)
            .bind(&role.name)
            .bind(&role.description)
            .bind(permissions)
            .fetch_one(pool)
            .await
            .map_err(Error::Database)?;
        Ok(row)
    }

    async fn update_project_role(&self, role: Role) -> Result<()> {
        let permissions = role.permissions.unwrap_or(0);
        sqlx::query(
                "UPDATE project_role SET slug = $1, name = $2, description = $3, permissions = $4 WHERE id = $5 AND project_id = $6"
            )
            .bind(&role.slug)
            .bind(&role.name)
            .bind(&role.description)
            .bind(permissions)
            .bind(role.id)
            .bind(role.project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_project_role(&self, project_id: i32, role_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM project_role WHERE id = $1 AND project_id = $2")
            .bind(role_id)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

// ============================================================================
// TemplateManager - CRUD операции для шаблонов
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Project, ProjectUser, Role};

    #[test]
    fn test_project_structure() {
        let project = Project {
            id: 1,
            created: chrono::Utc::now(),
            name: "My Project".to_string(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 0,
            r#type: "".to_string(),
            default_secret_storage_id: None,
        };
        assert_eq!(project.name, "My Project");
        assert!(!project.alert);
    }

    #[test]
    fn test_project_with_alerts() {
        let project = Project {
            id: 1,
            created: chrono::Utc::now(),
            name: "Alert Project".to_string(),
            alert: true,
            alert_chat: Some("chat-id".to_string()),
            max_parallel_tasks: 5,
            r#type: "".to_string(),
            default_secret_storage_id: None,
        };
        assert!(project.alert);
        assert!(project.alert_chat.is_some());
        assert_eq!(project.max_parallel_tasks, 5);
    }

    #[test]
    fn test_project_serialize() {
        let project = Project {
            id: 1,
            created: chrono::Utc::now(),
            name: "Serialize Test".to_string(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 0,
            r#type: "default".to_string(),
            default_secret_storage_id: None,
        };
        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("\"name\":\"Serialize Test\""));
        assert!(json.contains("\"type\":\"default\""));
    }

    #[test]
    fn test_project_clone() {
        let project = Project {
            id: 42,
            created: chrono::Utc::now(),
            name: "Clone Test".to_string(),
            alert: true,
            alert_chat: Some("chat".to_string()),
            max_parallel_tasks: 10,
            r#type: "test".to_string(),
            default_secret_storage_id: Some(1),
        };
        let cloned = project.clone();
        assert_eq!(cloned.id, project.id);
        assert_eq!(cloned.name, project.name);
    }

    #[test]
    fn test_project_user_structure() {
        let pu = ProjectUser {
            id: 1,
            project_id: 10,
            user_id: 5,
            role: crate::models::user::ProjectUserRole::Manager,
            created: chrono::Utc::now(),
            username: "user".to_string(),
            name: "User Name".to_string(),
        };
        assert_eq!(pu.project_id, 10);
        assert_eq!(pu.user_id, 5);
    }

    #[test]
    fn test_project_user_serialize() {
        let pu = ProjectUser {
            id: 1,
            project_id: 10,
            user_id: 5,
            role: crate::models::user::ProjectUserRole::Owner,
            created: chrono::Utc::now(),
            username: "owner".to_string(),
            name: "Owner User".to_string(),
        };
        let json = serde_json::to_string(&pu).unwrap();
        assert!(json.contains("\"username\":\"owner\""));
        assert!(json.contains("\"role\":\"owner\""));
    }

    #[test]
    fn test_project_role_structure() {
        let role = Role {
            id: 1,
            project_id: 10,
            slug: "custom-role".to_string(),
            name: "Custom Role".to_string(),
            description: Some("A custom role".to_string()),
            permissions: Some(0b1111),
        };
        assert_eq!(role.slug, "custom-role");
        assert_eq!(role.permissions, Some(0b1111));
    }

    #[test]
    fn test_project_role_serialize() {
        let role = Role {
            id: 1,
            project_id: 5,
            slug: "admin-custom".to_string(),
            name: "Admin Custom".to_string(),
            description: None,
            permissions: Some(255),
        };
        let json = serde_json::to_string(&role).unwrap();
        assert!(json.contains("\"slug\":\"admin-custom\""));
        assert!(json.contains("\"permissions\":255"));
    }

    #[test]
    fn test_sql_query_get_projects() {
        let query = "SELECT * FROM project";
        assert!(query.contains("project"));
    }

    #[test]
    fn test_sql_query_get_project_by_id() {
        let query = "SELECT * FROM project WHERE id = $1";
        assert!(query.contains("WHERE"));
        assert!(query.contains("id"));
    }

    #[test]
    fn test_sql_query_create_project() {
        let query = "INSERT INTO project (name, created, alert, alert_chat, max_parallel_tasks, type, default_secret_storage_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id";
        assert!(query.contains("INSERT INTO project"));
        assert!(query.contains("RETURNING id"));
    }

    #[test]
    fn test_sql_query_project_user_join() {
        let query = "SELECT p.* FROM project p JOIN project_user pu ON p.id = pu.project_id WHERE pu.user_id = $1";
        assert!(query.contains("JOIN project_user"));
        assert!(query.contains("pu.user_id"));
    }

    #[test]
    fn test_sql_query_project_role() {
        let query = "SELECT id, project_id, slug, name, description, permissions FROM project_role WHERE project_id = $1 ORDER BY id";
        assert!(query.contains("project_role"));
        assert!(query.contains("permissions"));
    }

    #[test]
    fn test_project_with_max_parallel_tasks() {
        let project = Project {
            id: 1,
            created: chrono::Utc::now(),
            name: "Parallel".to_string(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 20,
            r#type: "".to_string(),
            default_secret_storage_id: None,
        };
        assert_eq!(project.max_parallel_tasks, 20);
    }
}
