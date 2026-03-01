//! Template - операции с шаблонами в BoltDB
//!
//! Аналог db/bolt/template.go из Go версии

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::Result;
use crate::models::{Template, TemplateWithPerms, TemplateFilter, RetrieveQueryParams, TemplateRolePerm};
use chrono::Utc;

impl BoltStore {
    /// Создаёт новый шаблон
    pub async fn create_template(&self, mut template: Template) -> Result<Template> {
        // Валидация шаблона
        template.validate()?;
        
        template.created = Utc::now();
        
        let template_clone = template.clone();
        
        let new_template = self.db.update(|tx| {
            let bucket = tx.create_bucket_if_not_exists(b"templates")?;
            
            let str = serde_json::to_vec(&template_clone)?;
            
            let id = bucket.next_sequence()?;
            let id = i64::MAX - id as i64;
            
            let mut template_with_id = template_clone;
            template_with_id.id = id as i32;
            
            let str = serde_json::to_vec(&template_with_id)?;
            bucket.put(id.to_be_bytes(), str)?;
            
            Ok(template_with_id)
        }).await?;
        
        // Обновляем vaults
        self.update_template_vaults(template.project_id, new_template.id, template.vaults).await?;
        
        Ok(new_template)
    }

    /// Обновляет шаблон
    pub async fn update_template(&self, mut template: Template) -> Result<()> {
        // Валидация шаблона
        template.validate()?;
        
        self.db.update(|tx| {
            let bucket = tx.bucket(b"templates");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Шаблон не найден".to_string()));
            }
            
            let bucket = bucket.unwrap();
            let key = (i64::MAX - template.id as i64).to_be_bytes();
            
            if bucket.get(key).is_none() {
                return Err(crate::error::Error::NotFound("Шаблон не найден".to_string()));
            }
            
            let str = serde_json::to_vec(&template)?;
            bucket.put(key, str)?;
            
            Ok(())
        }).await?;
        
        // Обновляем vaults
        self.update_template_vaults(template.project_id, template.id, template.vaults).await?;
        
