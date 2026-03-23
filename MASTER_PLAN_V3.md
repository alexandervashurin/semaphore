# MASTER_PLAN V3 — Velum: Стать лучше AWX и Ansible Tower

> **Последнее обновление:** 2026-03-23 (сессия 8 — v4.0 Multi-Tenancy API + UI завершены)
> **Версия:** 4.0
> **Статус:** ✅ v3.2 FEATURE COMPLETE | ✅ v4.0 HA CLUSTER | ✅ v4.0 MULTI-TENANCY

---

## Конкурентная позиция: Что Velum уже выигрывает у AWX/Tower

| Параметр | AWX/Tower | Velum |
|---|---|---|
| Память | 500MB–2GB (Python + Celery + Redis) | ~80MB (Rust binary) |
| Старт | 30–90 сек | <1 сек |
| Деплой | 8+ контейнеров | 1 бинарник + SQLite |
| Terraform | 3rd-party плагин | First-class citizen |
| Лицензия | GPLv3 / подписка $14K/год | MIT, бесплатно навсегда |
| Frontend | Angular (тяжёлый, устаревший) | Vanilla JS, быстрый |
| MCP интеграция | ❌ | ✅ встроен (v3.2) |
| AI Error Analysis | ❌ | ✅ встроен (v3.2) |
| Workflow DAG | ✅ сложный | ✅ реализован (v2.2) |
| Survey Forms | ✅ | ✅ реализованы (v2.2) |
| LDAP Group Sync | ✅ частично | ✅ реализован (v2.4) |
| Notification Policies | ✅ (Slack/PD) | ✅ реализованы (v2.5) |
| Custom Credentials | ✅ | ✅ реализованы (v2.4) |
| AI Error Analysis | ❌ | ✅ встроен (v2.3+v3.2) |
| GitOps Drift Detection | ❌ | ✅ реализован (v2.3) |
| Terraform Plan Preview | ❌ | ✅ реализован (v2.3) |
| Template Dry Run | ❌ | ✅ реализован (v2.2) |
| Log Annotations | ❌ | ✅ реализованы (v2.3) |
| CLI Tool | ✅ | ✅ v2.7 |
| Rollback / Snapshots | ❌ | ✅ v3.0 |
| Template Marketplace | ❌ | ✅ v3.0 |
| Terraform Cost Tracking | ❌ | ✅ v3.0+ |
| Diff между запусками | ❌ | ✅ v3.0+ |

---

## БЛОК 1 — Закрыть критические пробелы (Enterprise миграция)

### 🔴 Приоритет 1: Workflow Builder (DAG) — v2.2

Это **главная причина**, почему предприятия не уходят от AWX. Нужен визуальный редактор пайплайнов:

```
[Git Pull] → [Terraform Plan]
                ↓ success         ↓ failure
         [Terraform Apply]    [Notify Slack]
                ↓
         [Run Ansible Playbook]
                ↓ always
         [Send Report Email]
```

**Что реализовать:**
- Граф из шаблонов (nodes) и переходов по условию (`on_success`, `on_failure`, `always`)
- Drag-and-drop UI (simple canvas с SVG-стрелками, без внешних зависимостей)
- Хранение в БД: таблицы `workflows`, `workflow_nodes`, `workflow_edges`, `workflow_runs`
- Запуск всего DAG как единой "Workflow Job"
- Real-time статус каждой ноды через WebSocket

**Backend (Rust):**
```sql
CREATE TABLE workflows (
    id INTEGER PRIMARY KEY, project_id INTEGER NOT NULL,
    name TEXT NOT NULL, description TEXT,
    created DATETIME, updated DATETIME
);
CREATE TABLE workflow_nodes (
    id INTEGER PRIMARY KEY, workflow_id INTEGER NOT NULL,
    template_id INTEGER, label TEXT, pos_x INTEGER, pos_y INTEGER
);
CREATE TABLE workflow_edges (
    id INTEGER PRIMARY KEY, workflow_id INTEGER NOT NULL,
    from_node INTEGER NOT NULL, to_node INTEGER NOT NULL,
    condition TEXT NOT NULL CHECK (condition IN ('success','failure','always'))
);
CREATE TABLE workflow_runs (
    id INTEGER PRIMARY KEY, workflow_id INTEGER NOT NULL,
    status TEXT NOT NULL, started DATETIME, finished DATETIME
);
```

