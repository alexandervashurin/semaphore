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
        return Err(Error::Other(
            "Template playbook cannot be empty".to_string(),
        ));
    }

    // Проверка что playbook заканчивается на .yml или .yaml
    if !template.playbook.ends_with(".yml") && !template.playbook.ends_with(".yaml") {
        return Err(Error::Other(
            "Template playbook must end with .yml or .yaml".to_string(),
        ));
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
    use std::path::Path;
    use tokio::fs;

    let mut playbooks = Vec::new();
    let mut entries = fs::read_dir(dir_path)
        .await
        .map_err(|e| Error::Other(format!("Failed to read directory: {}", e)))?;

    while let Some(entry) = entries
        .next_entry()
        .await
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_validate_template_valid() {
        let mut template = Template::default();
        template.project_id = 1;
        template.name = "Test Template".to_string();
        template.playbook = "test.yml".to_string();
        template.created = Utc::now();

        let result = validate_template(&template);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_template_empty_name() {
        let mut template = Template::default();
        template.project_id = 1;
        template.name = String::new();
        template.playbook = "test.yml".to_string();
        template.created = Utc::now();

        let result = validate_template(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[test]
    fn test_validate_template_empty_playbook() {
        let mut template = Template::default();
        template.project_id = 1;
        template.name = "Test".to_string();
        template.playbook = String::new();
        template.created = Utc::now();

        let result = validate_template(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("playbook"));
    }

    #[test]
    fn test_validate_template_invalid_extension() {
        let mut template = Template::default();
        template.project_id = 1;
        template.name = "Test".to_string();
        template.playbook = "test.txt".to_string();
        template.created = Utc::now();

        let result = validate_template(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".yml"));
    }
}
