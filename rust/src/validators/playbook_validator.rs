//! Валидация Playbook
//!
//! Модуль для валидации содержимого playbook файлов (Ansible, Terraform, Shell)

use serde_yaml::Value;
use thiserror::Error;

/// Ошибки валидации playbook
#[derive(Debug, Error)]
pub enum PlaybookValidationError {
    /// Ошибка парсинга YAML
    #[error("YAML парсинг: {0}")]
    YamlParse(String),

    /// Неверная структура playbook
    #[error("Неверная структура: {0}")]
    InvalidStructure(String),

    /// Отсутствует обязательное поле
    #[error("Отсутствует обязательное поле: {0}")]
    MissingField(String),

    /// Неверный тип поля
    #[error("Неверный тип поля {0}: {1}")]
    InvalidFieldType(String, String),

    /// Неверный тип playbook
    #[error("Неверный тип playbook: {0}")]
    InvalidPlaybookType(String),

    /// Превышен максимальный размер
    #[error("Превышен максимальный размер: {0} байт")]
    MaxSizeExceeded(usize),
}

/// Результат валидации
pub type ValidationResult = Result<(), PlaybookValidationError>;

/// Максимальный размер playbook (10 MB)
const MAX_PLAYBOOK_SIZE: usize = 10 * 1024 * 1024;

/// Валидатор playbook
pub struct PlaybookValidator;

impl PlaybookValidator {
    /// Валидирует playbook в зависимости от типа
    pub fn validate(content: &str, playbook_type: &str) -> ValidationResult {
        // Проверка размера
        if content.len() > MAX_PLAYBOOK_SIZE {
            return Err(PlaybookValidationError::MaxSizeExceeded(content.len()));
        }

        match playbook_type {
            "ansible" => Self::validate_ansible_playbook(content),
            "terraform" => Self::validate_terraform_config(content),
            "shell" => Self::validate_shell_script(content),
            _ => Err(PlaybookValidationError::InvalidPlaybookType(
                playbook_type.to_string(),
            )),
        }
    }

    /// Валидирует Ansible playbook
    ///
    /// Ansible playbook должен быть списком plays, где каждый play:
    /// - hosts: список хостов
    /// - tasks: список задач (опционально)
    /// - roles: список ролей (опционально)
    pub fn validate_ansible_playbook(content: &str) -> ValidationResult {
        // Парсинг YAML
        let playbook: Value = serde_yaml::from_str(content).map_err(|e| {
            PlaybookValidationError::YamlParse(e.to_string())
        })?;

        // Playbook должен быть списком
        let plays = playbook.as_sequence().ok_or_else(|| {
            PlaybookValidationError::InvalidStructure(
                "Playbook должен быть списком plays (YAML sequence)".to_string(),
            )
        })?;

        if plays.is_empty() {
            return Err(PlaybookValidationError::InvalidStructure(
                "Playbook не может быть пустым".to_string(),
            ));
        }

        // Валидация каждого play
        for (index, play) in plays.iter().enumerate() {
            Self::validate_ansible_play(play, index)?;
        }

        Ok(())
    }

