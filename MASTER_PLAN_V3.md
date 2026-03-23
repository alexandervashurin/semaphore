# MASTER_PLAN V3 — Velum: Стать лучше AWX и Ansible Tower

> **Последнее обновление:** 2026-03-23 (сессия 12 — добавлен БЛОК 12: Future Ideas v5+)
> **Версия:** 5.0-plan
> **Статус:** ✅ v4.2 COMPLETE | 🗺️ v5.0 PLAN READY | 💡 v5+ FUTURE IDEAS ADDED

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

---

## 🎯 ПЛАН РАЗРАБОТКИ v5.0 — Production-Ready & Developer-First

> Velum v4.x feature complete. v5.0 — hardening, ecosystem, developer experience.

---

### БЛОК 7 — Security Hardening (v5.0) 🔴

#### Приоритет 1: JWT Logout Blacklist

**Проблема:** Токены остаются валидными после logout до истечения TTL. Украденный токен работает.

**Что реализовать:**
- Redis-backed blacklist: при logout → `SET revoked:{jti} 1 EX {ttl_remaining}`
- При каждом запросе: проверить `EXISTS revoked:{jti}` → 401 если есть
- Fallback без Redis: in-memory `Arc<DashMap<String, Instant>>` с фоновой очисткой

**Backend:** `rust/src/api/extractors.rs` — добавить проверку в `AuthUser` extractor
**Конфиг:** `SEMAPHORE_JWT_BLACKLIST_BACKEND=redis|memory` (default: memory)

---

#### Приоритет 2: Webhook HMAC Подпись

**Проблема:** Исходящие webhook-уведомления не подписаны — получатель не может верифицировать источник.

**Что реализовать:**
```
X-Velum-Signature: sha256=<HMAC-SHA256(secret, body)>
X-Velum-Timestamp: <unix_timestamp>
```
- Секрет задаётся при создании webhook (хранится зашифрованным)
- Пример верификации на стороне получателя (Python/Node) в документации
- UI: поле "Webhook Secret" в форме создания webhook

---

#### Приоритет 3: Cargo Audit в CI

**Файл:** `.github/workflows/rust.yml`

```yaml
- name: Security audit
  run: cargo audit --deny warnings

- name: Dependency licenses
  run: cargo deny check
```

**Зависимости для обновления (критично):**
- `quinn-proto 0.11.13` → 0.11.14+ (CVSS 8.7, memory safety)
- `rsa 0.9.x` → мигрировать на ECDSA там где возможно

---

#### Приоритет 4: Dependabot

**Файл:** `.github/dependabot.yml`

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/rust"
    schedule:
      interval: weekly
    open-pull-requests-limit: 10
    ignore:
      - dependency-name: wasmtime   # Major updates требуют ручного review
        update-types: ["version-update:semver-major"]
```

---

### БЛОК 8 — Незавершённые Фичи (v5.1) 🟠

#### Приоритет 1: Telegram Bot — Уведомления

**Файл:** `rust/src/services/telegram_bot/mod.rs` — 4 заглушки
**Статус:** Зависимость `teloxide 0.13` уже в `Cargo.toml`

**Что реализовать:**
- Алерт при падении задачи: `❌ [prod] Deploy Backend — FAILED (2m 34s)`
- Уведомление об успехе: `✅ [prod] Deploy Backend — OK`
- Команды: `/status` (running tasks), `/run <template>`, `/stop <task_id>`, `/approve <plan_id>`
- Inline кнопки для approve/reject Terraform plans прямо в Telegram

**Конфиг:** `SEMAPHORE_TELEGRAM_TOKEN=...`, `SEMAPHORE_TELEGRAM_CHAT_ID=...`

---

#### Приоритет 2: SSH Key Installation

**Файл:** `rust/src/services/access_key_installation_service.rs` — 5 заглушек

**Что реализовать:**
- Запись SSH-ключа во временный файл с `chmod 600`
- Интеграция с `ssh-agent` через Unix socket (только Linux/Mac)
- Passphrase поддержка через `ssh2` crate
- Очистка после завершения задачи (cleanup hook)
- Поддержка ECDSA, Ed25519, RSA ключей

---

#### Приоритет 3: Remote Runners — Distributed Execution

**Цель:** Запуск задач на удалённых машинах без прямого доступа к БД

**Архитектура:**
```
Velum Server ──REST──> Runner Agent ──exec──> ansible-playbook
     ^                      │
     └──── WebSocket ───────┘  (live logs streaming)
```

**Что реализовать:**
- Runner регистрация: `POST /api/internal/runners` с токеном
- Heartbeat: `POST /api/internal/runners/{id}` каждые 30 сек
- Task assignment: `GET /api/internal/runners/{id}/task` (long polling)
- Log streaming: WebSocket туннель от runner к серверу
- Runner binary: отдельный `velum-runner` бинарник

**БД:** Таблица `runner_task_assignment(runner_id, task_id, assigned_at)`

---

#### Приоритет 4: Kubernetes-Native Runner

**Статус:** `k8s-openapi 0.24`, `kube 0.98` уже в `Cargo.toml`

**Что реализовать:**
- Каждая задача = отдельный Kubernetes Job
- Логи через `kubectl logs -f` → WebSocket к UI
- Namespace изоляция по проекту: `velum-project-{id}`
- ConfigMap для передачи переменных задачи
- Auto-cleanup завершённых Jobs

**Конфиг:**
```bash
SEMAPHORE_K8S_ENABLE=true
SEMAPHORE_K8S_NAMESPACE=velum-runners
SEMAPHORE_K8S_IMAGE=velum-runner:latest
```

---

#### Приоритет 5: GraphQL Mutations & Subscriptions

**Статус:** `async-graphql 7.0` уже в `Cargo.toml`

**Что добавить:**
```graphql
# Subscriptions (WebSocket)
subscription {
  taskOutput(taskId: 123) { line, timestamp, level }
  taskStatus(projectId: 1) { taskId, status, updatedAt }
}

# Mutations
mutation {
  runTemplate(projectId: 1, templateId: 5, extraVars: "{}") { taskId }
  stopTask(projectId: 1, taskId: 123) { success }
  approveplan(projectId: 1, planId: 7, comment: "LGTM") { success }
}
```

---

### БЛОК 9 — Developer Experience (v5.2) 🟡

#### Приоритет 1: Dark Mode

**Статус:** CSS переменные частично готовы

**Что реализовать:**
- CSS: полный набор `--dark-*` переменных для всех компонентов
- JS: `localStorage.setItem('theme', 'dark')` + `data-theme="dark"` на `<html>`
- Toggle: иконка луны/солнца в header
- OS preference: `prefers-color-scheme: dark` автодетект

---

#### Приоритет 2: Progressive Web App (PWA)

**Что реализовать:**
- `manifest.json` с иконками (192x192, 512x512)
- Service Worker: кэш статики + API responses для offline read
- Web Push API: уведомления о завершении задач без открытого браузера
- Install prompt для Chrome/Edge

---

#### Приоритет 3: Интернационализация (i18n)

**Что реализовать:**
- JSON файлы переводов: `web/public/locales/ru.json`, `en.json`, `de.json`, `zh.json`
- `t('key')` функция в `app.js` с fallback на `ru`
- Language switcher в header + сохранение в `localStorage`
- Поддержка: 🇷🇺 RU, 🇺🇸 EN, 🇩🇪 DE, 🇨🇳 ZH, 🇪🇸 ES

---

#### Приоритет 4: Keyboard Shortcuts

```
/ — глобальный поиск
g p — перейти к проектам
g t — перейти к шаблонам
j / k — навигация по списку
Enter — открыть/запустить выбранный элемент
? — показать справку по shortcuts
```

---

#### Приоритет 5: VS Code Extension

**Репозиторий:** `velum-vscode/` (отдельный)
**Технологии:** TypeScript + VS Code Extension API

**Фичи:**
- Tree view: проекты → шаблоны → последние задачи
- Run template из палитры команд (`Ctrl+Shift+P → Velum: Run`)
- Live logs в Output panel
- IntelliSense для `extra_vars` (подсказки из survey)
- Статус в status bar: 🟢 3 running / 🔴 1 failed

---

#### Приоритет 6: Terraform Provider

**Репозиторий:** `velum-terraform-provider/` (отдельный, Go)

```hcl
terraform {
  required_providers {
    velum = {
      source  = "tnl-o/velum"
      version = "~> 1.0"
    }
  }
}

resource "velum_project"   "infra"   { name = "Infrastructure" }
resource "velum_template"  "deploy"  { project_id = velum_project.infra.id; playbook = "deploy.yml" }
resource "velum_schedule"  "nightly" { template_id = velum_template.deploy.id; cron = "0 2 * * *" }
resource "velum_inventory" "prod"    { project_id = velum_project.infra.id; content = file("hosts.ini") }
```

---

### БЛОК 10 — Infrastructure & Observability (v5.3) 🟢

#### Приоритет 1: Helm Chart

**Путь:** `charts/velum/`

```
charts/velum/
├── Chart.yaml
├── values.yaml          # defaults: replicas=1, db=sqlite
├── values.prod.yaml     # PostgreSQL, Redis, 3 replicas
└── templates/
    ├── deployment.yaml
    ├── service.yaml
    ├── ingress.yaml
    ├── configmap.yaml
    ├── secret.yaml
    └── hpa.yaml         # HorizontalPodAutoscaler
