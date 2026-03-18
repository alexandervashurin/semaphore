# 📋 CRUD API - Структура и примеры

## Структура сущностей Velum

### 1. Проект (Project)

```json
{
  "id": 1,
  "created": "2026-03-06T10:00:00Z",
  "name": "My Project",
  "alert": false,
  "alert_chat": "",
  "max_parallel_tasks": 5,
  "type": "default"
}
```

**CRUD:**
- ✅ CREATE: `POST /api/projects`
- ✅ READ: `GET /api/projects`, `GET /api/projects/{id}`
- ✅ UPDATE: `PUT /api/projects/{id}`
- ✅ DELETE: `DELETE /api/projects/{id}`

---

### 2. Инвентарь (Inventory)

```json
{
  "id": 1,
  "project_id": 1,
  "name": "Production Servers",
  "inventory_type": "static",
  "inventory_data": "all:\n  children:\n    webservers:\n      hosts:\n        web1:\n        web2:",
  "key_id": 1,
  "ssh_key_id": 1,
  "ssh_login": "ansible",
  "ssh_port": 22,
  "extra_vars": null,
  "created": "2026-03-06T10:00:00Z"
}
```

**CRUD:**
- ✅ CREATE: `POST /api/project/{id}/inventory`
- ✅ READ: `GET /api/project/{id}/inventory`, `GET /api/project/{id}/inventory/{iid}`
- ✅ UPDATE: `PUT /api/project/{id}/inventory/{iid}`
- ✅ DELETE: `DELETE /api/project/{id}/inventory/{iid}`

**Примеры inventory_data:**

```yaml
# Static YAML
all:
  children:
    webservers:
      hosts:
        web1.example.com:
          ansible_user: ansible
        web2.example.com:
          ansible_user: ansible
    databases:
      hosts:
        db1.example.com:
```

```ini
; Static INI
[webservers]
web1.example.com ansible_user=ansible
web2.example.com ansible_user=ansible

[databases]
db1.example.com
```

```json
{
  "all": {
    "children": {
      "webservers": {
        "hosts": {
          "web1": {},
          "web2": {}
        }
      }
    }
  }
}
```

---

### 3. Репозиторий (Repository)

```json
{
  "id": 1,
  "project_id": 1,
  "name": "My Playbooks",
  "git_url": "https://github.com/myuser/my-playbooks.git",
  "git_type": "git",
  "git_branch": "main",
  "key_id": 1,
  "git_path": "",
  "created": "2026-03-06T10:00:00Z"
}
```

**CRUD:**
- ✅ CREATE: `POST /api/project/{id}/repository`
- ✅ READ: `GET /api/project/{id}/repository`, `GET /api/project/{id}/repository/{rid}`
- ✅ UPDATE: `PUT /api/project/{id}/repository/{rid}`
- ✅ DELETE: `DELETE /api/project/{id}/repository/{rid}`

**Типы репозиториев:**
- `git` - Git (SSH/HTTPS)
- `http` - HTTP
- `https` - HTTPS
- `file` - Локальный файл

---

### 4. Окружение (Environment)

```json
{
  "id": 1,
  "project_id": 1,
  "name": "Production Variables",
  "json": "{\"env\": \"production\", \"domain\": \"example.com\"}",
  "secret_storage_id": null,
  "secrets": null,
  "created": "2026-03-06T10:00:00Z"
}
```

**CRUD:**
- ✅ CREATE: `POST /api/project/{id}/environment`
- ✅ READ: `GET /api/project/{id}/environment`, `GET /api/project/{id}/environment/{eid}`
- ✅ UPDATE: `PUT /api/project/{id}/environment/{eid}`
- ✅ DELETE: `DELETE /api/project/{id}/environment/{eid}`

**Примеры JSON:**

```json
{
  "env": "production",
  "domain": "example.com",
  "ssl_enabled": true,
  "backup_enabled": true,
  "log_level": "warn"
}
```

```json
{
  "app_name": "MyWebApp",
  "app_port": 8080,
  "workers": 4,
  "cache_enabled": true
}
```

---

### 5. Ключ доступа (Access Key)

```json
{
  "id": 1,
  "project_id": 1,
  "name": "SSH Key",
  "type": "ssh",
  "user_id": null,
  "login_password_login": null,
  "login_password_password": null,
  "ssh_key": "-----BEGIN OPENSSH PRIVATE KEY-----\n...",
  "ssh_passphrase": null,
  "access_key_access_key": null,
  "access_key_secret_key": null,
  "secret_storage_id": null,
  "environment_id": null,
  "owner": "project",
  "created": "2026-03-06T10:00:00Z"
}
```

