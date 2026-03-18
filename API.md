# API документация Velum (Rust)

REST API для управления Velum.

## 🎯 CRUD Демо

> **Попробуйте интерактивное демо!** Полный CRUD для всех сущностей.

```bash
# Быстрый старт
./demo-start.sh

# Откройте в браузере
http://localhost/demo-crud.html
```

**Учетные данные для входа:**
| Логин | Пароль | Роль |
|-------|--------|------|
| `admin` | `demo123` | Администратор |
| `john.doe` | `demo123` | Менеджер |
| `jane.smith` | `demo123` | Менеджер |
| `devops` | `demo123` | Исполнитель |

📖 **Подробная документация**: [CRUD_DEMO.md](CRUD_DEMO.md)

---

## 🎯 Демонстрационное окружение

Для тестирования API используйте демонстрационное окружение:

```bash
# Запуск PostgreSQL с демонстрационными данными
./scripts/postgres-demo-start.sh
```

**Учетные данные для входа:**
| Логин | Пароль | Роль |
|-------|--------|------|
| `admin` | `demo123` | Администратор |
| `john.doe` | `demo123` | Менеджер |
| `jane.smith` | `demo123` | Менеджер |
| `devops` | `demo123` | Исполнитель |

**Быстрый тест API:**
```bash
# Вход и получение токена
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"demo123"}'

# Ответ: {"token":"eyJ...","token_type":"Bearer","expires_in":86400}

# Использование токена
curl -H "Authorization: Bearer eyJ..." \
  http://localhost:3000/api/projects
```

📖 **Подробная документация**: [db/postgres/DEMO.md](db/postgres/DEMO.md)

---

## Базовый URL

```
http://localhost:3000/api
```

## Аутентификация

Большинство endpoints требуют аутентификации через заголовок:

```http
Authorization: Bearer <token>
```

## Frontend

Приложение включает веб-интерфейс, доступный по адресу `http://localhost:3000`

### Быстрый старт через API

**Для демонстрационного окружения:**
```bash
# 1. Вход и получение токена
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"demo123"}'

# Ответ: {"token":"eyJ...","token_type":"Bearer","expires_in":86400}

# 2. Использование токена
curl -H "Authorization: Bearer eyJ..." \
  http://localhost:3000/api/projects

# 3. Получить все проекты
curl -H "Authorization: Bearer eyJ..." \
  http://localhost:3000/api/projects

# 4. Получить шаблоны проекта
curl -H "Authorization: Bearer eyJ..." \
  http://localhost:3000/api/project/1/templates
```

**Для тестовой БД (SQLite):**
```bash
# 1. Вход и получение токена
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'

# Ответ: {"token":"eyJ...","token_type":"Bearer","expires_in":86400}

# 2. Использование токена
curl -H "Authorization: Bearer eyJ..." \
  http://localhost:3000/api/projects
```

## Содержание