```

**Поддерживаемые конфигурации:**
- Single node (SQLite) — для dev/small teams
- HA (PostgreSQL + Redis + 3 replicas) — для enterprise

---

#### Приоритет 2: Multi-Platform Builds

**CI Matrix:**

| Platform | Target | Artifact |
|---|---|---|
| Linux x64 | `x86_64-unknown-linux-musl` | `velum-linux-amd64` |
| Linux ARM64 | `aarch64-unknown-linux-musl` | `velum-linux-arm64` |
| macOS x64 | `x86_64-apple-darwin` | `velum-macos-amd64` |
| macOS ARM64 | `aarch64-apple-darwin` | `velum-macos-arm64` |
| Windows x64 | `x86_64-pc-windows-msvc` | `velum-windows-amd64.exe` |
| Docker | multi-arch | `ghcr.io/tnl-o/velum:latest` |

**Docker образ:** оптимизировать с ~1.5GB до <50MB через `FROM scratch`

---

#### Приоритет 3: Grafana Dashboards

**Дашборды (JSON в `deployment/grafana/`):**

1. **Task Execution Overview** — success rate, duration heatmap, failures by project
2. **System Health** — CPU, memory, DB latency, connection pool, HTTP p99
3. **Queue Depth** — задачи в очереди по проектам, runner utilization
4. **Security** — failed logins, rate limit hits, audit events

**Prometheus метрики (добавить):**
```
velum_tasks_total{project, template, status}
velum_task_duration_seconds{project, template}
velum_queue_depth{project}
velum_runner_tasks{runner, status}
velum_http_requests_total{endpoint, method, status_code}
velum_rate_limit_hits_total{ip, endpoint}
velum_auth_failures_total{ip, reason}
```

---

#### Приоритет 4: OpenAPI / Swagger

**Что реализовать:**
- Crate: `utoipa` или `aide` для auto-generation из handler сигнатур
- Endpoints: `GET /api/openapi.json`, `GET /api/swagger`
- Аннотации: `#[utoipa::path(...)]` на все публичные handlers
- Польза: автогенерация SDK клиентов (Python, JS, Go)

---

### БЛОК 11 — Testing (v5.0 сквозной приоритет) 🧪

#### Цель: Покрытие 80%+ (сейчас ~67%)

| Область | Текущее | Цель | Файлы |
|---|---|---|---|
| Task Runner | ~40% | 80% | `services/task_runner/` |
| DB Managers | ~55% | 85% | `db/sql/managers/` |
| API Handlers | ~70% | 90% | `api/handlers/` |
| Auth/Security | ~60% | 95% | `api/extractors.rs`, `auth.rs` |

**Что добавить:**

1. **Integration tests** — `tests/api_integration.rs` с axum TestServer
2. **E2E Docker tests** — `docker-compose.test.yml` с PostgreSQL + тестовые сценарии
3. **Mutation testing** — `cargo mutants` на критические пути
4. **Security tests** — SQLi/XSS/CSRF suite в `tests/security/`
5. **Benchmark suite** — `benches/api_throughput.rs` с Criterion

---

### БЛОК 12 — Quick Wins (можно сделать за <1 дня) ⚡

| # | Задача | Файл | Impact |
|---|---|---|---|
| QW-1 | Валидация cron перед сохранением | `scheduler_pool.rs:57,68` | Prevent crashes |
| QW-2 | HMAC подпись в webhook уведомлениях | `alert.rs` | Security |
| QW-3 | `cargo audit` в CI | `.github/workflows/rust.yml` | Security |
| QW-4 | `.github/dependabot.yml` | — | Automation |
| QW-5 | Dark mode toggle (localStorage) | `styles.css`, `app.js` | UX |
| QW-6 | Virtual scroll для task history | `history.html`, `task.html` | Performance |
| QW-7 | Linux ARM64 в CI release matrix | GitHub Actions | Distribution |
| QW-8 | `manifest.json` + SW для PWA | `web/public/` | UX |
| QW-9 | `GET /api/openapi.json` заглушка | `routes.rs` | DX |
| QW-10 | Keyboard shortcut `?` — help modal | `app.js` | UX |

---

### Сводная таблица v5 задач

| ID | Блок | Задача | Приоритет | Версия | Статус |
|---|---|---|---|---|---|
| S-01 | Security | JWT logout blacklist | 🔴 | v5.0 | ⏳ |
| S-02 | Security | Webhook HMAC signature | 🔴 | v5.0 | ⏳ |
| S-03 | Security | cargo audit в CI | 🔴 | v5.0 | ⏳ |
| S-04 | Security | Dependabot | 🟠 | v5.0 | ⏳ |
| S-05 | Security | Уязвимые зависимости (quinn, rsa) | 🔴 | v5.0 | ⏳ |
| F-01 | Features | Telegram Bot уведомления | 🟠 | v5.1 | ⏳ |
| F-02 | Features | SSH Key Installation | 🟠 | v5.1 | ⏳ |
| F-03 | Features | Remote Runners | 🟠 | v5.1 | ⏳ |
| F-04 | Features | Kubernetes Runner | 🟡 | v5.1 | ⏳ |
| F-05 | Features | GraphQL mutations + subscriptions | 🟡 | v5.1 | ⏳ |
| DX-01 | Dev UX | Dark Mode | 🟡 | v5.2 | ⏳ |
| DX-02 | Dev UX | PWA + Service Worker | 🟡 | v5.2 | ⏳ |
| DX-03 | Dev UX | i18n (RU/EN/DE/ZH/ES) | 🟡 | v5.2 | ⏳ |
| DX-04 | Dev UX | Keyboard Shortcuts | 🟡 | v5.2 | ⏳ |
| DX-05 | Dev UX | VS Code Extension | 🟡 | v5.2 | ⏳ |
| DX-06 | Dev UX | Terraform Provider (Go) | 🟡 | v5.2 | ⏳ |
| I-01 | Infra | Helm Chart | 🟢 | v5.3 | ⏳ |
| I-02 | Infra | Multi-platform builds | 🟢 | v5.3 | ⏳ |
| I-03 | Infra | Grafana Dashboards | 🟢 | v5.3 | ⏳ |
| I-04 | Infra | OpenAPI/Swagger | 🟢 | v5.3 | ⏳ |
| T-01 | Testing | Integration tests (axum TestServer) | 🔴 | v5.0 | ⏳ |
| T-02 | Testing | E2E Docker tests | 🟠 | v5.0 | ⏳ |
| T-03 | Testing | Mutation testing (cargo mutants) | 🟡 | v5.1 | ⏳ |
| T-04 | Testing | Security test suite | 🔴 | v5.0 | ⏳ |
| T-05 | Testing | Benchmark suite (Criterion) | 🟢 | v5.3 | ⏳ |

---

## 📊 Дорожная карта

| Квартал | Версия | Фокус | Ключевые фичи | Статус |
|---------|--------|-------|---------------|--------|
| Q1 2026 | v3.2 | ✅ Завершено | MCP встроенный, AI Analysis, 60 инструментов | ✅ Готово |
| Q2 2026 | v4.0 | ✅ HA Cluster | Redis session store, Health checks, Graceful shutdown | ✅ Готово |
| Q2 2026 | v4.0 | ✅ Multi-Tenancy | Организации, квоты, API + UI | ✅ Готово |
| Q3 2026 | v4.1 | ✅ Готово | Audit Log Export (CSV/NDJSON), Rate Limiting (5/100 req/min) | ✅ Готово |
| Q4 2026 | v4.2 | ✅ Готово | Prometheus Metrics, Trace ID middleware (X-Trace-ID), White-labeling | ✅ Готово |
| Q1 2027 | v5.0 | 🗺️ План | Security hardening, Remote Runners, Testing 80%+ | ⏳ Ожидает |
| Q2 2027 | v5.1 | 🗺️ План | Telegram Bot, SSH keys, Kubernetes Runner, GraphQL mutations | ⏳ Ожидает |
| Q3 2027 | v5.2 | 🗺️ План | Dark Mode, PWA, i18n, VS Code Extension | ⏳ Ожидает |
| Q4 2027 | v5.3 | 🗺️ План | Helm chart, Multi-platform builds, Grafana dashboards | ⏳ Ожидает |

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
| **White-labeling** | Branding API (logo, color, app_name, CSS) + UI | ✅ Готово |

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

## 💡 БЛОК 12 — Future Ideas v5+ (Идеи для дальнейшего развития)

> Этот блок — **банк идей**, собранных в сессии 12 (2026-03-23).
> Это не план с обязательствами, а список направлений для обсуждения и приоритизации.
> Идеи **не дублируют** уже запланированные задачи в БЛОКАХ 7–11.

---

