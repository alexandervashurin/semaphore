# 📚 API Документация Velum

> **Версия API:** 2.16.14  
> **Base URL:** `http://localhost:3000/api`  
> **Формат:** OpenAPI 3.0

---

## 📋 Оглавление

1. [Аутентификация](#authentication)
2. [Пользователи](#users)
3. [Проекты](#projects)
4. [Шаблоны](#templates)
5. [Задачи](#tasks)
6. [Playbooks](#playbooks)
7. [Инвентари](#inventory)
8. [Репозитории](#repositories)
9. [Окружения](#environments)
10. [Ключи доступа](#keys)
11. [Расписания](#schedules)
12. [Webhooks](#webhooks)
13. [Аналитика](#analytics)
14. [Kubernetes](#kubernetes)
15. [Аудит](#audit-log)
16. [Системные](#system)

---

## 🔐 Аутентификация {#authentication}

### Вход в систему

```http
POST /api/auth/login
Content-Type: application/json
```

**Request:**
```json
{
  "auth": "admin",
  "password": "admin123"
}
```

**Response (200):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": 1,
    "name": "Admin",
    "username": "admin",
    "email": "admin@velum.local",
    "admin": true
  }
}
```

### Выход из системы

```http
POST /api/auth/logout
Authorization: Bearer <token>
```

### Проверка сессии

```http
POST /api/auth/verify
Authorization: Bearer <token>
```

### API Токены пользователя

```http
GET /api/user/tokens
Authorization: Bearer <token>
```

```http
POST /api/user/tokens
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "My API Token",
  "expiration": "2026-12-31T23:59:59Z"
}
```

---

## 👥 Пользователи {#users}

### Получить всех пользователей

```http
GET /api/users
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": 1,
    "name": "Admin",
    "username": "admin",
    "email": "admin@velum.local",
    "admin": true,
    "alert": true,
    "external": false,
    "created": "2026-03-01T00:00:00Z"
  }
]
```

### Создать пользователя

```http
POST /api/users
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "John Doe",
  "username": "john.doe",
  "email": "john@example.com",
  "password": "password123",
  "alert": true,
  "admin": false
}
```

### Обновить пользователя

```http
PUT /api/users/{id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "John Doe Updated",
  "email": "john.updated@example.com",
  "admin": false,
  "alert": false
}
```

### Удалить пользователя

```http
DELETE /api/users/{id}
Authorization: Bearer <token>
```

### Сменить пароль пользователя

```http
POST /api/users/{id}/password
Authorization: Bearer <token>
Content-Type: application/json

{
  "password": "newpassword123"
}
```

---

## 📁 Проекты {#projects}

### Получить все проекты

```http
GET /api/projects
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": 1,
    "name": "Infrastructure",
    "description": "Infrastructure automation",
    "created": "2026-03-01T00:00:00Z",
    "updated": "2026-03-30T00:00:00Z"
  }
]
```

### Создать проект

```http
POST /api/projects
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "My Project",
  "description": "Project description"
}
```

### Получить проект

```http
GET /api/projects/{id}
Authorization: Bearer <token>
```

### Обновить проект

```http
PUT /api/projects/{id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Project",
  "description": "New description"
}
```

### Удалить проект

```http
DELETE /api/projects/{id}
Authorization: Bearer <token>
```

### Получить статистику проекта

```http
GET /api/projects/{id}/stats
Authorization: Bearer <token>
```

### Получить пользователей проекта

```http
GET /api/projects/{id}/users
Authorization: Bearer <token>
```

### Добавить пользователя в проект

```http
POST /api/projects/{id}/users
Authorization: Bearer <token>
Content-Type: application/json

{
  "user_id": 2,
  "role": "manager"
}
```

**Роли:**
- `owner` — полный доступ
- `manager` — управление проектом
- `task_runner` — запуск задач
- `guest` — только просмотр

### Backup проекта

```http
GET /api/projects/{id}/backup
Authorization: Bearer <token>
```

### Restore проекта

```http
POST /api/projects/restore
Authorization: Bearer <token>
Content-Type: application/json

{
  "backup_data": { ... }
}
```

---

## 📋 Шаблоны {#templates}

### Получить все шаблоны

```http
GET /api/projects/{project_id}/templates
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": 1,
    "name": "Deploy Application",
    "playbook": "deploy.yml",
    "inventory_id": 1,
    "environment_id": 1,
    "repository_id": 1,
    "allow_override_limit": true,
    "allow_override_tags": true,
    "require_approval": false,
    "created": "2026-03-01T00:00:00Z"
  }
]
```

### Создать шаблон

```http
POST /api/projects/{project_id}/templates
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Deploy",
  "playbook": "deploy.yml",
  "inventory_id": 1,
  "environment_id": 1,
  "repository_id": 1,
  "vaults": [],
  "task_params": {
    "become": true,
    "forks": 10
  }
}
```

### Обновить шаблон

```http
PUT /api/projects/{project_id}/templates/{id}
Authorization: Bearer <token>
Content-Type: application/json
```

### Удалить шаблон

```http
DELETE /api/projects/{project_id}/templates/{id}
Authorization: Bearer <token>
```

### Остановить все задачи шаблона

```http
POST /api/projects/{project_id}/templates/{id}/stop_all_tasks
Authorization: Bearer <token>
```

---

## ⚡ Задачи {#tasks}

### Получить все задачи (глобально)

```http
GET /api/tasks
Authorization: Bearer <token>
```

### Получить задачи проекта

```http
GET /api/projects/{project_id}/tasks
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": 123,
    "template_id": 1,
    "status": "success",
    "playbook": "deploy.yml",
    "created": "2026-03-30T10:00:00Z",
    "start": "2026-03-30T10:00:05Z",
    "end": "2026-03-30T10:05:00Z",
    "output": "..."
  }
]
```

### Создать задачу

```http
POST /api/projects/{project_id}/tasks
Authorization: Bearer <token>
Content-Type: application/json

{
  "template_id": 1,
  "limit_hosts": "web01,web02",
  "tags": ["deploy", "app"],
  "skip_tags": ["debug"],
  "debug": 2,
  "override_args": {}
}
```

### Получить задачу

```http
GET /api/projects/{project_id}/tasks/{id}
Authorization: Bearer <token>
```

### Удалить задачу

```http
DELETE /api/projects/{project_id}/tasks/{id}
Authorization: Bearer <token>
```

### Остановить задачу

```http
POST /api/projects/{project_id}/tasks/{id}/stop
Authorization: Bearer <token>
```

### Получить вывод задачи

```http
GET /api/projects/{project_id}/tasks/{id}/output
Authorization: Bearer <token>
```

### Подтвердить задачу (требует approval)

```http
POST /api/projects/{project_id}/tasks/{id}/confirm
Authorization: Bearer <token>
```

### Отклонить задачу

```http
POST /api/projects/{project_id}/tasks/{id}/reject
Authorization: Bearer <token>
```

### Последние задачи (History)

```http
GET /api/project/{project_id}/tasks/last
Authorization: Bearer <token>
```

---

## 📖 Playbooks {#playbooks}

### Получить все playbooks проекта

```http
GET /api/project/{project_id}/playbooks
Authorization: Bearer <token>
```

### Создать playbook

```http
POST /api/project/{project_id}/playbooks
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "my-playbook.yml",
  "type": "ansible",
  "content": "---\n- hosts: all\n  tasks:\n    - debug: msg=\"Hello\""
}
```

### Получить playbook

```http
GET /api/project/{project_id}/playbooks/{id}
Authorization: Bearer <token>
```

### Обновить playbook

```http
PUT /api/project/{project_id}/playbooks/{id}
Authorization: Bearer <token>
Content-Type: application/json
```

### Удалить playbook

```http
DELETE /api/project/{project_id}/playbooks/{id}
Authorization: Bearer <token>
```

### Синхронизировать playbook (из repo)

```http
POST /api/project/{project_id}/playbooks/{id}/sync
Authorization: Bearer <token>
```

### Preview playbook

```http
GET /api/project/{project_id}/playbooks/{id}/preview
Authorization: Bearer <token>
```

### Запустить playbook

```http
POST /api/project/{project_id}/playbooks/{id}/run
Authorization: Bearer <token>
Content-Type: application/json

{
  "inventory_id": 1,
  "environment_id": 1,
  "limit_hosts": "web01",
  "tags": ["deploy"],
  "extra_vars": {}
}
```

### Playbook Runs

```http
GET /api/project/{project_id}/playbook-runs
Authorization: Bearer <token>
```

```http
GET /api/project/{project_id}/playbook-runs/{id}
Authorization: Bearer <token>
```

### Статистика запусков

```http
GET /api/project/{project_id}/playbooks/{playbook_id}/runs/stats
Authorization: Bearer <token>
```

---

## 🖥️ Инвентари {#inventory}

### Получить все инвентари

```http
GET /api/projects/{project_id}/inventories
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": 1,
    "name": "Production",
    "type": "static",
    "inventory": "[webservers]\nweb01\nweb02",
    "ssh_key_id": 1
  }
]
```

### Создать инвентарь

```http
POST /api/projects/{project_id}/inventories
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Staging",
  "type": "static",
  "inventory": "[webservers]\nstaging01",
  "ssh_key_id": 1
}
```

### Обновить инвентарь

```http
PUT /api/projects/{project_id}/inventories/{id}
Authorization: Bearer <token>
```

### Удалить инвентарь

```http
DELETE /api/projects/{project_id}/inventories/{id}
Authorization: Bearer <token>
```

---

## 📚 Репозитории {#repositories}

### Получить все репозитории

```http
GET /api/projects/{project_id}/repositories
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": 1,
    "name": "Main Repo",
    "git_url": "https://github.com/org/ansible.git",
    "git_branch": "main",
    "ssh_key_id": 1
  }
]
```

### Создать репозиторий

```http
POST /api/projects/{project_id}/repositories
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "My Repo",
  "git_url": "https://github.com/org/repo.git",
  "git_branch": "main",
  "ssh_key_id": 1
}
```

---

## 🔧 Окружения {#environments}

### Получить все окружения

```http
GET /api/projects/{project_id}/environments
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": 1,
    "name": "Production",
    "json": "{\"env\": \"prod\", \"debug\": false}",
    "password": "{\"db_password\": \"secret\"}"
  }
]
```

### Создать окружение

```http
POST /api/projects/{project_id}/environments
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Staging",
  "json": "{\"env\": \"staging\"}",
  "password": "{\"secret\": \"value\"}"
}
```

---

## 🔑 Ключи доступа {#keys}

### Получить все ключи

```http
GET /api/projects/{project_id}/keys
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": 1,
    "name": "SSH Key",
    "type": "ssh",
    "secret": "ssh-rsa AAAA..."
  }
]
```

### Создать ключ

```http
POST /api/projects/{project_id}/keys
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "My SSH Key",
  "type": "ssh",
  "secret": "-----BEGIN RSA PRIVATE KEY-----\n..."
}
```

**Типы ключей:**
- `ssh` — SSH ключ
- `login_password` — Логин/пароль
- `none` — Без ключа

---

## ⏰ Расписания {#schedules}

### Получить все расписания

```http
GET /api/projects/{project_id}/schedules
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": 1,
    "template_id": 1,
    "cron": "0 2 * * *",
    "name": "Nightly Deploy",
    "active": true
  }
]
```

### Создать расписание

```http
POST /api/projects/{project_id}/schedules
Authorization: Bearer <token>
Content-Type: application/json

