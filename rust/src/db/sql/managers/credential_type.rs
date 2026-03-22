//! CredentialTypeManager - управление пользовательскими типами учётных данных

use crate::db::sql::SqlStore;
use crate::db::store::CredentialTypeManager;
use crate::error::{Error, Result};
use crate::models::credential_type::{
    CredentialType, CredentialTypeCreate, CredentialTypeUpdate,
    CredentialInstance, CredentialInstanceCreate,
};
use async_trait::async_trait;
use chrono::Utc;

#[async_trait]
impl CredentialTypeManager for SqlStore {
    // =========================================================================
    // CredentialType CRUD
    // =========================================================================

    async fn get_credential_types(&self) -> Result<Vec<CredentialType>> {
        let rows = sqlx::query_as::<_, CredentialType>(
                "SELECT * FROM credential_type ORDER BY name"
            )
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(rows)
    }

    async fn get_credential_type(&self, id: i32) -> Result<CredentialType> {
        let row = sqlx::query_as::<_, CredentialType>(
                "SELECT * FROM credential_type WHERE id = $1"
            )
            .bind(id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn create_credential_type(&self, payload: CredentialTypeCreate) -> Result<CredentialType> {
        let now = Utc::now();
        let input_schema = payload.input_schema.to_string();
        let injectors = payload.injectors.to_string();

        let pool = self.get_postgres_pool()?;
            let row = sqlx::query_as::<_, CredentialType>(
                "INSERT INTO credential_type (name, description, input_schema, injectors, created, updated) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
            )
            .bind(&payload.name)
            .bind(&payload.description)
            .bind(&input_schema)
            .bind(&injectors)
            .bind(now)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn update_credential_type(&self, id: i32, payload: CredentialTypeUpdate) -> Result<CredentialType> {
        let now = Utc::now();
        let input_schema = payload.input_schema.to_string();
        let injectors = payload.injectors.to_string();

        let pool = self.get_postgres_pool()?;
            let row = sqlx::query_as::<_, CredentialType>(
                "UPDATE credential_type SET name = $1, description = $2, input_schema = $3, injectors = $4, updated = $5 WHERE id = $6 RETURNING *"
            )
            .bind(&payload.name)
            .bind(&payload.description)
            .bind(&input_schema)
            .bind(&injectors)
            .bind(now)
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn delete_credential_type(&self, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM credential_type WHERE id = $1")
                .bind(id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    // =========================================================================
    // CredentialInstance CRUD
    // =========================================================================

    async fn get_credential_instances(&self, project_id: i32) -> Result<Vec<CredentialInstance>> {
        let rows = sqlx::query_as::<_, CredentialInstance>(
                "SELECT * FROM credential_instance WHERE project_id = $1 ORDER BY name"
            )
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(rows)
    }

    async fn get_credential_instance(&self, id: i32, project_id: i32) -> Result<CredentialInstance> {
        let row = sqlx::query_as::<_, CredentialInstance>(
                "SELECT * FROM credential_instance WHERE id = $1 AND project_id = $2"
            )
            .bind(id)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn create_credential_instance(&self, project_id: i32, payload: CredentialInstanceCreate) -> Result<CredentialInstance> {
        let now = Utc::now();
        let values = payload.values.to_string();

        let pool = self.get_postgres_pool()?;
            let row = sqlx::query_as::<_, CredentialInstance>(
                "INSERT INTO credential_instance (project_id, credential_type_id, name, \"values\", description, created) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
            )
            .bind(project_id)
            .bind(payload.credential_type_id)
            .bind(&payload.name)
            .bind(&values)
            .bind(&payload.description)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn delete_credential_instance(&self, id: i32, project_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM credential_instance WHERE id = $1 AND project_id = $2")
                .bind(id)
                .bind(project_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }
}
