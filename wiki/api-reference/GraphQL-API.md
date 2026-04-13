# GraphQL API

> Query Velum data with GraphQL
>
> 📖 See also: [[REST API]], [[WebSocket API]], [[MCP Server]]

---

## Endpoint

```
POST /graphql
```

---

## Schema

### Query Types

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

### Subscription Types

```graphql
type Subscription {
  taskOutput(taskId: Int!): TaskOutputLine!
  taskStatus(taskId: Int!): TaskStatusEvent!
}
```

---

## Example Queries

### List Projects with Templates

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

### Watch Task Output

```graphql
subscription {
  taskOutput(taskId: 1) {
    line
    timestamp
  }
}
```

---

## Next Steps

- [[REST API]] — traditional REST endpoints
- [[WebSocket API]] — WebSocket events
- [[MCP Server]] — AI tools integration