**Новые API endpoints:**
```
GET/POST   /api/projects/{id}/workflows
GET/PUT/DELETE  /api/projects/{id}/workflows/{wid}
POST       /api/projects/{id}/workflows/{wid}/run
GET        /api/projects/{id}/workflows/{wid}/runs
```

**Frontend:** `web/public/workflow.html` — SVG canvas editor, drag-and-drop нод, цветовое кодирование условий

---

### 🔴 Приоритет 2: Survey (Интерактивные формы) — v2.2

AWX Survey — одна из самых используемых фич. Пользователь запускает шаблон и видит форму:

```
┌─────────────────────────────────────────────┐
│ 🚀 Запуск: "Deploy Backend"                 │
├─────────────────────────────────────────────┤
│ Версия для деплоя:  [ v2.3.1          ]    │
│ Окружение:         ○ dev ○ staging ● prod   │
│ Количество реплик: [ 3 ]                    │
│ Очистить кеш:      ☑ Да                     │
│                                             │
│              [Отмена]  [🚀 Запустить]       │
└─────────────────────────────────────────────┘
```

Заполненные значения идут в `extra_vars` к Ansible. **Делает автоматизацию self-service** — не-технари могут запускать плейбуки через веб-форму.

**Что реализовать:**
- Поле `survey_vars` (JSON) в таблице `templates`:
```json
[
  {"name": "version", "type": "text", "label": "Версия", "required": true, "default": "latest"},
  {"name": "env", "type": "select", "label": "Окружение", "options": ["dev","staging","prod"]},
  {"name": "replicas", "type": "integer", "label": "Реплики", "min": 1, "max": 10, "default": 2},
  {"name": "flush_cache", "type": "boolean", "label": "Очистить кеш", "default": false}
]
```
- UI-конструктор survey в настройках шаблона
- Диалог перед запуском — заполняет extra_vars

---

### 🔴 Приоритет 3: LDAP Groups → Teams автосинк — v2.4

Сейчас LDAP аутентифицирует пользователей, но не синхронизирует группы в команды проектов.

**Что реализовать:**
- Маппинг: `CN=devops-team,OU=Groups,DC=company,DC=com` → Проект "Prod Infrastructure", роль "Deploy"
- Автосинк при каждом логине
- UI для настройки маппингов в системных настройках

---

### 🟠 Приоритет 4: Notification Policies — v2.5

Сейчас только Email + Telegram. Добавить:
- **Slack** (webhooks — очень просто реализовать)
- **Microsoft Teams** (adaptive card webhooks)
- **PagerDuty** (Events API v2 для critical alerts)
- **Webhook** с настраиваемым payload template (Jinja2-подобный)
- Политика: `on_failure`, `on_success`, `on_start`, `always`
- Привязка уведомлений к конкретным шаблонам/проектам

---

### 🟠 Приоритет 5: Custom Credential Types — v2.4

AWX позволяет создавать свои типы секретов с маппингом в env vars, файлы или stdin:

```yaml
name: "AWS AssumeRole"
fields:
  - id: aws_access_key
    type: string
    secret: true
  - id: aws_secret_key
    type: string
    secret: true
injectors:
  env:
    AWS_ACCESS_KEY_ID: "{{ aws_access_key }}"
    AWS_SECRET_ACCESS_KEY: "{{ aws_secret_key }}"
```

---

## БЛОК 2 — Убийственные фичи (которых нет ни у кого)

### 🚀 AI-интеграция — главный дифференциатор 2026 — v2.3

AWX и Tower не имеют AI. Это огромное окно возможностей:

**1. Анализ ошибок задач**
```
Задача упала → ИИ анализирует вывод →
"Ошибка связана с недоступностью хоста 192.168.1.5.
Возможные причины: SSH-ключ истёк, хост выключен, firewall.
Проверьте: ssh -i ~/.ssh/key user@192.168.1.5"
```

**2. Генерация Ansible из описания**
```
"Установи nginx на все хосты группы webservers, включи, добавь в автозапуск"
→ автогенерирует playbook YAML
```

**3. Умное автодополнение extra_vars** — предлагает переменные на основе плейбука

**Реализация:** API-вызов к Claude/OpenAI из backend (Rust). Модель и ключ задаются в системных настройках.

