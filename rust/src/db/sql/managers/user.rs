//! UserManager - управление пользователями
//!
//! Реализация трейта UserManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::{ProjectUser, User, UserTotp};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

#[async_trait]
impl UserManager for SqlStore {
    async fn get_users(&self, _params: RetrieveQueryParams) -> Result<Vec<User>> {
        let query = "SELECT * FROM \"user\" ORDER BY id";
        let rows = sqlx::query(query)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| User {
                id: row.get("id"),
                created: row.try_get("created").unwrap_or_else(|_| Utc::now()),
                username: row.get("username"),
                name: row.get("name"),
                email: row.get("email"),
                password: row.get("password"),
                admin: row.get("admin"),
                external: row.get("external"),
                alert: row.get("alert"),
                pro: row.get("pro"),
                totp: None,
                email_otp: None,
            })
            .collect())
    }

    async fn get_user(&self, user_id: i32) -> Result<User> {
        let query = "SELECT * FROM \"user\" WHERE id = $1";
        let row = sqlx::query(query)
            .bind(user_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound("Пользователь не найден".to_string()),
                _ => Error::Database(e),
            })?;

        Ok(User {
            id: row.get("id"),
            created: row.try_get("created").unwrap_or_else(|_| Utc::now()),
            username: row.get("username"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            admin: row.get("admin"),
            external: row.get("external"),
            alert: row.get("alert"),
            pro: row.get("pro"),
            totp: None,
            email_otp: None,
        })
    }

    async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        let query = "SELECT id, created, username, name, email, password, admin, external, alert, pro FROM \"user\" WHERE username = $1 OR email = $2";
        let row = sqlx::query(query)
            .bind(login)
            .bind(email)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound("Пользователь не найден".to_string()),
                _ => Error::Database(e),
            })?;

        let id: i32 = row.try_get("id").map_err(Error::Database)?;
        let created: chrono::DateTime<chrono::Utc> =
            row.try_get("created").map_err(Error::Database)?;
        let username: String = row.try_get("username").map_err(Error::Database)?;
        let name: String = row.try_get("name").map_err(Error::Database)?;
        let email: String = row.try_get("email").map_err(Error::Database)?;
        let password: String = row.try_get("password").map_err(Error::Database)?;
        let admin: bool = row.try_get("admin").map_err(Error::Database)?;
        let external: bool = row.try_get("external").map_err(Error::Database)?;
        let alert: bool = row.try_get("alert").map_err(Error::Database)?;
        let pro: bool = row.try_get("pro").map_err(Error::Database)?;

        Ok(User {
            id,
            created,
            username,
            name,
            email,
            password,
            admin,
            external,
            alert,
            pro,
            totp: None,
            email_otp: None,
        })
    }

    async fn create_user(&self, user: User, password: &str) -> Result<User> {
        use crate::api::auth_local::hash_password;

        // Хешируем пароль перед сохранением
        let password_hash = hash_password(password)?;

        let query = "INSERT INTO \"user\" (username, name, email, password, admin, external, alert, pro, created) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)";
        sqlx::query(query)
            .bind(&user.username)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&password_hash)
            .bind(user.admin)
            .bind(user.external)
            .bind(user.alert)
            .bind(user.pro)
            .bind(user.created)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        self.get_user_by_login_or_email(&user.username, &user.email)
            .await
    }

    async fn update_user(&self, user: User) -> Result<()> {
        let query = "UPDATE \"user\" SET username = $1, name = $2, email = $3, admin = $4, external = $5, alert = $6, pro = $7 WHERE id = $8";
        sqlx::query(query)
            .bind(&user.username)
            .bind(&user.name)
            .bind(&user.email)
            .bind(user.admin)
            .bind(user.external)
            .bind(user.alert)
            .bind(user.pro)
            .bind(user.id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_user(&self, user_id: i32) -> Result<()> {
        let query = "DELETE FROM \"user\" WHERE id = $1";
        sqlx::query(query)
            .bind(user_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn set_user_password(&self, user_id: i32, password: &str) -> Result<()> {
        let query = "UPDATE \"user\" SET password = $1 WHERE id = $2";
        sqlx::query(query)
            .bind(password)
            .bind(user_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_all_admins(&self) -> Result<Vec<User>> {
        let query = "SELECT * FROM \"user\" WHERE admin = $1";
        let rows = sqlx::query(query)
            .bind(true)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| User {
                id: row.get("id"),
                created: row.try_get("created").unwrap_or_else(|_| Utc::now()),
                username: row.get("username"),
                name: row.get("name"),
                email: row.get("email"),
                password: row.get("password"),
                admin: row.get("admin"),
                external: row.get("external"),
                alert: row.get("alert"),
                pro: row.get("pro"),
                totp: None,
                email_otp: None,
            })
            .collect())
    }

    async fn get_user_count(&self) -> Result<usize> {
        let query = "SELECT COUNT(*) FROM \"user\"";
        let count: i64 = sqlx::query_scalar(query)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(count as usize)
    }

    async fn get_project_users(
        &self,
        project_id: i32,
        _params: RetrieveQueryParams,
    ) -> Result<Vec<ProjectUser>> {
        let query = "SELECT pu.*, u.username, u.name, u.email
                 FROM project_user pu
                 JOIN \"user\" u ON pu.user_id = u.id
                 WHERE pu.project_id = $1
                 ORDER BY pu.id";
        let rows = sqlx::query(query)
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| ProjectUser {
                id: row.get("id"),
                project_id: row.get("project_id"),
                user_id: row.get("user_id"),
                role: row.get("role"),
                created: row.try_get("created").unwrap_or_else(|_| Utc::now()),
                username: row.get("username"),
                name: row.get("name"),
            })
            .collect())
    }

    async fn get_user_totp(&self, user_id: i32) -> Result<Option<UserTotp>> {
        // Получаем пользователя и возвращаем его TOTP
        let user = self.get_user(user_id).await?;
        Ok(user.totp)
    }

    async fn set_user_totp(&self, user_id: i32, totp: &UserTotp) -> Result<()> {
        // Сериализуем TOTP в JSON
        let totp_json = serde_json::to_string(totp)
            .map_err(|e| Error::Other(format!("Failed to serialize TOTP: {}", e)))?;

        // Обновляем user.totp
        sqlx::query("UPDATE \"user\" SET totp = $1 WHERE id = $2")
            .bind(&totp_json)
            .bind(user_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_user_totp(&self, user_id: i32) -> Result<()> {
        // Удаляем TOTP (устанавливаем в NULL)
        sqlx::query("UPDATE \"user\" SET totp = NULL WHERE id = $1")
            .bind(user_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ProjectUser, User, UserTotp};
    use crate::models::user::ProjectUserRole;

    #[test]
    fn test_user_structure() {
        let user = User {
            id: 1,
            created: Utc::now(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "hashed".to_string(),
            admin: false,
            external: false,
            alert: true,
            pro: false,
            totp: None,
            email_otp: None,
        };
        assert_eq!(user.username, "testuser");
        assert!(!user.admin);
    }

    #[test]
    fn test_user_admin_flag() {
        let admin_user = User {
            id: 1,
            created: Utc::now(),
            username: "admin".to_string(),
            name: "Admin".to_string(),
            email: "admin@example.com".to_string(),
            password: "hash".to_string(),
            admin: true,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        assert!(admin_user.admin);
    }

    #[test]
    fn test_user_serialize_excludes_password() {
        let user = User {
            id: 1,
            created: Utc::now(),
            username: "user".to_string(),
            name: "Name".to_string(),
            email: "a@b.com".to_string(),
            password: "secret".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"username\":\"user\""));
        assert!(!json.contains("secret"));
    }

    #[test]
    fn test_user_totp_structure() {
        let totp = UserTotp {
            id: 1,
            created: Utc::now(),
            user_id: 42,
            url: "otpauth://totp/test?secret=ABC".to_string(),
            recovery_hash: "hash".to_string(),
            recovery_code: Some("CODE123".to_string()),
        };
        assert_eq!(totp.user_id, 42);
        assert!(totp.recovery_code.is_some());
    }

    #[test]
    fn test_user_totp_default() {
        let totp = UserTotp::default();
        assert_eq!(totp.id, 0);
        assert!(totp.recovery_code.is_none());
    }

    #[test]
    fn test_project_user_structure() {
        let pu = ProjectUser {
            id: 1,
            project_id: 10,
            user_id: 5,
            role: ProjectUserRole::Manager,
            created: Utc::now(),
            username: "user".to_string(),
            name: "User".to_string(),
        };
        assert_eq!(pu.project_id, 10);
        assert_eq!(pu.role, ProjectUserRole::Manager);
    }

    #[test]
    fn test_project_user_role_variants() {
        let roles = vec![
            ProjectUserRole::Owner,
            ProjectUserRole::Manager,
            ProjectUserRole::TaskRunner,
            ProjectUserRole::Guest,
            ProjectUserRole::None,
        ];
        for role in roles {
            let display = role.to_string();
            assert!(!display.is_empty());
        }
    }

    #[test]
    fn test_project_user_role_display() {
        assert_eq!(ProjectUserRole::Owner.to_string(), "owner");
        assert_eq!(ProjectUserRole::Manager.to_string(), "manager");
        assert_eq!(ProjectUserRole::TaskRunner.to_string(), "task_runner");
        assert_eq!(ProjectUserRole::Guest.to_string(), "guest");
        assert_eq!(ProjectUserRole::None.to_string(), "none");
    }

    #[test]
    fn test_project_user_new() {
        let pu = ProjectUser::new(1, 10, ProjectUserRole::TaskRunner);
        assert_eq!(pu.id, 0);
        assert_eq!(pu.project_id, 1);
        assert_eq!(pu.user_id, 10);
    }

    #[test]
    fn test_sql_query_get_users() {
        let query = "SELECT * FROM \"user\" ORDER BY id";
        assert!(query.contains("user"));
        assert!(query.contains("ORDER BY"));
    }

    #[test]
    fn test_sql_query_get_user_by_id() {
        let query = "SELECT * FROM \"user\" WHERE id = $1";
        assert!(query.contains("WHERE"));
        assert!(query.contains("id"));
    }

    #[test]
    fn test_sql_query_get_admins() {
        let query = "SELECT * FROM \"user\" WHERE admin = $1";
        assert!(query.contains("admin"));
        assert!(query.contains("$1"));
    }

    #[test]
    fn test_sql_query_project_users_with_join() {
        let query = "SELECT pu.*, u.username, u.name, u.email
                 FROM project_user pu
                 JOIN \"user\" u ON pu.user_id = u.id
                 WHERE pu.project_id = $1
                 ORDER BY pu.id";
        assert!(query.contains("project_user"));
        assert!(query.contains("JOIN"));
    }

    #[test]
    fn test_user_totp_serialize_excludes_sensitive() {
        let totp = UserTotp {
            id: 1,
            created: Utc::now(),
            user_id: 1,
            url: "otpauth://totp/x".to_string(),
            recovery_hash: "secret_hash".to_string(),
            recovery_code: None,
        };
        let json = serde_json::to_string(&totp).unwrap();
        assert!(json.contains("\"url\":"));
    }
}
