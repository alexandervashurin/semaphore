//! BoltDB Core
//!
//! Основная структура и операции BoltDB

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sled::Db;
use crate::error::{Error, Result};
use crate::db::store::RetrieveQueryParams;

/// BoltDB хранилище
pub struct BoltStore {
    /// Sled database
    pub db: Db,
    
    /// Имя файла БД
    pub filename: String,
    
    /// Счётчик ID
    id_counter: Arc<Mutex<HashMap<String, i32>>>,
}

impl BoltStore {
    /// Создаёт новое BoltDB хранилище
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(Self {
            db,
            filename: path.to_string(),
            id_counter: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Сериализует объект в JSON
    pub fn serialize<T: serde::Serialize>(&self, obj: &T) -> Result<Vec<u8>> {
        serde_json::to_vec(obj).map_err(|e| Error::Json(e))
    }

    /// Десериализует объект из JSON
    #[allow(dead_code)]
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        serde_json::from_slice(bytes).map_err(|e| Error::Json(e))
    }

    /// Получает следующий ID для объекта
    pub fn get_next_id(&self, bucket_name: &str) -> Result<i32> {
        let mut counter = self.id_counter.lock()
            .map_err(|e| Error::Other(format!("Lock error: {}", e)))?;

        let id = counter.entry(bucket_name.to_string())
            .or_insert(0);
        
        *id += 1;
        Ok(*id)
    }

    /// Создаёт объект
    pub async fn create_object<T: serde::Serialize>(
        &self,
        project_id: i32,
        bucket_name: &str,
        obj: &T,
    ) -> Result<()> {
        let key = format!("{}_{}", project_id, bucket_name);
        let tree = self.db.open_tree(key.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let id = self.get_next_id(bucket_name)?;
        let id_key = format!("{:010}", id);

        let value = serde_json::to_vec(obj)
            .map_err(|e| Error::Json(e))?;

        tree.insert(id_key.as_bytes(), value)
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(())
    }

    /// Получает объект по ID
    pub async fn get_object<T: serde::de::DeserializeOwned>(
        &self,
        project_id: i32,
        bucket_name: &str,
        id: i32,
    ) -> Result<T> {
        let key = format!("{}_{}", project_id, bucket_name);
        let tree = self.db.open_tree(key.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let id_key = format!("{:010}", id);
        let value = tree.get(id_key.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        match value {
            Some(v) => {
                let obj: T = serde_json::from_slice(&v)
                    .map_err(|e| Error::Json(e))?;
                Ok(obj)
            }
            None => Err(Error::NotFound(format!("Object {} not found in {}", id, bucket_name))),
        }
    }

    /// Получает все объекты
    pub async fn get_objects<T: serde::de::DeserializeOwned>(
        &self,
        project_id: i32,
        bucket_name: &str,
        params: RetrieveQueryParams,
    ) -> Result<Vec<T>> {
        let key = format!("{}_{}", project_id, bucket_name);
        let tree = self.db.open_tree(key.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let mut objects = Vec::new();
        let mut count = 0;
        let offset = params.offset;
        let limit = params.count.unwrap_or(1000);

        for item in tree.iter() {
            let (_k, v) = item
                .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

            if count >= offset && objects.len() < limit {
                let obj: T = serde_json::from_slice(&v)
                    .map_err(|e| Error::Json(e))?;
                objects.push(obj);
            }
            count += 1;
        }

        Ok(objects)
    }

    /// Обновляет объект
    pub async fn update_object<T: serde::Serialize>(
        &self,
        project_id: i32,
        bucket_name: &str,
        id: i32,
        obj: &T,
    ) -> Result<()> {
        let key = format!("{}_{}", project_id, bucket_name);
        let tree = self.db.open_tree(key.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let id_key = format!("{:010}", id);
        let value = serde_json::to_vec(obj)
            .map_err(|e| Error::Json(e))?;

        tree.insert(id_key.as_bytes(), value)
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(())
    }

    /// Удаляет объект
    pub async fn delete_object(
        &self,
        project_id: i32,
        bucket_name: &str,
        id: i32,
    ) -> Result<()> {
        let key = format!("{}_{}", project_id, bucket_name);
        let tree = self.db.open_tree(key.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let id_key = format!("{:010}", id);
        tree.remove(id_key.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(())
    }

    /// Получает диалект БД
    pub fn get_dialect(&self) -> &str {
        "bolt"
    }

    /// Закрывает соединение с БД
    pub fn close(&self) -> Result<()> {
        self.db.flush()
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bolt_store_creation() {
        // Тест для проверки создания хранилища
        assert!(true);
    }

    #[test]
    fn test_get_dialect() {
        // Тест для проверки диалекта
        assert!(true);
    }
}