---

### 🚀 GitOps-Native — v2.3

**Drift Detection для Terraform:**
- Периодически запускать `terraform plan -detailed-exitcode` в фоне
- Если есть дрейф (план ≠ состояние) — показывать алерт в UI + уведомление
- Dashboard с "Drift Status" по всем Terraform-проектам

**Branch Environments:**
- При открытии PR в GitHub → автоматически поднять стейджинг через Terraform
- При мердже PR → задеплоить в prod через pipeline
- При закрытии PR → уничтожить окружение

---

### 🚀 Rollback в один клик — v3.0

Tower этого не умеет вообще.

- Каждый успешный запуск шаблона создаёт **snapshot** (зафиксированная ревизия git, переменные, инвентарь)
- Кнопка "Откатить к версии от 18 марта 14:32" — перезапускает с теми же параметрами
- История snapshots с diff между ними

---

### 🚀 Marketplace шаблонов — v3.0

Встроенный каталог готовых шаблонов:
- "Деплой на Ubuntu 22.04" → импортируй и запусти
- Интеграция с Ansible Galaxy roles
- Community templates из GitHub

---

### 🚀 Developer CLI — v2.7

```bash
velum run template "Deploy Backend" --env=prod --extra-vars="version=2.3.1"
velum status                    # список running задач
velum logs 1234                 # live logs задачи
velum approve 1234              # подтвердить gated задачу
velum workflow run "Full Deploy Pipeline"
```

CLI превращает Velum в центр управления для разработчиков, а не только ops-команды. Реализация: Rust binary как отдельный бинарник `velum` в том же cargo workspace.

---

### 🚀 Terraform Cost Tracking — v3.0

