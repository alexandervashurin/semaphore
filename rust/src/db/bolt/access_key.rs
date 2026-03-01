//! AccessKey - операции с ключами доступа в BoltDB
//!
//! Аналог db/bolt/access_key.go из Go версии

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::Result;
use crate::models::AccessKey;

impl BoltStore {
    /// Получает ключ доступа по ID
    pub async fn get_access_key(&self, project_id: i32, access_key_id: i32) -> Result<AccessKey> {
        self.get_object(project_id, "access_keys", access_key_id).await
    }

    /// Получает рефереры ключа доступа
    pub async fn get_access_key_refs(&self, project_id: i32, access_key_id: i32) -> Result<crate::models::ObjectReferrers> {
        self.get_object_refs(project_id, "access_keys", access_key_id).await
    }

    /// Получает ключи доступа проекта
    pub async fn get_access_keys(&self, project_id: i32) -> Result<Vec<AccessKey>> {
        let params = crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: None,
            filter: None,
            sort_by: None,
            sort_inverted: false,
        };
        self.get_objects::<AccessKey>(project_id, "access_keys", params).await
    }

    /// Обновляет ключ доступа
    pub async fn update_access_key(&self, mut key: AccessKey) -> Result<()> {
        key.validate(key.override_secret)?;
        
        if !key.override_secret {
            // Принимаем только новое имя, игнорируем другие изменения
            let old_key = self.get_access_key(key.project_id.unwrap(), key.id).await?;
            key.name = old_key.name;
        }
        
        self.update_object(key.project_id.unwrap(), "access_keys", key).await
    }

    /// Создаёт ключ доступа
    pub async fn create_access_key(&self, mut key: AccessKey) -> Result<AccessKey> {
        key.validate(key.override_secret)?;
        key.created = chrono::Utc::now();
        
        let key_clone = key.clone();
        
        let new_key = self.db.update(|tx| {
            let bucket = tx.create_bucket_if_not_exists(b"access_keys")?;
            
            let str = serde_json::to_vec(&key_clone)?;
            
            let id = bucket.next_sequence()?;
            let id = i64::MAX - id as i64;
            
            let mut key_with_id = key_clone;
            key_with_id.id = id as i32;
            
            let str = serde_json::to_vec(&key_with_id)?;
            bucket.put(id.to_be_bytes(), str)?;
            
            Ok(key_with_id)
        }).await?;
        
        Ok(new_key)
    }

    /// Удаляет ключ доступа
    pub async fn delete_access_key(&self, project_id: i32, access_key_id: i32) -> Result<()> {
        self.delete_object(project_id, "access_keys", access_key_id).await
    }

    /// Перешифровывает ключи доступа (заглушка)
    pub async fn rekey_access_keys(&self, old_key: &str) -> Result<()> {
        // TODO: Реализовать перешифровку ключей
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_bolt_db() -> BoltStore {
        let path = PathBuf::from("/tmp/test_access_keys.db");
        BoltStore::new(path).unwrap()
    }

    fn create_test_access_key(project_id: i32, name: &str) -> AccessKey {
        AccessKey {
            id: 0,
            project_id: Some(project_id),
            name: name.to_string(),
            owner: AccessKeyOwner::Shared,
            key_type: crate::models::AccessKeyType::None,
            ssh_key: None,
            login_password: None,
            created: Utc::now(),
            override_secret: false,
            environment_id: None,
        }
    }

    #[tokio::test]
    async fn test_create_access_key() {
        let db = create_test_bolt_db();
        let key = create_test_access_key(1, "Test Key");
        
        let result = db.create_access_key(key).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_access_key() {
        let db = create_test_bolt_db();
        let key = create_test_access_key(1, "Test Key");
        let created = db.create_access_key(key).await.unwrap();
        
        let retrieved = db.get_access_key(1, created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().name, "Test Key");
    }

    #[tokio::test]
    async fn test_get_access_keys() {
        let db = create_test_bolt_db();
        
        // Создаём несколько ключей
        for i in 0..5 {
            let key = create_test_access_key(1, &format!("Key {}", i));
            db.create_access_key(key).await.unwrap();
        }
        
        let params = RetrieveQueryParams {
            offset: 0,
            count: Some(10),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        };
        
        let options = crate::models::GetAccessKeyOptions {
            owner: None,
        };
        
        let keys = db.get_access_keys(1, options, params).await;
        assert!(keys.is_ok());
        assert!(keys.unwrap().len() >= 5);
    }

    #[tokio::test]
    async fn test_update_access_key() {
        let db = create_test_bolt_db();
        let key = create_test_access_key(1, "Test Key");
        let mut created = db.create_access_key(key).await.unwrap();
        
        created.name = "Updated Key".to_string();
        created.override_secret = true;
        
        let result = db.update_access_key(created).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_access_key() {
        let db = create_test_bolt_db();
        let key = create_test_access_key(1, "Test Key");
        let created = db.create_access_key(key).await.unwrap();
        
        let result = db.delete_access_key(1, created.id).await;
        assert!(result.is_ok());
        
        let retrieved = db.get_access_key(1, created.id).await;
        assert!(retrieved.is_err());
    }
}
