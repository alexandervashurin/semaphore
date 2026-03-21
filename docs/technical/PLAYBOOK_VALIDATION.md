# Валидация Playbook

## Обзор

Модуль валидации playbook обеспечивает проверку содержимого playbook файлов перед сохранением в базу данных.

## Типы валидации

### 1. Ansible Playbook

**Проверки:**
- ✅ YAML синтаксис
- ✅ Playbook должен быть списком (YAML sequence)
- ✅ Каждый play должен содержать поле `hosts`
- ✅ Поле `hosts` должно быть строкой или списком
- ✅ Tasks (если есть) должны быть списком объектов
- ✅ Roles (если есть) должны быть списком

**Пример валидного playbook:**
```yaml
- hosts: all
  become: yes
  tasks:
    - name: Install nginx
      apt:
        name: nginx
        state: present
    
- hosts: webservers
  roles:
    - webserver
    - database
```

**Примеры ошибок:**

1. Отсутствует поле `hosts`:
```yaml
- tasks:
    - name: Test
      debug:
        msg: Hello
```
Ошибка: `MissingField("Play #1: hosts")`

2. Невалидный YAML:
```yaml
- hosts: all
  tasks:
    - name: Test
      debug:
        msg: [unclosed
```
Ошибка: `YamlParse("...")`

3. Пустой playbook:
```yaml
[]
```
Ошибка: `InvalidStructure("Playbook не может быть пустым")`

### 2. Terraform Config

**Проверки:**
- ✅ YAML/HCL синтаксис
- ✅ Конфигурация должна быть объектом (mapping)
- ⚠️ Проверка допустимых ключей верхнего уровня (warning)

**Допустимые ключи:**
- `resource`
- `variable`
- `output`
- `module`
- `provider`
- `data`
- `locals`
- `terraform`

**Пример валидного Terraform config:**
```yaml
resource:
  aws_instance:
    web:
      ami: "ami-12345678"
      instance_type: "t2.micro"

variable:
  region:
    default: "us-east-1"
```

### 3. Shell Script

**Проверки:**
- ✅ Скрипт не должен быть пустым
- ⚠️ Рекомендуется наличие shebang (warning)

**Пример валидного shell скрипта:**
```bash
#!/bin/bash
set -e

echo "Starting deployment..."
ansible-playbook -i inventory deploy.yml
```

## API Integration

### Создание Playbook

```http
POST /api/project/{project_id}/playbooks
Content-Type: application/json
Authorization: Bearer {token}

{
  "name": "Deploy App",
  "content": "- hosts: all\n  tasks:\n    - debug:\n        msg: Hello",
  "playbook_type": "ansible",
  "description": "Deployment playbook"
}
```

**Ответ при успехе:**
```json
{
  "id": 1,
  "project_id": 1,
  "name": "Deploy App",
  ...
}
```

**Ответ при ошибке валидации:**
```json
{
  "error": "Ошибка валидации: Отсутствует обязательное поле: Play #1: hosts"
}
```
HTTP Status: `400 Bad Request`

### Обновление Playbook

```http
PUT /api/project/{project_id}/playbooks/{id}
Content-Type: application/json
Authorization: Bearer {token}

{
  "name": "Deploy App Updated",
  "content": "- hosts: all\n  tasks:\n    - debug:\n        msg: Updated",
  "playbook_type": "ansible"
}
```

**Примечание:** При обновлении выполняется только проверка YAML синтаксиса (без полной валидации структуры).

## Ограничения

| Параметр | Значение |
|----------|----------|
| Максимальный размер | 10 MB |
| Поддерживаемые типы | ansible, terraform, shell |
| Кодировка | UTF-8 |

## Типы ошибок

### PlaybookValidationError

```rust
pub enum PlaybookValidationError {
    /// Ошибка парсинга YAML
    YamlParse(String),
    
    /// Неверная структура playbook
    InvalidStructure(String),
    
    /// Отсутствует обязательное поле
    MissingField(String),
    
    /// Неверный тип поля
    InvalidFieldType(String, String),
    
    /// Неверный тип playbook
    InvalidPlaybookType(String),
    
    /// Превышен максимальный размер
    MaxSizeExceeded(usize),
}
```

## Тестирование

### Запуск тестов

```bash
cd rust
cargo test playbook_validator
```

### Примеры тестов

```rust
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
```

## Быстрая проверка YAML

Для быстрой проверки YAML синтаксиса без полной валидации:

```rust
use crate::validators::PlaybookValidator;

match PlaybookValidator::check_yaml_syntax(content) {
    Ok(_) => println!("YAML валиден"),
    Err(e) => println!("Ошибка YAML: {}", e),
}
```

## Реализация

### Файлы

- `src/validators/playbook_validator.rs` - основной модуль валидации
- `src/validators/mod.rs` - экспорты модуля
- `src/api/handlers/playbook.rs` - интеграция в handlers

### Зависимости

- `serde_yaml = "0.9"` - парсинг YAML
- `thiserror = "2"` - обработка ошибок

## Roadmap

### v0.4.2 (Текущая)
- ✅ Базовая валидация YAML
- ✅ Валидация структуры Ansible playbook
- ✅ Интеграция в API

### v0.5.0 (Планируется)
- [ ] Полная валидация Terraform HCL (hcl-rs)
- [ ] Интеграция с ansible-lint
- [ ] Проверка на опасные модули
- [ ] Валидация переменных

### v0.6.0 (Планируется)
- [ ] Кастомные правила валидации
- [ ] Плагинная система валидаторов
- [ ] Статический анализ безопасности

## Ссылки

- [Ansible Playbook Syntax](https://docs.ansible.com/ansible/latest/reference_appendices/YAMLSyntax.html)
- [Terraform Configuration Language](https://www.terraform.io/docs/language/syntax/configuration.html)
- [serde_yaml documentation](https://docs.rs/serde_yaml/latest/serde_yaml/)

---

**Версия:** 0.4.2  
**Дата:** 2026-03-12  
**Статус:** ✅ Реализовано