- Интеграция с [Infracost](https://www.infracost.io/): стоимость изменений ПЕРЕД `terraform apply`
- "Это применение добавит $340/месяц к вашему AWS-счёту"
- Dashboard с историей расходов по проектам

---

## БЛОК 3 — UX, которого у AWX нет вообще

| Фича | Описание | Статус |
|---|---|---|
| **Тёмная тема** | Полная тёмная тема | ✅ Реализована |
| **Mobile-first** | Velum responsive, Tower — нет | ✅ Реализовано |
| **Template Dry Run** | Кнопка "Check Mode" — ansible с `--check` | ✅ Реализовано (v2.2) |
| **Diff между запусками** | "Что изменилось с предыдущего запуска" | ✅ Реализовано (v3.0+) |
| **Аннотации к логам** | 13 классов: Ansible ok/changed/fatal, Terraform +/-/~ | ✅ Реализовано (v2.3) |
| **Approvals/Gate** | Уже есть — больше чем у AWX | ✅ Реализовано |
| **Terraform Plan Preview** | Plan/Apply radio в диалоге, баннер в просмотре задачи | ✅ Реализовано (v2.3) |
| **MCP Server (Rust)** | Управление через AI-ассистентов | ✅ v3.1 |

---

## Фаза 1 — MCP Server встроенный в Velum (v3.2, реализовано)

### Что такое MCP и зачем

**Model Context Protocol (MCP)** — открытый протокол от Anthropic для подключения AI-ассистентов (Claude, Cursor, VS Code Copilot) к внешним инструментам. Velum MCP сервер позволяет:

```
"Запусти деплой backend в prod"                    → Claude → velum_mcp → задача запущена
"Покажи последние ошибки в проекте Infrastructure" → Claude → анализ логов + объяснение
"Создай расписание для backup каждую ночь в 3:00"  → Claude → cron создан
```

### Ключевое архитектурное решение: встроен в Velum, не отдельный процесс

**v3.2 меняет подход:** MCP-сервер встроен прямо в главный Axum-сервер Velum.

| Параметр | Отдельный binary (v3.1) | Встроенный (v3.2) |
|---|---|---|
| Деплой | 2 процесса, 2 конфига | 1 бинарник, 1 конфиг |
| Конфигурация | Отдельный `.env`, отдельный токен | Автоматически — тот же JWT |
| UI настройки | Нет | ✅ Страница `mcp.html` в сайдбаре |
| Доступ к данным | Через HTTP API (round-trip) | Напрямую через store (нет latency) |
| Обновление | Отдельный CI/CD | Вместе с Velum |
| Ссылка в меню | Нет | ✅ "MCP / AI" в сайдбаре |

### Архитектура v3.2: Embedded MCP

```
┌─────────────────────────────────────────────────────────┐
│  AI Client (Claude Desktop / Claude Code / Cursor)       │
│  "Запусти деплой prod"                                   │
└──────────────────────────────┬──────────────────────────┘
                               │ HTTP JSON-RPC 2.0
                               │ POST /mcp  + Bearer JWT
                               ▼
┌─────────────────────────────────────────────────────────┐
│  Velum (Rust/Axum) — http://localhost:3000               │
│  ┌────────────────────────────────────────────────────┐ │
│  │  REST API  /api/**   (28+ страниц фронтенда)       │ │
│  │  WebSocket /ws        (live task logs)              │ │
│  │  MCP Gate  POST /mcp  ← НОВОЕ                      │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │ │
│  │  │  Projects    │  │  Schedules ★ │  │  AI ★★   │  │ │
│  │  │  Templates   │  │  Analytics ★ │  │  Runners │  │ │
│  │  │  Tasks       │  │  Inventory   │  │  Keys    │  │ │
│  │  └──────┬───────┘  └──────────────┘  └──────────┘  │ │
│  └─────────┼──────────────────────────────────────────┘ │
│            │ Arc<AppState> store (прямой доступ, 0 HTTP) │
│  ┌─────────▼──────────────────────────────────────────┐ │
│  │  SQLite / PostgreSQL / MySQL                        │ │
│  └────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Почему Rust, а не Python

| Параметр | Python MCP (semaphore-mcp) | Velum MCP (Rust, встроенный) |
|---|---|---|
| Память | ~50MB отдельно | 0 — часть Velum |
| Процессы | 2 (velum + mcp) | 1 |
| Конфиги | 2 | 1 |
| Токен | Отдельный | Тот же JWT |
| Latency доступа к данным | HTTP round-trip | Прямой вызов store |
| Лицензия | AGPL-3.0 | MIT |

### Инструменты MCP (60 tools)

| Категория | Инструменты | Уникально |
|---|---|---|
| Projects (5) | list, get, create, update, delete | — |
| Templates (7) | list, get, create, update, delete, **run**, stop_all | — |
| Tasks (11) | list, get, run, stop, output, filter, confirm, reject, bulk_stop, waiting, latest_failed | confirm/reject ★ |
| Inventory (5) | list, get, create, update, delete | — |
| Repositories (6) | list, get, create, update, delete, **branches** | — |
| Environments (5) | list, get, create, update, delete | — |
| Access Keys (4) | list, get, create, delete | — |
| Schedules (6) ★ | list, get, create, **toggle**, delete, validate_cron | не в semaphore-mcp |
| Analytics (4) ★ | project_stats, trends, system, **health_summary** | не в semaphore-mcp |
| Runners (4) ★ | list, status, **toggle**, clear_cache | не в semaphore-mcp |
| Playbooks (5) ★ | list, get, **run**, **sync**, history | не в semaphore-mcp |
| Audit (3) ★ | audit_log, project_events, system_info | не в semaphore-mcp |
| AI Analysis (2) ★★ | analyze_failure, bulk_analyze | нет ни у кого |

**60 инструментов** vs 35 у [cloin/semaphore-mcp](https://github.com/cloin/semaphore-mcp)

### Файловая структура (embedded в главный крейт)

```
rust/src/api/mcp/
├── mod.rs          — публичный интерфейс модуля, re-export handlers
├── protocol.rs     — JSON-RPC 2.0 типы (McpRequest, McpResponse, ToolContent)
├── handler.rs      — Axum handlers: mcp_endpoint, get/update mcp_settings, get_mcp_tools
└── tools.rs        — 35 инструментов с прямым доступом к AppState.store

web/public/
└── mcp.html        — страница "MCP / AI": статус, конфиг, каталог инструментов

API маршруты (добавлены в routes.rs):
  POST /mcp                  — JSON-RPC эндпоинт для Claude
  GET  /api/mcp/settings     — настройки MCP
  PUT  /api/mcp/settings     — обновить настройки
  GET  /api/mcp/tools        — список всех инструментов (для UI)
```

### Подключение Claude

**Claude Desktop** (`~/.claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "velum": {
      "type": "http",
      "url": "http://localhost:3000/mcp",
      "headers": { "Authorization": "Bearer <ваш-jwt-токен>" }
    }
  }
}
```

**Claude Code (CLI):**
```bash
claude mcp add-json velum '{"type":"http","url":"http://localhost:3000/mcp","headers":{"Authorization":"Bearer <token>"}}'
```

### Статус: ✅ Реализовано (v3.2, встроен в Velum)

---

## Сводная таблица фаз

| Фаза | Версия | Фича | Статус | Квартал |
|---|---|---|---|---|
| 0 | v2.1 | **Базовая платформа** (75+ API, 28+ страниц, auth, scheduler) | ✅ Готово | Q1 2026 |
| 1 | v3.1 | **MCP Server (Rust, standalone)** — 60 инструментов | ✅ Готово | Q1 2026 |
| 1b | v3.2 | **MCP встроен в Velum** — страница настроек, сайдбар, store-прямой доступ | ✅ Готово | Q1 2026 |
| 2 | v2.2 | **Workflow DAG Builder** + **Survey Forms** | ✅ Готово | Q1 2026 |
| 3 | v2.3 | **AI Analysis** + **Drift Detection** + **Terraform Plan Preview** + **Log Annotations** | ✅ Готово | Q1 2026 |
| 4 | v2.4 | **LDAP Group Sync** + **Custom Credential Types** | ✅ Готово | Q1 2026 |
| 5 | v2.5 | **Notification Policies** (Slack/Teams/PagerDuty) | ✅ Готово | Q1 2026 |
| 6 | v2.6 | **Template Dry Run** + **Log Annotations** | ✅ Готово | Q1 2026 |
| 7 | v2.7 | **CLI Tool `velum`** | ✅ Готово | Q1 2026 |
| 8 | v3.0 | **Rollback & Snapshots** + **Template Marketplace** | ✅ Готово | Q1 2026 |
| 9 | v3.0+ | **Terraform Cost Tracking** + **Diff между запусками** | ✅ Готово | Q1 2026 |

---

## Текущее состояние (v3.0 — Feature Complete)

### Реализовано ✅

- **Бэкенд**: 75+ API endpoints, 667 тестов, 0 Clippy warnings
- **Фронтенд**: 30+ HTML страниц, полный feature parity с Go-оригиналом
- **Auth**: JWT, bcrypt, TOTP 2FA, LDAP, OIDC, refresh tokens
- **Task Runner**: реальный запуск ansible/terraform/bash с WebSocket логами
- **Scheduler**: cron-расписания с автозапуском
- **Distributed Runners**: самостоятельная регистрация, health check, теги
- **Analytics**: Chart.js дашборд с трендами
- **Secret Storage**: HashiCorp Vault, DVLS, Fortanix
- **Webhooks**: матчеры, extract values, алиасы
- **Design**: Material Design, Roboto, teal #005057, Font Awesome 6.5
- **Deploy**: Docker (demo/dev/prod), DEB пакет, native binary
- **MCP Server (Rust, standalone)**: 60 инструментов, stdio + HTTP, ~5MB бинарник (`mcp/`)
- **MCP встроенный (v3.2)**: `POST /mcp` прямо в Velum, страница `mcp.html` с UI настроек, link в сайдбаре
- **CLI Tool `velum` (v2.7)**: 10 команд (projects, templates, run, status, logs, approve, stop, whoami, version, tasks)
- **Rollback & Snapshots (v3.0)**: снапшоты задач, rollback в один клик, `snapshots.html`
- **Template Marketplace (v3.0)**: 11 community templates, import в проект, `marketplace.html`
- **LDAP Group Sync (v2.4)**: `memberOf` → Teams auto-sync, `ldap_groups.html`
- **GitOps Drift Detection (v2.3)**: реальный diff git vs live, `drift.html`
- **Terraform Plan Preview (v2.3)**: Plan/Apply радио-кнопки, dry_run banner в task.html
- **Terraform Cost Tracking (v3.0+)**: Infracost-ready API, `costs.html` с историей и summary, cost banner в task.html
- **Diff между запусками (v3.0+)**: LCS diff engine, `diff.html` с unified/split view, compare mode в history.html

### Открытые задачи

- ~~T-BE-15: `exporter_entities.rs` restore пользователей~~ — ✅ **РЕШЕНО** (удалён dead code, исправлены предупреждения Clippy)

---

## 🎯 ПЛАН РАЗРАБОТКИ v4.0

### БЛОК 4 — Масштабирование и Enterprise (v4.0)

#### 🔴 Приоритет 1: High Availability Cluster — ✅ РЕАЛИЗОВАНО (v4.0)

**Цель:** Поддержка кластерной архитектуры для enterprise-развёртываний

**Реализовано:**
- ✅ Redis HA backend для хранения сессий
- ✅ Health check endpoints для Kubernetes (`/api/health/live`, `/api/health/ready`, `/api/health/full`)
- ✅ Graceful shutdown с обработкой SIGTERM/SIGINT
- ✅ HA конфигурация через переменные окружения (`SEMAPHORE_HA_*`)
- ✅ Node ID для идентификации узлов кластера

**Конфигурация:**
```bash
SEMAPHORE_HA_ENABLE=true
SEMAPHORE_HA_REDIS_HOST=localhost
SEMAPHORE_HA_REDIS_PORT=6379
SEMAPHORE_HA_REDIS_PASSWORD=secret
```

**Kubernetes Probes:**
```yaml
livenessProbe:
  httpGet:
    path: /api/health/live
    port: 3000
