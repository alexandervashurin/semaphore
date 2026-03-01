//! Schedule - операции с расписаниями в BoltDB
//!
//! Аналог db/bolt/schedule.go из Go версии

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::Result;
use crate::models::{Schedule, ScheduleWithTpl, RetrieveQueryParams};

impl BoltStore {
    /// Получает все расписания
    pub async fn get_schedules(&self) -> Result<Vec<Schedule>> {
        let mut schedules = Vec::new();
        
        let all_projects = self.get_all_projects().await?;
        
        for project in all_projects {
            let project_schedules = self.get_project_schedules(project.id, None).await?;
            schedules.extend(project_schedules);
        }
        
        Ok(schedules)
    }

    /// Получает расписания проекта
    async fn get_project_schedules(&self, project_id: i32, filter: Option<&dyn Fn(&Schedule) -> bool>) -> Result<Vec<Schedule>> {
        let mut schedules = Vec::new();
        
        let all_schedules = self.get_objects::<Schedule>(project_id, "schedules", RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        }).await?;
        
        for schedule in all_schedules {
            if let Some(f) = filter {
                if !f(&schedule) {
                    continue;
                }
            }
            schedules.push(schedule);
        }
        
        Ok(schedules)
    }

    /// Получает расписания проекта с шаблонами
    pub async fn get_project_schedules_with_tpl(&self, project_id: i32, include_task_params: bool, include_commit_checkers: bool) -> Result<Vec<ScheduleWithTpl>> {
        let mut schedules = Vec::new();
        
        let orig = self.get_project_schedules(project_id, Some(&|s: &Schedule| {
            if include_commit_checkers {
                true
            } else {
                s.repository_id.is_none()
            }
        })).await?;
        
        for s in orig {
            let tpl = self.get_template(project_id, s.template_id).await?;
            schedules.push(ScheduleWithTpl {
                schedule: s,
                template_name: tpl.name,
            });
        }
        
        Ok(schedules)
    }

    /// Получает расписания шаблона
    pub async fn get_template_schedules(&self, project_id: i32, template_id: i32, only_commit_checkers: bool) -> Result<Vec<Schedule>> {
        self.get_project_schedules(project_id, Some(&|s: &Schedule| {
            s.template_id == template_id && (!only_commit_checkers || s.repository_id.is_some())
        })).await
    }

    /// Создаёт новое расписание
    pub async fn create_schedule(&self, mut schedule: Schedule) -> Result<Schedule> {
        schedule.created = chrono::Utc::now();
        
        let schedule_clone = schedule.clone();
        
        let new_schedule = self.db.update(|tx| {
            let bucket = tx.create_bucket_if_not_exists(b"schedules")?;
            
            let str = serde_json::to_vec(&schedule_clone)?;
            
            let id = bucket.next_sequence()?;
            let id = i64::MAX - id as i64;
            
            let mut schedule_with_id = schedule_clone;
            schedule_with_id.id = id as i32;
            
            let str = serde_json::to_vec(&schedule_with_id)?;
            bucket.put(id.to_be_bytes(), str)?;
            
            Ok(schedule_with_id)
        }).await?;
        
        Ok(new_schedule)
    }

    /// Обновляет расписание
    pub async fn update_schedule(&self, schedule: Schedule) -> Result<()> {
        self.db.update(|tx| {
            let bucket = tx.bucket(b"schedules");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Расписание не найдено".to_string()));
            }
            
            let bucket = bucket.unwrap();
            let key = (i64::MAX - schedule.id as i64).to_be_bytes();
            
            if bucket.get(key).is_none() {
                return Err(crate::error::Error::NotFound("Расписание не найдено".to_string()));
            }
            
            let str = serde_json::to_vec(&schedule)?;
            bucket.put(key, str)?;
            
            Ok(())
        }).await
    }

    /// Получает расписание по ID
    pub async fn get_schedule(&self, project_id: i32, schedule_id: i32) -> Result<Schedule> {
        self.db.view(|tx| {
            let bucket = tx.bucket(b"schedules");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Расписание не найдено".to_string()));
            }
            
            let bucket = bucket.unwrap();
            let key = (i64::MAX - schedule_id as i64).to_be_bytes();
            
            if let Some(v) = bucket.get(key) {
                let schedule: Schedule = serde_json::from_slice(&v)?;
                if schedule.project_id == project_id {
                    Ok(schedule)
                } else {
                    Err(crate::error::Error::NotFound("Расписание не найдено".to_string()))
                }
            } else {
                Err(crate::error::Error::NotFound("Расписание не найдено".to_string()))
            }
        }).await
    }

    /// Удаляет расписание
    pub async fn delete_schedule(&self, project_id: i32, schedule_id: i32) -> Result<()> {
        self.db.update(|tx| {
            let bucket = tx.bucket(b"schedules");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Расписание не найдено".to_string()));
            }
            
            let bucket = bucket.unwrap();
            let key = (i64::MAX - schedule_id as i64).to_be_bytes();
            
            if bucket.get(key).is_none() {
                return Err(crate::error::Error::NotFound("Расписание не найдено".to_string()));
            }
            
            bucket.remove(key)?;
            
            Ok(())
        }).await
    }

    /// Устанавливает активность расписания
    pub async fn set_schedule_active(&self, project_id: i32, schedule_id: i32, active: bool) -> Result<()> {
        let mut schedule = self.get_schedule(project_id, schedule_id).await?;
        schedule.active = active;
        self.update_schedule(schedule).await
    }

    /// Устанавливает hash коммита для расписания
    pub async fn set_schedule_commit_hash(&self, project_id: i32, schedule_id: i32, hash: &str) -> Result<()> {
        let mut schedule = self.get_schedule(project_id, schedule_id).await?;
        schedule.last_commit_hash = Some(hash.to_string());
        self.update_schedule(schedule).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_bolt_db() -> BoltStore {
        let path = PathBuf::from("/tmp/test_schedules.db");
        BoltStore::new(path).unwrap()
    }

    fn create_test_schedule(project_id: i32, template_id: i32) -> Schedule {
        Schedule {
            id: 0,
            created: Utc::now(),
            project_id,
            template_id,
            cron_format: "0 * * * *".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
        }
    }

    #[tokio::test]
    async fn test_create_schedule() {
        let db = create_test_bolt_db();
        let schedule = create_test_schedule(1, 1);
        
        let result = db.create_schedule(schedule).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_schedule() {
        let db = create_test_bolt_db();
        let schedule = create_test_schedule(1, 1);
        let created = db.create_schedule(schedule).await.unwrap();
        
        let retrieved = db.get_schedule(1, created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_get_project_schedules() {
        let db = create_test_bolt_db();
        
        // Создаём несколько расписаний
        for i in 0..5 {
            let schedule = create_test_schedule(1, 1);
            db.create_schedule(schedule).await.unwrap();
        }
        
        let schedules = db.get_project_schedules_with_tpl(1, false, false).await;
        assert!(schedules.is_ok());
        assert!(schedules.unwrap().len() >= 5);
    }

    #[tokio::test]
    async fn test_update_schedule() {
        let db = create_test_bolt_db();
        let schedule = create_test_schedule(1, 1);
        let mut created = db.create_schedule(schedule).await.unwrap();
        
        created.active = false;
        let result = db.update_schedule(created).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_schedule_active() {
        let db = create_test_bolt_db();
        let schedule = create_test_schedule(1, 1);
        let created = db.create_schedule(schedule).await.unwrap();
        
        let result = db.set_schedule_active(1, created.id, false).await;
        assert!(result.is_ok());
        
        let retrieved = db.get_schedule(1, created.id).await;
        assert!(retrieved.is_ok());
        assert!(!retrieved.unwrap().active);
    }

    #[tokio::test]
    async fn test_set_schedule_commit_hash() {
        let db = create_test_bolt_db();
        let schedule = create_test_schedule(1, 1);
        let created = db.create_schedule(schedule).await.unwrap();
        
        let result = db.set_schedule_commit_hash(1, created.id, "abc123").await;
        assert!(result.is_ok());
        
        let retrieved = db.get_schedule(1, created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().last_commit_hash, Some("abc123".to_string()));
    }

    #[tokio::test]
    async fn test_delete_schedule() {
        let db = create_test_bolt_db();
        let schedule = create_test_schedule(1, 1);
        let created = db.create_schedule(schedule).await.unwrap();
        
        let result = db.delete_schedule(1, created.id).await;
        assert!(result.is_ok());
        
        let retrieved = db.get_schedule(1, created.id).await;
        assert!(retrieved.is_err());
    }
}