    /// Валидирует отдельный play в Ansible playbook
    fn validate_ansible_play(play: &Value, index: usize) -> ValidationResult {
        // Play должен быть мапой
        let play_map = play.as_mapping().ok_or_else(|| {
            PlaybookValidationError::InvalidStructure(format!(
                "Play #{} должен быть объектом (YAML mapping)",
                index + 1
            ))
        })?;

        // Проверка обязательного поля hosts
        if !play_map.contains_key(&Value::String("hosts".to_string())) {
            return Err(PlaybookValidationError::MissingField(format!(
                "Play #{}: hosts",
                index + 1
            )));
        }

        // Проверка типа поля hosts
        let hosts_value = play_map.get(&Value::String("hosts".to_string())).unwrap();
        if !hosts_value.is_string() && !hosts_value.is_sequence() {
            return Err(PlaybookValidationError::InvalidFieldType(
                format!("Play #{}: hosts", index + 1),
                "должен быть строкой или списком".to_string(),
            ));
        }

        // Проверка tasks (если есть)
        if let Some(tasks) = play_map.get(&Value::String("tasks".to_string())) {
            if let Some(tasks_seq) = tasks.as_sequence() {
                for (task_idx, task) in tasks_seq.iter().enumerate() {
                    if !task.is_mapping() {
                        return Err(PlaybookValidationError::InvalidStructure(format!(
                            "Play #{} Task #{} должен быть объектом",
                            index + 1,
                            task_idx + 1
                        )));
                    }
                }
            }
        }

        // Проверка roles (если есть)
        if let Some(roles) = play_map.get(&Value::String("roles".to_string())) {
            if !roles.is_sequence() {
                return Err(PlaybookValidationError::InvalidFieldType(
                    format!("Play #{}: roles", index + 1),
                    "должен быть списком".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Валидирует Terraform конфигурацию
    ///
    /// Terraform config должен содержать:
    /// - resource (опционально)
    /// - variable (опционально)
    /// - output (опционально)
    /// - module (опционально)
    /// - provider (опционально)
    pub fn validate_terraform_config(content: &str) -> ValidationResult {
        // Парсинг HCL через YAML (упрощенная валидация)
        // В идеале нужно использовать hcl-rs для парсинга HCL
        let config: Value = serde_yaml::from_str(content).map_err(|e| {
            PlaybookValidationError::YamlParse(format!(
                "Terraform config должен быть валидным YAML/HCL: {}",
                e
            ))
        })?;

        // Конфигурация должна быть мапой
        if !config.is_mapping() && !config.is_null() {
            return Err(PlaybookValidationError::InvalidStructure(
                "Terraform config должен быть объектом".to_string(),
            ));
        }

        // Если это не null, проверяем структуру
        if let Some(config_map) = config.as_mapping() {
            // Допустимые ключи верхнего уровня в Terraform
            let valid_keys = [
                "resource",
                "variable",
                "output",
                "module",
                "provider",
                "data",
                "locals",
                "terraform",
            ];

            for key in config_map.keys() {
                if let Value::String(key_str) = key {
                    if !valid_keys.contains(&key_str.as_str()) {
                        // Предупреждение, но не ошибка
                        tracing::warn!(
                            "Необычный ключ верхнего уровня в Terraform: {}",
                            key_str
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Валидирует Shell скрипт
    ///
    /// Простая валидация:
    /// - Не пустой
    /// - Содержит shebang (опционально, но рекомендуется)
    pub fn validate_shell_script(content: &str) -> ValidationResult {
        if content.trim().is_empty() {
            return Err(PlaybookValidationError::InvalidStructure(
                "Shell скрипт не может быть пустым".to_string(),
            ));
        }

        // Проверка на наличие shebang (рекомендуется)
        if !content.starts_with("#!") {
            tracing::warn!("Shell скрипт не содержит shebang (#!/bin/bash)");
        }

        Ok(())
    }

    /// Быстрая проверка YAML синтаксиса без полной валидации
    pub fn check_yaml_syntax(content: &str) -> Result<(), String> {
        serde_yaml::from_str::<Value>(content)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ansible_playbook() {
        let content = r#"
- hosts: all
  tasks:
    - name: Test task
      debug:
        msg: Hello
"#;
        assert!(PlaybookValidator::validate_ansible_playbook(content).is_ok());
    }

    #[test]
    fn test_missing_hosts() {
        let content = r#"
- tasks:
    - name: Test task
      debug:
        msg: Hello
"#;
        let result = PlaybookValidator::validate_ansible_playbook(content);
        assert!(matches!(
            result,
            Err(PlaybookValidationError::MissingField(_))
        ));
    }

    #[test]
    fn test_invalid_yaml() {
        let content = r#"
- hosts: all
  tasks:
    - name: Test
      debug:
        msg: Hello
  invalid yaml: [
"#;
        let result = PlaybookValidator::validate_ansible_playbook(content);
        assert!(matches!(
            result,
            Err(PlaybookValidationError::YamlParse(_))
        ));
    }

    #[test]
    fn test_empty_playbook() {
        let content = "[]";
        let result = PlaybookValidator::validate_ansible_playbook(content);
        assert!(matches!(
            result,
            Err(PlaybookValidationError::InvalidStructure(_))
        ));
    }

    #[test]
    fn test_valid_shell_script() {
        let content = r#"#!/bin/bash
echo "Hello World"
"#;
        assert!(PlaybookValidator::validate_shell_script(content).is_ok());
    }

    #[test]
    fn test_empty_shell_script() {
        let content = "";
        let result = PlaybookValidator::validate_shell_script(content);
        assert!(matches!(
            result,
            Err(PlaybookValidationError::InvalidStructure(_))
        ));
    }

    #[test]
    fn test_max_size() {
        let content = "x".repeat(MAX_PLAYBOOK_SIZE + 1);
        let result = PlaybookValidator::validate(&content, "ansible");
        assert!(matches!(
            result,
            Err(PlaybookValidationError::MaxSizeExceeded(_))
        ));
    }

    #[test]
    fn test_invalid_playbook_type() {
        let content = "test";
        let result = PlaybookValidator::validate(content, "invalid_type");
        assert!(matches!(
            result,
            Err(PlaybookValidationError::InvalidPlaybookType(_))
        ));
    }
}