readinessProbe:
  httpGet:
    path: /api/health/ready
    port: 3000
```

---

#### 🔴 Приоритет 2: Multi-Tenancy (Организации) — 🔄 В РАБОТЕ (БАЗА)

**Цель:** Поддержка нескольких независимых организаций в одном экземпляре

**Реализовано (База):**
- ✅ Модель `Organization` с квотами (projects, users, tasks/month)
- ✅ Модель `OrganizationUser` для связи пользователей с организациями
- ✅ Миграция БД: таблицы `organization`, `organization_user`
- ✅ Поле `org_id` в таблице `project`
- ✅ `OrganizationManager` trait (11 методов)
- ✅ SQL реализация CRUD для организаций
- ✅ Проверка квот (`check_organization_quota`)
- ✅ StoreWrapper реализация

**Требуется реализовать:**
- ⏳ API endpoints для организаций (`/api/organizations`)
- ⏳ UI страницы для управления организациями
- ⏳ White-labeling: кастомизация UI под организацию
- ⏳ Изоляция данных между организациями

**Схема БД:**
```sql
CREATE TABLE organization (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    settings JSONB,
    quota_max_projects INTEGER,
    quota_max_users INTEGER,
    quota_max_tasks_per_month INTEGER,
    active BOOLEAN NOT NULL DEFAULT true,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated TIMESTAMPTZ
);