- [Аутентификация](#аутентификация)
- [Пользователи](#пользователи)
- [Проекты](#проекты)
- [Шаблоны](#шаблоны)
- [Задачи](#задачи)
- [Инвентари](#инвентари)
- [Репозитории](#репозитории)
- [Окружения](#окружения)
- [Ключи доступа](#ключи-доступа)
- [Playbooks](#playbooks)
- [Playbook Runs](#playbook-runs)
- [Analytics](#analytics)

---

## Аутентификация

### Вход в систему

**Для демонстрационного окружения:**
```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "demo123"
}
```

**Для тестовой БД (SQLite):**
```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "admin123"
}
```

**Ответ:**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### Выход из системы

```http
POST /api/auth/logout
Authorization: Bearer <token>
```

---

## Пользователи

### Получить список пользователей

```http
GET /api/users
Authorization: Bearer <token>
```

**Ответ:**

```json
[
  {
    "id": 1,
    "created": "2024-01-01T00:00:00Z",
    "username": "admin",
    "name": "Administrator",
    "email": "admin@localhost",
    "admin": true,
    "external": false,
    "alert": false,
    "pro": false
  }
]
```

### Получить пользователя по ID

```http
GET /api/users/:id
Authorization: Bearer <token>
```

### Обновить пользователя

```http
PUT /api/users/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "username": "new_username",
  "name": "New Name",
  "email": "new@example.com"
}
```

### Удалить пользователя

```http
DELETE /api/users/:id
Authorization: Bearer <token>
```

---

## Проекты

### Получить список проектов

```http
GET /api/projects
Authorization: Bearer <token>
```

**Ответ:**

```json
[
  {
    "id": 1,
    "created": "2024-01-01T00:00:00Z",
    "name": "My Project",
    "alert": false,
    "max_parallel_tasks": 10
  }
]
```

### Создать проект

```http
POST /api/projects
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "New Project"
}
```

### Получить проект по ID

```http
GET /api/projects/:id
Authorization: Bearer <token>
```

### Обновить проект

```http
PUT /api/projects/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Project Name",
  "alert": true
}
```

### Удалить проект

```http
DELETE /api/projects/:id
Authorization: Bearer <token>
```

---

## Шаблоны

### Получить список шаблонов

```http
GET /api/projects/:project_id/templates
Authorization: Bearer <token>
```

**Ответ:**

```json
[
  {
    "id": 1,
    "project_id": 1,
    "name": "Deploy Playbook",
    "playbook": "deploy.yml",
    "description": "Deployment playbook",
    "inventory_id": 1,
    "repository_id": 1,
    "environment_id": 1,
    "type": "default",
    "app": "ansible",
    "git_branch": "main",
    "created": "2024-01-01T00:00:00Z"
  }
]
```

### Создать шаблон

```http
POST /api/projects/:project_id/templates
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "New Template",
  "playbook": "deploy.yml",
  "inventory_id": 1,
  "repository_id": 1,
  "environment_id": 1
}
```

### Получить шаблон по ID

```http
GET /api/projects/:project_id/templates/:id
Authorization: Bearer <token>
```

### Обновить шаблон

```http
PUT /api/projects/:project_id/templates/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Template"
}
```

### Удалить шаблон

```http
DELETE /api/projects/:project_id/templates/:id
Authorization: Bearer <token>
```

---

## Задачи

### Получить список задач

```http
GET /api/projects/:project_id/tasks
Authorization: Bearer <token>
```

**Ответ:**

```json
[
  {
    "id": 1,
    "template_id": 1,
    "project_id": 1,
    "status": "success",
    "playbook": "deploy.yml",
    "created": "2024-01-01T00:00:00Z",
    "start": "2024-01-01T00:01:00Z",
    "end": "2024-01-01T00:05:00Z",
    "message": "Deployment completed"
  }
]
```

### Создать задачу

```http
POST /api/projects/:project_id/tasks
Authorization: Bearer <token>
Content-Type: application/json

{
  "template_id": 1,
  "playbook": "deploy.yml",
  "environment": "{}",
  "limit": "webservers"
}
```

### Получить задачу по ID

```http
GET /api/projects/:project_id/tasks/:id
Authorization: Bearer <token>
```

### Удалить задачу

```http
DELETE /api/projects/:project_id/tasks/:id
Authorization: Bearer <token>
```

---

## Инвентари

### Получить список инвентарей

```http
GET /api/projects/:project_id/inventories
Authorization: Bearer <token>
```

**Ответ:**

```json
[
  {
    "id": 1,
    "project_id": 1,
    "name": "Production Servers",
    "inventory": "static",
    "inventory_data": "[webservers]\nweb1.example.com\nweb2.example.com",
    "key_id": 1,
    "ssh_login": "root",
    "ssh_port": 22
  }
]
```

### Создать инвентарь

```http
POST /api/projects/:project_id/inventories
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "New Inventory",
  "inventory": "static",
  "inventory_data": "[all]\nlocalhost",
  "key_id": 1,
  "ssh_login": "root",
  "ssh_port": 22
}
```

### Получить инвентарь по ID

```http
GET /api/projects/:project_id/inventories/:id
Authorization: Bearer <token>
```

### Обновить инвентарь

```http
PUT /api/projects/:project_id/inventories/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Inventory",
  "inventory_data": "[updated]\nhost1\nhost2"
}
```

### Удалить инвентарь

```http
DELETE /api/projects/:project_id/inventories/:id
Authorization: Bearer <token>
```

---

## Репозитории

### Получить список репозиториев

```http
GET /api/projects/:project_id/repositories
Authorization: Bearer <token>
```

**Ответ:**

```json
[
  {
    "id": 1,
    "project_id": 1,
    "name": "Playbooks Repo",
    "git_url": "https://github.com/example/playbooks.git",
    "git_type": "https",
    "key_id": 1
  }
]
```

### Создать репозиторий

```http
POST /api/projects/:project_id/repositories
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "New Repo",
  "git_url": "https://github.com/example/repo.git",
  "git_type": "https",
  "key_id": 1
}
```

### Получить репозиторий по ID

```http
GET /api/projects/:project_id/repositories/:id
Authorization: Bearer <token>
```

### Обновить репозиторий

```http
PUT /api/projects/:project_id/repositories/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Repo",
  "git_url": "https://github.com/example/new-repo.git"
}
```

### Удалить репозиторий

```http
DELETE /api/projects/:project_id/repositories/:id
Authorization: Bearer <token>
```

---

## Окружения

### Получить список окружений

```http
GET /api/projects/:project_id/environments
Authorization: Bearer <token>
```

**Ответ:**

```json
[
  {
    "id": 1,
    "project_id": 1,
    "name": "Production Env",
    "json": "{\"ENV\": \"production\", \"DEBUG\": \"false\"}",
    "secret_storage_id": null
  }
]
```

### Создать окружение

```http
POST /api/projects/:project_id/environments
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "New Environment",
  "json": "{\"ENV\": \"development\"}"
}
```

### Получить окружение по ID

```http
GET /api/projects/:project_id/environments/:id
Authorization: Bearer <token>
```

### Обновить окружение

```http
PUT /api/projects/:project_id/environments/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Environment",
  "json": "{\"ENV\": \"staging\"}"
}
```

### Удалить окружение

```http
DELETE /api/projects/:project_id/environments/:id
Authorization: Bearer <token>
```

---

## Ключи доступа

### Получить список ключей доступа

```http
GET /api/projects/:project_id/keys
Authorization: Bearer <token>
```

**Ответ:**

```json
[
  {
    "id": 1,
    "project_id": 1,
    "name": "SSH Key",
    "type": "ssh",
    "ssh_key": "-----BEGIN RSA PRIVATE KEY-----...",
    "ssh_passphrase": null
  }
]
```

### Создать ключ доступа

```http
POST /api/projects/:project_id/keys
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "New SSH Key",
  "type": "ssh",
  "ssh_key": "-----BEGIN RSA PRIVATE KEY-----...",
  "ssh_passphrase": null
}
```

### Получить ключ доступа по ID

```http
GET /api/projects/:project_id/keys/:id
Authorization: Bearer <token>
```

### Обновить ключ доступа

```http
PUT /api/projects/:project_id/keys/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Key"
}
```

### Удалить ключ доступа

```http
DELETE /api/projects/:project_id/keys/:id
Authorization: Bearer <token>
```

---

## Статусы ответов

| Код | Описание |
|-----|----------|
| 200 | Успешный запрос |
| 201 | Ресурс создан |
| 204 | Ресурс удалён |
| 400 | Неверный запрос |
| 401 | Не аутентифицирован |
| 403 | Доступ запрещён |
| 404 | Ресурс не найден |
| 500 | Внутренняя ошибка сервера |

## Ошибки

Формат ответов об ошибках:

```json
{
  "error": "Описание ошибки"
}
```

---

## 📚 Playbook API

API для управления Playbook (Ansible, Terraform, Shell).

### Базовый URL

```
GET    /api/project/{project_id}/playbooks      # Получить список playbooks проекта
POST   /api/project/{project_id}/playbooks      # Создать playbook
GET    /api/project/{project_id}/playbooks/{id} # Получить playbook по ID
PUT    /api/project/{project_id}/playbooks/{id} # Обновить playbook
DELETE /api/project/{project_id}/playbooks/{id} # Удалить playbook
```

### Модель Playbook

```json
{
  "id": 1,                    // Уникальный идентификатор (auto)
  "project_id": 1,            // ID проекта
  "name": "Deploy App",       // Название плейбука
  "content": "- hosts: all\n  tasks:\n    - name: Deploy\n      debug:\n        msg: \"Deploying...\"",
  "description": "Deployment playbook",  // Описание (опционально)
  "playbook_type": "ansible", // Тип: ansible, terraform, shell
  "repository_id": null,      // ID репозитория Git (опционально)
  "created": "2026-03-11T10:00:00Z",
  "updated": "2026-03-11T10:00:00Z"
}
```

### Получить список Playbooks

**Запрос:**

```bash
GET /api/project/{project_id}/playbooks
Authorization: Bearer {token}
```

**Ответ:**

```json
[
  {
    "id": 1,
    "project_id": 1,
    "name": "Deploy App",
    "content": "- hosts: all...",
    "description": "Deployment playbook",
    "playbook_type": "ansible",
    "repository_id": null,
    "created": "2026-03-11T10:00:00Z",
    "updated": "2026-03-11T10:00:00Z"
  }
]
```

### Создать Playbook

**Запрос:**

```bash
POST /api/project/{project_id}/playbooks
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Deploy App",
  "content": "- hosts: all\n  tasks:\n    - name: Deploy\n      debug:\n        msg: \"Deploying...\"",
  "description": "Deployment playbook",
  "playbook_type": "ansible",
  "repository_id": null
}
```

**Ответ:**

```json
{
  "id": 1,
  "project_id": 1,
  "name": "Deploy App",
  "content": "- hosts: all...",
  "description": "Deployment playbook",
  "playbook_type": "ansible",
  "repository_id": null,
  "created": "2026-03-11T10:00:00Z",
  "updated": "2026-03-11T10:00:00Z"
}
```

### Получить Playbook по ID

**Запрос:**

```bash
GET /api/project/{project_id}/playbooks/{id}
Authorization: Bearer {token}
```

**Ответ:**

```json
{
  "id": 1,
  "project_id": 1,
  "name": "Deploy App",
  "content": "- hosts: all...",
  "description": "Deployment playbook",
  "playbook_type": "ansible",
  "repository_id": null,
  "created": "2026-03-11T10:00:00Z",
  "updated": "2026-03-11T10:00:00Z"
}
```

### Обновить Playbook

**Запрос:**

```bash
PUT /api/project/{project_id}/playbooks/{id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Deploy App Updated",
  "content": "- hosts: all\n  tasks:\n    - name: Updated Deploy\n      debug:\n        msg: \"Updated!\"",
  "description": "Updated deployment playbook",
  "playbook_type": "ansible"
}
```

**Ответ:**

```json
{
  "id": 1,
  "project_id": 1,
  "name": "Deploy App Updated",
  "content": "- hosts: all...",
  "description": "Updated deployment playbook",
  "playbook_type": "ansible",
  "repository_id": null,
  "created": "2026-03-11T10:00:00Z",
  "updated": "2026-03-11T11:00:00Z"
}
```

### Удалить Playbook

**Запрос:**

```bash
DELETE /api/project/{project_id}/playbooks/{id}
Authorization: Bearer {token}
```

**Ответ:**

```
204 No Content
```

---

### Тестирование Playbook API

Для тестирования используйте скрипт `test-playbook-api.sh`:

```bash
# Настроить переменные окружения
export BASE_URL=http://localhost:3000/api
export PROJECT_ID=1
export TOKEN=your_token_here

# Запустить тесты
./test-playbook-api.sh
```

Или через curl вручную:

```bash
# Получить токен
TOKEN=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"login":"admin","password":"admin123"}' \
  | jq -r '.token')

# Создать playbook
curl -X POST http://localhost:3000/api/project/1/playbooks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Test Playbook",
    "content": "- hosts: localhost\n  tasks:\n    - debug:\n        msg: Hello",
    "playbook_type": "ansible"
  }'
```

---

## Playbook Runs

API для просмотра истории запусков Playbook.

### Получить список запусков Playbook

Возвращает список всех запусков Playbook в проекте.

**Запрос:**

```bash
GET /api/project/{project_id}/playbook-runs
Authorization: Bearer {token}
```

**Параметры:**
| Параметр | Тип | Описание |
|----------|-----|----------|
| `project_id` | path | ID проекта |
| `playbook_id` | query | Фильтр по ID playbook (опционально) |
| `status` | query | Фильтр по статусу (опционально) |

**Ответ:**

```json
[
  {
    "id": 1,
    "project_id": 1,
    "playbook_id": 5,
    "status": "success",
    "started_at": "2026-03-14T10:00:00Z",
    "completed_at": "2026-03-14T10:05:00Z",
    "duration_ms": 300000
  }
]
```

### Получить запуск Playbook по ID

**Запрос:**

```bash
GET /api/project/{project_id}/playbook-runs/{id}
Authorization: Bearer {token}
```

**Ответ:**

```json
{
  "id": 1,
  "project_id": 1,
  "playbook_id": 5,
  "status": "success",
  "started_at": "2026-03-14T10:00:00Z",
  "completed_at": "2026-03-14T10:05:00Z",
  "duration_ms": 300000,
  "output": "Task completed successfully..."
}
```

### Получить статистику запусков Playbook

Возвращает агрегированную статистику по запускам конкретного Playbook.

**Запрос:**

```bash
GET /api/project/{project_id}/playbooks/{playbook_id}/runs/stats
Authorization: Bearer {token}
```

**Ответ:**

```json
{
  "total_runs": 50,
  "success_count": 45,
  "failed_count": 5,
  "avg_duration_ms": 280000,
  "last_run_at": "2026-03-14T10:00:00Z",
  "success_rate": 0.9
}
```

---

## Analytics

API для получения аналитики и метрик проекта.

### Получить статистику проекта

**Запрос:**

```bash
GET /api/projects/{project_id}/analytics/stats
Authorization: Bearer {token}
```

**Ответ:**

```json
{
  "total_tasks": 150,
  "total_templates": 12,
  "total_playbooks": 8,
  "active_users": 5,
  "success_rate": 0.92
}
```

### Получить метрики задач

**Запрос:**

```bash
GET /api/projects/{project_id}/analytics/tasks
Authorization: Bearer {token}
```

**Параметры:**
| Параметр | Тип | Описание |
|----------|-----|----------|
| `project_id` | path | ID проекта |
| `from` | query | Начальная дата (ISO 8601) |
| `to` | query | Конечная дата (ISO 8601) |

**Ответ:**

```json
{
  "tasks_by_status": {
    "waiting": 5,
    "running": 2,
    "success": 120,
    "failed": 23
  },
  "tasks_by_day": [
    {"date": "2026-03-01", "count": 10},
    {"date": "2026-03-02", "count": 15}
  ],
  "avg_duration_ms": 180000,
  "top_slow_tasks": [
    {"id": 42, "name": "Deploy Production", "duration_ms": 600000}
  ]
}
```
