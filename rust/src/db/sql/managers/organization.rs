//! OrganizationManager - управление организациями (Multi-Tenancy)

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::organization::*;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;

fn row_to_organization(row: sqlx::postgres::PgRow) -> Organization {
    Organization {
        id: row.get("id"),
        name: row.get("name"),
        slug: row.get("slug"),
        description: row.try_get("description").ok().flatten(),
        settings: row.try_get("settings").ok().flatten(),
        quota_max_projects: row.try_get("quota_max_projects").ok().flatten(),
        quota_max_users: row.try_get("quota_max_users").ok().flatten(),
        quota_max_tasks_per_month: row.try_get("quota_max_tasks_per_month").ok().flatten(),
        active: row.get("active"),
        created: row.get("created"),
        updated: row.try_get("updated").ok().flatten(),
    }
}

fn row_to_organization_user(row: sqlx::postgres::PgRow) -> OrganizationUser {
    OrganizationUser {
        id: row.get("id"),
        org_id: row.get("org_id"),
        user_id: row.get("user_id"),
        role: row.get("role"),
        created: row.get("created"),
    }
}

#[async_trait]
impl OrganizationManager for SqlStore {
    async fn get_organizations(&self) -> Result<Vec<Organization>> {
        let pool = self.get_postgres_pool()?;
        let rows = sqlx::query("SELECT * FROM organization ORDER BY name")
            .fetch_all(pool)
            .await
            .map_err(Error::Database)?;
        Ok(rows.into_iter().map(row_to_organization).collect())
    }

    async fn get_organization(&self, id: i32) -> Result<Organization> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query("SELECT * FROM organization WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(Error::Database)?
            .ok_or_else(|| Error::NotFound("Organization not found".to_string()))?;
        Ok(row_to_organization(row))
    }

    async fn get_organization_by_slug(&self, slug: &str) -> Result<Organization> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query("SELECT * FROM organization WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await
            .map_err(Error::Database)?
            .ok_or_else(|| Error::NotFound("Organization not found".to_string()))?;
        Ok(row_to_organization(row))
    }

    async fn create_organization(&self, payload: OrganizationCreate) -> Result<Organization> {
        let pool = self.get_postgres_pool()?;

        // Генерируем slug если не предоставлен
        let slug = payload.slug.unwrap_or_else(|| {
            payload
                .name
                .to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
        });

        let id: i32 = sqlx::query_scalar(
            "INSERT INTO organization (name, slug, description, settings, quota_max_projects, quota_max_users, quota_max_tasks_per_month, active, created)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"
        )
        .bind(&payload.name)
        .bind(&slug)
        .bind(&payload.description)
        .bind(&payload.settings)
        .bind(payload.quota_max_projects)
        .bind(payload.quota_max_users)
        .bind(payload.quota_max_tasks_per_month)
        .bind(true)
        .bind(Utc::now())
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

        Ok(Organization {
            id,
            name: payload.name,
            slug,
            description: payload.description,
            settings: payload.settings,
            quota_max_projects: payload.quota_max_projects,
            quota_max_users: payload.quota_max_users,
            quota_max_tasks_per_month: payload.quota_max_tasks_per_month,
            active: true,
            created: Utc::now(),
            updated: None,
        })
    }