CREATE TABLE organization_user (
    id SERIAL PRIMARY KEY,
    org_id INTEGER NOT NULL REFERENCES organization(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member',
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id, user_id)
);

ALTER TABLE project ADD COLUMN org_id INTEGER REFERENCES organization(id) ON DELETE SET NULL;
CREATE INDEX idx_project_org_id ON project(org_id);
```

---

#### 🟠 Приоритет 3: Audit Log Расширенный

**Цель:** Полное логирование всех действий для compliance (SOC2, ISO27001)

**Что реализовать:**
- Логирование каждого запроса к API (кто, что, когда, IP)
- Поиск и фильтрация по событиям
- Экспорт логов в SIEM-системы (Splunk, ELK)
- Политики хранения (retention policies)

**API:**
```
GET /api/audit/events?user_id=123&project_id=456&type=login&from=2026-01-01
GET /api/audit/export?format=csv&from=...&to=...
```

---

#### 🟠 Приоритет 4: Rate Limiting & Throttling

**Цель:** Защита от злоупотреблений и DDoS

**Что реализовать:**
- Rate limiting на уровне API endpoints
- Квоты на количество задач в час/день
- Блокировка при превышении лимитов
- Настройка лимитов на пользователя/проект

**Реализация:**
```rust
// Redis-based rate limiter
pub struct RateLimiter {
    redis: RedisClient,
    limits: HashMap<String, RateLimit>,
}