{
  "template_id": 1,
  "cron": "0 */6 * * *",
  "name": "Every 6 hours",
  "active": true
}
```

### Валидировать cron

```http
POST /api/projects/{project_id}/schedules/validate
Authorization: Bearer <token>
Content-Type: application/json

{
  "cron": "0 2 * * *"
}
```

---

## 🔗 Webhooks {#webhooks}

### Получить все webhooks

```http
GET /api/project/{project_id}/webhooks
Authorization: Bearer <token>
```

### Создать webhook

```http
POST /api/project/{project_id}/webhooks
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Slack Notification",
  "type": "slack",
  "url": "https://hooks.slack.com/...",
  "secret": "mysecret",
  "events": ["task.success", "task.failed"],
  "active": true
}
```

**Типы webhooks:**
- `generic` — Generic HTTP
- `slack` — Slack
- `teams` — Microsoft Teams
- `discord` — Discord
- `telegram` — Telegram

---

## 📊 Аналитика {#analytics}

### Аналитика проекта

```http
GET /api/project/{project_id}/analytics
Authorization: Bearer <token>
```

**Response:**
```json
{
  "total_tasks": 150,
  "success_rate": 95.5,
  "avg_duration": 180,
  "failed_tasks": 7,
  "running_tasks": 2
}
```

### График задач

```http
GET /api/project/{project_id}/analytics/tasks-chart
Authorization: Bearer <token>
```

### Распределение статусов

```http
GET /api/project/{project_id}/analytics/status-distribution
Authorization: Bearer <token>
```

### Системная аналитика

```http
GET /api/analytics/system
Authorization: Bearer <token>
```

---

## ☸️ Kubernetes {#kubernetes}

### Cluster Info

```http
GET /api/kubernetes/cluster/info
Authorization: Bearer <token>
```

### Cluster Summary

```http
GET /api/kubernetes/cluster/summary
Authorization: Bearer <token>
```

### Nodes

```http
GET /api/kubernetes/cluster/nodes
Authorization: Bearer <token>
```

### Namespaces

```http
GET /api/kubernetes/namespaces
Authorization: Bearer <token>
```

### Pods

```http
GET /api/kubernetes/pods
Authorization: Bearer <token>
```

### Deployments

```http
GET /api/kubernetes/deployments
Authorization: Bearer <token>
```

### Services

```http
GET /api/kubernetes/services
Authorization: Bearer <token>
```

### ConfigMaps

```http
GET /api/kubernetes/configmaps
Authorization: Bearer <token>
```

### Secrets

```http
GET /api/kubernetes/secrets
Authorization: Bearer <token>
```

### RBAC

```http
GET /api/kubernetes/rbac
Authorization: Bearer <token>
```

### Ingress

```http
GET /api/kubernetes/ingress
Authorization: Bearer <token>
```

### Health

```http
GET /api/kubernetes/health
Authorization: Bearer <token>
```

---

## 🔍 Audit Log {#audit-log}

### Получить логи аудита

```http
GET /api/audit-log
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": 1,
    "user_id": 1,
    "action": "task.created",
    "object_type": "task",
    "object_id": 123,
    "created": "2026-03-30T10:00:00Z"
  }
]
```

### Экспорт логов

```http
GET /api/audit-log/export
Authorization: Bearer <token>
```

### Очистить логи

```http
DELETE /api/audit-log/clear
Authorization: Bearer <token>
```

### Удалить старые логи

```http
DELETE /api/audit-log/expiry
Authorization: Bearer <token>
```

---

## ⚙️ Системные {#system}

### Health Check

```http
GET /api/health
```

**Response:** `OK`

### Health Live

```http
GET /api/health/live
```

### Health Ready

```http
GET /api/health/ready
```

### System Info

```http
GET /api/info
Authorization: Bearer <token>
```

**Response:**
```json
{
  "version": "2.16.14",
  "commit": "abc123",
  "build_date": "2026-03-30",
  "database": "postgres",
  "features": ["kubernetes", "analytics", "webhooks"]
}
```

### Metrics (Prometheus)

```http
GET /api/metrics
```

### Metrics JSON

```http
GET /api/metrics/json
Authorization: Bearer <token>
```

---

## 📝 Примеры использования

### cURL

```bash
# Login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"auth":"admin","password":"admin123"}'

# Get projects
curl -X GET http://localhost:3000/api/projects \
  -H "Authorization: Bearer YOUR_TOKEN"

# Create task
curl -X POST http://localhost:3000/api/projects/1/tasks \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"template_id":1}'
```

### Python

```python
import requests

BASE_URL = "http://localhost:3000/api"

# Login
response = requests.post(f"{BASE_URL}/auth/login", json={
    "auth": "admin",
    "password": "admin123"
})
token = response.json()["token"]

headers = {"Authorization": f"Bearer {token}"}

# Get projects
projects = requests.get(f"{BASE_URL}/projects", headers=headers).json()

# Create task
task = requests.post(
    f"{BASE_URL}/projects/1/tasks",
    headers=headers,
    json={"template_id": 1}
).json()
```

---

## 📊 Статистика API

| Метод | Количество |
|-------|------------|
| GET | 208 |
| POST | 104 |
| PUT | 52 |
| DELETE | 67 |
| **Всего** | **431** |

---

## 🔗 Ссылки

- [Swagger UI](http://localhost:3000/api-docs)
- [GitHub Repository](https://github.com/alexandervashurin/semaphore)
- [Основная документация](../README.md)
