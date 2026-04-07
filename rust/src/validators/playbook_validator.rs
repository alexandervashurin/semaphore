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
    /// Принимает любой валидный YAML — список plays, одиночный play, или имя файла.
    pub fn validate_ansible_playbook(content: &str) -> ValidationResult {
        if content.trim().is_empty() {
            return Err(PlaybookValidationError::InvalidStructure(
                "Содержимое playbook не может быть пустым".to_string(),
            ));
        }
        // Если это имя файла (.yml/.yaml/.sh) — пропускаем YAML парсинг
        let trimmed = content.trim();
        if !trimmed.contains('\n')
            && (trimmed.ends_with(".yml") || trimmed.ends_with(".yaml") || trimmed.ends_with(".sh"))
        {
            return Ok(());
        }
        // Проверяем только синтаксис YAML, не структуру
        serde_yaml::from_str::<Value>(content)
            .map(|_| ())
            .map_err(|e| PlaybookValidationError::YamlParse(e.to_string()))
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
        if !play_map.contains_key(Value::String("hosts".to_string())) {
            return Err(PlaybookValidationError::MissingField(format!(
                "Play #{}: hosts",
                index + 1
            )));
        }

        // Проверка типа поля hosts
        let hosts_value = play_map.get(Value::String("hosts".to_string())).unwrap();
        if !hosts_value.is_string() && !hosts_value.is_sequence() {
            return Err(PlaybookValidationError::InvalidFieldType(
                format!("Play #{}: hosts", index + 1),
                "должен быть строкой или списком".to_string(),
            ));
        }

        // Проверка tasks (если есть)
        if let Some(tasks) = play_map.get(Value::String("tasks".to_string())) {
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
        if let Some(roles) = play_map.get(Value::String("roles".to_string())) {
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
                        tracing::warn!("Необычный ключ верхнего уровня в Terraform: {}", key_str);
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
        // Lenient validator now accepts any valid YAML, including plays without hosts
        let content = r#"
- tasks:
    - name: Test task
      debug:
        msg: Hello
"#;
        let result = PlaybookValidator::validate_ansible_playbook(content);
        assert!(result.is_ok());
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
        assert!(matches!(result, Err(PlaybookValidationError::YamlParse(_))));
    }

    #[test]
    fn test_empty_playbook() {
        // Lenient validator accepts valid YAML (empty array is valid YAML)
        let content = "[]";
        let result = PlaybookValidator::validate_ansible_playbook(content);
        assert!(result.is_ok());
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

    #[test]
    fn test_validate_ansible_playbook_filename_only() {
        // Filename-only should pass
        assert!(PlaybookValidator::validate_ansible_playbook("deploy.yml").is_ok());
        assert!(PlaybookValidator::validate_ansible_playbook("site.yaml").is_ok());
        assert!(PlaybookValidator::validate_ansible_playbook("run.sh").is_ok());
    }

    #[test]
    fn test_validate_ansible_play_play_structure() {
        let content = r#"
- hosts: all
  tasks:
    - name: Step 1
      debug:
        msg: "Hello"
    - name: Step 2
      command: echo test
"#;
        assert!(PlaybookValidator::validate_ansible_playbook(content).is_ok());
    }

    #[test]
    fn test_validate_ansible_play_with_roles() {
        let content = r#"
- hosts: webservers
  roles:
    - common
    - webserver
  tasks:
    - name: Extra task
      debug:
        msg: "ok"
"#;
        assert!(PlaybookValidator::validate_ansible_playbook(content).is_ok());
    }

    #[test]
    fn test_validate_ansible_play_hosts_as_list() {
        let content = r#"
- hosts:
    - web1
    - web2
  tasks:
    - debug:
        msg: "ok"
"#;
        assert!(PlaybookValidator::validate_ansible_playbook(content).is_ok());
    }

    #[test]
    fn test_validate_ansible_playbook_invalid_structure_not_list() {
        // Scalar value is valid YAML but not a playbook list
        let content = "just_a_string";
        // Lenient validator accepts any valid YAML
        assert!(PlaybookValidator::validate_ansible_playbook(content).is_ok());
    }

    #[test]
    fn test_validate_terraform_config_valid() {
        let content = r#"
resource:
  aws_instance:
    web:
      ami: "ami-123456"
provider:
  aws:
    region: "us-east-1"
"#;
        assert!(PlaybookValidator::validate_terraform_config(content).is_ok());
    }

    #[test]
    fn test_validate_terraform_config_empty() {
        // Empty/null terraform config
        let content = "null";
        assert!(PlaybookValidator::validate_terraform_config(content).is_ok());
    }

    #[test]
    fn test_validate_terraform_config_with_variables() {
        let content = r#"
variable:
  instance_type:
    default: "t2.micro"
output:
  instance_ip:
    value: "10.0.0.1"
module:
  vpc:
    source: "hashicorp/vpc/aws"
"#;
        assert!(PlaybookValidator::validate_terraform_config(content).is_ok());
    }

    #[test]
    fn test_validate_terraform_config_invalid_not_mapping() {
        // Array is not a valid terraform config structure
        let content = "- item1\n- item2";
        let result = PlaybookValidator::validate_terraform_config(content);
        assert!(matches!(result, Err(PlaybookValidationError::InvalidStructure(_))));
    }

    #[test]
    fn test_validate_terraform_config_invalid_yaml() {
        let content = "invalid: yaml: [";
        let result = PlaybookValidator::validate_terraform_config(content);
        assert!(matches!(result, Err(PlaybookValidationError::YamlParse(_))));
    }

    #[test]
    fn test_validate_shell_with_shebang() {
        let content = "#!/bin/bash\necho hello\n";
        assert!(PlaybookValidator::validate_shell_script(content).is_ok());
    }

    #[test]
    fn test_validate_shell_without_shebang_warns_but_passes() {
        let content = "echo hello\n";
        // Should pass but warn (we can't easily test the warn, but check it passes)
        assert!(PlaybookValidator::validate_shell_script(content).is_ok());
    }

    #[test]
    fn test_validate_shell_whitespace_only() {
        let content = "   \n  \n  ";
        let result = PlaybookValidator::validate_shell_script(content);
        assert!(matches!(result, Err(PlaybookValidationError::InvalidStructure(_))));
    }

    #[test]
    fn test_check_yaml_syntax_valid() {
        let content = "key: value\nlist:\n  - a\n  - b";
        assert!(PlaybookValidator::check_yaml_syntax(content).is_ok());
    }

    #[test]
    fn test_check_yaml_syntax_invalid() {
        let content = "key: value\n  bad indentation: [";
        assert!(PlaybookValidator::check_yaml_syntax(content).is_err());
    }

    #[test]
    fn test_validate_ansible_playbook_single_play_object() {
        let content = r#"
hosts: all
tasks:
  - name: Single play
    debug:
      msg: "ok"
"#;
        assert!(PlaybookValidator::validate_ansible_playbook(content).is_ok());
    }

    #[test]
    fn test_validate_ansible_playbook_complex() {
        let content = r#"
- hosts: all
  become: true
  vars:
    app_port: 8080
  pre_tasks:
    - name: Update apt cache
      apt:
        update_cache: yes
  roles:
    - common
  tasks:
    - name: Install nginx
      apt:
        name: nginx
        state: present
    - name: Start nginx
      service:
        name: nginx
        state: started
"#;
        assert!(PlaybookValidator::validate_ansible_playbook(content).is_ok());
    }

    #[test]
    fn test_validate_max_size_exceeded() {
        let content = "x".repeat(MAX_PLAYBOOK_SIZE + 100);
        let result = PlaybookValidator::validate(&content, "ansible");
        assert!(matches!(result, Err(PlaybookValidationError::MaxSizeExceeded(s)) if s > MAX_PLAYBOOK_SIZE));
    }

    #[test]
    fn test_validate_with_ansible_exact_content() {
        let content = "---\n- hosts: localhost\n  tasks:\n    - debug:\n        msg: test";
        assert!(PlaybookValidator::validate(content, "ansible").is_ok());
    }
}
