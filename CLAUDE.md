# CLAUDE.md — Инструкции для AI-агента

> Этот файл читается Claude Code при каждом запуске. Следуй порядку действий строго.

---

## Конечная цель проекта

**Полная миграция Velum с Go на Rust** — feature parity с Go-оригиналом, опубликованная на GitHub.

| Репозиторий | URL |
|---|---|
| Наш (origin) | https://github.com/tnl-o/rust_semaphore |
| Upstream (alexandervashurin) | https://github.com/alexandervashurin/semaphore |
| Go-оригинал (эталон фич) | https://github.com/velum/velum |

Ориентируйся на **Go-оригинал** как источник правды о том, что должно работать.
Ориентируйся на **MASTER_PLAN.md** как живой план задач.

---

## Текущее состояние проекта (2026-03-17)

### Что готово
- **Бэкенд: 99%+** — все 75+ API эндпоинтов реализованы, тесты зелёные (cargo test + clippy 0 warnings)
- **Cron Scheduler** — SchedulePool подключён к запуску сервера (`cmd_server.rs`), задачи стартуют автоматически
- **Shared Task Execution** — `services/task_execution.rs` — единая точка запуска задач (HTTP + scheduler)
- **Store Chain** — рефакторинг `Box<dyn Store>` → `Arc<dyn Store + Send + Sync>` по всей цепочке
- **Дизайн** — Material Design (Roboto, teal sidebar #005057, logo.svg, background.svg)
- **Docker** — `docker-compose.demo.yml` для быстрого запуска с SQLite, автосид admin/admin123
- **Auth** — JWT, bcrypt, TOTP, LDAP, OIDC, refresh token — всё реализовано
- **Task Runner** — реальный запуск ansible/terraform/bash с WebSocket логами
- **Фаза 6** (Vanilla JS frontend) — 100% CRUD форм, 28+ HTML-страниц, полный feature parity

### Открытые задачи
- **T-BE-15** — `exporter_entities.rs` restore пользователей/проектов (⏸ dead code, низкий приоритет)

### Текущий приоритет
Все критичные и высокоприоритетные задачи закрыты. Смотри MASTER_PLAN.md раздел 2е для полного списка.
Следующие работы: E2E тестирование реального деплоя, production hardening, документация.

---

## Порядок действий при каждом запуске

### 1. Проверка состояния git

```bash
git status
git log --oneline -5
```

- Если есть незакоммиченные изменения — сначала разберись с ними

---

### 2. Fetch + Merge из upstream

```bash
git fetch upstream
git log upstream/main --oneline -5
git merge upstream/main --no-edit
```

- Remote `upstream` = `https://github.com/alexandervashurin/semaphore`
- При конфликтах: разрешай сохраняя наши изменения, затем `cargo check`
- После успешного merge — сразу пуш в origin:
  ```bash
  git push origin main
  ```

---

### 3. Проверка MASTER_PLAN.md

Читай секции **"Текущее состояние"** и **"Известные проблемы и блокеры"** (раздел 17).
Также читай **раздел 2 и 2.4** — таблица фронтенд задач B-FE-01..B-FE-22.

- Код — источник правды. Если план расходится с кодом — обнови план
- Выбирай следующую задачу по приоритету: 🔴 → 🟠 → 🟡
- Смотри на Go-оригинал чтобы понять что ещё нужно реализовать

---

### 4. Работа: реализация задач

1. Создай todo-список через `TodoWrite`
2. Читай файлы перед редактированием
3. Помечай задачи `in_progress` → `completed` в реальном времени
4. `cargo check` после каждого значимого изменения Rust

**Текущий приоритет (2026-03-17):**
- Все B-FE и B-BE задачи закрыты (B-FE-11..75, B-BE-01..25, T-BE-01..14, T-FE-01..10)
- Единственная открытая: T-BE-15 (⏸ dead code, низкий приоритет)
- Следующий шаг: E2E тестирование, production hardening, документация

---

### 5. Коммиты — после каждой завершённой задачи

Формат **Conventional Commits**:

```
feat(frontend): add CRUD forms for templates page — closes B-FE-11
fix(frontend): fix inventory create form validation
docs(plan): update MASTER_PLAN — close B-FE-11, B-FE-12
```

Стейджинг конкретными файлами (не `git add -A`):
```bash
git add web/public/templates.html MASTER_PLAN.md
git commit -m "feat(frontend): ..."
```

---

### 6. Пуш на GitHub — после каждого коммита

```bash
git push origin main
```

Пушить сразу после коммита — это рабочий процесс проекта.
Не накапливать локальные коммиты без пуша.

---

### 7. Обновление MASTER_PLAN.md — обязательно

После каждой реализованной задачи:

1. Обновить статус в таблице **2.4**: `⬜` → `✅ Закрыт YYYY-MM-DD`
2. Обновить дату: `**Последнее обновление:** YYYY-MM-DD`
3. Коммитить и пушить вместе с кодом или отдельно

---

## Технический стек

| Компонент | Технология |
|---|---|
| Бэкенд | Rust, Axum 0.8, SQLx 0.8, Tokio 1 |
| БД | SQLite (dev/demo) / PostgreSQL (prod) / MySQL |
| Auth | JWT (jsonwebtoken), bcrypt, TOTP, LDAP, OIDC |
| Фронтенд | Vanilla JS (без фреймворков), Roboto font |
| Дизайн | Material Design (как upstream Vuetify), teal `#005057` sidebar |
| CI | `.github/workflows/rust.yml` (build + clippy + test) |

---

## Команды разработки

```bash
# Проверка компиляции
cd rust && cargo check

# Линтер (должен быть 0 warnings)
cd rust && cargo clippy -- -D warnings

# Тесты
cd rust && cargo test

# Запуск локально (SQLite)
cd rust && SEMAPHORE_DB_PATH=/tmp/semaphore.db cargo run -- server

# Demo через Docker (SQLite, admin/admin123, порт 8088)
docker compose -f docker-compose.demo.yml up --build -d
# Открыть: http://localhost:8088

# Полный стек через Docker (PostgreSQL)
docker compose up -d
```

---

## Структура фронтенда (web/public/)

```
web/public/
├── app.js          # API client, renderSidebar(), утилиты (escapeHtml, formatDate)
├── styles.css      # Material Design CSS (Roboto, teal sidebar, elevation shadows)
├── logo.svg        # Официальный логотип Semaphore (из upstream assets)
├── background.svg  # Фон логин-страницы (из upstream assets/background.svg)
├── login.html      # Страница входа (teal фон + белая карточка, как Auth.vue)
├── index.html      # Dashboard (список проектов, создание)
├── project.html    # Обзор проекта + настройки
├── templates.html  # ✅ CRUD + NewTaskDialog + Views/Tabs
├── inventory.html  # ✅ CRUD (все типы: static/dynamic/terraform/etc)
├── keys.html       # ✅ CRUD (ssh/token/password/none)
├── repositories.html # ✅ CRUD
├── environments.html # ✅ CRUD + secrets tab
├── schedules.html  # ✅ CRUD + visual cron builder
├── task.html       # ✅ Лог задачи с WebSocket live-стримингом
├── template.html   # ✅ Template View — Tasks/Details/Permissions tabs
├── users.html      # ✅ Управление пользователями + TOTP
├── analytics.html  # ✅ Аналитика с Chart.js
├── webhooks.html   # ✅ Webhooks CRUD + матчеры + алиасы
├── playbooks.html  # ✅ Playbooks CRUD + sync/run + история запусков
├── team.html       # ✅ Team members + Invites + Custom Roles
├── history.html    # ✅ История задач проекта
├── activity.html   # ✅ Audit log событий
├── runners.html    # ✅ Runners CRUD
├── apps.html       # ✅ Apps управление
├── global_tasks.html # ✅ Глобальный список задач
└── ... (28+ страниц)
```

---

## Правила

- Читай файл перед редактированием — всегда
- Не создавай новые файлы если можно отредактировать существующий
- Не добавляй комментарии и docstrings к коду который не менял
- Пуш делать после каждого коммита (это норма для этого проекта)
- При merge-конфликтах: HEAD (наш код) имеет приоритет если сомневаешься
- Смотри на Go-оригинал (`velum/velum`) как эталон API и поведения
- Образец CRUD форм на фронтенде — `web/public/users.html`