### 🌐 FI-1 — GitHub / GitLab / Gitea нативная интеграция

| Идея | Описание | Сложность |
|------|----------|-----------|
| **PR Status Checks** | Публиковать результат задачи как GitHub Check Run — блокировать merge до успеха deploy | 🟡 Средняя |
| **PR Auto-комментарии** | Вставлять Terraform plan diff прямо в комментарий Pull Request | 🟡 Средняя |
| **GitHub Actions ↔ Velum** | Двусторонняя триггеризация: Action → Velum template → результат обратно | 🟠 Высокая |
| **GitLab CI пайплайны** | Артефакты CI как Inventory; автопередача `CI_COMMIT_SHA` в extra_vars | 🟡 Средняя |

**Файлы:** `rust/src/services/github_status.rs`, `rust/src/services/gitlab_ci.rs`

---

### ☁️ FI-2 — Облачные исполнители задач (Serverless Execution)

| Идея | Описание | Сложность |
|------|----------|-----------|
| **AWS Lambda Executor** | `executor: aws_lambda` — каждая задача = Lambda function, sub-second billing | 🔴 Высокая |
| **Azure Container Instances** | `executor: azure_aci` — без управления инфраструктурой, Managed Identity | 🔴 Высокая |
| **Google Cloud Run** | `executor: gcloud_run` — автомасштабирование 0→N, Knative-совместимость | 🔴 Высокая |
| **Умный выбор исполнителя** | Авто-роутинг: задача <30 сек → Lambda, большая → Kubernetes | 🟠 Средняя |

**Файлы:** `rust/src/services/lambda_executor.rs`, `rust/src/services/executor_selector.rs`

---

### 🔑 FI-3 — Продвинутое управление секретами

| Идея | Описание | Сложность |
|------|----------|-----------|
| **AWS Secrets Manager** | Нативный клиент (aws-sdk-secretsmanager), авторотация, TTL-кеш | 🟠 Средняя |
| **GCP Secret Manager** | Service Account JSON / Workload Identity (`google-secretmanager1` crate) | 🟡 Средняя |
| **Azure Key Vault** | Managed Identity без хардкода — `azure_security_keyvault` crate | 🟡 Средняя |
| **HSM / PKCS#11** | Подпись SSH ключей через YubiKey / Thales Luna HSM (`pkcs11` crate) | 🔴 Высокая |

**Файлы:** `rust/src/services/aws_secrets_manager.rs`, `rust/src/services/pkcs11_provider.rs`

---

### 📋 FI-4 — Compliance & Аудит (SOC2 / ISO27001 / GDPR)

| Идея | Описание | Сложность |
|------|----------|-----------|
| **Иммутабельный аудит-лог** | Hash chain (каждая запись подписывает предыдущую), WORM-хранение, export в SIEM | 🟠 Средняя |
| **PII маскировка в логах** | Авто-редактирование email/IP/ключей regex-паттернами (GDPR/CCPA) | 🟡 Низкая |
| **SBOM генерация** | Опись зависимостей Ansible Galaxy / Terraform modules перед apply | 🟡 Средняя |
| **Field-level шифрование + ротация ключей** | FIPS 140-2, `age` или `tink` crates, auto-rotation N дней | 🔴 Высокая |
| **Data Export для GDPR** | `GET /api/user/{id}/data-export` → полный JSON всех данных пользователя | 🟡 Низкая |

**Файлы:** `rust/src/services/immutable_log.rs`, `rust/src/services/pii_scrubber.rs`, `rust/src/services/sbom_generator.rs`

---

### 🗓️ FI-5 — Умное планирование задач

| Идея | Описание | Сложность |
|------|----------|-----------|
| **Рабочие часы и праздники** | "Запускать только Пн–Пт 9–18, не в праздники US/EU/RU/JP" | 🟡 Средняя |
| **Blackout windows** | Запрет деплоев в указанные временные периоды (настраивается per-project) | 🟡 Низкая |
| **Cost-Optimized Scheduling** | Откладывать задачи до дешёвых AWS Spot-часов | 🟠 Высокая |
| **Resource-aware scheduler** | Теги `gpu=required`, `memory=16gb` — выбор раннера по capabilities | 🟡 Средняя |
| **Иерархический Resource Scheduler** | GPU/FPGA очереди, приоритизация по доступности ресурсов | 🔴 Высокая |

**Файлы:** `rust/src/services/advanced_scheduler.rs`, `rust/src/services/resource_scheduler.rs`

---

### 📊 FI-6 — Продвинутая аналитика и ML

| Идея | Описание | Сложность |
|------|----------|-----------|
| **Anomaly Detection** | Статистическое обнаружение: задача вдруг стала на 47% медленнее (`linfa` crate) | 🟠 Высокая |
| **Failure Prediction** | "73% вероятность падения (исторический паттерн)" — alert до запуска | 🔴 Высокая |
| **Cost Attribution / Chargeback** | Внутренний биллинг: стоимость облака по проектам/командам/департаментам | 🟡 Средняя |
| **Auto-scale рекомендации** | "У вас 40% раннеров простаивает" — ML recommendation engine | 🟠 Высокая |
| **Task Performance Baseline** | Авто-установка SLA из исторических p50/p95/p99 данных | 🟡 Низкая |

**Файлы:** `rust/src/services/anomaly_detection.rs`, `rust/src/services/failure_predictor.rs`, `rust/src/services/cost_attribution.rs`

---

### 🛡️ FI-7 — Безопасность & Zero-Trust

| Идея | Описание | Сложность |
|------|----------|-----------|
| **Secrets Scanning в логах** | Авто-детект утечек AWS ключей/GitHub токенов в output задач + авто-ротация | 🟡 Средняя |
| **Runtime Security** | Интеграция Crowdstrike/Wiz/Snyk: блокировать задачи при детекте аномалии | 🔴 Высокая |
| **Supply Chain SBOM Check** | CVE-проверка Ansible roles/Terraform modules перед apply | 🟡 Средняя |
| **Device Trust / MDM** | Блокировать логин с неуправляемых устройств (Okta/Intune Device Compliance) | 🔴 Высокая |
| **Continuous CWPP** | Detect malicious task execution в реальном времени (regex на stderr pattern) | 🟠 Средняя |

**Файлы:** `rust/src/services/secrets_scanner.rs`, `rust/src/services/runtime_security.rs`

---

### 📜 FI-8 — Policy-as-Code (OPA / Rego)

| Идея | Описание | Сложность |
|------|----------|-----------|
| **OPA / Rego Policy Engine** | `regorus` crate (Rust OPA); политики: "только бизнес-часы", "approval если >100 ресурсов" | 🟠 Средняя |
| **Cost Policy Enforcement** | Блокировать задачу если estimated_cost > лимит проекта | 🟡 Средняя |
| **Security Policy** | "Terraform apply только на раннерах в private VPC", "no plaintext secrets" | 🟡 Средняя |
| **Compliance Policies** | PCI-DSS, HIPAA, ISO27001 predefined policy bundles | 🔴 Высокая |

**Файлы:** `rust/src/services/opa_engine.rs`, `rust/src/services/policy_enforcer.rs`

---

### 💬 FI-9 — Мессенджеры & Управление инцидентами

| Идея | Описание | Сложность |
|------|----------|-----------|
| **Slack App (официальный Bot)** | Интерактивные кнопки approve/reject прямо в Slack, `/velum run deploy` slash-команды | 🟠 Средняя |
| **Microsoft Teams App** | Adaptive Cards с результатами задач, bot commands + message extensions | 🟠 Средняя |
| **PagerDuty интеграция** | Авто-создание инцидента при падении задачи, авто-резолв при успехе ретрая | 🟡 Низкая |
| **Jira + Confluence** | Создавать тикеты при ошибках, линковать runbooks к шаблонам | 🟡 Средняя |
| **OpsGenie / AlertManager** | Multi-channel алерты с дедупликацией (1 алерт/час) и политиками эскалации | 🟡 Средняя |

**Файлы:** `rust/src/services/slack_app.rs`, `rust/src/services/pagerduty.rs`, `rust/src/services/atlassian.rs`

---

### 🔀 FI-10 — Расширенные Workflow

| Идея | Описание | Сложность |
|------|----------|-----------|
| **Conditional DAG Branches** | Ветки по результату задачи: `if output['status'] == 'critical' → rollback` | 🟠 Средняя |
| **Fan-Out / Fan-In** | Параллельный деплой на 100 серверов → агрегированный отчёт | 🟠 Средняя |
| **Multi-level Approval + Escalation** | Таймаут 2 часа → эскалация выше, SLA-метрики по approval time | 🟡 Средняя |
| **Loop nodes в DAG** | `for each item in inventory_hosts` — итерация через ноды графа | 🔴 Высокая |
| **DAG Snapshot & Replay** | Воспроизвести прошлый run с теми же параметрами | 🟡 Средняя |

---

### 🏪 FI-11 — SaaS, Monetization & Multi-Tenancy Extended

