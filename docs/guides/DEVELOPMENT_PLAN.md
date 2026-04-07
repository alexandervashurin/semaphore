# План разработки Velum

## Текущее состояние проекта (2026-04-07)

### Backend (Rust)

| Метрика | Значение |
|---------|----------|
| **Rust файлов** | ~491 |
| **Строк Rust кода** | ~54 500 |
| **Тестов** | 1283 passed, 0 failed |
| **Покрытие (tarpaulin)** | ~16.5% (4517/27348) |
| **Clippy** | 0 warnings |
| **cargo audit** | 0 CVE |

### Frontend

| Метрика | Значение |
|---------|----------|
| **Технология** | Vanilla JS + Material Design |
| **HTML страниц** | 76 файлов в `web/public/` |
| **Kubernetes страниц** | 33 (`k8s-*.html`) |
| **CSS** | `styles.css` + inline |
| **JS** | `app.js` + inline в HTML |

### Kubernetes интеграция

| Компонент | Статус |
|-----------|--------|
| **K8s API endpoints** | 180+ REST endpoints |
| **K8s UI страницы** | 33 страницы |
| **WebSocket streaming** | Exec, Port-forward, Logs, Events |
| **Helm** | Install, upgrade, rollback, uninstall |
| **Multi-cluster** | Cluster switcher |
| **RBAC** | can-i cache, audit logging |
| **kube-rs** | 0.98, k8s-openapi 0.24 |

---

## Завершённые спринты

### S0 — Critical Security Fixes ✅
- Обновление уязвимых зависимостей
- Cargo audit в CI
- JWT logout blacklist
- Alert HMAC signature
- Cron validation
- OIDC email claim

### S1 — Stabilization ✅
- Telegram Bot уведомления
- SSH Key Installation service
- Exporter/Importer fix
- Subscription service integration
- Kubernetes runbook integration
- Prometheus label aggregation
- Inventory sync completion

### S2 — Verification & Test Coverage ✅
- E2E deployment testing
- Postman collection verification
- K8s API performance benchmarks
- Тесты: 1250 → 1283

### S3 — Production Hardening & Docs ✅ (частично)
- Redis-backed task queue ✅
- WebSocket pub/sub via Redis ✅
- README обновлён ✅
- DEPLOY.md обновлён ✅
- CHANGELOG.md обновлён ✅
- Coverage в CI ✅

### S4 — Release ✅
- GitHub Release v2.5.2 опубликован
- Финальный quality gate пройден

---

## Открытые задачи

### TEST-01 — Test Coverage 80%+ (Critical)

**Текущее покрытие:** ~16.5% (tarpaulin LLVM), ~78% (оценка по unit-тестам)

**Что нужно покрыть:**
- `telegram_bot/mod.rs` — 153 строки, ~2% покрытие
- `workflow_executor.rs` — 188 строк, ~16%
- `ssh_agent.rs` — 242 строки, ~36%
- `restore.rs` — 127 строк, ~24%
- `job_pool.rs` — 135 строк, ~20%
- `task_pool_runner.rs` — 152 строки, ~22%
- `mailer.rs` — 70 строк, ~41%

**Сложность:** Многие модули требуют мокирования внешних зависимостей (Telegram API, SSH, SMTP).

### PROD-01 — Docker образ <50MB (High)
- Multi-stage build с scratch
- Статическая линковка (musl)

### PROD-02 — Cross-platform builds (High)
- macOS amd64/arm64
- Linux arm64

---

## Архитектура проекта

```
├── rust/src/
│   ├── api/            HTTP handlers, middleware, routes
│   │   ├── handlers/   36 handler файлов, 576+ pub fn
│   │   │   └── kubernetes/  44 файла K8s API
│   │   ├── extractors.rs  Auth user extractors
│   │   └── middleware/    CORS, rate limiting, auth
│   ├── models/         Data models (25+ structs)
│   ├── db/             Database layer
│   │   ├── sql/        SQLx implementations
│   │   │   └── managers/  50+ manager файлов
│   │   └── mock/       MockStore для тестов
│   ├── services/       Business logic
│   │   ├── task_runner/    Task execution
│   │   ├── task_pool*      Task queue management
│   │   ├── webhook.rs      Webhook sending
│   │   ├── scheduler.rs    Cron scheduling
│   │   └── telegram_bot/   Telegram notifications
│   ├── kubernetes/     K8s client, Helm, Jobs
│   ├── config/         Configuration loading
│   ├── plugins/        WASM plugin system
│   └── grpc/           gRPC server (stub)
├── web/public/         Frontend — 76 HTML файлов
│   ├── k8s-*.html      33 K8s страниц
│   ├── app.js          API client, sidebar
│   └── styles.css      Material Design
├── mcp/                MCP server (Rust)
└── deploy/             Docker compose файлы
```

---

## Технологический стек

| Компонент | Версия | Назначение |
|-----------|--------|------------|
| Rust | stable | Язык |
| Axum | 0.8 | Web framework |
| SQLx | 0.8 | Database |
| Tokio | 1.x | Runtime |
| kube-rs | 0.98 | Kubernetes client |
| jsonwebtoken | 9.3 | JWT |
| bcrypt | 0.17 | Password hashing |
| redis | 0.29 | Cache, pub/sub |
| wasmtime | 41.0.4 | WASM plugins |
| teloxide | 0.13 | Telegram (без runtime) |
| ldap3 | 0.11 | LDAP auth |
| oauth2 | 5.0 | OIDC |
| lettre | 0.11 | Email |
| async-graphql | 7.0 | GraphQL API |
| cron | 0.15 | Cron parsing |

---

## Документация

| Файл | Статус |
|------|--------|
| README.md | ✅ Обновлено |
| DEPLOY.md | ✅ Обновлено |
| CHANGELOG.md | ✅ Обновлено |
| docs/technical/CONFIG.md | ✅ Переписан |
| docs/technical/AUTH.md | ✅ Исправлен |
| docs/technical/WEBHOOK.md | ✅ Исправлен |
| docs/guides/TESTING.md | ✅ Переписан |
| docs/guides/DOCKER_DEMO.md | ✅ Переписан |
| docs/API.md | ⚠️ Требует исправления |
| docs/guides/CRUD_ENTITIES.md | ⚠️ Требует исправления |
| docs/future/* | ⚠require reorganization |
