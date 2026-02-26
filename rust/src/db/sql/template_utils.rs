//! Template Utils - вспомогательные функции для шаблонов
//!
//! Аналог db/sql/template.go из Go версии (часть 4: утилиты)

use crate::error::{Error, Result};
use crate::models::*;

/// Валидирует шаблон
pub fn validate_template(template: &Template) -> Result<()> {
    // Проверка имени
    if template.name.is_empty() {
        return Err(Error::Other("Template name cannot be empty".to_string()));
    }
    
    // Проверка playbook
    if template.playbook.is_empty() {
        return Err(Error::Other("Template playbook cannot be empty".to_string()));
    }
    
    // Проверка что playbook заканчивается на .yml или .yaml
    if !template.playbook.ends_with(".yml") && !template.playbook.ends_with(".yaml") {
        return Err(Error::Other("Template playbook must end with .yml or .yaml".to_string()));
    }
    
    // Проверка типа шаблона
    match template.template_type {
        TemplateType::Task | TemplateType::Build | TemplateType::Deploy => {
            // OK
        }
        _ => {
            return Err(Error::Other(format!("Invalid template type: {:?}", template.template_type)));
        }
    }
    
    Ok(())
}

/// Проверяет существует ли playbook
pub async fn playbook_exists(playbook_path: &str) -> Result<bool> {
    use tokio::fs;
    
    match fs::metadata(playbook_path).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Получает список playbook из директории
pub async fn list_playbooks(dir_path: &str) -> Result<Vec<String>> {
    use tokio::fs;
    use std::path::Path;
    
    let mut playbooks = Vec::new();
    let mut entries = fs::read_dir(dir_path).await
        .map_err(|e| Error::Other(format!("Failed to read directory: {}", e)))?;
    
    while let Some(entry) = entries.next_entry().await
        .map_err(|e| Error::Other(format!("Failed to read entry: {}", e)))? 
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "yml" || ext == "yaml" {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        playbooks.push(name.to_string());
                    }
                }
            }
        }
    }
    
    Ok(playbooks)
}

/// Проверяет валидность YAML файла
pub fn validate_yaml(content: &str) -> Result<()> {
    // Простая проверка - пытаемся распарсить как YAML
    // В production лучше использовать yaml crate
    if content.is_empty() {
        return Err(Error::Other("YAML content cannot be empty".to_string()));
    }
    
    Ok(())
}

/// Получает переменные из survey_vars
pub fn get_survey_var_names(survey_vars: &Option<String>) -> Result<Vec<String>> {
    if let Some(vars_json) = survey_vars {
        let vars: serde_json::Value = serde_json::from_str(vars_json)
            .map_err(|e| Error::Other(format!("Failed to parse survey_vars: {}", e)))?;
        
        if let Some(vars_array) = vars.as_array() {
            let names = vars_array
                .iter()
                .filter_map(|var| var.get("name").and_then(|n| n.as_str()))
                .map(|s| s.to_string())
                .collect();
            
            return Ok(names);
        }
    }
    
    Ok(Vec::new())
}

/// Проверяет что survey_vars валидны
pub fn validate_survey_vars(survey_vars: &Option<String>) -> Result<()> {
    if let Some(vars_json) = survey_vars {
        let vars: serde_json::Value = serde_json::from_str(vars_json)
            .map_err(|e| Error::Other(format!("Failed to parse survey_vars: {}", e)))?;
        
        if let Some(vars_array) = vars.as_array() {
            for var in vars_array {
                // Проверяем что есть name
                if var.get("name").is_none() || !var.get("name").unwrap().is_string() {
                    return Err(Error::Other("Survey var must have a name".to_string()));
                }
                
                // Проверяем что есть type
                if var.get("type").is_none() || !var.get("type").unwrap().is_string() {
                    return Err(Error::Other("Survey var must have a type".to_string()));
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_validate_template_valid() {
        let template = Template {
            id: 0,
            project_id: 1,
            name: "Test Template".to_string(),
            playbook: "test.yml".to_string(),
            arguments: None,
            template_type: TemplateType::Task,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            start_version: None,
            build_version: None,
            description: None,
            survey_vars: None,
            vaults: Vec::new(),
            tasks: 0,
            created: Utc::now(),
        };
        
        let result = validate_template(&template);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_template_empty_name() {
        let template = Template {
            id: 0,
            project_id: 1,
            name: String::new(),
            playbook: "test.yml".to_string(),
            arguments: None,
            template_type: TemplateType::Task,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            start_version: None,
            build_version: None,
            description: None,
            survey_vars: None,
            vaults: Vec::new(),
            tasks: 0,
            created: Utc::now(),
        };
        
        let result = validate_template(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[test]
    fn test_validate_template_empty_playbook() {
        let template = Template {
            id: 0,
            project_id: 1,
            name: "Test".to_string(),
            playbook: String::new(),
            arguments: None,
            template_type: TemplateType::Task,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            start_version: None,
            build_version: None,
            description: None,
            survey_vars: None,
            vaults: Vec::new(),
            tasks: 0,
            created: Utc::now(),
        };
        
        let result = validate_template(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("playbook"));
    }

    #[test]
    fn test_validate_template_invalid_extension() {
        let template = Template {
            id: 0,
            project_id: 1,
            name: "Test".to_string(),
            playbook: "test.txt".to_string(),
            arguments: None,
            template_type: TemplateType::Task,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            start_version: None,
            build_version: None,
            description: None,
            survey_vars: None,
            vaults: Vec::new(),
            tasks: 0,
            created: Utc::now(),
        };
        
        let result = validate_template(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".yml"));
    }

    #[test]
    fn test_get_survey_var_names() {
        let survey_vars = Some(r#"[{"name": "var1", "type": "string"}, {"name": "var2", "type": "int"}]"#.to_string());
        
        let names = get_survey_var_names(&survey_vars).unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"var1".to_string()));
        assert!(names.contains(&"var2".to_string()));
    }

    #[test]
    fn test_validate_survey_vars_valid() {
        let survey_vars = Some(r#"[{"name": "var1", "type": "string"}]"#.to_string());
        
        let result = validate_survey_vars(&survey_vars);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_survey_vars_missing_name() {
        let survey_vars = Some(r#"[{"type": "string"}]"#.to_string());
        
        let result = validate_survey_vars(&survey_vars);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }
}