| Идея | Описание | Сложность |
|------|----------|-----------|
| **Usage-Based Billing** | Stripe Metering API, тарификация по количеству запусков задач | 🔴 Высокая |
| **Template Marketplace с рейтингом** | Community contrib + premium templates от вендоров (Hashicorp, CloudBees) | 🟠 Высокая |
| **Tenant Network Isolation** | Отдельный Redis cache per org, row-level security в БД | 🔴 Высокая |
| **SaaS Trial Mode** | 14-дневный trial с авто-downgrade; onboarding wizard | 🟡 Средняя |

**Файлы:** `rust/src/services/billing.rs`, `rust/src/services/tenant_isolation.rs`

---

### 🔄 FI-12 — Template & GitOps Advanced

| Идея | Описание | Сложность |
|------|----------|-----------|
| **Template Git Sync** | Источник правды в Git: pull repo → parse playbooks → создать шаблоны авто | 🟡 Средняя |
| **Template Versioning** | Полный diff между версиями шаблона, rollback к любой версии | 🟡 Средняя |
| **Export as Helm Chart** | Velum template → Helm chart package одной кнопкой | 🟠 Средняя |
| **IntelliSense для Ansible/Terraform** | Авто-дополнение переменных из playbook в survey form | 🟠 Высокая |

---

### 🌍 FI-13 — Федеративное развёртывание (Multi-Region)

| Идея | Описание | Сложность |
|------|----------|-----------|
| **Multi-Region Routing** | Авто-маршрут задачи на ближайший раннер (us-east / eu-west / ap-south) | 🔴 Высокая |
| **Data Residency Enforcement** | Данные не покидают регион (GDPR), шифрование per-region | 🔴 Высокая |
| **Federated Control Plane** | Центральный control plane + edge execution nodes | 🔴 Очень высокая |

---

### ⚡ FI-14 — Быстрые улучшения (< 1 дня каждое)

| ID | Идея | Приоритет |
|----|------|-----------|
| FI-14-1 | **GitHub Commit Status** — публиковать статус задачи как commit status в PR | 🟡 |
| FI-14-2 | **Slack Webhook Test button** — кнопка "Проверить" прямо в settings/notifications UI | 🟢 |
| FI-14-3 | **Task Deduplication** — авто-слияние одинаковых задач в очереди, предупреждение | 🟡 |
| FI-14-4 | **Batch Approvals** — одобрить N Terraform планов одним кликом в UI | 🟡 |
| FI-14-5 | **Query Language для задач** — JMESPath фильтр: `?filter=status==failed&&duration>300` | 🟡 |
| FI-14-6 | **Estimated Cost в превью задачи** — показывать стоимость до запуска (Infracost) | 🟡 |
| FI-14-7 | **Keyboard shortcut `?`** — help modal со всеми шорткатами (как в GitHub) | 🟢 |
| FI-14-8 | **Task Dry-Run Mode** — показать что выполнится без реального запуска (для Ansible) | 🟡 |
| FI-14-9 | **CLI Plugin System** — `velum plugin install <repo>` для кастомных команд | 🟠 |
| FI-14-10 | **PLAY RECAP таблица** — парсить Ansible PLAY RECAP в красивую таблицу в task.html | 🟢 |

---

### 🎯 FI-15 — Конкурентное позиционирование (см. БЛОК 13 ниже)

> Полная матрица конкурентов вынесена в **БЛОК 13** для удобства навигации.

---

### 📊 Сводная таблица Future Ideas

| ID | Категория | Идея | Сложность | Версия |
|----|-----------|------|-----------|--------|
| FI-1-1 | Git | PR Status Checks (GitHub) | 🟡 Средняя | v6.0 |
| FI-1-2 | Git | PR Auto-комментарии с Terraform diff | 🟡 Средняя | v6.0 |
| FI-1-3 | Git | GitHub Actions ↔ Velum bidirectional | 🟠 Высокая | v6.1 |
| FI-2-1 | Cloud | AWS Lambda Executor | 🔴 Высокая | v6.1 |
| FI-2-2 | Cloud | Azure Container Instances Executor | 🔴 Высокая | v6.1 |
| FI-2-3 | Cloud | Умный выбор исполнителя | 🟠 Средняя | v6.1 |
| FI-3-1 | Secrets | AWS Secrets Manager нативный | 🟠 Средняя | v6.0 |
| FI-3-2 | Secrets | Azure Key Vault нативный | 🟡 Средняя | v6.0 |
| FI-3-3 | Secrets | HSM / PKCS#11 | 🔴 Высокая | v6.2 |
| FI-4-1 | Compliance | Иммутабельный аудит-лог (hash chain) | 🟠 Средняя | v6.0 |
| FI-4-2 | Compliance | PII маскировка в логах | 🟡 Низкая | v5.1 |
| FI-4-3 | Compliance | SBOM генерация | 🟡 Средняя | v6.0 |
| FI-4-5 | Compliance | GDPR Data Export endpoint | 🟡 Низкая | v5.1 |
| FI-5-1 | Scheduler | Рабочие часы + праздники | 🟡 Средняя | v5.1 |
| FI-5-2 | Scheduler | Blackout windows | 🟡 Низкая | v5.1 |
| FI-5-3 | Scheduler | Cost-Optimized Scheduling | 🟠 Высокая | v6.1 |
| FI-6-1 | ML/Analytics | Anomaly Detection | 🟠 Высокая | v6.1 |
| FI-6-2 | ML/Analytics | Failure Prediction | 🔴 Высокая | v6.2 |
| FI-6-3 | ML/Analytics | Cost Attribution / Chargeback | 🟡 Средняя | v6.0 |
| FI-6-5 | ML/Analytics | Task Performance Baseline (SLA) | 🟡 Низкая | v5.1 |
| FI-7-1 | Security | Secrets Scanning в логах | 🟡 Средняя | v5.0 |
| FI-7-3 | Security | Supply Chain SBOM Check | 🟡 Средняя | v6.0 |
| FI-8-1 | Policy | OPA / Rego Policy Engine | 🟠 Средняя | v6.0 |
| FI-8-2 | Policy | Cost Policy Enforcement | 🟡 Средняя | v6.0 |
| FI-9-1 | Messaging | Slack App (официальный Bot) | 🟠 Средняя | v5.1 |
| FI-9-2 | Messaging | Microsoft Teams App | 🟠 Средняя | v6.0 |
| FI-9-3 | Messaging | PagerDuty интеграция | 🟡 Низкая | v5.1 |
| FI-9-4 | Messaging | Jira + Confluence | 🟡 Средняя | v6.0 |
| FI-10-1 | Workflow | Conditional DAG Branches | 🟠 Средняя | v5.1 |
| FI-10-2 | Workflow | Fan-Out / Fan-In patterns | 🟠 Средняя | v5.1 |
| FI-10-3 | Workflow | Multi-level Approval + Escalation | 🟡 Средняя | v5.1 |
| FI-11-1 | SaaS | Usage-Based Billing (Stripe) | 🔴 Высокая | v6.2 |
| FI-11-2 | SaaS | Template Marketplace с рейтингом | 🟠 Высокая | v6.1 |
| FI-12-1 | GitOps | Template Git Sync | 🟡 Средняя | v5.1 |
| FI-12-2 | GitOps | Template Versioning | 🟡 Средняя | v5.1 |
| FI-12-3 | GitOps | Export as Helm Chart | 🟠 Средняя | v6.0 |
| FI-13-1 | Multi-Region | Multi-Region Routing | 🔴 Высокая | v6.2 |
| FI-14-1..10 | Quick Wins | 10 быстрых улучшений | 🟢 Низкая | v5.0–v5.1 |

**Итого: 40+ направлений** | 🔴 10 высоких | 🟠 14 средне-высоких | 🟡 16 средних/низких | 🟢 5 быстрых

---

## 🏟️ БЛОК 13 — Матрица конкурентов (2026)

> **Velum** — Rust, MIT, ~15 MB бинарник, <1 с старт, ~80 MB RAM, 1 бинарник + SQLite, native Ansible + Terraform, AI built-in, MCP сервер.

---

### 13.1 — Категории инструментов

| Категория | Инструменты |
|-----------|------------|
| **Ansible-ориентированные** | AWX/Tower, Ansible Semaphore, **Velum** |
| **Universal CI/CD** | Jenkins, GitLab CI/CD, GitHub Actions, Harness |
| **GitOps / K8s-native** | Argo CD, Flux CD, Spinnaker |
| **Terraform-focused** | Terraform Cloud (HCP), Spacelift, Atlantis, Pulumi Cloud |
| **Operations / RunOps** | Rundeck / PagerDuty Process Automation |
| **Developer Portal** | Backstage (Spotify), Port.io |

---

### 13.2 — Технические характеристики

