# API документация Semaphore UI (Rust)

REST API для управления Semaphore UI.

## Базовый URL

```
http://localhost:3000/api
```

## Аутентификация

Большинство endpoints требуют аутентификации через заголовок:

```http
Authorization: Bearer <token>
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

---

## Аутентификация

### Вход в систему

```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "changeme"
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
