# MCP сервер

> Инструменты Model Context Protocol для AI-интеграции
>
> 📖 См. также: [REST API](./rest-api.md), [GraphQL API](./graphql-api.md), [WebSocket API](./websocket-api.md)

---

## Обзор

Velum предоставляет **60+ MCP-инструментов** для интеграции с AI-ассистентами (Claude, Cursor и др.).

## Категории инструментов

- **Управление проектами** — создание, обновление, удаление проектов
- **Управление шаблонами** — CRUD шаблонов
- **Запуск задач** — выполнение задач из шаблонов
- **Мониторинг** — получение статусов и логов
- **Kubernetes** — управление ресурсами K8s

## Подключение

MCP-сервер работает как отдельный процесс и подключается к AI-ассистенту через stdio или SSE.

```bash
cd mcp
cargo run
```

## Конфигурация

```json
{
  "mcpServers": {
    "velum": {
      "command": "velum",
      "args": ["mcp"],
      "env": {
        "VELUM_API_URL": "http://localhost:3000",
        "VELUM_API_TOKEN": "your-token"
      }
    }
  }
}
```

---

## Следующие шаги

- [REST API](./rest-api.md) — REST-эндпоинты
- [GraphQL API](./graphql-api.md) — GraphQL-интерфейс
- [WebSocket API](./websocket-api.md) — события в реальном времени