| Инструмент | Язык | Лицензия | RAM (idle) | Старт | Деплой | Размер образа |
|------------|------|----------|-----------|-------|--------|---------------|
| **Velum** | **Rust** | **MIT** | **~80 MB** | **<1 с** | **1 бинарник** | **~50 MB** |
| AWX | Python | GPLv3 | 500 MB–2 GB | 30–90 с | 8+ контейнеров | ~3 GB |
| Ansible Tower | Python | Коммерческий | 1–2 GB | 60–120 с | RPM/VM | ~3 GB |
| Ansible Semaphore | Go | MIT | ~80 MB | <1 с | 1 бинарник | ~80 MB |
| Jenkins | Java | MIT | 512 MB–1 GB | 30–60 с | WAR / Docker | ~700 MB |
| Rundeck | Java/Groovy | Apache 2.0 | 512 MB–1 GB | 15–30 с | WAR / Docker | ~600 MB |
| Argo CD | Go | Apache 2.0 | 200–500 MB | 10–30 с | K8s manifests | ~300 MB |
| Flux CD | Go | Apache 2.0 | ~200 MB | 10–20 с | K8s CRDs | ~200 MB |
| Spinnaker | Java | Apache 2.0 | 1–2 GB | 2–5 мин | 10+ сервисов | ~4 GB |
| Terraform Cloud | Go (SaaS) | BUSL 1.1 | SaaS | SaaS | SaaS / Enterprise | N/A |
| Spacelift | Go (SaaS) | Коммерческий | SaaS | SaaS | SaaS | N/A |
| Atlantis | Go | Apache 2.0 | ~50 MB | <1 с | 1 бинарник | ~60 MB |
| Pulumi Cloud | Go (SaaS) | Apache 2.0 (SDK) | SaaS | SaaS | SaaS / Self-hosted | N/A |
| Harness | Java (SaaS) | BSL / Коммерческий | SaaS | SaaS | SaaS / Self-hosted | ~2 GB |
| GitLab CI | Ruby/Go | MIT (CE) / Koммерч. | ~800 MB | 15–30 с | Многосервисный | ~2 GB |

---

### 13.3 — Поддержка технологий автоматизации

| Инструмент | Ansible | Terraform | Pulumi | Bash/Script | K8s | OpenTofu |
|------------|---------|-----------|--------|------------|-----|---------|
| **Velum** | ✅ **Native** | ✅ **Native** | ⏳ FI-plan | ✅ | ⏳ K8s Runner | ✅ |
| AWX / Tower | ✅ Native | ❌ Plugin | ❌ | ✅ | ✅ | ❌ |
| Ansible Semaphore | ✅ Native | ✅ | ❌ | ✅ | ❌ | ✅ |
| Jenkins | ⚙️ Plugin | ⚙️ Plugin | ⚙️ Plugin | ✅ | ⚙️ Plugin | ⚙️ |
| Rundeck | ⚙️ Plugin | ⚙️ Plugin | ❌ | ✅ | ⚙️ Plugin | ❌ |
| Argo CD | ❌ | ❌ | ❌ | ❌ | ✅ **Native** | ❌ |
| Flux CD | ❌ | ✅ (tf-controller) | ❌ | ❌ | ✅ **Native** | ✅ |
| Spinnaker | ⚙️ Plugin | ⚙️ Plugin | ❌ | ❌ | ✅ | ❌ |
| Terraform Cloud | ❌ | ✅ **Native** | ❌ | ⚙️ | ✅ | ✅ |
| Spacelift | ✅ | ✅ **Native** | ✅ | ✅ | ✅ | ✅ |
| Atlantis | ❌ | ✅ **Native** | ❌ | ❌ | ❌ | ✅ |
| Pulumi Cloud | ❌ | ✅ (compatible) | ✅ **Native** | ✅ | ✅ | ❌ |
| Harness | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| GitLab CI | ✅ (CI jobs) | ✅ (state backend) | ✅ | ✅ | ✅ (Agent) | ✅ |

> Легенда: ✅ Встроено | ⚙️ Плагин/интеграция | ⏳ В плане | ❌ Нет

---

### 13.4 — Аутентификация и безопасность

| Инструмент | LDAP | OIDC/OAuth2 | SAML | TOTP 2FA | SSO | RBAC | Audit Log |
|------------|------|-------------|------|----------|-----|------|-----------|
| **Velum** | ✅ | ✅ | ❌ | ✅ **Built-in** | ✅ | ✅ | ✅ |
| AWX / Tower | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ |
| Ansible Semaphore | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Jenkins | ⚙️ | ⚙️ | ⚙️ | ⚙️ | ⚙️ | ⚙️ | ⚙️ |
| Rundeck | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ |
| Argo CD | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ |
| Flux CD | — | K8s RBAC | — | — | K8s | ✅ | ✅ |
| Spinnaker | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ |
| Terraform Cloud | — | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Spacelift | — | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Atlantis | — | — | — | — | ❌ | ❌ | ❌ |
| Pulumi Cloud | — | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Harness | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| GitLab CI | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

---

### 13.5 — Продвинутые функции

| Инструмент | WebSocket логи | Approval Workflow | DAG / Пайплайны | Survey/Формы | Уведомления |
|------------|---------------|-------------------|-----------------|-------------|-------------|
| **Velum** | ✅ | ✅ | ✅ **DAG UI** | ✅ **Drag&Drop** | ✅ Политики |
| AWX / Tower | ✅ | ✅ | ✅ Workflow | ✅ Survey | ✅ |
| Ansible Semaphore | ✅ | ❌ | ❌ | ❌ | ✅ |
| Jenkins | ✅ | ✅ (plugin) | ✅ Pipeline | ⚙️ Plugin | ✅ (plugin) |
| Rundeck | ✅ | ✅ | ✅ (jobs) | ✅ | ✅ |
| Argo CD | ✅ | ✅ Sync gates | ✅ Argo WF | ❌ | ✅ |
| Flux CD | Partial | ❌ | ❌ | ❌ | ✅ |
| Spinnaker | ✅ | ✅ Manual judge | ✅ Pipelines | ❌ | ✅ |
| Terraform Cloud | ✅ | ✅ | ❌ | ❌ | ✅ |
| Spacelift | ✅ | ✅ | ✅ (stacks) | ❌ | ✅ |
| Atlantis | PR comments | ✅ (PR-based) | ❌ | ❌ | ❌ |
| Pulumi Cloud | ✅ | ✅ | ❌ | ❌ | ✅ |
| Harness | ✅ | ✅ | ✅ Pipelines | ✅ | ✅ |
| GitLab CI | ✅ | ✅ Environments | ✅ Pipelines | ❌ | ✅ |

---

### 13.6 — DevOps-специфичные функции

| Инструмент | Drift Detection | Rollback | Cost Tracking | GitOps | Template Marketplace | Diff между запусками |
|------------|----------------|----------|---------------|--------|---------------------|----------------------|
| **Velum** | ✅ **Terraform** | ✅ **Snapshots** | ✅ **Infracost** | ✅ | ✅ **11 шаблонов** | ✅ **LCS engine** |
| AWX / Tower | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Ansible Semaphore | ❌ | ❌ | ❌ | Partial | ❌ | ❌ |
| Jenkins | ❌ | ⚙️ Plugin | ❌ | Jenkins X ✅ | ✅ 1800+ plugins | ❌ |
| Rundeck | ❌ | ❌ | ❌ | ❌ | ✅ Plugin hub | ❌ |
| Argo CD | ✅ **Native** | ✅ | ❌ | ✅ **Native** | ❌ | ❌ |
| Flux CD | ✅ **Native** | ✅ | ❌ | ✅ **Native** | ❌ | ❌ |
| Spinnaker | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Terraform Cloud | ✅ | ✅ State | ✅ (v2) | ✅ | ✅ Registry | ❌ |
| Spacelift | ✅ | ✅ | ✅ **Infracost** | ✅ | ❌ | ❌ |
| Atlantis | ❌ | ❌ | ✅ Infracost | ✅ **PR-based** | ❌ | ❌ |
| Pulumi Cloud | ✅ | ✅ | ✅ | ✅ | ✅ Registry | ❌ |
| Harness | ✅ | ✅ | ✅ **CCM module** | ✅ | ❌ | ❌ |
| GitLab CI | ❌ | ✅ Environments | ❌ | ✅ | ❌ | ❌ |

---

### 13.7 — AI и современные интеграции

| Инструмент | AI / LLM | MCP Server | REST API | GraphQL | Webhook | White-label |
|------------|----------|-----------|---------|---------|---------|------------|
| **Velum** | ✅ **Claude+OpenAI** | ✅ **60 tools** | ✅ 75+ endpoints | ⏳ | ✅ | ✅ **Branding API** |
| AWX / Tower | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ |
| Ansible Semaphore | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ |
| Jenkins | ❌ | ❌ | ✅ | ❌ | ✅ | ⚙️ Plugin |
| Rundeck | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ (Enterprise) |
| Argo CD | ❌ | ❌ | ✅/gRPC | ❌ | ✅ | ❌ |
| Flux CD | ❌ | ❌ | K8s API | ❌ | ✅ | ❌ |
| Spinnaker | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ |
| Terraform Cloud | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ (Enterprise) |
| Spacelift | ✅ **Spacelift AI** | ❌ | ✅ | ✅ | ✅ | ❌ |
| Atlantis | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ |
| Pulumi Cloud | ✅ **Pulumi AI** | ✅ **MCP Server** | ✅ | ❌ | ✅ | ❌ |
| Harness | ✅ **AIDA (AI)** | ❌ | ✅ | ✅ | ✅ | ✅ |
| GitLab CI | ✅ **GitLab Duo** | ❌ | ✅ | ✅ | ✅ | ✅ (EE) |

