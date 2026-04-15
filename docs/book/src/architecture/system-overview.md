# System Overview

> High-level architecture of Velum
>
> 📖 See also: [[Database Schema]], [[Auth & Security]], [[Task Execution Flow]], [[Kubernetes Integration]]

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│                  Web UI (Vanilla JS)             │
│            48 pages (28 base + 20 K8s)           │
└────────────────────┬────────────────────────────┘
                     │ HTTP / WebSocket
┌────────────────────▼────────────────────────────┐
│              API Layer (Axum 0.8)                │
│                                                  │
│  ┌──────────┐ ┌─────────┐ ┌──────────────────┐  │
│  │ REST API │ │GraphQL  │ │ WebSocket / MCP  │  │
│  │ 135+     │ │ Queries │ │ 60 Tools         │  │
│  └────┬─────┘ └────┬────┘ └────────┬─────────┘  │
│       │            │               │             │
│       └────────────┼───────────────┘             │
│                    ▼                              │
│           Middleware Stack                        │
│  • Auth (JWT/LDAP/OIDC)                          │
│  • Rate Limiting                                 │
│  • Correlation ID                                │
│  • Security Headers                              │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│              Service Layer                       │
│                                                  │
│  ┌──────────┐ ┌─────────┐ ┌──────────────────┐  │
│  │ Task     │ │Scheduler│ │ Alert/Notify     │  │
│  │ Runner   │ │ (cron)  │ │ Webhooks         │  │
│  └──────────┘ └─────────┘ └──────────────────┘  │
│  ┌──────────┐ ┌─────────┐ ┌──────────────────┐  │
│  │ Backup/  │ │ Export/ │ │ Remote Runners   │  │
│  │ Restore  │ │ Import  │ │ (Heartbeat)      │  │
│  └──────────┘ └─────────┘ └──────────────────┘  │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│              Data Layer                          │
│                                                  │
│  ┌──────────────────┐  ┌──────────────────────┐  │
│  │ PostgreSQL       │  │ Redis (HA mode)      │  │
│  │ (SQLx 0.8)       │  │ • Task Queue         │  │
│  │ • Users          │  │ • WebSocket Pub/Sub  │  │
│  │ • Projects       │  │ • Session Cache      │  │
│  │ • Tasks          │  └──────────────────────┘  │
│  │ • Templates      │                             │
│  └──────────────────┘                             │
└──────────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│          External Integrations                   │
│                                                  │
│  • Kubernetes (kube-rs 0.98)                     │
│  • Ansible / Terraform / Bash / PowerShell       │
│  • LDAP / OIDC                                   │
│  • Telegram Bot (teloxide)                       │
│  • Prometheus Metrics                            │
│  • HashiCorp Vault                               │
└──────────────────────────────────────────────────┘
```

---

## Key Components

### Backend (Rust)

| Module | Lines | Description |
|--------|-------|-------------|
| `api/` | 9000+ | HTTP handlers, routes, middleware |
| `services/` | 12000+ | Business logic |
| `db/` | 5000+ | SQLx PostgreSQL |
| `kubernetes/` | 2300+ | K8s client integration |
| `models/` | 8000+ | Data structures |

### Frontend (Vanilla JS)

| Component | Count | Description |
|-----------|-------|-------------|
| HTML Pages | 48 | Core + K8s pages |
| CSS | 1692 lines | Material Design, responsive |
| JS | 1 app.js | API client, sidebar |

---

## Next Steps

- [[Database Schema]] — PostgreSQL schema details
- [[Auth & Security]] — authentication architecture
- [[Task Execution Flow]] — how tasks run
- [[Kubernetes Integration]] — K8s integration details