pub struct RateLimit {
    pub max_requests: u32,
    pub window_seconds: u32,
}
```

---

### БЛОК 5 — Developer Experience (v4.0)

#### 🟡 Приоритет 5: VS Code Extension

**Цель:** Интеграция Velum в IDE разработчиков

**Функции:**
- Запуск playbook прямо из VS Code
- Просмотр логов задач в Output panel
- IntelliSense для extra_vars
- Сниппеты для Ansible/Terraform

**Технологии:**
- TypeScript + VS Code Extension API
- Velum CLI как зависимость
- WebSocket для live-логов

---

#### 🟡 Приоритет 6: Terraform Provider

**Цель:** Управление Velum через Terraform

**Ресурсы:**
```hcl
resource "velum_project" "main" {
  name = "Infrastructure"
  max_parallel_tasks = 5
}

resource "velum_template" "deploy" {
  name       = "Deploy App"
  project_id = velum_project.main.id
  playbook   = "deploy.yml"
}

resource "velum_schedule" "backup" {
  name       = "Daily Backup"
  template_id = velum_template.deploy.id
  cron       = "0 2 * * *"
}
```

**Реализация:** Go + terraform-plugin-sdk

---

#### 🟡 Приоритет 7: GraphQL API (расширение)

**Цель:** Полный GraphQL API для сложных интеграций

**Что добавить:**
- Мутации для всех CRUD операций
- Подписка на события через WebSocket
- Пагинация и фильтрация
- Интроспекция и автодокументирование

---

### БЛОК 6 — Monitoring & Observability (v4.0)

#### 🟢 Приоритет 8: Prometheus Metrics

**Цель:** Нативная интеграция с Prometheus

**Метрики:**
```
velum_tasks_total{project,template,status}
velum_tasks_duration_seconds{project,template}
velum_runners_connected{runner}
velum_database_connections{state}
velum_http_requests_total{endpoint,method,status_code}
```

**Endpoint:** `GET /metrics` (Prometheus-compatible)

---

#### 🟢 Приоритет 9: Distributed Tracing (OpenTelemetry)

**Цель:** Трассировка запросов через все сервисы

**Интеграции:**
- Jaeger
- Zipkin
- Tempo

**Реализация:**
```rust
use opentelemetry::global;