---

### 13.8 — Ценообразование

| Инструмент | Бесплатно | Коммерческий тариф | Ограничения Free tier |
|------------|----------|-------------------|----------------------|
| **Velum** | **∞ MIT** | — | **Нет ограничений** |
| AWX | ✅ GPLv3 | Tower $14K–$50K/год | — |
| Ansible Semaphore | ✅ MIT | — | Нет ограничений |
| Jenkins | ✅ MIT | Support $5K+/год | — |
| Rundeck | ✅ Apache | Process Auto $от $15K/год | — |
| Argo CD | ✅ Apache | — | — |
| Flux CD | ✅ Apache | Weave GitOps Enterprise | — |
| Spinnaker | ✅ Apache | Support/hosted | Сложный деплой |
| Terraform Cloud | 500 ресурсов | $20/user/мес | 500 managed resources |
| Spacelift | ❌ | ~$200–$500/мес (min) | 5 запусков/мес trial |
| Atlantis | ✅ Apache | — | Нет UI |
| Pulumi Cloud | 1 пользователь | $50/user/мес | 1 user, 3 stacks |
| Harness | Ограниченный | ~$100–300/user/мес | CI: 2000 билд-минут |
| GitLab CI | 400 мин CI/мес | $19–99/user/мес | 400 мин/мес |

---

### 13.9 — Self-hosted и операционная сложность

| Инструмент | Self-hosted | Сложность деплоя | Docker single-container | SQLite | PostgreSQL | MySQL |
|------------|------------|-----------------|------------------------|--------|-----------|-------|
| **Velum** | ✅ | ⭐ **Минимальная** | ✅ | ✅ | ✅ | ✅ |
| AWX | ✅ | ⭐⭐⭐⭐ Высокая | ❌ (operator) | ❌ | ✅ | ❌ |
| Ansible Semaphore | ✅ | ⭐ Минимальная | ✅ | ✅ | ✅ | ✅ |
| Jenkins | ✅ | ⭐⭐ Низкая | ✅ | ❌ | ✅ | ✅ |
| Rundeck | ✅ | ⭐⭐ Низкая | ✅ | ❌ | ✅ | ✅ |
| Argo CD | ✅ | ⭐⭐⭐ Средняя (K8s) | ❌ (K8s only) | ❌ | ✅ | ❌ |
| Flux CD | ✅ | ⭐⭐⭐ Средняя (K8s) | ❌ (K8s only) | ❌ | ❌ | ❌ |
| Spinnaker | ✅ | ⭐⭐⭐⭐⭐ Очень высокая | ❌ (10+ сервисов) | ❌ | ✅ | ✅ |
| Terraform Cloud | Enterprise | ⭐⭐⭐ Средняя | ❌ | ❌ | ✅ | ❌ |
| Spacelift | Worker pools | ⭐⭐ Низкая | ❌ | ❌ | N/A | ❌ |
| Atlantis | ✅ | ⭐ Минимальная | ✅ | ❌ | ✅ | ✅ |
| Pulumi Cloud | ✅ (Business) | ⭐⭐ Низкая | ✅ | ❌ | ✅ | ❌ |
| Harness | ✅ | ⭐⭐⭐⭐ Высокая | ❌ | ❌ | ✅ | ✅ |
| GitLab CI | ✅ | ⭐⭐⭐ Средняя | ❌ (GitLab Runner ✅) | ❌ | ✅ | ✅ |

---

### 13.10 — Итоговая сравнительная таблица (Score Card)

> Оценка от 0 до 5 по каждому критерию. **Velum выделен жирным.**

| Критерий | **Velum** | AWX | Semaphore | Jenkins | Rundeck | Argo CD | TF Cloud | Spacelift | Harness |
|----------|----------|-----|-----------|---------|---------|---------|---------|-----------|---------|
| **Простота деплоя** | **5** | 1 | 5 | 4 | 4 | 2 | 4 | 5 | 2 |
| **Лёгкость (RAM/CPU)** | **5** | 1 | 5 | 3 | 3 | 4 | 5 | 5 | 2 |
| **Ansible поддержка** | **5** | 5 | 5 | 3 | 3 | 0 | 0 | 4 | 4 |
| **Terraform поддержка** | **5** | 1 | 4 | 2 | 2 | 1 | 5 | 5 | 4 |
| **AI интеграция** | **5** | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 4 |
| **MCP сервер** | **5** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| **Cost Tracking** | **4** | 0 | 0 | 0 | 0 | 0 | 4 | 5 | 5 |
| **Drift Detection** | **4** | 0 | 0 | 0 | 0 | 5 | 5 | 5 | 4 |
| **Rollback** | **5** | 0 | 0 | 2 | 2 | 5 | 4 | 4 | 5 |
| **DAG Workflow** | **4** | 5 | 0 | 5 | 4 | 4 | 2 | 3 | 5 |
| **Survey/Forms** | **5** | 5 | 0 | 2 | 4 | 0 | 0 | 0 | 2 |
| **GitOps** | **3** | 1 | 2 | 3 | 1 | 5 | 5 | 5 | 5 |
| **Marketplace** | **3** | 2 | 0 | 5 | 3 | 0 | 5 | 0 | 0 |
| **Multi-tenancy** | **4** | 4 | 2 | 3 | 3 | 4 | 5 | 5 | 5 |
| **RBAC + Auth** | **4** | 5 | 3 | 3 | 4 | 5 | 5 | 5 | 5 |
| **Цена (TCO)** | **5** | 3 | 5 | 4 | 3 | 5 | 3 | 2 | 1 |
| **White-labeling** | **5** | 0 | 0 | 1 | 3 | 0 | 3 | 0 | 3 |
| **WebSocket логи** | **5** | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| **Итого (из 90)** | **82** | 38 | 41 | 49 | 48 | 51 | 61 | 66 | 61 |

---

### 13.11 — Позиционирование Velum по сегментам

```
Кто использует Velum?

┌─────────────────────────────────────────────────────────────────┐
│  СЕГМЕНТ               │ ПОЧЕМУ VELUM           │ VS            │
├────────────────────────┼────────────────────────┼───────────────┤
│ Стартапы / SMB         │ 1 бинарник, $0, SQLite  │ AWX слишком   │
│                        │ Запуск за 5 минут       │  тяжёлый      │
├────────────────────────┼────────────────────────┼───────────────┤
│ Ansible-команды        │ Семафор + Enterprise фичи│ Semaphore без │
│                        │ TOTP, AI, DAG, Survey   │  этих фич     │
├────────────────────────┼────────────────────────┼───────────────┤
│ Mixed Ansible+Terraform│ Оба — first-class       │ AWX нет TF,   │
│                        │ Cost tracking built-in  │  TF Cloud нет │
│                        │                         │  Ansible      │
├────────────────────────┼────────────────────────┼───────────────┤
│ Edge / Air-gapped      │ 1 бинарник, offline,    │ SaaS-решения  │
│                        │ нет внешних зависимостей│  не работают  │
├────────────────────────┼────────────────────────┼───────────────┤
│ AI-первые команды      │ MCP сервер (60 tools),  │ Никто не имеет│
│                        │ Claude/OpenAI built-in  │  MCP нативно  │
├────────────────────────┼────────────────────────┼───────────────┤
│ White-label SaaS       │ Branding API,           │ Большинство   │
│                        │ организации, квоты      │  нет white-   │
│                        │                         │  label в OSS  │
└─────────────────────────────────────────────────────────────────┘
```

---

### 13.12 — Что реализовать, чтобы закрыть оставшиеся разрывы

| Разрыв | Конкурент лидирует | Что реализовать в Velum | Версия |
|--------|-------------------|------------------------|--------|
| GitOps нативный | Argo CD, Flux, TF Cloud | Auto-sync Git → templates, drift alerts | v6.0 |
| K8s Runner | AWX, Argo CD | `executor: kubernetes` pod per task | v5.1 |
| SAML SSO | AWX, TF Cloud, Harness | `samlauth` crate или SAML proxy | v6.0 |
| PR-based Terraform | Atlantis, Spacelift | GitHub PR webhook → plan → comment | v6.0 |
| Visual Pipeline Builder | Jenkins, Harness | Улучшить DAG UI: условия, переменные | v5.1 |
| Cost Management Module | Harness CCM, Spacelift | Детальный дашборд cost by project/user | v6.0 |
| Plugin / Extension SDK | Jenkins 1800+ plugins | `velum plugin install <repo>` API | v6.1 |
| GraphQL API | Spacelift, Harness | mutations + subscriptions | v5.1 |
| Pulumi support | Spacelift, Harness | `TemplateApp::Pulumi` + runner | v6.0 |

---

## 🧬 БЛОК 14 — Лучшее от каждого конкурента: что взять в Velum

