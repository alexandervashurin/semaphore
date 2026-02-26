//! Event - операции с событиями в BoltDB
//!
//! Аналог db/bolt/event.go из Go версии

use std::sync::Arc;
use crate::db::bolt::BoltDb;
use crate::error::Result;
use crate::models::{Event, RetrieveQueryParams};

impl BoltDb {
    /// Создаёт новое событие
    pub async fn create_event(&self, mut evt: Event) -> Result<Event> {
        evt.created = chrono::Utc::now();
        
        let evt_clone = evt.clone();
        
        self.db.update(|tx| {
            let bucket = tx.create_bucket_if_not_exists(b"events")?;
            
            let str = serde_json::to_vec(&evt_clone)?;
            
            let id = bucket.next_sequence()?;
            let id = i64::MAX - id as i64;
            
            bucket.put(id.to_be_bytes(), str)?;
            
            Ok(())
        }).await?;
        
        Ok(evt)
    }

    /// Получает события пользователя
    pub async fn get_user_events(&self, user_id: i32, params: RetrieveQueryParams) -> Result<Vec<Event>> {
        let mut events = Vec::new();
        
        self.db.view(|tx| {
            let bucket = tx.bucket(b"events");
            if bucket.is_none() {
                return Ok(());
            }
            
            let bucket = bucket.unwrap();
            let mut cursor = bucket.cursor();
            
            let mut i = 0;
            let mut n = 0;
            
            while let Some((k, v)) = cursor.first() {
                if params.offset > 0 && i < params.offset {
                    i += 1;
                    continue;
                }
                
                let evt: Event = serde_json::from_slice(v)?;
                
                if evt.project_id.is_none() {
                    continue;
                }
                
                // Проверяем права пользователя
                match self.get_project_user(evt.project_id.unwrap(), user_id) {
                    Ok(_) => {
                        events.push(evt);
                        n += 1;
                    }
                    Err(_) => continue,
                }
                
                if n > params.count {
                    break;
                }
            }
            
            Ok(())
        }).await?;
        
        Ok(events)
    }

    /// Получает события проекта
    pub async fn get_events(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<Event>> {
        let mut events = Vec::new();
        
        self.db.view(|tx| {
            let bucket = tx.bucket(b"events");
            if bucket.is_none() {
                return Ok(());
            }
            
            let bucket = bucket.unwrap();
            let mut cursor = bucket.cursor();
            
            let mut i = 0;
            let mut n = 0;
            
            while let Some((k, v)) = cursor.first() {
                if params.offset > 0 && i < params.offset {
                    i += 1;
                    continue;
                }
                
                let evt: Event = serde_json::from_slice(v)?;
                
                if evt.project_id != Some(project_id) {
                    continue;
                }
                
                events.push(evt);
                n += 1;
                
                if n > params.count {
                    break;
                }
            }
            
            Ok(())
        }).await?;
        
        Ok(events)
    }

    /// Получает все события
    pub async fn get_all_events(&self, params: RetrieveQueryParams) -> Result<Vec<Event>> {
        let mut events = Vec::new();
        
        self.db.view(|tx| {
            let bucket = tx.bucket(b"events");
            if bucket.is_none() {
                return Ok(());
            }
            
            let bucket = bucket.unwrap();
            let mut cursor = bucket.cursor();
            
            let mut i = 0;
            let mut n = 0;
            
            while let Some((k, v)) = cursor.first() {
                if params.offset > 0 && i < params.offset {
                    i += 1;
                    continue;
                }
                
                let evt: Event = serde_json::from_slice(v)?;
                events.push(evt);
                n += 1;
                
                if n > params.count {
                    break;
                }
            }
            
            Ok(())
        }).await?;
        
        Ok(events)
    }

    /// Вспомогательный метод для получения событий с фильтром
    async fn get_events_internal<F>(
        &self,
        params: RetrieveQueryParams,
        filter: F,
    ) -> Result<Vec<Event>>
    where
        F: Fn(&Event) -> bool,
    {
        let mut events = Vec::new();
        
        self.db.view(|tx| {
            let bucket = tx.bucket(b"events");
            if bucket.is_none() {
                return Ok(());
            }
            
            let bucket = bucket.unwrap();
            let mut cursor = bucket.cursor();
            
            let mut i = 0;
            let mut n = 0;
            
            while let Some((k, v)) = cursor.first() {
                if params.offset > 0 && i < params.offset {
                    i += 1;
                    continue;
                }
                
                let evt: Event = serde_json::from_slice(v)?;
                
                if !filter(&evt) {
                    continue;
                }
                
                events.push(evt);
                n += 1;
                
                if n > params.count {
                    break;
                }
            }
            
            Ok(())
        }).await?;
        
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EventType;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_bolt_db() -> BoltDb {
        let path = PathBuf::from("/tmp/test_events.db");
        BoltDb::new(path).unwrap()
    }

    fn create_test_event(project_id: i32) -> Event {
        Event {
            id: 0,
            object_type: EventType::Task,
            object_id: 1,
            project_id: Some(project_id),
            project_name: None,
            user_id: None,
            description: "Test event".to_string(),
            created: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_event() {
        let db = create_test_bolt_db();
        let evt = create_test_event(1);
        
        let result = db.create_event(evt).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_events() {
        let db = create_test_bolt_db();
        
        // Создаём тестовое событие
        let evt = create_test_event(1);
        db.create_event(evt).await.unwrap();
        
        // Получаем события
        let params = RetrieveQueryParams {
            offset: 0,
            count: 10,
            filter: String::new(),
        };
        
        let events = db.get_events(1, params).await;
        assert!(events.is_ok());
    }

    #[tokio::test]
    async fn test_get_all_events() {
        let db = create_test_bolt_db();
        
        // Создаём несколько событий
        for i in 1..=5 {
            let evt = create_test_event(i);
            db.create_event(evt).await.unwrap();
        }
        
        // Получаем все события
        let params = RetrieveQueryParams {
            offset: 0,
            count: 100,
            filter: String::new(),
        };
        
        let events = db.get_all_events(params).await;
        assert!(events.is_ok());
        assert!(events.unwrap().len() >= 5);
    }
}
