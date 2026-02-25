//! Модуль конвертации типов
//!
//! Предоставляет функции для конвертации между типами данных:
//! - Конвертация float в int
//! - Преобразование структур в плоские карты

use serde::Serialize;
use serde_json::{Map, Value};

/// Конвертирует float в int, если это возможно
///
/// Возвращает Some(i64), если float представляет целое число,
/// иначе возвращает None
pub fn convert_float_to_int_if_possible(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(i)
            } else if let Some(f) = n.as_f64() {
                let i = f as i64;
                if (f - i as f64).abs() < f64::EPSILON {
                    Some(i)
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Преобразует структуру в плоскую HashMap
///
/// Рекурсивно обходит структуру и создаёт плоское представление
/// с ключами в формате "field.subfield.value"
pub fn struct_to_flat_map(obj: &impl Serialize) -> Map<String, Value> {
    let mut result = Map::new();
    
    // Сериализуем объект в Value
    let value = serde_json::to_value(obj).ok().unwrap_or(Value::Null);
    
    // Рекурсивно flattening
    flatten_value(&value, "", &mut result);
    
    result
}

/// Рекурсивно flattening Value в плоскую карту
fn flatten_value(value: &Value, prefix: &str, result: &mut Map<String, Value>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let new_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_value(val, &new_key, result);
            }
        }
        Value::Array(arr) => {
            for (index, item) in arr.iter().enumerate() {
                let new_key = if prefix.is_empty() {
                    format!("{}", index)
                } else {
                    format!("{}.{}", prefix, index)
                };
                flatten_value(item, &new_key, result);
            }
        }
        Value::Null => {
            let key = if prefix.is_empty() {
                "value".to_string()
            } else {
                prefix.to_string()
            };
            result.insert(key, Value::Null);
        }
        _ => {
            let key = if prefix.is_empty() {
                "value".to_string()
            } else {
                prefix.to_string()
            };
            result.insert(key, value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_convert_float_to_int_whole_number() {
        let value = json!(5.0);
        assert_eq!(convert_float_to_int_if_possible(&value), Some(5));
    }

    #[test]
    fn test_convert_float_to_int_fractional_number() {
        let value = json!(5.5);
        assert_eq!(convert_float_to_int_if_possible(&value), None);
    }

    #[test]
    fn test_convert_float_to_int_integer() {
        let value = json!(5);
        assert_eq!(convert_float_to_int_if_possible(&value), Some(5));
    }

    #[test]
    fn test_convert_float_to_int_null() {
        let value = json!(null);
        assert_eq!(convert_float_to_int_if_possible(&value), None);
    }

    #[test]
    fn test_struct_to_flat_map_simple() {
        #[derive(Serialize)]
        struct TestStruct {
            name: String,
            age: i32,
        }

        let obj = TestStruct {
            name: "John".to_string(),
            age: 30,
        };

        let flat = struct_to_flat_map(&obj);
        
        assert_eq!(flat.get("name"), Some(&Value::String("John".to_string())));
        assert_eq!(flat.get("age"), Some(&Value::Number(30.into())));
    }

    #[test]
    fn test_struct_to_flat_map_nested() {
        #[derive(Serialize)]
        struct Address {
            city: String,
            zip: String,
        }

        #[derive(Serialize)]
        struct Person {
            name: String,
            address: Address,
        }

        let obj = Person {
            name: "John".to_string(),
            address: Address {
                city: "Moscow".to_string(),
                zip: "101000".to_string(),
            },
        };

        let flat = struct_to_flat_map(&obj);
        
        assert_eq!(flat.get("name"), Some(&Value::String("John".to_string())));
        assert_eq!(flat.get("address.city"), Some(&Value::String("Moscow".to_string())));
        assert_eq!(flat.get("address.zip"), Some(&Value::String("101000".to_string())));
    }

    #[test]
    fn test_struct_to_flat_map_with_null() {
        #[derive(Serialize)]
        struct TestStruct {
            name: Option<String>,
            value: i32,
        }

        let obj = TestStruct {
            name: None,
            value: 42,
        };

        let flat = struct_to_flat_map(&obj);
        
        assert!(flat.get("name").is_some());
        assert_eq!(flat.get("value"), Some(&Value::Number(42.into())));
    }
}