> Анализ: какую **одну лучшую идею** взять из каждого инструмента, адаптировать под Velum и реализовать лучше оригинала.

---

### 🔴 AWX / Ansible Tower → **Execution Environments (EE)**

**Что это:** AWX запускает каждую задачу в изолированном OCI-контейнере (`ee-minimal-rhel8`), где заранее собраны нужные версии Ansible, коллекций, Python-зависимостей. Playbook никогда не ломается из-за "у меня работало" — окружение воспроизводимо.

**Как адаптировать для Velum:**
```
Шаблон → поле "execution_image": "ghcr.io/myorg/ee-network:2.15"
Velum запускает задачу внутри docker run --rm <image> ansible-playbook ...
Логи стримятся через WebSocket как обычно
```

**Почему лучше AWX:** в AWX EE обязательны и сложно настраиваются. В Velum — опциональное поле, по умолчанию локальный ansible, но если указан образ — задача идёт в контейнере.

**Реализация:** `rust/src/services/local_job/run.rs` — добавить ветку `if let Some(ee) = template.execution_image`.
**Сложность:** 🟡 Средняя | **Версия:** v5.1 | **ID:** `FI-AWX-1`

---

### 🔵 Ansible Semaphore → **App Runner (интеграция приложений)**

**Что это:** Semaphore v2.10+ ввёл Apps — возможность подключить внешние системы (GitLab, GitHub, Telegram) как источники триггеров, получать данные и реагировать на события. Каждый App имеет свой webhook и конфиг.

**Что уже есть в Velum:** Apps страница (`apps.html`) реализована.

**Что взять дополнительно:** механизм **App Templates** из Semaphore — пре-конфигурированные шаблоны интеграций для популярных сервисов (GitHub, Bitbucket, Azure DevOps, Gitea) с автозаполнением webhook URL и валидацией секретов.

**Реализация:** галерея из 10+ готовых App-конфигураций в `apps.html` — кнопка "Добавить из галереи" → выбор GitHub/Gitea/etc → автозаполнение полей.
**Сложность:** 🟢 Низкая | **Версия:** v5.0 | **ID:** `FI-SEM-1`

---

### ☕ Jenkins → **Shared Libraries / Reusable Pipeline Code**

**Что это:** Jenkins Shared Libraries — возможность вынести повторяющийся Groovy-код в отдельный Git репозиторий и импортировать: `@Library('my-pipeline-lib') _`. Тысячи команд используют это для стандартизации пайплайнов.

**Как адаптировать для Velum:**
```
"Template Includes" — шаблон может наследовать переменные, vault-файлы
и параметры из другого шаблона (родительского).

parent_template: "base-deploy"   # наследуем переменные
extra_vars: { env: "prod" }       # переопределяем только нужное
```
**Результат:** базовый шаблон `base-deploy` хранит общие переменные (AWS region, become_user и т.д.), дочерние шаблоны только переопределяют специфику.

**Реализация:** поле `parent_template_id` в таблице `project_template` + merge extra_vars при запуске.
**Сложность:** 🟡 Средняя | **Версия:** v5.1 | **ID:** `FI-JEN-1`

---

### 🔧 Rundeck → **Node Health Checks + Auto-disable**

**Что это:** Rundeck умеет проверять доступность нод перед запуском задачи (`node health check`). Если нода недоступна — задача автоматически пропускает её или выдаёт понятную ошибку, а не зависает на SSH timeout.

**Как адаптировать для Velum:**
```
Inventory → настройка "health check command": "ping -c1 {host} || echo UNREACHABLE"
Перед запуском задачи Velum проверяет все целевые хосты параллельно (timeout 5с)
Недоступные хосты: warn + skip или fail fast (настраивается)
Результат health check виден в UI задачи до начала основного лога
```

**Реализация:** `rust/src/services/local_job/preflight.rs` — новый модуль предстартовых проверок.
**Сложность:** 🟡 Средняя | **Версия:** v5.1 | **ID:** `FI-RUN-1`

---

### 🐙 Argo CD → **Sync Waves + Sync Hooks**

**Что это:** Argo CD позволяет задать порядок применения ресурсов через аннотацию `argocd.argoproj.io/sync-wave: "5"`. Ресурсы с меньшим числом применяются первыми. Hook'и (`PreSync`, `PostSync`, `SyncFail`) — запускают задачи до/после/при ошибке синхронизации.

**Как адаптировать для Velum:**
```
DAG Workflow → атрибут "wave": 1, 2, 3 для нод графа
Все ноды одной волны запускаются параллельно
Следующая волна стартует только после успеха предыдущей

Hooks в шаблоне:
  pre_run:  "check-disk-space"    # шаблон, выполняемый перед основным
  post_run: "send-report"         # после успеха
  on_fail:  "rollback-snapshot"   # при ошибке
```

**Реализация:** расширить `workflow_nodes` полем `wave INT`, расширить `project_template` полями `pre_template_id`, `post_template_id`, `fail_template_id`.
**Сложность:** 🟠 Средняя | **Версия:** v5.1 | **ID:** `FI-ARGO-1`

---

### 🌊 Flux CD → **Image Automation Controller**

**Что это:** Flux CD отслеживает OCI registry на появление новых тегов образов и автоматически обновляет Git-репозиторий (меняет тег в YAML), что запускает новый деплой. Полностью автоматизированный GitOps без кнопки "Deploy".

**Как адаптировать для Velum:**
```
Repository → настройка "image watch":
  registry: "ghcr.io/myorg/myapp"
  tag_pattern: "^v[0-9]+\.[0-9]+\.[0-9]+$"  # только semver
  on_new_tag: запустить шаблон "deploy-production"
  auto_approve: false  # требовать подтверждение в UI

Velum polling каждые N минут → новый тег найден → создать pending task
```

**Реализация:** `rust/src/services/image_watcher.rs` — периодический poll registry API, интеграция с cron scheduler.
**Сложность:** 🟠 Средняя | **Версия:** v6.0 | **ID:** `FI-FLUX-1`

---

### 🎡 Spinnaker → **Canary Analysis (Automated Canary)**

**Что это:** Spinnaker Kayenta — автоматический анализ канареечного деплоя. Сравнивает метрики canary vs baseline через Prometheus/Datadog/Stackdriver и автоматически принимает решение: продвигать или откатывать.

**Как адаптировать для Velum:**
```
Шаблон типа "Canary Deploy":
  1. Деплоить на 10% трафика (canary)
  2. Ждать N минут (настраивается)
  3. Запросить метрики из Prometheus: error_rate, p99_latency, cpu
  4. Сравнить canary vs baseline (статистический тест)
  5. Если OK → approval request для 100% (или auto-promote)
  6. Если хуже → auto-rollback + уведомление

Велум показывает: score, графики метрик, decision log
```

**Реализация:** `rust/src/services/canary_analysis.rs` + новый тип шаблона `TemplateApp::Canary` + Prometheus query в scheduler.
**Сложность:** 🔴 Высокая | **Версия:** v6.1 | **ID:** `FI-SPI-1`

---

### 🏗️ Terraform Cloud (HCP) → **State Locking UI + Remote State Browser**

**Что это:** TFC показывает кто и когда заблокировал Terraform state, позволяет принудительно разблокировать через UI. State browser — просмотр всех ресурсов в state прямо в веб-интерфейсе без `terraform show`.

**Как адаптировать для Velum:**
```
Task view → раздел "Terraform State":
  - Locked by: user@example.com  [Force Unlock]
  - Resources: 47 managed, 3 added, 1 destroyed
  - State version: #23 (2026-03-23 14:52)
  - Browser: список ресурсов с типом, именем, атрибутами

GET /api/projects/{id}/terraform/state        → parse state JSON
GET /api/projects/{id}/terraform/state/lock   → проверить лок
DELETE /api/projects/{id}/terraform/state/lock → разблокировать
```

**Реализация:** `rust/src/api/handlers/terraform_state.rs` + `web/public/terraform-state.html`.
**Сложность:** 🟡 Средняя | **Версия:** v5.1 | **ID:** `FI-TFC-1`

---

### 🚀 Spacelift → **Stack Dependencies + Trigger Chains**

**Что это:** Spacelift Stacks могут зависеть друг от друга. Stack `network` → Stack `database` → Stack `app`. При изменении `network` автоматически запускается цепочка зависимых стеков. Output одного стека становится Input следующего.

**Как адаптировать для Velum:**
```
Шаблон → поле "triggers":
  - after: "network-deploy"   # запускаться после успеха этого шаблона
    pass_outputs: true          # передать output как extra_vars

network-deploy завершился → output: { vpc_id: "vpc-123", subnet: "10.0.0.0/24" }
  → auto-trigger database-deploy с extra_vars: { vpc_id: "vpc-123", ... }
  → auto-trigger app-deploy с extra_vars: { ... }
```

**Реализация:** расширить `project_template` полями `trigger_on_template_id`, `pass_outputs_as_vars BOOL`.
**Сложность:** 🟠 Средняя | **Версия:** v5.1 | **ID:** `FI-SPA-1`

---

