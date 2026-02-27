//! Exporter Utils - утилиты для экспорта
//!
//! Аналог services/export/ утилит из Go версии

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Сериализует данные в JSON
pub fn serialize_to_json<T: Serialize>(data: &T) -> Result<String, String> {
    serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize to JSON: {}", e))
}

/// Десериализует данные из JSON
pub fn deserialize_from_json<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, String> {
    serde_json::from_str(json)
        .map_err(|e| format!("Failed to deserialize from JSON: {}", e))
}

/// Проверяет зависимости между сущностями
pub fn check_dependencies(exported: &HashMap<String, bool>, required: &[&str]) -> Result<(), String> {
    for dep in required {
        if !exported.get(*dep).unwrap_or(&false) {
            return Err(format!("Missing dependency: {}", dep));
        }
    }
    Ok(())
}

/// Валидирует данные перед экспортом
pub fn validate_before_export<T: Serialize>(data: &T) -> Result<(), String> {
    // Проверяем что данные можно сериализовать
    serde_json::to_string(data)
        .map_err(|e| format!("Validation failed: {}", e))?;
    Ok(())
}

/// Валидирует данные после импорта
pub fn validate_after_import<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, String> {
    deserialize_from_json(json)
}

/// Создаёт мапу зависимостей
pub fn create_dependency_map() -> HashMap<&'static str, Vec<&'static str>> {
    let mut deps = HashMap::new();
    
    // User не имеет зависимостей
    deps.insert("User", vec![]);
    
    // Project зависит от User
    deps.insert("Project", vec!["User"]);
    
    // AccessKey зависит от Project
    deps.insert("AccessKey", vec!["Project"]);
    
    // Environment зависит от Project
    deps.insert("Environment", vec!["Project"]);
    
    // Repository зависит от Project и AccessKey
    deps.insert("Repository", vec!["Project", "AccessKey"]);
    
    // Inventory зависит от Project и AccessKey
    deps.insert("Inventory", vec!["Project", "AccessKey"]);
    
    // Template зависит от Project, Inventory, Repository, Environment
    deps.insert("Template", vec!["Project", "Inventory", "Repository", "Environment"]);
    
    // View зависит от Project
    deps.insert("View", vec!["Project"]);
    
    // Schedule зависит от Project и Template
    deps.insert("Schedule", vec!["Project", "Template"]);
    
    // Integration зависит от Project и Template
    deps.insert("Integration", vec!["Project", "Template"]);
    
    // Task зависит от Project и Template
    deps.insert("Task", vec!["Project", "Template"]);
    
    deps
}

/// Получает порядок экспорта на основе зависимостей
pub fn get_export_order() -> Result<Vec<&'static str>, String> {
    let deps = create_dependency_map();
    let mut order = Vec::new();
    let mut visited = std::collections::HashSet::new();
    
    fn visit(
        name: &str,
        deps: &HashMap<&str, Vec<&str>>,
        order: &mut Vec<&str>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<(), String> {
        if visited.contains(name) {
            return Ok(());
        }
        
        visited.insert(name.to_string());
        
        if let Some(dependencies) = deps.get(name) {
            for dep in dependencies {
                visit(dep, deps, order, visited)?;
            }
        }
        
        order.push(name);
        Ok(())
    }
    
    for name in deps.keys() {
        visit(name, &deps, &mut order, &mut visited)?;
    }
    
    Ok(order)
}

/// Получает порядок импорта (обратный экспорту)
pub fn get_import_order() -> Result<Vec<&'static str>, String> {
    let mut order = get_export_order()?;
    order.reverse();
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_to_json() {
        let data = vec!["a".to_string(), "b".to_string()];
        let json = serialize_to_json(&data).unwrap();
        assert!(json.contains("a"));
        assert!(json.contains("b"));
    }

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"["a", "b"]"#;
        let data: Vec<String> = deserialize_from_json(json).unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], "a");
        assert_eq!(data[1], "b");
    }

    #[test]
    fn test_check_dependencies_success() {
        let mut exported = HashMap::new();
        exported.insert("User", true);
        exported.insert("Project", true);
        
        let required = vec!["User", "Project"];
        let result = check_dependencies(&exported, &required);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_dependencies_missing() {
        let mut exported = HashMap::new();
        exported.insert("User", true);
        
        let required = vec!["User", "Project"];
        let result = check_dependencies(&exported, &required);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Project"));
    }

    #[test]
    fn test_validate_before_export() {
        let data = "test".to_string();
        let result = validate_before_export(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_dependency_map() {
        let deps = create_dependency_map();
        
        assert!(deps.contains_key("User"));
        assert!(deps.contains_key("Project"));
        assert!(deps.contains_key("Template"));
        
        // Проверяем зависимости Template
        let template_deps = deps.get("Template").unwrap();
        assert!(template_deps.contains(&"Project"));
        assert!(template_deps.contains(&"Inventory"));
        assert!(template_deps.contains(&"Repository"));
        assert!(template_deps.contains(&"Environment"));
    }

    #[test]
    fn test_get_export_order() {
        let order = get_export_order().unwrap();
        
        // User должен быть первым (нет зависимостей)
        assert_eq!(order[0], "User");
        
        // Project должен быть после User
        let user_pos = order.iter().position(|&x| x == "User").unwrap();
        let project_pos = order.iter().position(|&x| x == "Project").unwrap();
        assert!(project_pos > user_pos);
        
        // Template должен быть после Project, Inventory, Repository, Environment
        let template_pos = order.iter().position(|&x| x == "Template").unwrap();
        assert!(template_pos > project_pos);
    }

    #[test]
    fn test_get_import_order() {
        let import_order = get_import_order().unwrap();
        let export_order = get_export_order().unwrap();
        
        // Импорт должен быть в обратном порядке
        assert_eq!(import_order[0], *export_order.last().unwrap());
        assert_eq!(import_order.last().unwrap(), export_order[0]);
    }
}
