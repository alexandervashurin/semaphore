# MASTER_PLAN V3 — Velum: Стать лучше AWX и Ansible Tower

> **Последнее обновление:** 2026-03-21 (сессия 5 — Diff между запусками, Terraform Cost Tracking)
> **Версия:** 3.2
> **Статус:** АКТИВНЫЙ ПЛАН РАЗРАБОТКИ

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

- T-BE-15: `exporter_entities.rs` restore пользователей (⏸ dead code, низкий приоритет)

---

## Ссылки

| Репозиторий | URL |
|---|---|
| Velum (origin) | https://github.com/tnl-o/velum |
| Upstream (alexandervashurin) | https://github.com/alexandervashurin/semaphore |
| Go-оригинал (эталон) | https://github.com/velum/velum |
| Semaphore MCP (референс, Python) | https://github.com/cloin/semaphore-mcp |