### 🌊 Atlantis → **PR Plan → Comment → Apply Workflow**

**Что это:** Atlantis реализует классический GitOps для Terraform через PR-комментарии: `atlantis plan` в PR → автокомментарий с diff → `atlantis apply` → merge разблокирован. Прост как гвоздь, работает везен.

**Как адаптировать для Velum:**
```
Webhook от GitHub PR → Velum:
  1. PR opened/updated → auto-run terraform plan
  2. Результат plan → comment в PR с diff (через GitHub API)
     "## Velum Terraform Plan
      + aws_instance.web (create)
      ~ aws_security_group.main (update)
      Plan: 2 to add, 1 to change, 0 to destroy
      Cost delta: +$45/month
      [✅ Approve in Velum](https://velum.example.com/...)"
  3. Approval в Velum UI → terraform apply → PR comment обновляется
  4. Apply успех → GitHub status check ✅ → merge разблокирован

```

**Реализация:** `rust/src/services/github_pr.rs` — GitHub API client для комментариев и status checks.
**Сложность:** 🟠 Средняя | **Версия:** v6.0 | **ID:** `FI-ATL-1`

---

### 🌐 Pulumi Cloud → **Structured Outputs + Stack References**

**Что это:** Pulumi экспортирует именованные выходные значения стека (`pulumi.export("bucket_name", bucket.name)`), которые другие стеки могут читать как `StackReference`. Это создаёт граф зависимостей между инфраструктурными компонентами.

**Как адаптировать для Velum:**
```
После завершения задачи Velum парсит структурированный output:
  - Terraform: terraform output -json → сохранить в БД
  - Ansible: зарегистрированные факты через callback plugin
  - Bash: JSON строки из stdout (convention: "VELUM_OUTPUT: {...}")

Сохранить в таблицу task_outputs (task_id, key, value, type)
UI: вкладка "Outputs" в task view
API: GET /api/tasks/{id}/outputs
Другие шаблоны могут ссылаться: extra_vars: { vpc_id: "{{outputs.network-deploy.vpc_id}}" }
```

**Реализация:** `rust/src/services/output_extractor.rs` + таблица `task_outputs` в миграции.
**Сложность:** 🟡 Средняя | **Версия:** v5.1 | **ID:** `FI-PUL-1`

---

### ⚙️ Harness → **Pipeline Studio (Visual + YAML dual-mode)**

**Что это:** Harness Pipeline Studio — лучший в индустрии редактор пайплайнов. Переключение между визуальным drag-and-drop и raw YAML в реальном времени, с валидацией и автодополнением. Изменил в визуальном — YAML обновился, изменил в YAML — граф обновился.

**Как адаптировать для Velum:**
```
DAG Workflow Editor → добавить кнопку "YAML" рядом с "Visual":

# workflow.yaml
name: Deploy Production
nodes:
  - id: plan
    template: terraform-plan
    wave: 1
  - id: approve
    type: approval
    approvers: [managers]
    wave: 2
  - id: apply
    template: terraform-apply
    depends_on: [approve]
    wave: 3
  - id: notify
    template: slack-notify
    on: always
    wave: 4

Двустороннее редактирование: визуальный ↔ YAML через serde
```

**Реализация:** добавить YAML import/export в `web/public/workflows.html` + `rust/src/api/handlers/workflows.rs` endpoint `POST /workflows/from-yaml`.
**Сложность:** 🟡 Средняя | **Версия:** v5.1 | **ID:** `FI-HAR-1`

---

### 🦊 GitLab CI → **Environments + Deployment Tracking**

**Что это:** GitLab Environments — центральный реестр окружений (production, staging, dev), где видна история деплоев: кто, когда, какой коммит, статус. Кнопка "Rollback" прямо в списке окружений. Ссылка на живое окружение (`environment_url`).

**Как адаптировать для Velum:**
```
Новая сущность "Environment" (отдельно от Inventory):
  name: "production"
  url: "https://app.example.com"
  template: "deploy-prod"       # какой шаблон деплоит
  tier: production|staging|dev|review

Страница /environments:
  ┌─────────────┬──────────┬────────────────┬──────────────┬──────────┐
  │ Environment │ Status   │ Last Deploy    │ Deployed By  │ Actions  │
  ├─────────────┼──────────┼────────────────┼──────────────┼──────────┤
  │ production  │ ✅ Active │ v2.3.1 (2ч назад)│ alice      │ 🚀 🔄 ⏪ │
  │ staging     │ ✅ Active │ v2.4.0-rc1     │ ci-bot      │ 🚀 🔄 ⏪ │
  │ dev         │ 🔶 Stale  │ v2.4.0-dev     │ bob         │ 🚀 🔄 ⏪ │
  └─────────────┴──────────┴────────────────┴──────────────┴──────────┘

Интеграция: шаблон помечает свой environment → history группируется по env
```

**Реализация:** таблица `environments`, `rust/src/api/handlers/environments.rs`, `web/public/environments.html`.
**Сложность:** 🟡 Средняя | **Версия:** v5.1 | **ID:** `FI-GL-1`

---

### ✨ Velum → **AI-assisted Template Creation (уникальная фича)**

**Что это:** Единственная фича которой нет **нигде**. Пользователь описывает задачу на естественном языке, AI генерирует шаблон, extra_vars, survey form и inventory.

```
Пользователь пишет: "Хочу деплоить Django app на 3 сервера с nginx и PostgreSQL"
AI возвращает:
  - Заполненный шаблон с playbook path
  - Survey form: db_password, app_version, domain_name
  - Suggested inventory structure
  - Extra vars: become=true, python_version=3.11
  - Рекомендованные vault keys для секретов
  - Ссылки на 2 похожих шаблона из Marketplace
```

**Реализация:** MCP tool `create_template_from_description` → Claude API → structured output → шаблон создаётся в один клик.
**Сложность:** 🟡 Средняя | **Версия:** v5.0 | **ID:** `FI-VEL-1`

---

### 📊 Сводная таблица "Взять лучшее"

| ID | Источник | Фича | Суть | Сложность | Версия |
|----|---------|------|------|-----------|--------|
| FI-AWX-1 | AWX | **Execution Environments** | Docker-образ для каждого запуска | 🟡 | v5.1 |
| FI-SEM-1 | Semaphore | **App Gallery** | 10+ готовых App-конфигов (GitHub, Gitea...) | 🟢 | v5.0 |
| FI-JEN-1 | Jenkins | **Template Inheritance** | `parent_template` + merge extra_vars | 🟡 | v5.1 |
| FI-RUN-1 | Rundeck | **Node Health Preflight** | Проверка хостов до старта задачи | 🟡 | v5.1 |
| FI-ARGO-1 | Argo CD | **Sync Waves + Hooks** | Волны + pre/post/fail хуки | 🟠 | v5.1 |
| FI-FLUX-1 | Flux CD | **Image Watcher** | Auto-trigger при новом Docker-теге | 🟠 | v6.0 |
| FI-SPI-1 | Spinnaker | **Canary Analysis** | Авто-анализ метрик canary vs baseline | 🔴 | v6.1 |
| FI-TFC-1 | TF Cloud | **State Browser + Lock UI** | Просмотр state, force unlock | 🟡 | v5.1 |
| FI-SPA-1 | Spacelift | **Stack Trigger Chains** | Output одного → Input следующего | 🟠 | v5.1 |
| FI-ATL-1 | Atlantis | **PR Plan→Comment→Apply** | GitHub PR comment workflow | 🟠 | v6.0 |
| FI-PUL-1 | Pulumi | **Structured Outputs** | Именованные outputs → доступны другим шаблонам | 🟡 | v5.1 |
| FI-HAR-1 | Harness | **YAML ↔ Visual dual-mode** | Переключение редактора DAG | 🟡 | v5.1 |
| FI-GL-1 | GitLab CI | **Environments Registry** | Реестр окружений + история деплоев | 🟡 | v5.1 |
| FI-VEL-1 | Velum | **AI Template Creator** | Описание → готовый шаблон | 🟡 | v5.0 |

---

### 🎯 Топ-5 приоритетов по impact/effort

| # | Фича | Почему важно | Effort |
|---|------|-------------|--------|
| 1 | **Environments Registry** (GitLab) | Самая запрашиваемая фича у Semaphore-пользователей | 3 дня |
| 2 | **AI Template Creator** (Velum) | Уникальный дифференциатор, MCP уже есть | 2 дня |
| 3 | **Template Inheritance** (Jenkins) | Устраняет дублирование у 90% команд | 2 дня |
| 4 | **Structured Outputs** (Pulumi) | Разблокирует Stack Trigger Chains | 3 дня |
| 5 | **Sync Waves + Hooks** (Argo CD) | Делает DAG значительно мощнее | 4 дня |

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
| 5.0-plan+ideas | 2026-03-23 | 💡 БЛОК 12: Future Ideas — 40+ направлений развития v5+ (15 категорий) |
| 5.0-plan | 2026-03-23 | 🗺️ Добавлен план v5.0: Security, Remote Runners, Testing, DX, Infra (42 задачи) |
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