**Типы ключей:**
- `ssh` - SSH ключ
- `login_password` - Логин/пароль
- `access_key` - AWS Access Key
- `none` - Без аутентификации

**CRUD:**
- ✅ CREATE: `POST /api/project/{id}/keys`
- ✅ READ: `GET /api/project/{id}/keys`, `GET /api/project/{id}/keys/{kid}`
- ✅ UPDATE: `PUT /api/project/{id}/keys/{kid}`
- ✅ DELETE: `DELETE /api/project/{id}/keys/{kid}`

---

### 6. Шаблон (Template)

```json
{
  "id": 1,
  "project_id": 1,
  "inventory_id": 1,
  "repository_id": 1,
  "environment_id": 1,
  "name": "Deploy Web Application",
  "description": "Production deployment",
  "playbook": "deploy.yml",
  "arguments": "[]",
  "allow_override_args_in_task": false,
  "survey_var": null,
  "created": "2026-03-06T10:00:00Z",
  "vault_key_id": null,
  "type": "ansible",
  "app": "ansible",
  "git_branch": "main",
  "git_depth": 1,
  "diff": false,
  "operator_id": null,
  "last_task_id": null
}
```

**CRUD:**
- ✅ CREATE: `POST /api/project/{id}/templates`
- ✅ READ: `GET /api/project/{id}/templates`, `GET /api/project/{id}/templates/{tid}`
- ✅ UPDATE: `PUT /api/project/{id}/templates/{tid}`
- ✅ DELETE: `DELETE /api/project/{id}/templates/{tid}`

**Типы шаблонов:**
- `ansible` - Ansible playbook
- `terraform` - Terraform
- `shell` - Shell скрипт
- `default` - По умолчанию

**Примеры playbook:**
- `site.yml` - Основной playbook
- `deploy.yml` - Деплой
- `backup.yml` - Резервное копирование

---

### 7. Задача (Task)

```json
{
  "id": 1,
  "template_id": 1,
  "project_id": 1,
  "status": "success",
  "playbook": "deploy.yml",
  "arguments": "[]",
  "task_limit": null,
  "debug": false,
  "dry_run": false,
  "diff": false,
  "user_id": 1,
  "created": "2026-03-06T10:00:00Z",
  "start_time": "2026-03-06T10:01:00Z",
  "end_time": "2026-03-06T10:05:00Z",
  "message": "Deployment completed successfully",
  "commit_hash": null,
  "commit_message": null,
  "commit_author": null
}
```

**Статусы задач:**
- `waiting` - Ожидание
- `running` - Выполняется
- `success` - Успешно
- `failed` - Ошибка

**CRUD:**
- ✅ CREATE: `POST /api/project/{id}/tasks`
- ✅ READ: `GET /api/project/{id}/tasks`, `GET /api/project/{id}/tasks/{tid}`
- ❌ UPDATE: (не применяется)
- ❌ DELETE: `DELETE /api/project/{id}/tasks/{tid}`

**Запуск задачи:**

```json
{
  "template_id": 1,
  "debug": false,
  "dry_run": false,
  "diff": false,
  "arguments": "[]",
  "task_limit": null
}
```

---

### 8. Расписание (Schedule)

```json
{
  "id": 1,
  "project_id": 1,
  "template_id": 1,
  "cron": "0 2 * * *",
  "name": "Daily Backup",
  "active": true,
  "created": "2026-03-06T10:00:00Z"
}
```

**CRUD:**
- ✅ CREATE: `POST /api/project/{id}/schedule`
- ✅ READ: `GET /api/project/{id}/schedule`, `GET /api/project/{id}/schedule/{sid}`
- ✅ UPDATE: `PUT /api/project/{id}/schedule/{sid}`
- ✅ DELETE: `DELETE /api/project/{id}/schedule/{sid}`

**Примеры cron:**
- `0 2 * * *` - Ежедневно в 2:00
- `0 3 * * 0` - Еженедельно в воскресенье в 3:00
- `*/15 * * * *` - Каждые 15 минут
- `0 9-17 * * 1-5` - Каждый час с 9 до 17 в будни

---

### 9. Событие (Event)

```json
{
  "id": 1,
  "project_id": 1,
  "user_id": 1,
  "task_id": 1,
  "object_id": 1,
  "object_type": "template",
  "description": "Template 'Deploy App' created",
  "created": "2026-03-06T10:00:00Z"
}
```

