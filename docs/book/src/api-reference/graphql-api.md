# GraphQL API

> Запрос данных Velum через GraphQL
>
> 📖 См. также: [REST API](./rest-api.md), [WebSocket API](./websocket-api.md), [MCP сервер](./mcp-server.md), [OpenAPI](./openapi.md)

---

## Эндпоинт

```
POST /graphql
```

---

## Схема

### Типы запросов (Query)

```graphql
type Query {
  users: [User!]!
  projects: [Project!]!
  templates(projectId: Int): [Template!]!
  tasks(projectId: Int, status: String): [Task!]!
  runners: [Runner!]!
  inventory: [Inventory!]!
  repositories: [Repository!]!
  environments: [Environment!]!
  schedules: [Schedule!]!

  # Kubernetes
  kubernetesNamespaces: [KubernetesNamespace!]!
  kubernetesNodes: [KubernetesNode!]!
  kubernetesCluster: KubernetesClusterInfo!
}
```

### Типы подписок (Subscription)

```graphql
type Subscription {
  taskOutput(taskId: Int!): TaskOutputLine!
  taskStatus(taskId: Int!): TaskStatusEvent!
}
```

---

## Примеры запросов

### Список проектов с шаблонами

```graphql
query {
  projects {
    id
    name
    templates {
      id
      name
      playbook
    }
  }
}
```

### Отслеживание вывода задачи

```graphql
subscription {
  taskOutput(taskId: 1) {
    line
    timestamp
  }
}
```

---

## Следующие шаги

- [REST API](./rest-api.md) — традиционные REST-эндпоинты
- [WebSocket API](./websocket-api.md) — WebSocket-события
- [MCP сервер](./mcp-server.md) — интеграция с AI-инструментами
