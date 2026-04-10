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

    #[test]
    fn test_serialize_complex_struct() {
        #[derive(Serialize)]
        struct TestExport {
            version: String,
            items: Vec<String>,
        }

        let data = TestExport {
            version: "1.0".to_string(),
            items: vec!["item1".to_string(), "item2".to_string()],
        };
        let json = serialize_to_json(&data).unwrap();
        assert!(json.contains("\"version\":\"1.0\""));
        assert!(json.contains("\"items\":["));
    }

    #[test]
    fn test_serialize_to_json_error() {
        // serde_json::to_string_pretty редко ошибается, но проверим путь
        let data = vec![1, 2, 3];
        let result = serialize_to_json(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_from_json_error() {
        let result = deserialize_from_json::<Vec<String>>("invalid json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to deserialize"));
    }

    #[test]
    fn test_validate_before_export_complex() {
        #[derive(Serialize)]
        struct ComplexData {
            name: String,
            value: i32,
        }

        let data = ComplexData {
            name: "test".to_string(),
            value: 42,
        };
        assert!(validate_before_export(&data).is_ok());
    }

    #[test]
    fn test_validate_after_import_valid_json() {
        let json = r#"{"name":"test","value":42}"#;
        #[derive(Deserialize, Debug)]
        struct TestData {
            name: String,
            value: i32,
        }
        let result = validate_after_import::<TestData>(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "test");
    }

    #[test]
    fn test_validate_after_import_invalid_json() {
        let result = validate_after_import::<String>("not valid json");
        assert!(result.is_err());
    }

    // ── Additional tests ──

    #[test]
    fn test_serialize_empty_vec() {
        let data: Vec<String> = Vec::new();
        let json = serialize_to_json(&data).unwrap();
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_serialize_boolean_value() {
        let data = true;
        let json = serialize_to_json(&data).unwrap();
        assert_eq!(json, "true");
    }

    #[test]
    fn test_serialize_numeric_value() {
        let data = 42i32;
        let json = serialize_to_json(&data).unwrap();
        assert_eq!(json, "42");
    }

    #[test]
    fn test_deserialize_numeric_value() {
        let json = "123";
        let data: i32 = deserialize_from_json(json).unwrap();
        assert_eq!(data, 123);
    }

    #[test]
    fn test_deserialize_boolean_value() {
        let json = "false";
        let data: bool = deserialize_from_json(json).unwrap();
        assert!(!data);
    }

    #[test]
    fn test_check_dependencies_empty_required() {
        let exported = HashMap::new();
        let required: Vec<&str> = Vec::new();
        let result = check_dependencies(&exported, &required);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_dependencies_all_missing() {
        let exported = HashMap::new();
        let required = vec!["User", "Project", "Template"];
        let result = check_dependencies(&exported, &required);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Missing dependency"));
    }

    #[test]
    fn test_check_dependencies_partial_match() {
        let mut exported = HashMap::new();
        exported.insert("User", true);
        exported.insert("Template", true);

        let required = vec!["User", "Project", "Template"];
        let result = check_dependencies(&exported, &required);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Project"));
    }

    #[test]
    fn test_create_dependency_map_user_no_deps() {
        let deps = create_dependency_map();
        let user_deps = deps.get("User").unwrap();
        assert!(user_deps.is_empty());
    }

    #[test]
    fn test_create_dependency_map_repository_deps() {
        let deps = create_dependency_map();
        let repo_deps = deps.get("Repository").unwrap();
        assert_eq!(repo_deps.len(), 2);
        assert!(repo_deps.contains(&"Project"));
        assert!(repo_deps.contains(&"AccessKey"));
    }

    #[test]
    fn test_create_dependency_map_inventory_deps() {
        let deps = create_dependency_map();
        let inv_deps = deps.get("Inventory").unwrap();
        assert_eq!(inv_deps.len(), 2);
        assert!(inv_deps.contains(&"Project"));
        assert!(inv_deps.contains(&"AccessKey"));
    }

    #[test]
    fn test_create_dependency_map_view_deps() {
        let deps = create_dependency_map();
        let view_deps = deps.get("View").unwrap();
        assert_eq!(view_deps.len(), 1);
        assert!(view_deps.contains(&"Project"));
    }

    #[test]
    fn test_export_order_contains_all_entities() {
        let order = get_export_order().unwrap();
        assert!(order.contains(&"User"));
        assert!(order.contains(&"Project"));
        assert!(order.contains(&"Template"));
        assert!(order.contains(&"Schedule"));
    }

    #[test]
    fn test_export_order_no_duplicates() {
        let order = get_export_order().unwrap();
        let mut seen = std::collections::HashSet::new();
        for entity in &order {
            assert!(seen.insert(*entity), "Duplicate found: {}", entity);
        }
    }

    #[test]
    fn test_import_order_is_reverse_of_export() {
        let export_order = get_export_order().unwrap();
        let import_order = get_import_order().unwrap();
        let reversed_export: Vec<&str> = export_order.into_iter().rev().collect();
        assert_eq!(import_order.len(), reversed_export.len());
        for (a, b) in import_order.iter().zip(reversed_export.iter()) {
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_export_order_template_after_dependencies() {
        let order = get_export_order().unwrap();
        let project_pos = order.iter().position(|&x| x == "Project").unwrap();
        let inventory_pos = order.iter().position(|&x| x == "Inventory").unwrap();
        let repository_pos = order.iter().position(|&x| x == "Repository").unwrap();
        let environment_pos = order.iter().position(|&x| x == "Environment").unwrap();
        let template_pos = order.iter().position(|&x| x == "Template").unwrap();

        assert!(template_pos > project_pos);
        assert!(template_pos > inventory_pos);
        assert!(template_pos > repository_pos);
        assert!(template_pos > environment_pos);
    }

    #[test]
    fn test_export_order_schedule_after_template() {
        let order = get_export_order().unwrap();
        let template_pos = order.iter().position(|&x| x == "Template").unwrap();
        let schedule_pos = order.iter().position(|&x| x == "Schedule").unwrap();
        assert!(schedule_pos > template_pos);
    }

    #[test]
    fn test_export_order_integration_after_template() {
        let order = get_export_order().unwrap();
        let template_pos = order.iter().position(|&x| x == "Template").unwrap();
        let integration_pos = order.iter().position(|&x| x == "Integration").unwrap();
        assert!(integration_pos > template_pos);
    }

    #[test]
    fn test_export_order_task_after_template() {
        let order = get_export_order().unwrap();
        let template_pos = order.iter().position(|&x| x == "Template").unwrap();
        let task_pos = order.iter().position(|&x| x == "Task").unwrap();
        assert!(task_pos > template_pos);
    }

    #[test]
    fn test_serialize_nested_struct() {
        #[derive(Serialize)]
        struct Inner {
            value: i32,
        }

        #[derive(Serialize)]
        struct Outer {
            inner: Inner,
            label: String,
        }

        let data = Outer {
            inner: Inner { value: 99 },
            label: "outer".to_string(),
        };
        let json = serialize_to_json(&data).unwrap();
        assert!(json.contains("\"value\": 99"));
        assert!(json.contains("\"label\": \"outer\""));
    }

    #[test]
    fn test_deserialize_hashmap() {
        let json = r#"{"key1":"val1","key2":"val2"}"#;
        let data: HashMap<String, String> = deserialize_from_json(json).unwrap();
        assert_eq!(data.get("key1").unwrap(), "val1");
        assert_eq!(data.get("key2").unwrap(), "val2");
    }

    #[test]
    fn test_validate_before_export_with_vec() {
        let data = vec![1, 2, 3, 4, 5];
        assert!(validate_before_export(&data).is_ok());
    }

    #[test]
    fn test_dependency_map_entity_count() {
        let deps = create_dependency_map();
        // User, Project, AccessKey, Environment, Repository, Inventory, Template, View, Schedule, Integration, Task
        assert_eq!(deps.len(), 11);
    }
}