**CRUD:**
- ❌ CREATE: (создаётся автоматически)
- ✅ READ: `GET /api/project/{id}/events`, `GET /api/events/{eid}`
- ❌ UPDATE: (не применяется)
- ❌ DELETE: `DELETE /api/project/{id}/events/{eid}`

**Типы объектов:**
- `project` - Проект
- `template` - Шаблон
- `task` - Задача
- `inventory` - Инвентарь
- `repository` - Репозиторий
- `environment` - Окружение
- `key` - Ключ доступа

---

## 📊 Рабочий процесс (Ansible-like)

### Типичный сценарий использования

```
1. Создать Проект
   └─> "My Infrastructure"

2. Создать Ключ доступа
   └─> SSH ключ для подключения к серверам

3. Создать Инвентарь
   └─> Список серверов (webservers, databases)

4. Создать Репозиторий
   └─> Git с Ansible playbook

5. Создать Окружение
   └─> Переменные (env: production, domain: example.com)

6. Создать Шаблон
   └─> Связать: Инвентарь + Репозиторий + Окружение + Playbook

7. Запустить Задачу
   └─> Выполнение шаблона

8. (Опционально) Создать Расписание
   └─> Автоматический запуск по cron
```

### Пример: Деплой веб-приложения

```bash
# 1. Проект
POST /api/projects
{"name": "Web App Deployment"}

# 2. Ключ
POST /api/project/1/keys
{
  "name": "Production SSH",
  "type": "ssh",
  "ssh_key": "-----BEGIN OPENSSH PRIVATE KEY-----..."
}

# 3. Инвентарь
POST /api/project/1/inventory
{
  "name": "Production Servers",
  "inventory_type": "static",
  "inventory_data": "all:\n  children:\n    webservers:\n      hosts:\n        web1:\n        web2:",
  "ssh_key_id": 1
}

# 4. Репозиторий
POST /api/project/1/repository
{
  "name": "Web App Playbooks",
  "git_url": "https://github.com/myorg/webapp-playbooks.git",
  "git_branch": "main"
}

# 5. Окружение
POST /api/project/1/environment
{
  "name": "Production Config",
  "json": "{\"env\": \"production\", \"domain\": \"example.com\"}"
}

# 6. Шаблон
POST /api/project/1/templates
{
  "name": "Deploy Web App",
  "playbook": "deploy.yml",
  "inventory_id": 1,
  "repository_id": 1,
  "environment_id": 1,
  "type": "ansible"
}

# 7. Задача
POST /api/project/1/tasks
{
  "template_id": 1,
  "debug": false
}
```

---

## 🔗 Связи между сущностями

```
Project (1)
├── Inventory (N)
├── Repository (N)
├── Environment (N)
├── Access Key (N)
├── Template (N)
│   ├── inventory_id → Inventory
│   ├── repository_id → Repository
│   └── environment_id → Environment
├── Task (N)
│   └── template_id → Template
└── Schedule (N)
    └── template_id → Template
```

---

## 💡 Лучшие практики

### 1. Именование

- **Проекты:** `My Infrastructure`, `Web App Deployment`
- **Инвентарь:** `Production Servers`, `Staging Environment`
- **Репозитории:** `Ansible Playbooks`, `Terraform Modules`
- **Окружения:** `Production Variables`, `Staging Config`
- **Ключи:** `Production SSH`, `AWS Access Key`
- **Шаблоны:** `Deploy Web App`, `Backup Database`

### 2. Организация

```
Project: E-Commerce Platform
├── Inventory:
│   ├── Production (webservers, databases, cache)
│   └── Staging (all-in-one)
├── Repository:
│   ├── Main Playbooks (git@github.com:org/ecom-playbooks)
│   └── Database Playbooks (git@github.com:org/db-playbooks)
├── Environment:
│   ├── Production (env=prod, ssl=true)
│   └── Staging (env=staging, debug=true)
├── Keys:
│   ├── Production SSH
│   └── AWS Deploy Key
└── Templates:
    ├── Deploy Frontend
    ├── Deploy Backend
    ├── Database Backup
    └── SSL Renewal
```

### 3. Безопасность

- Используйте разные ключи для production/staging
- Храните секреты в Environment с флагом secret
- Ограничивайте доступ через Project Users
- Включайте логирование (Events)

---

## 📖 Дополнительные ресурсы

- [CRUD_DEMO.md](CRUD_DEMO.md) - Полное руководство
- [API.md](API.md) - Документация API
- [ЗАПУСК_ДЕМО.md](ЗАПУСК_ДЕМО.md) - Инструкция по запуску
