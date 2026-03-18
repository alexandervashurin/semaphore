# 🗺️ Дорожная карта проекта Velum (Rust)

> **Репозиторий:** https://github.com/tnl-o/rust_semaphore
> **Upstream (Go-оригинал — эталон фич):** https://github.com/velum/velum
> **Последнее обновление:** 14 марта 2026 г.
> **Цель:** Полная миграция Velum с Go на Rust + замена Vue 2 на Vanilla JS, feature parity с Go-оригиналом.

---

## 📚 Содержание

1. [Цель проекта](#цель-проекта)
2. [Стек технологий](#стек-технологий)
3. [Архитектура](#архитектура)
4. [Запуск одним Docker-контейнером](#запуск-одним-docker-контейнером)
5. [План разработки](#план-разработки)
6. [Статус функций](#статус-функций)

---

## 🎯 Цель проекта

**Velum** — open-source веб-интерфейс для Ansible, Terraform, OpenTofu, PowerShell и других DevOps-инструментов. Оригинал написан на Go + Gin + Vue 2.

Этот форк:
- **Мигрирует бэкенд** с Go на Rust (Axum + SQLx + Tokio) — производительнее, безопаснее, меньше памяти
- **Мигрирует фронтенд** с Vue 2 (EOL декабрь 2023) на **Vanilla JS** — меньше зависимостей, быстрее загрузка, вечная поддержка
- Сохраняет **полный feature parity** с Go-оригиналом (`velum/velum`)

**Что должно работать:**
- Управление проектами с ролевой моделью (admin/manager/runner)
- Templates, Inventories, Keys, Repositories, Environments
- Task Runner — реальный запуск ansible-playbook, terraform, bash
- WebSocket для стриминга логов в реальном времени
- Schedules (cron), Webhooks (входящие и исходящие)
- Auth: JWT, bcrypt, TOTP (2FA), LDAP, OIDC/OAuth2, refresh token
- Audit log, Notifications (email, Slack, Telegram)
- Prometheus metrics, Backup/Restore

---

## 🛠️ Стек технологий

### Backend (Rust)

| Категория | Технология | Версия | Назначение |
|-----------|-----------|--------|------------|
| **Язык** | Rust | 1.80+ | Основной язык backend |
| **Веб-фреймворк** | Axum | 0.8 | HTTP сервер, роутинг, middleware |
| **Асинхронность** | Tokio | 1.x | Async runtime |
| **База данных** | SQLx | 0.8 | Асинхронный SQL клиент |
| **БД (поддержка)** | PostgreSQL, MySQL, SQLite | — | Хранение данных |
| **Auth** | JWT (jsonwebtoken), bcrypt, TOTP | 9.3, 0.17 | Токены, пароли, 2FA |
| **OIDC/OAuth2** | oauth2 | 5.0 | Внешняя аутентификация |
| **LDAP** | ldap3 | 0.11 | LDAP/AD аутентификация |
| **WebSocket** | axum ws, tokio-tungstenite | — | Real-time лог-стриминг |
| **Логирование** | tracing, tracing-subscriber | 0.1 | Структурированное логирование |
| **Email** | lettre | 0.11 | SMTP уведомления |
| **Git** | git2 | 0.20 | Работа с Git репозиториями |
| **SSH** | ssh2 | 0.9 | SSH подключения |
| **Шифрование** | AES-256, sha2, hmac | — | Хранение секретов |
| **Метрики** | prometheus | 0.13 | Prometheus экспортёр |
| **CLI** | clap | 4.5 | Командная строка |
| **GraphQL** | async-graphql | 7.0 | GraphQL API |

### Frontend (Vanilla JS)

| Категория | Технология | Описание |
|-----------|-----------|--------|
| **Фреймворк** | Vanilla JS (нет фреймворка) | Браузерные стандарты: History API, Fetch, WebSocket |
| **Стили** | SCSS (Gulp сборка) | Material Design-inspired компоненты |
| **Роутинг** | router.js (History API) | Клиентский роутер |
| **State** | store.js (Proxy) | Реактивный state без зависимостей |
| **API-клиент** | api.js (fetch + interceptors) | REST-клиент |
| **Bundle** | ~50 KB (vs ~500 KB Vue 2) | В 10 раз меньше Vue 2 |
| **Поддержка** | Вечная | Нет EOL зависимостей |

> ⚠️ Vue 2 достиг End-of-Life в декабре 2023. Vanilla JS миграция завершена (базовые страницы). В работе: Charts ✅ готов (Chart.js Analytics page).

### DevOps

| Категория | Технология |
|-----------|-----------|
| **Контейнеризация** | Docker multi-stage |
| **Оркестрация** | Docker Compose |
| **БД (prod)** | PostgreSQL 16 |
| **БД (dev)** | SQLite |
| **CI/CD** | GitHub Actions (build + test + clippy) |
| **API-документация** | OpenAPI (api-docs.yml) |

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────┐
│                    Velum (Rust)                   │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐  │
│  │  Frontend   │    │   Backend   │    │  Database   │  │
│  │ Vanilla JS  │◄──►│  Rust Axum  │◄──►│ PostgreSQL  │  │
│  │  ~50 KB     │    │  0.8 Tokio  │    │   MySQL     │  │
│  │  No EOL     │    │  SQLx 0.8   │    │   SQLite    │  │
│  └─────────────┘    └─────────────┘    └─────────────┘  │
└─────────────────────────────────────────────────────────┘
         │                   │
         ▼                   ▼
  ┌─────────────┐    ┌─────────────┐
  │  Static     │    │  REST API   │
  │  Files      │    │  WebSocket  │
  └─────────────┘    └─────────────┘
```

### Структура проекта

```
rust_semaphore/
├── rust/                    # Backend на Rust
│   ├── src/
│   │   ├── api/            # HTTP handlers, routes, middleware
│   │   ├── db/             # Store traits + SQLite/PG/MySQL impl
│   │   ├── services/       # Task runner, scheduler, alert, metrics...
│   │   ├── models/         # Сущности данных (serde)
│   │   ├── config/         # Конфигурация
│   │   └── utils/          # Encryption, mailer, SSH...
│   ├── tests/              # Integration tests
│   └── Cargo.toml
├── web/
│   ├── vanilla/            # Vanilla JS фронтенд (активный)
│   │   ├── js/             # app.js, router.js, api.js, components/
│   │   └── css/            # SCSS стили
│   └── src/                # Vue 2 (legacy, не используется)
├── Dockerfile
├── docker-compose.yml
├── MASTER_PLAN.md          # Живой план задач
└── ROADMAP.md              # Этот файл
```

---

## 🐳 Запуск одним Docker-контейнером

### Быстрый старт (SQLite + встроенный сервер)

```bash
# Запуск backend (SQLite)
cd rust
export SEMAPHORE_DB_PATH=/tmp/semaphore.db
cargo run -- server --host 0.0.0.0 --port 3000

# Создать admin-пользователя
cargo run -- user add --username admin --name "Admin" \
  --email admin@localhost --password admin123 --admin

# Открыть UI
http://localhost:3000
```

### Docker Compose (PostgreSQL + backend)

```bash
docker compose up -d
```

### Параметры конфигурации (env)

```bash
SEMAPHORE_DB_PATH=/data/semaphore.db   # SQLite
SEMAPHORE_DB_URL=postgres://...        # PostgreSQL
SEMAPHORE_JWT_SECRET=your-secret       # JWT-ключ (обязательно в prod)
SEMAPHORE_ACCESS_KEY_ENCRYPTION=...    # AES ключ для секретов
```

---

## 📋 План разработки

### ✅ Завершено

- [x] Миграция бэкенда с Go на Rust (Axum + SQLx + Tokio)
- [x] Поддержка SQLite, PostgreSQL, MySQL
- [x] Auth: JWT, bcrypt, refresh token, logout
- [x] Auth: TOTP 2FA, OIDC/OAuth2, LDAP
- [x] CRUD: Projects, Templates, Inventories, Keys, Repositories, Environments
- [x] CRUD: Views, Schedules, Integrations (входящие webhooks)
- [x] Task Runner — реальный запуск ansible-playbook, terraform, bash
- [x] WebSocket — стриминг логов задач в реальном времени
- [x] Scheduler (cron-runner) — фоновый tokio task
- [x] Webhooks исходящие (POST на смену статуса задачи)
- [x] Notifications: Email (SMTP/TLS), Slack, Telegram
- [x] Audit Log
- [x] Prometheus Metrics
- [x] Secret storage (AES-256 шифрование ключей)
- [x] Backup / Restore
- [x] GraphQL API (async-graphql 7.0)
- [x] Security middleware: rate limiting, CORS, security headers
- [x] CI/CD (GitHub Actions: build + test + clippy)
- [x] **Vanilla JS фронтенд** — все страницы CRUD (100% базовая миграция)
- [x] **Task Log Viewer** — WebSocket + ANSI-цвета в браузере
- [x] **Integration tests** — 10 тестов (auth, projects, health)
- [x] **Windows SQLite path fix** — корректная обработка путей

### 🔄 В работе

- [x] **Integration tests** — 20 тестов (auth, projects, keys, inventories, repositories, environments, templates, tasks, 2026-03-14)
- [x] **Charts/Analytics** — Chart.js line + doughnut charts на странице Analytics (2026-03-14)
- [x] **Docker multi-stage** — distroless/cc-debian12:nonroot, цель < 50 MB (2026-03-14)

### 📅 Запланировано (ближайшие)

- [ ] Проверка паритета схем SQLite / PostgreSQL
- [ ] Dark theme в Vanilla JS фронтенде
- [ ] Keyboard shortcuts
- [ ] `cargo clippy -- -D warnings` — 0 предупреждений
- [ ] Покрытие тестами ≥ 60% критических путей

### 🔮 Долгосрочно (после feature parity)

После достижения полного pariteta с Go-оригиналом:

- [ ] Кластерный режим / HA
- [ ] Redis кэширование
- [ ] Helm chart для Kubernetes
- [ ] Distributed tracing (OpenTelemetry)
- [ ] gRPC API для внутренних сервисов

> **Примечание:** Desktop/Mobile приложения, AI-ассистент и другие фичи, выходящие
> за рамки Go-оригинала, не входят в текущую область проекта.

---

## 📊 Статус функций

### Ядро

| Функция | Статус | Описание |
|--------|--------|----------|
| **Аутентификация** | ✅ Готово | JWT + refresh, bcrypt, 2FA TOTP |
| **LDAP** | ✅ Готово | Конфиг + handler подключён |
| **OAuth2/OIDC** | ✅ Готово | Multi-provider |
| **REST API** | ✅ Готово | Feature parity с Go-оригиналом |
| **WebSocket** | ✅ Готово | Стриминг логов задач |
| **База данных** | ✅ Готово | PostgreSQL, MySQL, SQLite |

### Управление задачами

| Функция | Статус |
|--------|--------|
| Task Runner (ansible/terraform/bash) | ✅ Готово |
| WebSocket лог-стриминг (бэкенд) | ✅ Готово |
| Task Log Viewer (фронтенд, ANSI) | ✅ Готово |
| Schedules (cron) | ✅ Готово |
| Stop task | ✅ Готово |

### Уведомления

| Функция | Статус |
|--------|--------|
| Email (SMTP) | ✅ Готово |
| Slack webhook | ✅ Готово |
| Telegram Bot API | ✅ Готово |
| Webhooks исходящие | ✅ Готово |

### Фронтенд (Vanilla JS)

| Страница | Статус |
|---------|--------|
| Login | ✅ |
| Dashboard (проекты) | ✅ |
| Templates (CRUD) | ✅ |
| Inventories (CRUD) | ✅ |
| Repositories (CRUD) | ✅ |
| Environments (CRUD) | ✅ |
| Keys (CRUD) | ✅ |
| История задач | ✅ |
| **Task detail + live log (WS)** | ✅ |
| Team / Users | ✅ |
| Schedules | ✅ |
| Integrations | ✅ |
| Audit Log | ✅ |
| Analytics | ✅ (с Chart.js charts) |
| Settings | ✅ |

### Инфраструктура

| Функция | Статус |
|--------|--------|
| Docker Compose | ✅ Готово |
| GitHub Actions CI (Rust) | ✅ Готово |
| Unit-тесты (524) | ✅ Готово |
| Integration tests (20) | ✅ Готово |
| Docker multi-stage (distroless) | ✅ Готово |

---

## 🚀 Запуск для разработки

```bash
# Backend
cd rust && cargo run -- server --host 0.0.0.0 --port 3000

# Тесты
cd rust && cargo test
cd rust && cargo test --test api_integration

# Линтер
cd rust && cargo clippy -- -D warnings

# Frontend сборка
cd web && npm run vanilla:build

# Всё через Docker
docker compose up -d
```

---

*Последнее обновление: 14 марта 2026 г. (обновление 2 — 20 tests, Charts, Docker distroless)*
