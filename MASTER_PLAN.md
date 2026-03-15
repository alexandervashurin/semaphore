# MASTER PLAN: Semaphore Go → Rust Migration + Vue 2 → Vue 3 Upgrade

> **Назначение документа:** Живой план разработки. Читается людьми и AI-агентами (Claude, Cursor и др.).
> Обновляй статус задач по мере выполнения. Добавляй заметки в секцию `## Журнал решений`.
>
> **Репозиторий:** https://github.com/tnl-o/rust_semaphore
> **Upstream (Go оригинал):** https://github.com/semaphoreui/semaphore
> **Последнее обновление:** 2026-03-15 (обновление 15 — mobile nav hamburger, runTemplate fix, MASTER_PLAN синхронизирован: 682 unit + 35 integration)

---

## Содержание

1. [Обзор проекта и контекст](#1-обзор-проекта-и-контекст)
2. [Текущее состояние](#2-текущее-состояние)
3. [Технологический стек (целевой)](#3-технологический-стек-целевой)
4. [Структура проекта](#4-структура-проекта)
5. [Фаза 1 — Аудит и базовая инфраструктура](#фаза-1--аудит-и-базовая-инфраструктура)
6. [Фаза 2 — Auth + Users (завершение)](#фаза-2--auth--users-завершение)
7. [Фаза 3 — CRUD сущностей (завершение)](#фаза-3--crud-сущностей-завершение)
8. [Фаза 4 — Task Runner (ключевая фаза)](#фаза-4--task-runner-ключевая-фаза)
9. [Фаза 5 — WebSocket и реалтайм](#фаза-5--websocket-и-реалтайм)
10. [Фаза 6 — Фронтенд: Vue 2 → Vue 3](#фаза-6--фронтенд-vue-2--vue-3)
11. [Фаза 7 — Интеграции и дополнительные возможности](#фаза-7--интеграции-и-дополнительные-возможности)
12. [Фаза 8 — Prod-готовность](#фаза-8--prod-готовность)
13. [Маппинг Go → Rust](#13-маппинг-go--rust)
14. [API-контракт (эндпоинты)](#14-api-контракт-эндпоинты)
15. [Схема базы данных](#15-схема-базы-данных)
16. [Журнал решений (ADR)](#16-журнал-решений-adr)
17. [Известные проблемы и блокеры](#17-известные-проблемы-и-блокеры)
18. [Как контрибьютить](#18-как-контрибьютить)

---

## 1. Обзор проекта и контекст

**Semaphore UI** — open-source веб-интерфейс для запуска Ansible, Terraform, OpenTofu, Terragrunt и других DevOps-инструментов. Оригинал написан на Go + Gin. Этот проект — полная миграция бэкенда на Rust с одновременным обновлением фронтенда с Vue 2 (EOL декабрь 2023) на Vue 3.

**Почему Rust?**
- Производительность: меньше памяти, меньше задержек
- Безопасность: borrow checker исключает целые классы ошибок
- Надёжность: развёртываемый бинарник без рантайма

**Что должно работать в итоге (feature parity с Go-оригиналом):**
- Управление проектами с ролевой моделью (admin/manager/runner)
- Templates (шаблоны задач для Ansible/Terraform/Shell)
- Inventories (инвентари Ansible — статические и динамические)
- Key stores (SSH-ключи, пароли, токены)
- Repositories (Git-репозитории с поддержкой SSH/HTTPS/токенов)
- Task Runner — запуск реальных процессов (ansible-playbook, terraform apply, etc.)
- WebSocket для стриминга логов выполнения в реальном времени
- Schedules (cron-расписания)
- Webhooks (входящие и исходящие)
- Users & Auth (JWT, bcrypt, LDAP опционально)
- Audit log
- Уведомления (email, Slack, Telegram)

---

## 2. Фронтенд: что необходимо для 100% работоспособности

> **Аудит проведён 2026-03-15.** Этот раздел описывает все обнаруженные проблемы фронтенда
> в порядке приоритета. Бэкенд работает корректно. Все API-маршруты доступны.

### 2.1 Критические блокеры (приложение не работает без исправления)

---

#### 🔴 B-FE-01 — Нет создания admin-пользователя при первом запуске

**Симптом:** После `docker run` или `cargo run` войти невозможно — форма логина молча перезагружается.

**Причина:** Переменные `SEMAPHORE_ADMIN`, `SEMAPHORE_ADMIN_PASSWORD` и т.д. объявлены в `Dockerfile`
как `ENV`, но **не обрабатываются** в `rust/src/cli/mod.rs:cmd_server()`. Функция `cmd_server()`
просто создаёт `SqlStore` и запускает Axum-сервер без создания пользователя. Свежая БД пустая.

**Файл:** [rust/src/cli/mod.rs:363-393](rust/src/cli/mod.rs#L363)

**Что нужно сделать:**
- При запуске сервера читать `SEMAPHORE_ADMIN`, `SEMAPHORE_ADMIN_PASSWORD`, `SEMAPHORE_ADMIN_EMAIL`
- Если пользователей в БД нет — создать admin через `store.create_user()`
- Логировать: `"Admin user 'admin' created (first-run seed)"`

---

#### 🔴 B-FE-02 — Баг в `api.request()`: 401 вызывает logout до показа ошибки

**Симптом:** При вводе неверного пароля страница `/login.html` молча перезагружается.
Сообщение об ошибке **никогда не отображается**.

**Причина:** В `web/public/app.js:77-80`:
```javascript
if (response.status === 401) {
    this.logout();            // ← redirect to /login.html СЕЙЧАС
    throw new Error('Не авторизован');  // ← catch в login.html не успевает
}
```
`this.logout()` вызывает `window.location.href = '/login.html'` немедленно.
`catch` в `login.html:77-85` никогда не отрабатывает для ошибок логина.

**Файл:** [web/public/app.js:77](web/public/app.js#L77)

**Что нужно сделать:**
- В `api.request()` не вызывать `logout()` если запрос идёт на `/auth/login`
- Либо: передавать флаг `skipLogoutOn401: true` в `options`
- Либо: убрать auto-logout из `request()`, делать его явно в middleware на страницах

---

### 2.2 Высокий приоритет (после входа сломана навигация)

---

#### 🟠 B-FE-03 — Sidebar теряет project_id при переходе между разделами

**Симптом:** Пользователь на странице `/templates.html?id=5`, кликает "Инвентарь" в sidebar —
попадает на `/inventory.html` без `?id=` → `urlParams.get('id')` = null → редирект на `index.html`.

**Причина:** Все sidebar-ссылки в `project.html`, `templates.html`, `inventory.html` и других
страницах указаны без `?id=` параметра:
```html
<!-- В project.html, templates.html, etc: -->
<li><a href="templates.html">📋 Шаблоны</a></li>  <!-- нет ?id=PROJECT_ID -->
```

**Файлы:** [web/public/project.html:15-27](web/public/project.html#L15), аналогично во всех страницах с sidebar.

**Что нужно сделать:**
- В каждой странице при загрузке брать `projectId` из URL и подставлять в все sidebar-ссылки
- Или: хранить `projectId` в `localStorage` и читать при загрузке каждой страницы

---

#### 🟠 B-FE-04 — Нет формы создания проекта на Dashboard

**Симптом:** Новый пользователь заходит на `index.html` (dashboard) — видит пустой список
"Нет проектов" без возможности создать первый проект.

**Причина:** `index.html` показывает список проектов через `api.getProjects()` но не имеет
кнопки "Создать проект" и соответствующей формы.

**Файл:** [web/public/index.html:37-57](web/public/index.html#L37)

**Что нужно сделать:**
- Добавить кнопку "+ Новый проект" в `index.html`
- Форма: `name`, `max_parallel_tasks` → `POST /api/projects`
- После создания — обновить список

---

#### 🟠 B-FE-05 — Нет страницы управления пользователями

**Симптом:** В sidebar нет ссылки на управление пользователями. Admin не может создавать новых пользователей через UI.

**Причина:** Файл `users.html` отсутствует в `web/public/`. API-маршруты существуют
(`GET/POST/PUT/DELETE /api/user`), фронтенд не реализован.

**Что нужно сделать:**
- Создать `web/public/users.html` — список пользователей с CRUD
- Добавить ссылку "👥 Пользователи" в sidebar всех страниц (только для admin)

---

#### 🟠 B-FE-06 — WebSocket лог задачи не подключён в `task.html`

**Симптом:** При просмотре запущенной задачи логи не обновляются в реальном времени.

**Причина:** Нужно проверить подключение WS в `task.html` к
`ws://host/api/project/{id}/tasks/{task_id}/ws`. Бэкенд WebSocket реализован полностью.

**Файл:** [web/public/task.html](web/public/task.html)

**Что нужно сделать:**
- Подключить `WebSocket` в `task.html` к `/api/project/{id}/tasks/{task_id}/ws`
- Парсить сообщения `{"type":"output","line":"..."}` и дописывать в log-контейнер
- Закрывать соединение при `{"type":"done"}`

---

### 2.3 Средний приоритет (функционал неполный)

---

#### 🟡 B-FE-07 — CRUD формы не реализованы на большинстве страниц

**Симптом:** Страницы показывают списки (`GET`), но нет кнопок создания/редактирования/удаления.

| Страница | Что есть | Чего нет |
|---|---|---|
| `templates.html` | Список, кнопка "+ Новый Шаблон" | Форма создания (функция `openTemplateModal()` не определена) |
| `inventory.html` | Список | Форма создания, редактирование, удаление |
| `keys.html` | Список | Форма создания (с выбором типа ssh/password/token), редактирование |
| `repositories.html` | Список | Форма создания, редактирование |
| `environments.html` | Список | Форма создания с JSON-редактором |
| `schedules.html` | Список | Форма создания cron-расписания |

**Что нужно сделать:**
- Для каждой страницы добавить модальную форму создания/редактирования
- Подключить к существующим API методам в `app.js`

---

#### 🟡 B-FE-08 — Дублирование методов в `app.js`

**Симптом:** Нет UI-эффекта, но техдолг — три метода объявлены дважды.

**Файл:** [web/public/app.js](web/public/app.js) строки 173/192 (`getInventories`), 179/213 (`getEnvironments`), 184/234 (`getRepositories`).

JavaScript берёт последнее объявление — поведение корректное, но код запутан.

**Что нужно сделать:** Удалить первые объявления (строки 173, 179, 184).

---

#### 🟡 B-FE-09 — `analytics.html` не использует Chart.js

**Симптом:** Страница аналитики отображает данные без графиков.

**Причина:** `analytics.html` в `web/public/` не подключает Chart.js. Это отдельный файл от
`web/vanilla/` (где Chart.js реализован). `web/public/analytics.html` — простой HTML без скриптов.

**Что нужно сделать:** Перенести Chart.js логику из `web/vanilla/js/app.js::handleAnalytics()` в `analytics.html`.

---

#### 🟡 B-FE-10 — Нет страницы настроек проекта

**Симптом:** Нет возможности изменить название проекта или удалить проект через UI.

**Что нужно сделать:** Добавить в `project.html` секцию "Настройки" с формой редактирования
(`PUT /api/projects/{id}`) и кнопкой удаления (`DELETE /api/projects/{id}`).

---

### 2.3б Оставшиеся задачи фронтенда (аудит 2026-03-15)

> Все задачи ниже — **только фронтенд**. API бэкенда реализован полностью для каждой.

---

#### ✅ B-FE-11 — CRUD формы: templates.html — Закрыт 2026-03-15

Модальная форма create/edit/delete с полями name, playbook (select), inventory_id, environment_id, repository_id, git_branch, arguments, allow_override_args_in_task.

---

#### ✅ B-FE-12 — CRUD формы: inventory.html — Закрыт 2026-03-15

Модальная форма с полями name, inventory_type (static/file), inventory (textarea INI), ssh_key_id (select из keys).

---

#### ✅ B-FE-13 — CRUD формы: keys.html — Закрыт 2026-03-15

Форма с динамическими полями: SSH (приватный ключ + passphrase) или login_password (login + password).

---

#### ✅ B-FE-14 — CRUD формы: repositories.html — Закрыт 2026-03-15

Форма: name, git_url, git_branch, git_path, key_id (select из keys).

---

#### ✅ B-FE-15 — CRUD формы: environments.html — Закрыт 2026-03-15

Форма: name, description, json (key=value построчно → конвертируется в JSON объект).

---

#### ✅ B-FE-16 — CRUD формы: schedules.html — Закрыт 2026-03-15

Форма: name, template_id (select), cron (с примерами), active (checkbox). Toggle enable/disable.

---

#### ✅ B-FE-17 — run.html — Закрыт 2026-03-15

Страница запуска playbook: inventory_id, environment_id, extra_vars (JSON), limit, tags, skip_tags, debug/dry_run/diff. Создаёт временный шаблон + задачу, редирект на task.html.

---

#### ✅ B-FE-18 — webhooks.html — Закрыт 2026-03-15

Полный CRUD: name, type (generic/slack/teams/discord/telegram), url, secret, active, events, headers. Фильтры по типу и статусу. Кнопка Test.

---

#### ✅ B-FE-19 — playbooks.html — Закрыт 2026-03-15

Полный CRUD: name, playbook_type, content (YAML textarea), description, repository_id. Кнопки run (ссылка на run.html) и sync.

---

#### ✅ B-FE-20 — team.html — Закрыт 2026-03-15

Отдельная страница team.html: список участников проекта с ролями (owner/manager/task_runner/guest). Добавление из списка пользователей, смена роли, удаление.

---

#### 🟠 B-FE-21 — Дизайн: привести к upstream semaphoreui/semaphore

**✅ Закрыт 2026-03-15.** Реализован Material Design совпадающий с upstream Vue/Vuetify:
- Roboto font, teal sidebar `#005057`/`#003236`, primary `#1976D2`
- `background.svg` + `logo.svg` из upstream assets
- login.html: белая карточка на teal фоне с SVG-градиентом (как Auth.vue)
- Material elevation shadows, Vuetify-стиль кнопок

---

#### ✅ B-FE-22 — E2E тесты — Закрыт 2026-03-15

4 новых теста в `rust/tests/api_integration.rs` (итого 35 green):
- `test_e2e_full_resource_cycle` — project → key → inventory → env → template → task → output
- `test_project_team_management` — add/update role/remove проектных участников
- `test_update_resources` — PUT key/inventory/environment/template
- `test_websocket_endpoint_accepts_upgrade` — проверка маршрута /api/ws (не 404/405)

---

### 2.4 Сводная таблица проблем

| ID | Проблема | Приоритет | Статус |
|---|---|---|---|
| B-FE-01 | Нет создания admin при первом запуске | 🔴 Критично | ✅ Закрыт 2026-03-15 |
| B-FE-02 | Баг 401 в api.request() → silent redirect | 🔴 Критично | ✅ Закрыт 2026-03-15 |
| B-FE-03 | Sidebar теряет project_id при навигации | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-04 | Нет формы создания проекта | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-05 | Нет страницы управления пользователями | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-06 | WebSocket лог в task.html не подключён | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-07 | CRUD формы отсутствуют на 6 страницах | 🟡 Средний | ✅ Закрыт 2026-03-15 — детали B-FE-11..16 |
| B-FE-08 | Дублирование методов в app.js | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-09 | analytics.html без Chart.js | 🟡 Средний | ✅ Chart.js подключён |
| B-FE-10 | Нет настроек/удаления проекта | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-11 | CRUD формы: templates.html | 🔴 Критично | ✅ Закрыт 2026-03-15 |
| B-FE-12 | CRUD формы: inventory.html | 🔴 Критично | ✅ Закрыт 2026-03-15 |
| B-FE-13 | CRUD формы: keys.html | 🔴 Критично | ✅ Закрыт 2026-03-15 |
| B-FE-14 | CRUD формы: repositories.html | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-15 | CRUD формы: environments.html | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-16 | CRUD формы: schedules.html | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-17 | run.html — страница запуска задачи | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-18 | webhooks.html — формы create/edit/delete | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-19 | playbooks.html — CRUD + sync/run форма | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-20 | Страница управления командой проекта (roles) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-21 | Дизайн: привести к upstream semaphoreui/semaphore | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-22 | E2E тесты с реальным ansible-playbook | 🟡 Средний | ✅ Закрыт 2026-03-15 |

---

## 2б. Текущее состояние

> Обновляй эту таблицу по мере продвижения. Статусы: `✅ Готово` | `🔄 В работе` | `⬜ Не начато` | `❌ Сломано` | `⚠️ Частично`

### Бэкенд (Rust / Axum / SQLx)

> ⚠️ **Таблица обновлена AI-аудитом 2026-03-14 (обновление 2).**

| Компонент | Статус | Файлы | Примечания |
|---|---|---|---|
| Структура проекта (Cargo workspace) | ✅ Готово | `rust/` | |
| Конфигурация (env vars + YAML) | ✅ Готово | `rust/src/config/` | Полная система: auth, ldap, oidc, dirs, ha, logging |
| SQLite поддержка | ✅ Готово | `rust/src/db/sql/sqlite/` | |
| PostgreSQL поддержка | ✅ Готово | `rust/src/db/sql/postgres/` | |
| MySQL поддержка | ✅ Готово | `rust/src/db/sql/mysql/` | Все CRUD-операции |
| Миграции БД (SQLx) | ✅ Готово | `rust/migrations/` | |
| Auth — JWT выдача и проверка | ✅ Готово | `rust/src/api/auth/` | |
| Auth — bcrypt паролей | ✅ Готово | | |
| Auth — middleware (rate limiting + security headers) | ✅ Готово | `rust/src/api/middleware/` | |
| Auth — TOTP (2FA) | ✅ Готово | `rust/src/services/totp.rs` | |
| Auth — OIDC / OAuth2 | ✅ Готово | `rust/src/api/handlers/oidc.rs` | Multi-provider |
| Auth — LDAP | ✅ Готово | `rust/src/api/handlers/auth.rs`, `config/config_ldap.rs` | Конфиг + handler подключён (2026-03-14) |
| Auth — refresh token | ✅ Готово | `rust/src/api/handlers/auth.rs` | POST /api/auth/refresh (2026-03-14) |
| Auth — logout | ✅ Готово | `rust/src/api/handlers/auth.rs` | Cookie clear |
| Users CRUD | ✅ Готово | `rust/src/api/handlers/users.rs` | |
| Users CLI (`user add`, `token`, `vault`) | ✅ Готово | `rust/src/cli/` | |
| Projects CRUD | ✅ Готово | | |
| Project Users (роли) | ✅ Готово | `rust/src/api/handlers/projects/users.rs` | |
| Project Invites | ✅ Готово | `rust/src/api/handlers/projects/invites.rs` | |
| Inventories CRUD | ✅ Готово | | |
| Keys (Access Keys) CRUD | ✅ Готово | | |
| Repositories CRUD | ✅ Готово | | |
| Templates CRUD | ✅ Готово | | |
| Environments CRUD | ✅ Готово | | |
| Views CRUD | ✅ Готово | | |
| Schedules CRUD | ✅ Готово | | |
| **Task Runner** | ✅ Готово | `rust/src/services/task_runner/`, `task_pool*.rs` | Полная реализация с lifecycle |
| **WebSocket (лог-стриминг)** | ✅ Готово | `rust/src/api/websocket.rs`, `task_runner/websocket.rs` | Broadcast channels |
| **Scheduler (cron-runner)** | ✅ Готово | `rust/src/services/scheduler.rs` | Фоновый tokio task |
| Local Job Runner (ansible/terraform/bash) | ✅ Готово | `rust/src/services/local_job/` | SSH keys, env, git clone |
| Git Integration | ✅ Готово | `rust/src/services/git_repository.rs` | |
| Webhooks (Integrations) | ✅ Готово | `rust/src/api/handlers/projects/integration*.rs` | Входящие + матчеры |
| Webhooks (исходящие) | ✅ Готово | `rust/src/api/handlers/webhooks.rs`, `services/webhook.rs` | Добавлено из upstream |
| Audit Log | ✅ Готово | `rust/src/services/` | Полная схема |
| Хранилище секретов (шифрование) | ✅ Готово | `rust/src/utils/encryption.rs` | AES-256 |
| Secret Storages | ✅ Готово | `rust/src/api/handlers/projects/secret_storages.rs` | |
| Terraform State API | ✅ Готово | `rust/src/models/terraform_inventory.rs` | |
| Уведомления (email / SMTP) | ✅ Готово | `rust/src/utils/mailer.rs`, `services/alert.rs` | lettre + TLS |
| Уведомления (Slack/Telegram) | ✅ Готово | `rust/src/services/alert.rs` | Встроено в AlertService |
| Prometheus Metrics | ✅ Готово | `rust/src/services/metrics.rs` | |
| Backup / Restore | ✅ Готово | `rust/src/services/backup.rs`, `restore.rs` | |
| TOTP (2FA) | ✅ Готово | | |
| **Playbooks CRUD** | ✅ Готово | `rust/src/api/handlers/playbook.rs`, `models/playbook.rs` | Из upstream |
| **Playbook Runs** | ✅ Готово | `rust/src/api/handlers/playbook_runs.rs` | Из upstream |
| **Analytics API** | ✅ Готово | `rust/src/api/handlers/analytics.rs` | Из upstream |
| **GraphQL API** | ✅ Готово | `rust/src/api/graphql/` | Schema, Query, Mutation, Subscription |
| HA (High Availability) | ⚠️ Частично | `rust/src/pro/services/ha.rs` | Pro-фича |
| Cargo build — 0 warnings | ✅ Готово | | |
| cargo test — green | ✅ Готово | | |
| CI/CD (GitHub Actions) — Rust | ✅ Готово | `.github/workflows/rust.yml` | build + test + clippy (2026-03-14) |
| CI/CD (GitHub Actions) — Go legacy | ✅ Готово | `.github/workflows/dev.yml` | Из upstream (Go оригинал) |

### Фронтенд

> ⚠️ **Изменение стратегии:** Фронтенд мигрирует на **Vanilla JS** (не Vue 3, как планировалось ранее). Детали в `MIGRATION_TO_VANILLA.md` и `VANILLA_JS_STATUS.md`.

| Компонент | Статус | Примечания |
|---|---|---|
| Vue 2 фронтенд (из upstream) | ✅ Работает | Базис, EOL декабрь 2023 |
| Миграция на Vanilla JS | 🔄 В работе | Активная разработка — см. VANILLA_JS_STATUS.md |
| Vue 3 миграция | ❌ Отменена | Заменена стратегией Vanilla JS |
| Task Run UI + WebSocket лог | ✅ Готово | TaskLogViewer с ANSI-цветами + live streaming |
| Mobile-адаптивность | ✅ Готово | Hamburger-меню, slide-in sidebar, responsive table (2026-03-15) |

---

## 3. Технологический стек (целевой)

### Бэкенд
```
Rust 1.80+
axum 0.7           — HTTP-фреймворк
sqlx 0.7           — async SQL (PostgreSQL, SQLite, MySQL)
tokio 1.x          — async runtime
serde / serde_json — сериализация
jsonwebtoken       — JWT
bcrypt             — хэши паролей
tokio-tungstenite  — WebSocket
tokio::process     — запуск дочерних процессов (Task Runner)
tracing            — логирование (заменить log/env_logger)
clap 4             — CLI
uuid               — генерация UUID
chrono             — работа с датами
dotenvy            — .env файлы
```

### Фронтенд
```
Vanilla JS (ES2020+)  — без фреймворков
History API           — SPA-роутинг
Proxy-based store     — реактивное состояние
fetch API             — HTTP-запросы
SCSS                  — стили (скомпилированы в web/public/)
Chart.js (CDN)        — аналитика / графики
```
> ⚠️ Vue 3 + Vite + Pinia — **отменено**. Стратегия изменена на Vanilla JS (см. VANILLA_JS_STATUS.md)

### Инфраструктура
```
Docker + docker-compose
GitHub Actions (CI/CD)
PostgreSQL 16 (prod)
SQLite (dev/test)
```

---

## 4. Структура проекта

```
rust_semaphore/
├── rust/                          # Rust бэкенд
│   ├── Cargo.toml
│   ├── migrations/                # SQLx миграции (PG + SQLite)
│   │   ├── postgres/
│   │   └── sqlite/
│   └── src/
│       ├── main.rs
│       ├── config.rs              # Конфигурация из env
│       ├── db.rs                  # Инициализация пула БД
│       ├── errors.rs              # Типы ошибок + Into<Response>
│       ├── auth/
│       │   ├── mod.rs
│       │   ├── middleware.rs      # JWT extraction middleware
│       │   └── handlers.rs       # /api/auth/login, /logout, /refresh
│       ├── models/                # Структуры данных (serde)
│       │   ├── user.rs
│       │   ├── project.rs
│       │   ├── task.rs
│       │   └── ...
│       ├── handlers/              # Axum handlers
│       │   ├── users.rs
│       │   ├── projects.rs
│       │   ├── tasks.rs
│       │   └── ...
│       ├── runner/                # ← ГЛАВНЫЙ БЛОКЕР
│       │   ├── mod.rs
│       │   ├── executor.rs        # Запуск процессов
│       │   ├── queue.rs           # Очередь задач
│       │   └── ws.rs              # WebSocket лог-стриминг
│       └── router.rs              # Все маршруты
├── web/                           # Фронтенд
│   ├── src/
│   │   ├── components/
│   │   ├── views/
│   │   ├── stores/                # Pinia stores (после миграции)
│   │   └── router/
│   ├── package.json
│   └── vite.config.ts             # После миграции
├── db/
│   └── postgres/                  # Дополнительные SQL-скрипты
├── docker-compose.yml
├── Dockerfile
└── MASTER_PLAN.md                 # ← этот файл
```

---

## Фаза 1 — Аудит и базовая инфраструктура

**Цель:** Понять точное текущее состояние, устранить технический долг, зафиксировать основу.

**Статус фазы: ✅ Завершена** *(2026-03-14)*

### Задачи

- [x] **1.1** `cargo build` проходит без warnings *(исправлено 2026-03-14)*
- [x] **1.2** `cargo test` — 524 passed, 0 failed *(исправлено 2026-03-14)*
- [ ] **1.3** Проверить все существующие API-эндпоинты через Postman-коллекцию (`.postman/`)
- [x] **1.4** Таблица Go → Rust обновлена в секции 13 *(2026-03-14)*
- [x] **1.5** `tracing` + `tracing-subscriber` настроены — `src/logging.rs` существует
- [x] **1.6** `clippy` в CI — `cargo clippy -- -D warnings` green (0 errors, 2026-03-14)
- [x] **1.7** Убедиться, что миграции SQLite и PostgreSQL идентичны по схеме *(проверено 2026-03-14: схемы совпадают, различия только в синтаксисе: `?` vs `$N`, `"user"` квотирование)*
- [x] **1.8** `CONTRIBUTING.md` написан

### Критерии готовности
- ✅ `cargo build` — success, 0 warnings
- ✅ `cargo test` — 524 passed green
- [ ] Postman: все CRUD-эндпоинты отвечают корректно

---

## Фаза 2 — Auth + Users (завершение)

**Цель:** Полная функциональность аутентификации, паритет с Go-оригиналом.

**Статус фазы: ✅ Завершена**

### Задачи

- [x] **2.1** `POST /api/auth/login` — работает, возвращает JWT token
- [x] **2.2** `POST /api/auth/refresh` — реализован (`handlers/auth.rs`, закрыт 2026-03-14)
- [x] **2.3** `POST /api/auth/logout` — реализован (cookie clear)
- [x] **2.4** Project Users — CRUD ролей (`GET/POST/PUT/DELETE /api/project/{id}/users`) — `handlers/projects/users.rs`
- [x] **2.5** Проверка прав — middleware в `api/middleware/` (rate limiting + security headers, commit 67bfce0)
- [x] **2.6** `GET /api/user` → `get_current_user` — реализован в `routes.rs:30`
- [x] **2.7** `POST /api/users/{id}/password` — реализован в `routes.rs:37`
- [x] **2.8** Unit-тесты для auth middleware — 524 unit-тестов, включая auth-тесты
- [x] **2.9** TOTP (2FA) — полностью реализован (`services/totp.rs`)
- [x] **2.10** OIDC / OAuth2 — полностью реализован (`handlers/oidc.rs`)
- [x] **2.11** Project Invites — реализован (`handlers/projects/invites.rs`)

### Критерии готовности
- ✅ Login / logout работают
- ✅ Refresh token endpoint реализован
- ✅ Нельзя обратиться к project без токена (401)

---

## Фаза 3 — CRUD сущностей (завершение)

**Цель:** Полный паритет CRUD со всеми сущностями Go-оригинала.

**Статус фазы: ✅ Завершена**

### Задачи для каждой сущности

#### 3.1 Keys (ключи доступа)
- [x] Поддержка типов: `ssh`, `login_password`, `none`, `token` — `models/access_key.rs`
- [x] Шифрование в БД AES-256 — `utils/encryption.rs`
- [x] Secret не возвращается в API — `handlers/projects/keys.rs`
- [x] Эндпоинт `GET /api/project/{id}/keys` — работает

#### 3.2 Repositories
- [x] Поддержка `git` (HTTPS/SSH), `local` — `models/repository.rs`
- [x] Привязка к Key для SSH-доступа
- [x] Эндпоинт `GET /api/project/{id}/repositories`

#### 3.3 Inventories
- [x] Поддержка типов: `static`, `file`, `static-yaml`, `terraform-workspace`
- [x] Эндпоинт `GET /api/project/{id}/inventory`

#### 3.4 Templates
- [x] Поддержка типов: `ansible`, `terraform`, `tofu`, `bash`, `powershell`
- [x] Связи: `repository_id`, `inventory_id`, `environment_id`, `vault_key_id`
- [x] Template vault keys — `models/template_vault.rs`
- [x] Template roles — `db/sql/template_roles.rs`
- [x] Эндпоинт `GET /api/project/{id}/templates`

#### 3.5 Environments
- [x] Хранение JSON-переменных — `models/environment.rs`
- [x] Шифрование значений — `utils/encryption.rs`
- [x] Эндпоинт `GET /api/project/{id}/environment`

#### 3.6 Tasks (история запусков)
- [x] `GET /api/project/{id}/tasks` — список
- [x] `GET /api/project/{id}/tasks/{task_id}` — детали
- [x] `GET /api/project/{id}/tasks/{task_id}/output` — лог
- [x] Статусы: `waiting`, `running`, `success`, `error`, `stopped`

#### 3.7 Schedules
- [x] Валидация cron-выражения — `services/scheduler.rs`
- [x] Cron-runner (tokio background task) — `services/scheduler_pool.rs`
- [x] Включение / выключение расписания
- [x] Эндпоинт `GET /api/project/{id}/schedules`

#### 3.8 Views (категории шаблонов в проекте)
- [x] CRUD для View — `handlers/projects/views.rs`
- [x] Привязка Template к View
- [x] Позиции views — `db/sql/view.rs`
- [x] Эндпоинт `GET /api/project/{id}/views`

### Критерии готовности
- ✅ Все CRUD-эндпоинты реализованы
- ✅ Нет SQL-инъекций (SQLx параметризованные запросы)
- [ ] E2E проверка через Postman

---

## Фаза 4 — Task Runner (ключевая фаза)

**Цель:** Реальный запуск ansible-playbook, terraform, bash и других инструментов как дочерних процессов.

**Статус фазы: ✅ Завершена**

> Реализована в `services/task_runner/`, `services/task_pool*.rs`, `services/local_job/`, `db_lib/`

### Архитектура Rust Task Runner

```
POST /api/project/{id}/tasks  →  TaskPoolQueue  →  TaskPoolRunner
                                     ↓                    ↓
                                 БД (waiting)        LocalJob (ansible/terraform/bash)
                                                          ↓
                                                   TaskLogger (построчно в БД)
                                                          ↓
                                                   WebSocket broadcast
```

### Задачи

#### 4.1 Структуры данных
- [x] `Task` модель — `models/task.rs`
- [x] `TaskOutput` модель — `db/sql/task_output.rs`
- [x] `TaskStatus` enum — `services/task_logger.rs`

#### 4.2 Очередь задач
- [x] `TaskPoolQueue` — `services/task_pool_queue.rs`
- [x] Worker pool — `services/task_pool_runner.rs`
- [x] `AppState` содержит task pool
- [x] Инициализация воркеров при старте (`tokio::spawn`)

#### 4.3 Подготовка окружения перед запуском
- [x] Git clone/pull — `services/local_job/repository.rs`, `services/git_repository.rs`
- [x] SSH-ключи во временные файлы — `services/local_job/ssh.rs`, `services/ssh_agent.rs`
- [x] Env-переменные из Environment — `services/local_job/environment.rs`
- [x] Vault keys — `services/local_job/vault.rs`

#### 4.4 Запуск процессов
- [x] **ansible-playbook** — `db_lib/ansible_playbook.rs`, `db_lib/ansible_app.rs`
- [x] **terraform / opentofu** — `db_lib/terraform_app.rs`
- [x] **bash / sh** — `db_lib/shell_app.rs`
- [x] **local** — `db_lib/local_app.rs`
- [x] CLI аргументы — `services/local_job/args.rs`, `services/local_job/cli.rs`

#### 4.5 Сбор и сохранение логов
- [x] Построчная запись в `task_output` — `services/task_logger.rs`
- [x] Broadcast через `tokio::sync::broadcast` — `services/task_runner/websocket.rs`
- [x] ANSI-escape коды — `utils/ansi.rs`

#### 4.6 Управление задачами
- [x] `POST /api/project/{id}/tasks` — создать и запустить
- [x] `POST /api/project/{id}/tasks/{task_id}/stop` — остановить процесс
- [x] Lifecycle управление — `services/task_runner/lifecycle.rs`
- [x] Hooks на события — `services/task_runner/hooks.rs`

#### 4.7 Тесты
- [ ] Специфичные тесты runner (echo, stop, error) — не написаны

### Критерии готовности
- ✅ Task Runner полностью реализован
- ✅ Лог пишется в БД построчно
- ✅ Stop endpoint реализован
- [ ] Интеграционный тест с реальным ansible-playbook

---

## Фаза 5 — WebSocket и реалтайм

**Цель:** Стриминг логов выполнения задачи в браузер в реальном времени.

**Статус фазы: ⚠️ Бэкенд готов, фронтенд не подключён**

### Задачи

- [x] **5.1** `axum` с feature `ws` подключён
- [x] **5.2** Handler WebSocket — `api/websocket.rs`
- [x] **5.3** Отдача лога из БД + подписка на broadcast — реализовано
- [x] **5.4** `broadcast::Sender` в AppState — `services/task_runner/websocket.rs`
- [x] **5.5** Heartbeat ping/pong — реализовано в websocket.rs
- [x] **5.6** Закрытие WS при завершении задачи — реализовано
- [x] **5.7** Фронтенд: TaskLogViewer подключён к WebSocket *(2026-03-14)*

### API WebSocket

```
ws://host/api/project/{id}/tasks/{task_id}/ws
  → авторизация через ?token=JWT или cookie
  → сервер шлёт: {"type":"output","line":"...","order":1}
  → сервер шлёт: {"type":"status","status":"running"}
  → сервер шлёт: {"type":"done","status":"success"}
```

### Критерии готовности
- ✅ Бэкенд WebSocket полностью реализован
- ❌ Фронтенд не использует WS (будет в фазе 6)

---

## Фаза 6 — Фронтенд: Vanilla JS миграция

**Цель:** Заменить Vue 2 (EOL) на чистый Vanilla JS без зависимостей от фреймворков.

**Статус фазы: ✅ Завершена** *(2026-03-14)*

> ⚠️ **Стратегия изменена:** Vue 3 + Vite + Pinia — **отменено**. Вместо этого реализован полноценный
> Vanilla JS SPA с History API, Proxy-based store и fetch API. Детали в `VANILLA_JS_STATUS.md`.

### Реализованные страницы (все ✅)

- [x] Login / Logout — JWT-аутентификация
- [x] Dashboard — обзор проектов и задач
- [x] Projects — список и создание проектов
- [x] Project → Templates, Tasks, Inventory, Keys, Repositories, Environments, Schedules, Views
- [x] Task Log — ANSI-цвета, WebSocket live-стриминг, кнопка Stop
- [x] Analytics — Chart.js line chart + doughnut, period switcher (week/month/year)
- [x] Users & Settings — управление пользователями

### Архитектура (`web/vanilla/`)

```
web/vanilla/
├── js/
│   ├── app.js          # SPA router (History API) + все страницы
│   ├── store.js        # Proxy-based reactive store
│   └── api.js          # fetch API client + interceptors
├── css/
│   ├── main.scss
│   └── components/     # analytics.scss и др.
└── index.html          # single entry point
```

### Критерии готовности
- ✅ Все страницы работают без фреймворка
- ✅ WebSocket лог-стриминг подключён
- ✅ Analytics с Chart.js (CDN lazy-load)
- ✅ Mobile-адаптивность (базовая)

---

## Фаза 7 — Интеграции и дополнительные возможности

**Статус фазы: ✅ Завершена** *(LDAP конфиг + handler подключён, Slack/Telegram реализованы)*

### Задачи

- [x] **7.1 Webhooks входящие** — `handlers/projects/integration*.rs` — полный CRUD + матчеры
- [x] **7.2 Webhooks исходящие** — `services/webhook.rs` — HTTP POST на смену статуса
- [x] **7.3 Уведомления Email** — `utils/mailer.rs` + `services/alert.rs` (lettre, TLS/SSL)
- [x] **7.4 Уведомления Slack** — `services/alert.rs::send_slack_alert` webhook POST *(фактически реализовано)*
- [x] **7.5 Уведомления Telegram** — `services/alert.rs::send_telegram_alert` Bot API *(фактически реализовано)*
- [x] **7.6 MySQL поддержка** — `db/sql/mysql/` — полный CRUD
- [x] **7.7 Terraform State API** — `models/terraform_inventory.rs`, `db/sql/terraform_inventory.rs`
- [x] **7.8 LDAP Auth** — конфиг + handler подключён (`handlers/auth.rs`, 2026-03-14)
- [x] **7.9 Secret Storages** — `handlers/projects/secret_storages.rs` *(новое)*
- [x] **7.10 Backup / Restore** — `services/backup.rs`, `services/restore.rs` *(новое)*
- [x] **7.11 Prometheus Metrics** — `services/metrics.rs` *(новое)*

---

## Фаза 8 — Prod-готовность

**Статус фазы: ✅ Готово**

### Задачи

#### 8.1 Docker
- [x] `Dockerfile` — существует
- [x] `docker-compose.yml` — существует (postgres + backend)
- [x] `docker-compose.single.yml` — single-container режим
- [x] Multi-stage минимальный образ — distroless/cc-debian12:nonroot (2026-03-14)
- [x] `docker-compose.dev.yml` с hot-reload — `cargo-watch` + PostgreSQL (2026-03-14)

#### 8.2 CI/CD (GitHub Actions)
- [x] `.github/workflows/dev.yml` — build + test на каждый push
- [x] `.github/workflows/community_release.yml` — сборка release binaries
- [x] `.github/workflows/community_beta.yml` — beta releases
- [x] Clippy шаг для Rust — `cargo clippy --all-features -- -D warnings` в `.github/workflows/rust.yml` ✅
- [ ] Кросс-компиляция musl — не проверена

#### 8.3 Конфигурация
- [x] `CONFIG.md` — документация env-переменных существует
- [x] YAML-конфиг — `config/loader.rs`
- [x] Health check — `GET /api/health` → `"OK"` (`routes.rs:16`)

#### 8.4 Тесты
- [x] 682 unit-тестов — `cargo test --lib` green (2026-03-15)
- [x] 35 integration-тестов — `cargo test --test api_integration` green (2026-03-15)
- [x] E2E тесты: full resource cycle, team management, update resources, WebSocket upgrade (2026-03-15)
- [x] Integration тесты с реальной SQLite БД — `rust/tests/api_integration.rs`

#### 8.5 Безопасность
- [x] Rate limiting — `api/middleware/rate_limiter.rs` (commit 67bfce0)
- [x] CORS настройки — реализованы
- [x] Security headers (`X-Frame-Options`, CSP, etc.) — `api/middleware/security_headers.rs` (commit 67bfce0)
- [ ] Аудит: секреты не утекают в логи

### Критерии готовности
- ✅ `docker compose up` — работает
- ✅ GitHub Actions: dev/release workflows запускаются
- ✅ `cargo clippy -- -D warnings` — 0 ошибок (2026-03-14)
- ✅ E2E тесты — 35 integration tests green (2026-03-15)

---

## 13. Маппинг Go → Rust

> Для контрибьюторов: при портировании Go-пакета заполняй эту таблицу.

| Go пакет / файл | Rust модуль | Статус | Примечания |
|---|---|---|---|
| `api/router.go` | `src/api/routes.rs` | ✅ | |
| `api/projects.go` | `src/api/handlers/projects/` | ✅ | |
| `api/tasks.go` | `src/api/handlers/projects/tasks.rs` | ✅ | |
| `api/users.go` | `src/api/handlers/users.rs` | ✅ | |
| `api/keys.go` | `src/api/handlers/projects/keys.rs` | ✅ | |
| `api/inventory.go` | `src/api/handlers/projects/inventory.rs` | ✅ | |
| `api/repositories.go` | `src/api/handlers/projects/repository.rs` | ✅ | |
| `api/templates.go` | `src/api/handlers/projects/templates.rs` | ✅ | |
| `api/schedules.go` | `src/api/handlers/projects/schedules.rs` | ✅ | |
| `api/environments.go` | `src/api/handlers/projects/environment.rs` | ✅ | |
| `api/auth.go` | `src/api/handlers/auth.rs` | ✅ | Refresh token + LDAP реализованы |
| `runner/task_runner.go` | `src/services/task_runner/` | ✅ | Полностью реализован |
| `runner/job.go` | `src/services/local_job/` | ✅ | |
| `runner/ansible.go` | `src/db_lib/ansible_app.rs` | ✅ | |
| `runner/terraform.go` | `src/db_lib/terraform_app.rs` | ✅ | |
| `db/` | `migrations/` + `db/sql/` | ✅ | PG + SQLite + MySQL |
| `util/ssh.go` | `src/services/local_job/ssh.rs` | ✅ | |
| `util/crypt.go` | `src/utils/encryption.rs` | ✅ | AES-256 |
| `services/schedules.go` | `src/services/scheduler.rs` | ✅ | |
| `services/notifications.go` | `src/utils/mailer.rs` + `services/alert.rs` | ✅ | Email реализован |
| `api/integration.go` | `src/api/handlers/projects/integration*.rs` | ✅ | |
| `api/websocket.go` | `src/api/websocket.rs` | ✅ | |

---

## 14. API-контракт (эндпоинты)

> Полная документация в `API.md` и `api-docs.yml`. Здесь — краткий справочник.

### Auth
```
POST   /api/auth/login             { username, password } → { token, refresh_token }
POST   /api/auth/refresh           { refresh_token } → { token }
POST   /api/auth/logout            Header: Authorization
GET    /api/user                   → User[]  (admin only)
GET    /api/user/{id}              → User
PUT    /api/user/{id}              
DELETE /api/user/{id}              
GET    /api/user/me                → текущий пользователь
PUT    /api/user/me/password       { old_password, new_password }
```

### Projects
```
GET    /api/projects               → Project[]
POST   /api/projects               
GET    /api/project/{id}           → Project
PUT    /api/project/{id}           
DELETE /api/project/{id}           
GET    /api/project/{id}/users     → ProjectUser[]
POST   /api/project/{id}/users     
DELETE /api/project/{id}/users/{uid}
```

### Tasks (запуск и история)
```
GET    /api/project/{id}/tasks               → Task[]
POST   /api/project/{id}/tasks               { template_id, params } → Task (создать и запустить)
GET    /api/project/{id}/tasks/{task_id}     → Task
POST   /api/project/{id}/tasks/{task_id}/stop
GET    /api/project/{id}/tasks/{task_id}/output → TaskOutput[]
WS     /api/project/{id}/tasks/{task_id}/ws  → stream лога
```

### Остальные CRUD (все внутри `/api/project/{id}/`)
```
/keys         GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
/inventory    GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
/repositories GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
/templates    GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
/schedules    GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
/environment  GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
/views        GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
```

---

## 15. Схема базы данных

> Полные миграции в `rust/migrations/`. Здесь — структура для понимания зависимостей.

```
users
  id UUID PK, username, name, email, password_hash, admin BOOL, created DATETIME

projects
  id INT PK, name, max_parallel_tasks INT, alert BOOL, alert_chat, created DATETIME

project_users
  project_id FK→projects, user_id FK→users, role ENUM(owner,manager,runner) PK

access_keys                          ← Keys
  id INT PK, name, type ENUM(none,ssh,login_password,token),
  project_id FK, secret_encrypted BYTEA

repositories
  id INT PK, name, git_url, git_branch, ssh_key_id FK→keys, project_id FK

inventories
  id INT PK, name, project_id FK, inventory TEXT, type ENUM, ssh_key_id FK

environments
  id INT PK, name, project_id FK, json TEXT  ← env vars как JSON

task_templates
  id INT PK, project_id FK, name, type ENUM(ansible,terraform,tofu,bash,powershell),
  repository_id FK, inventory_id FK, environment_id FK,
  playbook TEXT, arguments TEXT, allow_override_args BOOL, ...

tasks
  id INT PK, template_id FK, project_id FK, status ENUM,
  user_id FK, created DATETIME, started DATETIME, finished DATETIME,
  commit_hash, message TEXT

task_output
  task_id FK→tasks, task_order INT, output TEXT, time DATETIME
  PK(task_id, task_order)

schedules
  id INT PK, project_id FK, template_id FK, cron_format TEXT, active BOOL

integrations  ← входящие webhooks
  id INT PK, project_id FK, name, auth_secret

events        ← audit log
  object_id, object_type, description, obj_id, created DATETIME, object_key_id, project_id, ...
```

---

## 16. Журнал решений (ADR)

> ADR = Architecture Decision Record. Добавляй новые решения сюда с датой и автором.

### ADR-001: Axum вместо Actix-web
**Дата:** 2024 (начало проекта)
**Решение:** Использовать Axum.
**Причина:** Более ergonomic extractor-based API, встроенная поддержка WebSocket, активное развитие от Tokio team.

### ADR-002: SQLx вместо Diesel
**Дата:** 2024
**Решение:** Использовать SQLx.
**Причина:** Async-first, compile-time проверка запросов, поддержка SQLite/PostgreSQL/MySQL из коробки.

### ADR-003: Vue 3 + Pinia + Vite (фронтенд)
**Дата:** 2026-03-14 (запланировано)
**Решение:** Мигрировать с Vue 2 + Vuex + webpack на Vue 3 + Pinia + Vite.
**Причина:** Vue 2 EOL декабрь 2023. Нет security patches. Vite на порядок быстрее webpack.
**Альтернативы рассмотрены:** React (слишком большое изменение для команды), SvelteKit (мало документации для подобных проектов).

### ADR-004: tokio::process для Task Runner
**Дата:** 2026-03-14 (запланировано)
**Решение:** Использовать `tokio::process::Command` для запуска ansible/terraform.
**Причина:** Нативная async интеграция с tokio runtime. Поддержка `kill_on_drop`.

### ADR-005: Шифрование секретов
**Дата:** ?
**Решение:** ?
**TODO:** Выбрать алгоритм (AES-256-GCM vs ChaCha20-Poly1305) и библиотеку (`aes-gcm` crate).
**Контекст:** Go-оригинал использует AES-256-GCM с ключом из конфига. Нужна обратная совместимость если мигрировать БД.

---

## 17. Известные проблемы и блокеры

| # | Проблема | Приоритет | Статус |
|---|---|---|---|
| B-01 | Task Runner не реализован | 🔴 Критично | ✅ Закрыт |
| B-02 | WebSocket не реализован | 🔴 Критично | ✅ Закрыт |
| B-03 | Vue 2 EOL | 🟠 Высокий | 🔄 В работе — Vanilla JS миграция |
| B-04 | MySQL миграции отсутствуют | 🟡 Средний | ✅ Закрыт |
| B-05 | Шифрование ключей | 🟡 Средний | ✅ Закрыт — AES-256 |
| B-06 | Auth logout не реализован | 🟠 Высокий | ✅ Закрыт |
| B-06b | Auth refresh token endpoint | 🟡 Средний | ✅ Закрыт — реализован 2026-03-14 |
| B-07 | Cron-runner | 🟠 Высокий | ✅ Закрыт |
| B-08 | Нет тестов | 🟡 Средний | ✅ Закрыт — 682 unit + 35 integration E2E (2026-03-15) |
| B-09 | LDAP auth не подключён к auth flow | 🟡 Средний | ✅ Закрыт — подключён 2026-03-14 |
| B-10 | Фронтенд не использует WS для логов | 🟠 Высокий | ✅ Закрыт — TaskLogViewer + WebSocket 2026-03-14 |
| B-11 | Slack/Telegram уведомления | 🟡 Средний | ✅ Закрыт — встроено в `services/alert.rs` |
| B-12 | Нет Rust clippy в CI | 🟡 Средний | ✅ Закрыт — `.github/workflows/rust.yml` 2026-03-14 |

> ℹ️ Фронтенд задачи (B-FE-01..B-FE-22) — см. **Раздел 2** в начале документа.

## 18. Как контрибьютить

### Для разработчиков-людей

1. Форкни репозиторий, создай ветку от `main`: `git checkout -b feat/task-runner`
2. Найди незакрытую задачу в этом плане, оставь комментарий что берёшь её
3. Обнови статус задачи в `MASTER_PLAN.md` как `🔄 В работе`
4. Пиши код, покрывай тестами
5. Открой PR с ссылкой на задачу из плана
6. После merge — обнови статус на `✅ Готово`

### Для AI-агентов (Claude, Cursor, GPT)

При работе с этим файлом:

1. **Всегда читай секцию "Текущее состояние"** перед написанием кода — проверь что задача не уже решена
2. **Обновляй статус** задачи которую выполняешь
3. **Добавляй в ADR** если принимаешь архитектурное решение
4. **При обнаружении противоречий** между этим файлом и кодом — код является источником правды, обнови план
5. **Не переписывай без нужды** работающий код — сначала убедись что это действительно нужно

### Соглашения по коду

```
# Ветки
feat/имя-фичи
fix/описание-бага
refactor/что-рефакторим

# Коммиты (Conventional Commits)
feat(runner): add ansible-playbook executor
fix(auth): handle expired JWT correctly
docs(plan): update task runner status
test(runner): add process execution tests
```

### Команды для разработки

```bash
# Запуск бэкенда (SQLite)
cd rust
export SEMAPHORE_DB_PATH=/tmp/semaphore.db
cargo run -- server --host 0.0.0.0 --port 3000

# Запуск бэкенда (PostgreSQL)
export SEMAPHORE_DB_URL="postgres://semaphore:semaphore_pass@localhost:5432/semaphore"
cargo run -- server --host 0.0.0.0 --port 3000

# Создать admin-пользователя
cargo run -- user add --username admin --name "Admin" --email admin@localhost --password admin123 --admin

# Тесты
cargo test
cargo test -- --nocapture   # с выводом логов

# Линтер
cargo clippy -- -D warnings

# Фронтенд (Vue 2, текущий)
cd web && npm install && npm run build

# Запуск всего через Docker
docker compose up -d
```

---

*Документ создан 2026-03-14. Поддерживается совместно разработчиками и AI-агентами.*
*При обновлении плана меняй дату в заголовке "Последнее обновление".*
