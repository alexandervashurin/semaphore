# REST API

> Full REST API reference
>
> 📖 See also: [[GraphQL API]], [[WebSocket API]], [[MCP Server]]

---

## Authentication

All API calls (except login) require a JWT token:

```
Authorization: Bearer <token>
```

### Login

```bash
POST /api/auth/login
Content-Type: application/json

{"username": "admin", "password": "admin123"}
```

Response: `{ "token": "jwt...", "user": {...} }`

---

## Core Endpoints

### Projects

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/projects` | List all projects |
| `POST` | `/api/projects` | Create project |
| `GET` | `/api/projects/{id}` | Get project |
| `PUT` | `/api/projects/{id}` | Update project |
| `DELETE` | `/api/projects/{id}` | Delete project |

### Templates

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/templates` | List templates |
| `POST` | `/api/templates` | Create template |
| `PUT` | `/api/templates/{id}` | Update template |
| `DELETE` | `/api/templates/{id}` | Delete template |

### Tasks

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/tasks` | List tasks |
| `POST` | `/api/tasks` | Run task |
| `GET` | `/api/tasks/{id}` | Get task |
| `GET` | `/api/tasks/{id}/output` | Get task logs |

---

## Health Endpoints

| Endpoint | Description |
|----------|-------------|
| `/healthz` | Liveness probe — returns "OK" |
| `/readyz` | Readiness probe — returns JSON with checks |
| `/api/health` | Full health status |

---

## OpenAPI Spec

Full spec available at:
- [`api-docs.yml`](../api-docs.yml)
- Swagger UI at `/swagger` (when enabled)

---

## Next Steps

- [[GraphQL API]] — alternative query interface
- [[WebSocket API]] — real-time events
- [[MCP Server]] — AI tool integration