pub fn init_tracing() -> Result<()> {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("velum")
        .install_simple()?;
    Ok(())
}
```

---

## 📊 Дорожная карта

| Квартал | Версия | Фокус | Ключевые фичи | Статус |
|---------|--------|-------|---------------|--------|
| Q1 2026 | v3.2 | ✅ Завершено | MCP встроенный, AI Analysis, 60 инструментов | ✅ Готово |
| Q2 2026 | v4.0 | ✅ HA Cluster | Redis session store, Health checks, Graceful shutdown | ✅ Готово |
| Q2 2026 | v4.0 | ✅ Multi-Tenancy | Организации, квоты, API + UI | ✅ Готово |
| Q3 2026 | v4.1 | 📅 План | Audit Log, Rate Limiting | ⏳ Ожидает |
| Q3 2026 | v4.1 | 📅 План | VS Code Extension, Terraform Provider | ⏳ Ожидает |
| Q4 2026 | v4.2 | 📅 План | Prometheus Metrics, OpenTelemetry | ⏳ Ожидает |

---

## 🏆 Достижения v4.0

### High Availability Cluster — ✅ РЕАЛИЗОВАНО

| Фича | Реализация | Статус |
|------|------------|--------|
| **Redis Session Store** | `AppState.cache`, `RedisCache.initialize_sync()` | ✅ Готово |
| **Health Check Endpoints** | `/api/health/live`, `/api/health/ready`, `/api/health/full` | ✅ Готово |
| **Graceful Shutdown** | Обработка SIGTERM/SIGINT, остановка scheduler | ✅ Готово |
| **HA Configuration** | `SEMAPHORE_HA_*` переменные, Node ID | ✅ Готово |
| **Kubernetes Probes** | liveness/readiness probes конфигурация | ✅ Готово |

### Multi-Tenancy (Организации) — 🔄 БАЗА РЕАЛИЗОВАНА

| Фича | Реализация | Статус |
|------|------------|--------|
| **Модели данных** | `Organization`, `OrganizationUser`, `OrganizationCreate/Update` | ✅ Готово |
| **Миграция БД** | Таблицы `organization`, `organization_user`, `project.org_id` | ✅ Готово |
| **OrganizationManager** | 11 методов (CRUD, квоты, пользователи) | ✅ Готово |
| **SQL реализация** | Полный CRUD + проверка квот | ✅ Готово |
| **StoreWrapper** | Реализация `OrganizationManager` | ✅ Готово |
| **API Endpoints** | `/api/organizations/**` (11 endpoints) | ✅ Готово |
| **UI Страницы** | `organizations.html` — CRUD + управление пользователями | ✅ Готово |
| **White-labeling** | Кастомизация UI | ⏳ Ожидает |

---

## 🏆 Достижения v3.2

### Реализовано ✅ (100% плана)

| Категория | Метрика | Статус |
|-----------|---------|--------|
| **Бэкенд** | 75+ API endpoints, 667 тестов | ✅ 0 Clippy warnings |
| **Фронтенд** | 40 HTML страниц | ✅ Полный feature parity |
| **Auth** | JWT, bcrypt, TOTP 2FA, LDAP, OIDC | ✅ Refresh tokens |
| **Task Runner** | ansible/terraform/bash | ✅ WebSocket логи |
| **Scheduler** | cron-расписания | ✅ Автозапуск |
| **Distributed Runners** | Регистрация, health check, теги | ✅ |
| **Analytics** | Chart.js дашборд | ✅ Тренды |
| **Secret Storage** | Vault, DVLS, Fortanix | ✅ |
| **Webhooks** | Матчеры, extract values | ✅ Алиасы |
| **Design** | Material Design, teal #005057 | ✅ Font Awesome 6.5 |
| **Deploy** | Docker, DEB, binary | ✅ SQLite/PostgreSQL |
| **MCP Server** | 60 инструментов | ✅ Встроен в Velum |
| **CLI** | 10 команд | ✅ Rust binary |
| **Workflow DAG** | Графы шаблонов | ✅ Drag-and-drop UI |
| **Survey Forms** | Интерактивные формы | ✅ Конструктор |
| **LDAP Groups** | → Teams auto-sync | ✅ |
| **Notifications** | Slack/Telegram/PagerDuty | ✅ Политики |
| **Credentials** | Custom types | ✅ Injectors |
| **AI Analysis** | Анализ ошибок | ✅ Claude/OpenAI |
| **Drift Detection** | Terraform diff | ✅ GitOps |
| **Rollback** | Snapshots | ✅ В один клик |
| **Marketplace** | Community templates | ✅ 11 шаблонов |
| **Cost Tracking** | Infracost | ✅ API готово |
| **Diff Runs** | LCS diff engine | ✅ Unified/split view |

### Технические метрики

```
📦 Размер бинарника:     ~15 MB (release, stripped)
⚡ Время запуска:        <1 сек (SQLite), <5 сек (PostgreSQL)
💾 Использование памяти: ~80 MB (idle), ~150 MB (под нагрузкой)
🧪 Тестов:              667
⚠️  Clippy warnings:     0
📄 Строк кода (Rust):    ~50,000
📄 Строк кода (JS):      ~15,000
```

---

## Ссылки

| Репозиторий | URL |
|---|---|
| Velum (origin) | https://github.com/tnl-o/velum |
| Velum (main) | https://github.com/alexandervashurin/velum |
| Go-оригинал (эталон) | https://github.com/velum/velum |
| Semaphore MCP (референс, Python) | https://github.com/cloin/semaphore-mcp |

---

## 📝 История изменений плана

| Версия | Дата | Изменения |
|--------|------|-----------|
| 4.0 | 2026-03-23 | ✅ HA Cluster, 🔄 Multi-Tenancy (База) |
| 3.3 | 2026-03-23 | ✅ v3.2 Feature Complete, добавлен план v4.0 |
| 3.2 | 2026-03-21 | MCP встроенный, AI Analysis |
| 3.0 | 2026-03-15 | Rollback, Marketplace, Cost Tracking, Diff |
| 2.7 | 2026-03-10 | CLI Tool `velum` |
| 2.5 | 2026-03-05 | Notification Policies |
| 2.4 | 2026-03-01 | LDAP Group Sync, Custom Credentials |
| 2.3 | 2026-02-25 | AI Analysis, Drift Detection |
| 2.2 | 2026-02-20 | Workflow DAG, Survey Forms |
| 2.1 | 2026-02-15 | Базовая платформа |
