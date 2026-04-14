//! CredentialTypeManager - управление пользовательскими типами учётных данных

use crate::db::sql::SqlStore;
use crate::db::store::CredentialTypeManager;
use crate::error::{Error, Result};
use crate::models::credential_type::{
    CredentialInstance, CredentialInstanceCreate, CredentialType, CredentialTypeCreate,
    CredentialTypeUpdate,
};
use async_trait::async_trait;
use chrono::Utc;

#[async_trait]
impl CredentialTypeManager for SqlStore {
    // =========================================================================
    // CredentialType CRUD
    // =========================================================================

    async fn get_credential_types(&self) -> Result<Vec<CredentialType>> {
        let rows =
            sqlx::query_as::<_, CredentialType>("SELECT * FROM credential_type ORDER BY name")
                .fetch_all(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(rows)
    }

    async fn get_credential_type(&self, id: i32) -> Result<CredentialType> {
        let row =
            sqlx::query_as::<_, CredentialType>("SELECT * FROM credential_type WHERE id = $1")
                .bind(id)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(row)
    }

    async fn create_credential_type(
        &self,
        payload: CredentialTypeCreate,
    ) -> Result<CredentialType> {
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

    async fn update_credential_type(
        &self,
        id: i32,
        payload: CredentialTypeUpdate,
    ) -> Result<CredentialType> {
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
            "SELECT * FROM credential_instance WHERE project_id = $1 ORDER BY name",
        )
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(rows)
    }

    async fn get_credential_instance(
        &self,
        id: i32,
        project_id: i32,
    ) -> Result<CredentialInstance> {
        let row = sqlx::query_as::<_, CredentialInstance>(
            "SELECT * FROM credential_instance WHERE id = $1 AND project_id = $2",
        )
        .bind(id)
        .bind(project_id)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(row)
    }

    async fn create_credential_instance(
        &self,
        project_id: i32,
        payload: CredentialInstanceCreate,
    ) -> Result<CredentialInstance> {
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

#[cfg(test)]
mod tests {
    use crate::models::credential_type::{
        CredentialField, CredentialInjector, CredentialInstance, CredentialInstanceCreate,
        CredentialType, CredentialTypeCreate, CredentialTypeUpdate,
    };
    use chrono::Utc;

    #[test]
    fn test_credential_field_serialization() {
        let field = CredentialField {
            id: "username".to_string(),
            label: "Username".to_string(),
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            help_text: Some("Enter username".to_string()),
        };
        let json = serde_json::to_string(&field).unwrap();
        assert!(json.contains("\"id\":\"username\""));
        assert!(json.contains("\"required\":true"));
    }

    #[test]
    fn test_credential_field_skip_nulls() {
        let field = CredentialField {
            id: "token".to_string(),
            label: "Token".to_string(),
            field_type: "password".to_string(),
            required: false,
            default_value: None,
            help_text: None,
        };
        let json = serde_json::to_string(&field).unwrap();
        assert!(!json.contains("default_value"));
        assert!(!json.contains("help_text"));
    }

    #[test]
    fn test_credential_injector_serialization() {
        let injector = CredentialInjector {
            injector_type: "env".to_string(),
            key: "API_TOKEN".to_string(),
            value_template: "{{ token }}".to_string(),
        };
        let json = serde_json::to_string(&injector).unwrap();
        assert!(json.contains("\"injector_type\":\"env\""));
        assert!(json.contains("\"key\":\"API_TOKEN\""));
    }

    #[test]
    fn test_credential_injector_file_type() {
        let injector = CredentialInjector {
            injector_type: "file".to_string(),
            key: "/tmp/cred_{{ id }}".to_string(),
            value_template: "{{ secret }}".to_string(),
        };
        assert_eq!(injector.injector_type, "file");
    }

    #[test]
    fn test_credential_type_create_serialization() {
        let create = CredentialTypeCreate {
            name: "API Credentials".to_string(),
            description: Some("Custom API creds".to_string()),
            input_schema: serde_json::json!([]),
            injectors: serde_json::json!([]),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"API Credentials\""));
    }

    #[test]
    fn test_credential_type_update_serialization() {
        let update = CredentialTypeUpdate {
            name: "Updated Name".to_string(),
            description: None,
            input_schema: serde_json::json!([]),
            injectors: serde_json::json!([]),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"Updated Name\""));
    }

    #[test]
    fn test_credential_instance_serialization() {
        let instance = CredentialInstance {
            id: 1,
            project_id: 10,
            credential_type_id: 5,
            name: "My API Key".to_string(),
            values: r#"{"token":"enc"}"#.to_string(),
            description: Some("API key".to_string()),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&instance).unwrap();
        assert!(json.contains("\"name\":\"My API Key\""));
        assert!(json.contains("\"project_id\":10"));
    }

    #[test]
    fn test_credential_instance_skip_nulls() {
        let instance = CredentialInstance {
            id: 1,
            project_id: 10,
            credential_type_id: 5,
            name: "No Desc".to_string(),
            values: "{}".to_string(),
            description: None,
            created: Utc::now(),
        };
        let json = serde_json::to_string(&instance).unwrap();
        assert!(!json.contains("description"));
    }

    #[test]
    fn test_credential_instance_create_serialization() {
        let create = CredentialInstanceCreate {
            credential_type_id: 5,
            name: "New Cred".to_string(),
            values: serde_json::json!({"token": "secret"}),
            description: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"credential_type_id\":5"));
        assert!(json.contains("\"name\":\"New Cred\""));
    }

    #[test]
    fn test_credential_type_clone() {
        let ct = CredentialType {
            id: 1,
            name: "Clone".to_string(),
            description: None,
            input_schema: "[]".to_string(),
            injectors: "[]".to_string(),
            created: Utc::now(),
            updated: Utc::now(),
        };
        let cloned = ct.clone();
        assert_eq!(cloned.name, ct.name);
        assert_eq!(cloned.input_schema, ct.input_schema);
    }

    #[test]
    fn test_credential_field_clone() {
        let field = CredentialField {
            id: "f1".to_string(),
            label: "F1".to_string(),
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            help_text: None,
        };
        let cloned = field.clone();
        assert_eq!(cloned.id, field.id);
    }

    #[test]
    fn test_credential_injector_clone() {
        let inj = CredentialInjector {
            injector_type: "env".to_string(),
            key: "KEY".to_string(),
            value_template: "{{ val }}".to_string(),
        };
        let cloned = inj.clone();
        assert_eq!(cloned.injector_type, inj.injector_type);
    }

    #[test]
    fn test_credential_type_create_description_none() {
        let create = CredentialTypeCreate {
            name: "NoDesc".to_string(),
            description: None,
            input_schema: serde_json::json!([]),
            injectors: serde_json::json!([]),
        };
        assert!(create.description.is_none());
    }

    #[test]
    fn test_credential_instance_create_with_description() {
        let create = CredentialInstanceCreate {
            credential_type_id: 1,
            name: "With Desc".to_string(),
            values: serde_json::json!({}),
            description: Some("Has description".to_string()),
        };
        assert_eq!(create.description, Some("Has description".to_string()));
    }
}
