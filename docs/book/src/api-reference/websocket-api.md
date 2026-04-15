# WebSocket API

> Real-time event streaming via WebSocket
>
> 📖 See also: [[REST API]], [[GraphQL API]], [[MCP Server]]

---

## Connection

```
ws://localhost:3000/api/ws
```

---

## Events Stream

### Subscribe to Cluster Events

```
GET /api/kubernetes/events/stream
```

Streams all Kubernetes events in the cluster.

### Subscribe to Namespace Events

```
GET /api/kubernetes/namespaces/{namespace}/events/stream
```

Streams events for a specific namespace.

---

## Message Format

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

## Event Types

| Type | Description |
|------|-------------|
| `Connected` | Connection established |
| `Event` | Kubernetes event |
| `Error` | Error message |
| `Heartbeat` | Keep-alive ping |

---

## Next Steps

- [[REST API]] — traditional REST endpoints
- [[GraphQL API]] — GraphQL subscriptions
- [[Task Execution Flow]] — how task events are emitted
