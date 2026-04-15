# REST API

> Полная справка по REST API Velum
>
> 📖 См. также: [GraphQL API](./graphql-api.md), [WebSocket API](./websocket-api.md), [MCP сервер](./mcp-server.md), [OpenAPI](./openapi.md)

---

## Аутентификация

Все вызовы API (кроме входа) требуют JWT-токен:

```
Authorization: Bearer <token>
```

### Вход

```
POST /api/auth/login
Content-Type: application/json

{"username": "admin", "password": "admin123"}
```

Ответ: `{ "token": "jwt...", "user": {...} }`

---

## Основные эндпоинты

### Проекты

| Метод | Путь | Описание |
|-------|------|----------|
| `GET` | `/api/projects` | Список всех проектов |
| `POST` | `/api/projects` | Создать проект |
| `GET` | `/api/projects/{id}` | Получить проект |
| `PUT` | `/api/projects/{id}` | Обновить проект |
| `DELETE` | `/api/projects/{id}` | Удалить проект |

### Шаблоны

| Метод | Путь | Описание |
|-------|------|----------|
| `GET` | `/api/templates` | Список шаблонов |
| `POST` | `/api/templates` | Создать шаблон |
| `PUT` | `/api/templates/{id}` | Обновить шаблон |
| `DELETE` | `/api/templates/{id}` | Удалить шаблон |

### Задачи

| Метод | Путь | Описание |
|-------|------|----------|
| `GET` | `/api/tasks` | Список задач |
| `POST` | `/api/tasks` | Запустить задачу |
| `GET` | `/api/tasks/{id}` | Получить задачу |
| `GET` | `/api/tasks/{id}/output` | Получить логи задачи |

---

## Эндпоинты здоровья

| Эндпоинт | Описание |
|----------|----------|
| `/healthz` | Liveness-зонд — возвращает "OK" |
| `/readyz` | Readiness-зонд — возвращает JSON с проверками |
| `/api/health` | Полный статус здоровья |

---

## Спецификация OpenAPI

Полная спецификация доступна:
- [Интерактивная документация (ReDoc)](./openapi.md)
- [Файл YAML](../../openapi.yml)
- [Файл Swagger](../../api-docs.yml)

---

## Следующие шаги

- [GraphQL API](./graphql-api.md) — альтернативный интерфейс запросов
- [WebSocket API](./websocket-api.md) — события в реальном времени
- [MCP сервер](./mcp-server.md) — интеграция с AI
- [OpenAPI](./openapi.md) — интерактивная документация
