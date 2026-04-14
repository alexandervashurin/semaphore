//! PlaybookManager - управление playbook

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::playbook::{Playbook, PlaybookCreate, PlaybookUpdate};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl PlaybookManager for SqlStore {
    async fn get_playbooks(&self, project_id: i32) -> Result<Vec<Playbook>> {
        let playbooks = sqlx::query_as::<_, Playbook>(
            "SELECT * FROM playbook WHERE project_id = $1 ORDER BY name",
        )
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(playbooks)
    }

    async fn get_playbook(&self, id: i32, project_id: i32) -> Result<Playbook> {
        let playbook = sqlx::query_as::<_, Playbook>(
            "SELECT * FROM playbook WHERE id = $1 AND project_id = $2",
        )
        .bind(id)
        .bind(project_id)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(playbook)
    }

    async fn create_playbook(&self, project_id: i32, payload: PlaybookCreate) -> Result<Playbook> {
        let playbook = sqlx::query_as::<_, Playbook>(
                "INSERT INTO playbook (project_id, name, content, description, playbook_type, repository_id, created, updated)
                 VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) RETURNING *"
            )
            .bind(project_id)
            .bind(&payload.name)
            .bind(&payload.content)
            .bind(&payload.description)
            .bind(&payload.playbook_type)
            .bind(payload.repository_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(playbook)
    }

    async fn update_playbook(
        &self,
        id: i32,
        project_id: i32,
        payload: PlaybookUpdate,
    ) -> Result<Playbook> {
        let playbook = sqlx::query_as::<_, Playbook>(
                "UPDATE playbook SET name = $1, content = $2, description = $3, playbook_type = $4, updated = NOW()
                 WHERE id = $5 AND project_id = $6 RETURNING *"
            )
            .bind(&payload.name)
            .bind(&payload.content)
            .bind(&payload.description)
            .bind(&payload.playbook_type)
            .bind(id)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(playbook)
    }

    async fn delete_playbook(&self, id: i32, project_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM playbook WHERE id = $1 AND project_id = $2")
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
    use crate::models::playbook::{Playbook, PlaybookCreate, PlaybookUpdate};
    use chrono::Utc;

    #[test]
    fn test_playbook_serialization() {
        let playbook = Playbook {
            id: 1,
            project_id: 10,
            name: "deploy.yml".to_string(),
            content: "---\n- hosts: all\n  tasks: []".to_string(),
            description: Some("Main deploy playbook".to_string()),
            playbook_type: "ansible".to_string(),
            repository_id: Some(5),
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&playbook).unwrap();
        assert!(json.contains("\"name\":\"deploy.yml\""));
        assert!(json.contains("\"playbook_type\":\"ansible\""));
    }

    #[test]
    fn test_playbook_skip_nulls() {
        let playbook = Playbook {
            id: 1,
            project_id: 10,
            name: "simple.yml".to_string(),
            content: "---".to_string(),
            description: None,
            playbook_type: "ansible".to_string(),
            repository_id: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&playbook).unwrap();
        assert!(!json.contains("\"description\""));
        assert!(!json.contains("\"repository_id\""));
    }

    #[test]
    fn test_playbook_create_serialization() {
        let create = PlaybookCreate {
            name: "new-playbook.yml".to_string(),
            content: "---\nhosts: localhost".to_string(),
            description: None,
            playbook_type: "shell".to_string(),
            repository_id: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"new-playbook.yml\""));
        assert!(json.contains("\"playbook_type\":\"shell\""));
    }

    #[test]
    fn test_playbook_update_serialization() {
        let update = PlaybookUpdate {
            name: "updated.yml".to_string(),
            content: "---\nupdated content".to_string(),
            description: Some("Updated".to_string()),
            playbook_type: "terraform".to_string(),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"updated.yml\""));
        assert!(json.contains("\"playbook_type\":\"terraform\""));
    }

    #[test]
    fn test_playbook_create_deserialize() {
        let json = r#"{"name":"test.yml","content":"---\ntasks: []","description":null,"playbook_type":"ansible","repository_id":null}"#;
        let create: PlaybookCreate = serde_json::from_str(json).unwrap();
        assert_eq!(create.name, "test.yml");
        assert_eq!(create.playbook_type, "ansible");
    }

    #[test]
    fn test_playbook_update_deserialize() {
        let json =
            r#"{"name":"upd.yml","content":"---","description":"desc","playbook_type":"ansible"}"#;
        let update: PlaybookUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.name, "upd.yml");
        assert_eq!(update.description, Some("desc".to_string()));
    }

    #[test]
    fn test_playbook_clone() {
        let playbook = Playbook {
            id: 1,
            project_id: 5,
            name: "clone.yml".to_string(),
            content: "---".to_string(),
            description: None,
            playbook_type: "ansible".to_string(),
            repository_id: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let cloned = playbook.clone();
        assert_eq!(cloned.name, playbook.name);
        assert_eq!(cloned.playbook_type, playbook.playbook_type);
    }

    #[test]
    fn test_playbook_create_clone() {
        let create = PlaybookCreate {
            name: "clone.yml".to_string(),
            content: "---\nclone".to_string(),
            description: Some("Clone test".to_string()),
            playbook_type: "ansible".to_string(),
            repository_id: Some(1),
        };
        let cloned = create.clone();
        assert_eq!(cloned.name, create.name);
    }

    #[test]
    fn test_playbook_update_clone() {
        let update = PlaybookUpdate {
            name: "clone_upd.yml".to_string(),
            content: "---\nclone".to_string(),
            description: None,
            playbook_type: "shell".to_string(),
        };
        let cloned = update.clone();
        assert_eq!(cloned.playbook_type, update.playbook_type);
    }

    #[test]
    fn test_playbook_different_types() {
        let types = ["ansible", "terraform", "shell"];
        for ptype in types {
            let create = PlaybookCreate {
                name: "test".to_string(),
                content: "---".to_string(),
                description: None,
                playbook_type: ptype.to_string(),
                repository_id: None,
            };
            let json = serde_json::to_string(&create).unwrap();
            assert!(json.contains(&format!("\"playbook_type\":\"{}\"", ptype)));
        }
    }
}