        Ok(())
    }

    /// Устанавливает описание шаблона
    pub async fn set_template_description(&self, project_id: i32, template_id: i32, description: String) -> Result<()> {
        self.db.update(|tx| {
            let bucket = tx.bucket(b"templates");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Шаблон не найден".to_string()));
            }
            
            let bucket = bucket.unwrap();
            let key = (i64::MAX - template_id as i64).to_be_bytes();
            
            if let Some(v) = bucket.get(key) {
                let mut template: Template = serde_json::from_slice(&v)?;
                
                if description.is_empty() {
                    template.description = None;
                } else {
                    template.description = Some(description);
                }
                
                let str = serde_json::to_vec(&template)?;
                bucket.put(key, str)?;
                
                Ok(())
            } else {
                Err(crate::error::Error::NotFound("Шаблон не найден".to_string()))
            }
        }).await
    }

    /// Получает шаблоны с правами
    pub async fn get_templates_with_permissions(&self, project_id: i32, user_id: i32, filter: TemplateFilter, params: RetrieveQueryParams) -> Result<Vec<TemplateWithPerms>> {
        let templates = self.get_templates(project_id, filter, params).await?;
        
        let result = templates.iter().map(|tpl| {
            TemplateWithPerms {
                template: tpl.clone(),
                permissions: Default::default(), // TODO: Заполнить права
            }
        }).collect();
        
        Ok(result)
    }

    /// Получает шаблоны проекта
    pub async fn get_templates(&self, project_id: i32, filter: TemplateFilter, params: RetrieveQueryParams) -> Result<Vec<Template>> {
        let mut templates = Vec::new();
        
        self.db.view(|tx| {
            let bucket = tx.bucket(b"templates");
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
                
                let template: Template = serde_json::from_slice(&v)?;
                
                if template.project_id != project_id {
                    continue;
                }
                
                // Фильтрация по view
                if let Some(view_id) = filter.view_id {
                    // TODO: Проверка принадлежности к view
                    if view_id != 0 {
                        continue;
                    }
                }
                
                templates.push(template);
                n += 1;
                
                if n > params.count {
                    break;
                }
            }
            
            Ok(())
        }).await?;
        
        // Сортировка по имени
        templates.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(templates)
    }

    /// Получает шаблон по ID
    pub async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        self.db.view(|tx| {
            let bucket = tx.bucket(b"templates");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Шаблон не найден".to_string()));
            }
            
            let bucket = bucket.unwrap();
            let key = (i64::MAX - template_id as i64).to_be_bytes();
            
            if let Some(v) = bucket.get(key) {
                let template: Template = serde_json::from_slice(&v)?;
                if template.project_id == project_id {
                    Ok(template)
                } else {
                    Err(crate::error::Error::NotFound("Шаблон не найден".to_string()))
                }
            } else {
                Err(crate::error::Error::NotFound("Шаблон не найден".to_string()))
            }
        }).await
    }

    /// Удаляет шаблон
    pub async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()> {
        self.db.update(|tx| {
            let bucket = tx.bucket(b"templates");
            if bucket.is_none() {
                return Err(crate::error::Error::NotFound("Шаблон не найден".to_string()));
            }
            
            let bucket = bucket.unwrap();
            let key = (i64::MAX - template_id as i64).to_be_bytes();
            
            if bucket.get(key).is_none() {
                return Err(crate::error::Error::NotFound("Шаблон не найден".to_string()));
            }
            
            bucket.remove(key)?;
            
            // Удаляем vaults шаблона
            let vaults_bucket_name = format!("template_vaults_{}", template_id);
            tx.delete_bucket(vaults_bucket_name.as_bytes())?;
            
            // Удаляем роли шаблона
            let roles_bucket_name = format!("template_roles_{}", template_id);
            tx.delete_bucket(roles_bucket_name.as_bytes())?;
            
            Ok(())
        }).await
    }

    /// Получает рефереры шаблона
    pub async fn get_template_refs(&self, project_id: i32, template_id: i32) -> Result<crate::models::ObjectReferrers> {
        // TODO: Реализовать поиск ссылок на шаблон
        Ok(crate::models::ObjectReferrers {
            schedules: vec![],
            tasks: vec![],
        })
    }

    /// Получает права шаблона
    pub async fn get_template_permission(&self, project_id: i32, template_id: i32, user_id: i32) -> Result<String> {
        // TODO: Реализовать получение прав
        Ok("admin".to_string())
    }

    /// Получает роли шаблона
    pub async fn get_template_roles(&self, project_id: i32, template_id: i32) -> Result<Vec<TemplateRolePerm>> {
        let roles = self.get_objects::<TemplateRolePerm>(template_id, "template_roles", RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        }).await?;
        
        Ok(roles)
    }

    /// Создаёт роль шаблона
    pub async fn create_template_role(&self, role: TemplateRolePerm) -> Result<TemplateRolePerm> {
        self.create_object(role.template_id, "template_roles", role).await
    }

    /// Удаляет роль шаблона
    pub async fn delete_template_role(&self, project_id: i32, template_id: i32, role_id: i32) -> Result<()> {
        self.delete_object(template_id, "template_roles", role_id).await
    }

    /// Обновляет роль шаблона
    pub async fn update_template_role(&self, role: TemplateRolePerm) -> Result<()> {
        self.update_object(role.template_id, "template_roles", role).await
    }

    /// Получает роль шаблона
    pub async fn get_template_role(&self, project_id: i32, template_id: i32, role_id: i32) -> Result<TemplateRolePerm> {
        self.get_object(template_id, "template_roles", role_id).await
    }

    /// Обновляет vaults шаблона
    async fn update_template_vaults(&self, project_id: i32, template_id: i32, vaults: Vec<crate::models::TemplateVault>) -> Result<()> {
        // TODO: Реализовать обновление vaults
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_bolt_db() -> BoltStore {
        let path = PathBuf::from("/tmp/test_templates.db");
        BoltStore::new(path).unwrap()
    }

    fn create_test_template(project_id: i32, name: &str) -> Template {
        Template {
            id: 0,
            created: Utc::now(),
            project_id,
            name: name.to_string(),
            playbook: "test.yml".to_string(),
            arguments: None,
            template_type: crate::models::TemplateType::Task,
            survey_vars: None,
            start_version: None,
            build_version: None,
            description: None,
            vaults: vec![],
            tasks: 0,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
        }
    }

    #[tokio::test]
    async fn test_create_template() {
        let db = create_test_bolt_db();
        let template = create_test_template(1, "Test Template");
        
        let result = db.create_template(template).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_template() {
        let db = create_test_bolt_db();
        let template = create_test_template(1, "Test Template");
        let created = db.create_template(template).await.unwrap();
        
        let retrieved = db.get_template(1, created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().name, "Test Template");
    }

    #[tokio::test]
    async fn test_get_templates() {
        let db = create_test_bolt_db();
        
        // Создаём несколько шаблонов
        for i in 0..5 {
            let template = create_test_template(1, &format!("Template {}", i));
            db.create_template(template).await.unwrap();
        }
        
        let params = RetrieveQueryParams {
            offset: 0,
            count: Some(10),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        };
        
        let templates = db.get_templates(1, TemplateFilter { view_id: None }, params).await;
        assert!(templates.is_ok());
        assert!(templates.unwrap().len() >= 5);
    }

    #[tokio::test]
    async fn test_update_template() {
        let db = create_test_bolt_db();
        let template = create_test_template(1, "Test Template");
        let mut created = db.create_template(template).await.unwrap();
        
        created.name = "Updated Template".to_string();
        let result = db.update_template(created).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_template_description() {
        let db = create_test_bolt_db();
        let template = create_test_template(1, "Test Template");
        let created = db.create_template(template).await.unwrap();
        
        let result = db.set_template_description(1, created.id, "Test description".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_template() {
        let db = create_test_bolt_db();
        let template = create_test_template(1, "Test Template");
        let created = db.create_template(template).await.unwrap();
        
        let result = db.delete_template(1, created.id).await;
        assert!(result.is_ok());
        
        let retrieved = db.get_template(1, created.id).await;
        assert!(retrieved.is_err());
    }

    #[tokio::test]
    async fn test_create_template_role() {
        let db = create_test_bolt_db();
        let template = create_test_template(1, "Test Template");
        let created = db.create_template(template).await.unwrap();
        
        let role = TemplateRolePerm {
            id: 0,
            template_id: created.id,
            role_id: 2,
        };
        
        let result = db.create_template_role(role).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_template_roles() {
        let db = create_test_bolt_db();
        let template = create_test_template(1, "Test Template");
        let created = db.create_template(template).await.unwrap();
        
        // Создаём несколько ролей
        for i in 0..3 {
            let role = TemplateRolePerm {
                id: 0,
                template_id: created.id,
                role_id: i + 2,
            };
            db.create_template_role(role).await.unwrap();
        }
        
        let roles = db.get_template_roles(1, created.id).await;
        assert!(roles.is_ok());
        assert!(roles.unwrap().len() >= 3);
    }
}