    async fn update_organization(
        &self,
        id: i32,
        payload: OrganizationUpdate,
    ) -> Result<Organization> {
        let pool = self.get_postgres_pool()?;

        // Получаем текущую организацию
        let org = self.get_organization(id).await?;

        let updated = Organization {
            name: payload.name.unwrap_or(org.name),
            description: payload.description.or(org.description),
            settings: payload.settings.or(org.settings),
            quota_max_projects: payload.quota_max_projects.or(org.quota_max_projects),
            quota_max_users: payload.quota_max_users.or(org.quota_max_users),
            quota_max_tasks_per_month: payload
                .quota_max_tasks_per_month
                .or(org.quota_max_tasks_per_month),
            active: payload.active.unwrap_or(org.active),
            updated: Some(Utc::now()),
            ..org
        };

        sqlx::query(
            "UPDATE organization SET name = $1, description = $2, settings = $3, 
             quota_max_projects = $4, quota_max_users = $5, quota_max_tasks_per_month = $6, 
             active = $7, updated = $8 WHERE id = $9",
        )
        .bind(&updated.name)
        .bind(&updated.description)
        .bind(&updated.settings)
        .bind(updated.quota_max_projects)
        .bind(updated.quota_max_users)
        .bind(updated.quota_max_tasks_per_month)
        .bind(updated.active)
        .bind(updated.updated)
        .bind(id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        Ok(updated)
    }

    async fn delete_organization(&self, id: i32) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query("DELETE FROM organization WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_organization_users(&self, org_id: i32) -> Result<Vec<OrganizationUser>> {
        let pool = self.get_postgres_pool()?;
        let rows =
            sqlx::query("SELECT * FROM organization_user WHERE org_id = $1 ORDER BY created")
                .bind(org_id)
                .fetch_all(pool)
                .await
                .map_err(Error::Database)?;
        Ok(rows.into_iter().map(row_to_organization_user).collect())
    }

    async fn add_user_to_organization(
        &self,
        payload: OrganizationUserCreate,
    ) -> Result<OrganizationUser> {
        let pool = self.get_postgres_pool()?;
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO organization_user (org_id, user_id, role, created) VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(payload.org_id)
        .bind(payload.user_id)
        .bind(&payload.role)
        .bind(Utc::now())
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

        Ok(OrganizationUser {
            id,
            org_id: payload.org_id,
            user_id: payload.user_id,
            role: payload.role,
            created: Utc::now(),
        })
    }

    async fn remove_user_from_organization(&self, org_id: i32, user_id: i32) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query("DELETE FROM organization_user WHERE org_id = $1 AND user_id = $2")
            .bind(org_id)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn update_user_organization_role(
        &self,
        org_id: i32,
        user_id: i32,
        role: &str,
    ) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query("UPDATE organization_user SET role = $1 WHERE org_id = $2 AND user_id = $3")
            .bind(role)
            .bind(org_id)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_user_organizations(&self, user_id: i32) -> Result<Vec<Organization>> {
        let pool = self.get_postgres_pool()?;
        let rows = sqlx::query(
            "SELECT o.* FROM organization o
             JOIN organization_user ou ON o.id = ou.org_id
             WHERE ou.user_id = $1
             ORDER BY o.name",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;
        Ok(rows.into_iter().map(row_to_organization).collect())
    }

    async fn check_organization_quota(&self, org_id: i32, quota_type: &str) -> Result<bool> {
        let pool = self.get_postgres_pool()?;

        // Получаем организацию
        let org = self.get_organization(org_id).await?;

        match quota_type {
            "projects" => {
                if let Some(max) = org.quota_max_projects {
                    let count: i64 =
                        sqlx::query_scalar("SELECT COUNT(*) FROM project WHERE org_id = $1")
                            .bind(org_id)
                            .fetch_one(pool)
                            .await
                            .map_err(Error::Database)?;
                    Ok(count < max as i64)
                } else {
                    Ok(true) // Квота не установлена
                }
            }
            "users" => {
                if let Some(max) = org.quota_max_users {
                    let count: i64 = sqlx::query_scalar(
                        "SELECT COUNT(*) FROM organization_user WHERE org_id = $1",
                    )
                    .bind(org_id)
                    .fetch_one(pool)
                    .await
                    .map_err(Error::Database)?;
                    Ok(count < max as i64)
                } else {
                    Ok(true)
                }
            }
            "tasks_per_month" => {
                if let Some(max) = org.quota_max_tasks_per_month {
                    let count: i64 = sqlx::query_scalar(
                        "SELECT COUNT(*) FROM task t
                         JOIN template tpl ON t.template_id = tpl.id
                         WHERE tpl.project_id IN (SELECT id FROM project WHERE org_id = $1)
                         AND t.created >= NOW() - INTERVAL '30 days'",
                    )
                    .bind(org_id)
                    .fetch_one(pool)
                    .await
                    .map_err(Error::Database)?;
                    Ok(count < max as i64)
                } else {
                    Ok(true)
                }
            }
            _ => Ok(true),
        }
    }
}
