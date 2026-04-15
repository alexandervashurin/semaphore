# WebSocket API

> Потоковая передача событий в реальном времени через WebSocket
>
> 📖 См. также: [REST API](./rest-api.md), [GraphQL API](./graphql-api.md), [MCP сервер](./mcp-server.md), [Выполнение задач](../architecture/task-execution.md)

---

## Подключение

```
ws://localhost:3000/api/ws
```

---

## Поток событий

### Подписка на события кластера

```
GET /api/kubernetes/events/stream
```

Передаёт все события Kubernetes в кластере.

### Подписка на события пространства имён

```
GET /api/kubernetes/namespaces/{namespace}/events/stream
```

Передаёт события для конкретного пространства имён.

---

## Формат сообщений

```json
{
  "type": "event",
  "data": {
    "kind": "Pod",
    "reason": "Started",
    "message": "Started container",
    "namespace": "default",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

---

## Типы событий

| Тип | Описание |
|-----|----------|
| `Connected` | Соединение установлено |
| `Event` | Событие Kubernetes |
| `Error` | Сообщение об ошибке |
| `Heartbeat` | Keep-alive пинг |

---

## Следующие шаги

- [REST API](./rest-api.md) — традиционные REST-эндпоинты
- [GraphQL API](./graphql-api.md) — GraphQL-подписки
- [Выполнение задач](../architecture/task-execution.md) — как emitятся события задач
