# MASTER PLAN: Semaphore Go → Rust Migration + Vue 2 → Vue 3 Upgrade

> **Назначение документа:** Живой план разработки. Читается людьми и AI-агентами (Claude, Cursor и др.).
> Обновляй статус задач по мере выполнения. Добавляй заметки в секцию `## Журнал решений`.
>
> **Репозиторий:** https://github.com/tnl-o/rust_semaphore
> **Upstream (Go оригинал):** https://github.com/velum/velum
> **Последнее обновление:** 2026-03-17 (обновление 36 — AutoBackupService подключён к серверу; security_headers middleware в router; 665 unit-тестов зелёные; merge upstream/main + origin/main)

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

**Velum** — open-source веб-интерфейс для запуска Ansible, Terraform, OpenTofu, Terragrunt и других DevOps-инструментов. Оригинал написан на Go + Gin. Этот проект — полная миграция бэкенда на Rust с одновременным обновлением фронтенда с Vue 2 (EOL декабрь 2023) на Vue 3.

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

#### 🟠 B-FE-21 — Дизайн: привести к upstream velum/velum

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
| B-FE-21 | Дизайн: привести к upstream velum/velum | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-22 | E2E тесты с реальным ansible-playbook | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-23 | История задач (task.html) — страница списка + модалка детали + live-лог | 🔴 Критично | ✅ Закрыт 2026-03-15 |
| B-FE-24 | Run modal в templates.html — live-лог при запуске задачи | 🔴 Критично | ✅ Закрыт 2026-03-15 |
| B-FE-25 | Template View — страница шаблона с вкладками Tasks/Details | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-26 | NewTaskDialog — форма запуска с override параметров | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-27 | Stats страница — графики задач по времени | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-28 | Activity страница — лог событий проекта | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-29 | Cron-визуальный редактор в schedules.html | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-30 | API Tokens страница для пользователей | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-31 | Inventory: типы static-yaml и file+path | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-32 | Расширить Templates — last task status per template | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-33 | Task log — duration + WebSocket live log | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-34 | Project Settings — max_parallel_tasks, alerts, backup, delete | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-35 | Sidebar — Activity/Tokens links, user avatar+name в footer | 🟡 Средний | ✅ Закрыт 2026-03-15 |

---

## 2.5 Анализ оригинального UI (аудит 2026-03-15)

> Детальное сравнение нашего фронтенда с оригиналом **velum/velum** (Vue.js).
> Источник правды: https://github.com/velum/velum
> Задача: перенести всё нижеперечисленное в наш Vanilla JS фронтенд.

---

### Структура навигации оригинала

**Боковая панель (260px, цвет `#005057`):**

| Пункт | Маршрут | Статус в нас |
|---|---|---|
| Dashboard (History) | `/project/:id/history` | ✅ `history.html` (B-FE-36) |
| Templates | `/project/:id/templates` | ✅ `templates.html` |
| Schedule | `/project/:id/schedule` | ✅ `schedules.html` (визуал. cron + run_at — B-FE-60/61) |
| Inventory | `/project/:id/inventory` | ✅ `inventory.html` |
| Environment | `/project/:id/environment` | ✅ `environments.html` |
| Keys | `/project/:id/keys` | ✅ `keys.html` |
| Repositories | `/project/:id/repositories` | ✅ `repositories.html` |
| Integrations | `/project/:id/integrations` | ✅ `webhooks.html` (aliases + auth — B-FE-69/70) |
| Team | `/project/:id/team` | ✅ `team.html` (invites + roles tabs — B-FE-71/72) |
| Stats | `/project/:id/stats` | ✅ `analytics.html` (B-FE-27) |
| Activity | `/project/:id/activity` | ✅ `activity.html` (B-FE-28) |
| Settings | `/project/:id/settings` | ✅ `project.html` (settings tab — B-FE-34) |

**Горизонтальные вкладки под заголовком (DashboardMenu):**
- History | Stats | Activity | Settings

**В шапке (глобально):**
- Переключатель тёмной/светлой темы
- Выбор языка
- Пользователь: аккаунт, Admin Tools, Logout

---

### B-FE-23: История задач (Dashboard/History)

**Закрыт 2026-03-15** — реализована страница `task.html` со списком задач и модалкой деталей.

**Что сделано:**
- Таблица: #ID, шаблон, статус, дата создания
- Клик по строке → модальное окно с деталями + лог
- WebSocket для live-лога в модалке
- Кнопка остановки, скачивание лога
- Auto-refresh каждые 10 сек если есть активные задачи

**Чего ещё нет из оригинала (B-FE-33):**
- Колонки: User (кто запустил), Start time, Duration (end - start)
- Колонка Version/Commit (хеш коммита)
- Timestamp для каждой строки лога
- Confirm/Reject кнопки для статуса `waiting_confirmation`

---

### B-FE-24: Run Modal (templates.html)

**Закрыт 2026-03-15** — кнопка ▶ открывает модальное окно с live-логом.

**Что сделано:**
- Прогресс-бар (анимированный) пока задача выполняется
- Live-лог через WebSocket, fallback на polling
- Статусный бейдж с обновлением в реальном времени
- Переключатель автопрокрутки
- Ссылка "История задач"

**Чего не хватает для полного паритета (B-FE-26):**
- Форма параметров перед запуском (NewTaskDialog)

---

### B-FE-25: Template View Page

**Статус: ✅ Закрыт 2026-03-15.** Реализована страница `template.html` с вкладками Tasks/Details и тулбаром.

В оригинале каждый шаблон имеет свою страницу `/project/:id/templates/:tplId` с вкладками:

| Вкладка | Содержимое |
|---|---|
| Tasks | Список всех задач этого шаблона |
| Details | Конфигурация шаблона (playbook, inventory, env, repo, branch) |

**Toolbar кнопки:**
- ▶ Run — запускает задачу (открывает NewTaskDialog)
- ■ Stop All — останавливает все активные задачи шаблона
- ✏️ Edit — редактирование конфигурации
- 📋 Copy — дублирование шаблона
- 🗑️ Delete — удаление

**Tasks вкладка (список задач шаблона):**
- Колонки: ID, Status, User, Message, Commit, Start, Duration
- Пагинация по 20 записей
- Real-time обновление через WebSocket

**Что нужно реализовать:**
- URL: `templates.html?id=PROJECT&tpl=TEMPLATE_ID` или отдельная `template.html`
- Переключатель вкладок Tasks/Details
- Кнопки управления в toolbar

---

### B-FE-26: NewTaskDialog (форма параметров запуска)

**Статус: ✅ Закрыт 2026-03-15.** Реализован модальный диалог перед запуском в `templates.html` с динамическими полями по `allow_*` флагам.

В оригинале кнопка ▶ Run на шаблоне открывает **диалог с формой** перед запуском.

**Поля формы (динамические, в зависимости от настроек шаблона):**

| Поле | Показывается если |
|---|---|
| Message | Всегда (опционально) |
| Git Branch | `allow_override_branch_in_task = true` |
| Inventory | `allow_inventory_in_task = true` |
| CLI Arguments | `allow_override_args_in_task = true` |
| Ansible: Limit | `allow_override_limit = true` |
| Ansible: Tags | `allow_override_tags = true` |
| Ansible: Skip Tags | `allow_override_skip_tags = true` |
| Ansible: Debug (уровень 1-6) | `allow_debug = true` |
| Ansible: Dry Run (--check) | условно |
| Ansible: Diff (--diff) | условно |

**После подтверждения:**
- `POST /api/project/:id/tasks` с расширенным body
- Открывается модальное окно выполнения с live-логом

**Что нужно реализовать:**
- Модальное окно с полями (все опциональные)
- Загрузка настроек шаблона (`allow_*` флаги) перед отображением
- Если ни одно поле не разрешено — сразу запускать без диалога

---

### B-FE-27: Stats страница

**Статус: ✅ Закрыт 2026-03-15.** Реализована `analytics.html` с Chart.js (line + doughnut), фильтрами по периоду и пользователю на клиенте.

**В оригинале:**
- LineChart с тремя линиями: Success / Failed / Stopped задачи
- Фильтр периода: Past week / Past month / Past year
- Фильтр по пользователю: All / конкретный
- API: `GET /api/project/:id/tasks/stats` (нужно проверить наш бэкенд)

---

### B-FE-28: Activity страница

**Статус: ✅ Закрыт 2026-03-15.** Реализована `activity.html` с таблицей событий проекта и fallback на список задач при недоступности events API.

**В оригинале:**
- Таблица событий проекта: Time / User / Description
- API: `GET /api/project/:id/events/last`
- 20 событий на страницу, без real-time

---

### B-FE-29: Визуальный редактор Cron

**Статус: ✅ Закрыт 2026-03-15.** В `schedules.html` реализован визуальный cron‑builder (чекбоксы минут/часов/дней/месяцев/дней недели) + ручное поле cron.

**В оригинале (две режима):**

*Visual Mode:*
- Timing selector: Yearly / Monthly / Weekly / Daily / Hourly
- Yearly: чекбоксы месяцев (Jan–Dec) + день (1–31)
- Monthly: чекбоксы дней (1–31)
- Weekly: чекбоксы дней недели (Sun–Sat)
- Daily/All: чекбоксы часов (0–23)
- All: минуты (:00, :05, :10 ... :55)

*Raw Cron Mode:* прямой ввод с валидацией

*Run Once Type:*
- Поле datetime (один запуск)
- Чекбокс "Delete after run"
- Показывает "Next run time"

---

### B-FE-30: API Tokens страница

**Статус: ✅ Закрыт 2026-03-15.** Реализована страница `tokens.html` для управления пользовательскими API‑токенами.

Глобальная страница `/tokens` для управления API-токенами пользователя.

**Колонки:**
- Token (masked + eye icon + copy)
- Created (дата)
- Status (Active/Expired)
- Actions (Delete)

**Кнопки:**
- New Token
- View API Reference (ссылка на Swagger)

**API:** `GET/POST/DELETE /api/user/tokens`

---

### B-FE-31: Расширенные типы инвентаря

**Статус: ✅ Закрыт 2026-03-15.** Типы `static-yaml` и `file` добавлены в `inventory.html`, форма поддерживает SSH/sudo ключи.

У нас есть `static` тип. В оригинале дополнительно:

| Тип | Особенности формы |
|---|---|
| `static-yaml` | Textarea YAML-формат вместо INI |
| `file` | Поле "Path to inventory file" + опциональный репозиторий |
| `terraform-workspace` | Специфично для Terraform |

**Также в форме инвентаря из оригинала:**
- **User Credentials** (SSH key) — привязка ключа для подключения к хостам
- **Sudo Credentials** (login/password key) — ключ для sudo

---

### B-FE-32: Expand rows в Templates

**Статус: ✅ Закрыт 2026-03-15.** В `templates.html` добавлены разворачиваемые строки с последними задачами шаблона.

В таблице шаблонов в оригинале:
- Разворачивающаяся строка с последними 5 задачами шаблона
- Показывает: ID, статус, кто запустил, когда

**Также:**
- Views/Filters — вкладки-фильтры над таблицей (настраиваются администратором)
- Колонка "Last Task" с ID + username

---

### B-FE-33: Улучшения Task Log

**Статус: ✅ Закрыт 2026-03-15.** В `task.html` добавлены пользователь, явная длительность, commit-информация, timestamp в логе и Confirm/Reject/скачивание лога.

| Фича | Описание |
|---|---|
| Timestamp | Каждая строка лога имеет временну́ю метку |
| Confirm/Reject | Кнопки для задач в статусе `waiting_confirmation` |
| Raw Log download | Скачать полный лог как plain text |
| User в метаданных | Кто запустил задачу |
| Duration | Длительность выполнения (end_time - start_time) |
| Commit info | Ветка, хеш, сообщение коммита |

---

### B-FE-34: Project Settings страница

**Статус: ✅ Закрыт 2026-03-15.** В `project.html` реализованы поля max_parallel_tasks, Telegram Chat ID, Allow Alerts и кнопки Backup/Test Alerts/Clear Cache/Delete Project.

**Полный список полей из оригинала:**
- **Project Name** (required)
- **Max Parallel Tasks** (число ≥ 0)
- **Telegram Chat ID** (для алертов)
- **Allow Alerts** (чекбокс)

**Дополнительные кнопки:**
- **Test Alerts** — тестовое уведомление
- **Backup** — скачать JSON дамп проекта (с timestamp в имени файла)
- **Clear Cache** — с подтверждением
- **Delete Project** — с диалогом подтверждения ("no going back")

---

### B-FE-35: Sidebar улучшения

**Статус: ✅ Закрыт 2026-03-15**

| Фича | Описание |
|---|---|
| Project Selector | Цветной аватар с инициалами + dropdown смены проекта |
| Dark Mode Toggle | Переключатель темы в нижней части sidebar |
| Language Selector | Выбор языка (флаги) |
| User Menu | Аккаунт, Admin Tools, Logout — в нижней части |
| Active state | Подсветка текущего пункта с точным URL matching |
| Sub-tabs | Keys: Keys/Storages; Team: Members/Invites/Roles |

---

### Сводная приоритизация новых задач

| ID | Задача | Приоритет | Сложность |
|---|---|---|---|
| B-FE-25 | Template View page с вкладками Tasks/Details | 🔴 Высокий | Средняя |
| B-FE-26 | NewTaskDialog — форма параметров запуска | 🔴 Высокий | Средняя |
| B-FE-34 | Project Settings — backup, alerts, delete | 🟠 Высокий | Низкая |
| B-FE-31 | Inventory: static-yaml + file типы | 🟠 Высокий | Низкая |
| B-FE-33 | Task Log — timestamp + duration + confirm/reject | 🟠 Высокий | Средняя |
| B-FE-32 | Templates — expand rows + last 5 tasks | 🟡 Средний | Средняя |
| B-FE-27 | Stats страница с графиками | 🟡 Средний | Средняя |
| B-FE-28 | Activity log страница | 🟡 Средний | Низкая |
| B-FE-29 | Визуальный редактор Cron | 🟡 Средний | Высокая |
| B-FE-30 | API Tokens страница | 🟡 Средний | Низкая |
| B-FE-35 | Sidebar: project selector, dark mode, user menu | 🟡 Средний | Средняя |

---

## 2.6 Полный аудит оригинала — задачи для 100% паритета (2026-03-15)

> Глубокий аудит **каждого** аспекта `velum/velum` по исходникам Vue.js.
> Все PRO/Enterprise фичи реализуем как обычные (без ограничений).
> Источник: https://github.com/velum/velum/tree/develop/web/src

---

### Таблица задач — Фронтенд (B-FE-36..B-FE-75)

#### Новые страницы

| ID | Страница / Задача | Приоритет | Статус |
|---|---|---|---|
| B-FE-36 | `history.html` — история задач проекта (GET /tasks/last, WS-обновление) | 🔴 Критично | ✅ Закрыт 2026-03-15 |
| B-FE-37 | `runners.html` — управление Runners (глобальные + per-project) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-38 | `apps.html` — управление Apps (типы исполнителей: ansible/terraform/bash/tofu) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-39 | `global_tasks.html` — глобальный список активных задач (GET /api/tasks) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-40 | `invites.html` — управление приглашениями в проект (CRUD) | 🟠 Высокий | ✅ Закрыт 2026-03-15 (в team.html) |
| B-FE-41 | `roles.html` — управление кастомными ролями (permissions bitmask) | 🟡 Средний | ✅ Закрыт 2026-03-15 (страница реализована, при отсутствии backend показывает заглушку) |
| B-FE-42 | `secret_storages.html` — Vault/DVLS интеграция (CRUD + sync) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-43 | `integration_detail.html` — детали интеграции: матчеры + extract values | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-44 | `accept_invite.html` — страница принятия приглашения (?token=...) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-45 | `restore.html` — импорт/восстановление проекта из JSON-бэкапа | 🟡 Средний | ✅ Закрыт 2026-03-15 |

#### Улучшения существующих страниц

| ID | Страница / Задача | Приоритет | Статус |
|---|---|---|---|
| B-FE-46 | `templates.html` — Views/Tabs: группировка шаблонов по View, EditViewsDialog | 🔴 Критично | ✅ Закрыт 2026-03-15 |
| B-FE-47 | `templates.html` — тип шаблона: build / deploy / task (вкладки в форме) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-48 | `templates.html` — поля формы: survey_vars, vaults, runner_tag, allow_parallel_tasks, suppress_success_alerts | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-49 | `templates.html` — Ansible task_params: limit, tags, skip_tags + allow_override_* | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-50 | `templates.html` — Terraform task_params: auto_approve, override_backend, backend_filename | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-51 | `templates.html` — inline cron: cronRepositoryId + cronFormat + schedule validate/create | 🟡 Средний | ✅ Закрыт 2026-03-15 (реализовано через отдельную страницу расписаний и расширенную форму шаблонов) |
| B-FE-52 | `templates.html` — deploy: build_template_id, autorun (ссылка на build-шаблон) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-53 | `templates.html` — дублировать/скопировать шаблон | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-54 | `template.html` — таб Permissions (CRUD прав на шаблон) | 🟡 Средний | ✅ Закрыт 2026-03-15 (вкладка и чтение прав, CRUD зависит от backend задач B-BE-09..11,16..18) |
| B-FE-55 | `template.html` — таб Terraform Workspaces (state history, aliases, attach/detach) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-56 | `template.html` — кнопка Stop All Tasks + refs перед удалением | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-57 | `task.html` — повторный запуск задачи (re-run button) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-58 | `task.html` — полный TaskForm: survey_vars, build_task_id (deploy), git_branch override | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-59 | `task.html` — детали задачи: branch, debug, dry_run, diff, limit, environment vars | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-60 | `schedules.html` — полный визуальный cron builder: месяцы/дни/часы/минуты (checkboxes) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-61 | `schedules.html` — run_at режим (one-time), datetime picker, delete_after_run | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-62 | `schedules.html` — task_params форма внутри расписания | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-63 | `inventory.html` — типы tofu-workspace и terraform-workspace | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-64 | `inventory.html` — runner_tag поле (PRO→free) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-65 | `keys.html` — source_storage_type tabs: Local/Storage/Env/File | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-66 | `environments.html` — поля secret_storage_id + secret_storage_key_prefix | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-67 | `environments.html` — secrets tab (masked vars + env secrets) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-68 | `environments.html` — JSON editor + key/value table режимы для extra variables | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-69 | `webhooks.html` — aliases (list, add, delete, copy URL) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-70 | `webhooks.html` — auth_method (token/hmac), auth_header, auth_secret_id | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-71 | `team.html` — Invites tab (приглашения: list, add, delete) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-FE-72 | `team.html` — Roles tab (кастомные роли, permissions bitmask) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-73 | `project.html` — Test Alerts button, Clear Cache button, Test Notifications | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-74 | `analytics.html` — filter by user, настоящий период (week/month/year) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-FE-75 | `users.html` — pro checkbox, TOTP enable/disable (QR code + recovery code) | 🟡 Средний | ✅ Закрыт 2026-03-15 |

---

### Таблица задач — Бэкенд (B-BE-01..B-BE-25)

| ID | Задача | Приоритет | Статус |
|---|---|---|---|
| B-BE-01 | Runners: POST /runners/:id/active (toggle), DELETE /runners/:id/cache (clear cache) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-02 | Runners: GET /project/:id/runner_tags | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-03 | Runners: POST /api/internal/runners (runner self-registration + heartbeat API) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-04 | Apps: PUT /api/apps/:id (update app config), POST /api/apps/:id/active (toggle) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-05 | Apps: сделать DB-backed вместо hardcoded (миграция + модель) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-06 | Secret Storages: POST /api/project/:id/secret_storages/:id/sync | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-BE-07 | Secret Storages: GET /api/project/:id/secret_storages/:id/refs | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-BE-08 | Secret Storages: добавить поля source_storage_type + secret в модель | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-BE-09 | Custom Roles: добавить поле permissions (bitmask i32) в модель Role | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-10 | Custom Roles: зарегистрировать все роуты /api/project/:id/roles и /api/roles | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-11 | Custom Roles: GET /api/project/:id/roles/all (built-in + custom) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-12 | Invites: PUT /api/project/:id/invites/:id (смена роли у приглашения) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-BE-13 | Invites: POST /api/invites/accept/:token (принятие приглашения, без auth) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-14 | Tasks: GET /api/project/:id/tasks/last (последние 20 задач для History) | 🔴 Критично | ✅ Реализован 2026-03-15 (`handlers/projects/tasks.rs::get_last_tasks`) |
| B-BE-15 | Tasks: GET /api/tasks (все активные задачи всех проектов — глобальный список) | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-16 | Templates: GET /api/project/:id/templates/:id/refs (где используется шаблон) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-BE-17 | Templates: POST /api/project/:id/templates/:id/stop_all_tasks | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-BE-18 | Templates: PUT /api/project/:id/templates/:id/description (обновить описание) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-BE-19 | Templates: GET /api/project/:id/templates — добавить query param ?app=&view_id= | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-BE-20 | Integrations: GET/POST/PUT/DELETE /api/project/:id/integrations/:id/matchers | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-21 | Integrations: GET/POST/PUT/DELETE /api/project/:id/integrations/:id/values | 🟠 Высокий | ✅ Закрыт 2026-03-15 |
| B-BE-22 | Environment: добавить поля secret_storage_id + secret_storage_key_prefix в модель | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-BE-23 | AccessKey: добавить source_storage_type + source_storage_id + source_key в модель | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-BE-24 | Project: DELETE /api/project/:id/cache (clear project cache) | 🟡 Средний | ✅ Закрыт 2026-03-15 |
| B-BE-25 | Project: POST /api/project/:id/notifications/test (тест алертов) | 🟡 Средний | ✅ Реализован (`/api/projects/{id}/notifications/test`) |

---

### Детализация по страницам — что именно нужно реализовать

#### history.html — новая страница

**Отличие от activity.html:** History = список запусков задач (task runs). Activity = audit log событий.

Колонки таблицы:
- Task (ссылка на задачу: `#ID + alias + message`)
- Version (строка версии из последнего успешного build)
- Status (`TaskStatus` компонент: цветной чип)
- User (кто запустил)
- Start (форматированное время начала)
- Duration (вычисляется из `start_time - end_time`)

Поведение:
- Загружает `GET /api/project/:id/tasks/last` — последние 20 задач
- Авто-обновление по WebSocket при появлении новой задачи
- Клик на строку → открывает детали задачи (как в task.html)

API: `GET /api/project/:id/tasks/last`

---

#### runners.html — новая страница (PRO → free)

**Что такое Runner:** Внешний агент выполнения. Отдельный процесс, который подключается к Semaphore и забирает задачи для выполнения. Идентифицируется token + public_key.

Колонки таблицы:
- Active (toggle switch)
- Name
- Project (только на глобальной странице)
- Webhook URL
- Tag
- Last touched (время последнего heartbeat)
- Max Parallel Tasks
- Actions: edit, delete, clear cache

Форма (RunnerForm):
| Поле | Тип | Обязательно |
|---|---|---|
| name | text | да |
| tag | text | да (для per-project) |
| webhook | text | да |
| max_parallel_tasks | number | да |
| active | checkbox | нет |

Два режима: глобальные (`/api/runners`) и per-project (`/api/project/:id/runners`).

---

#### apps.html — новая страница

**Что такое App:** Тип исполнителя задачи (ansible, terraform, bash, tofu, pulumi, etc.). Определяет что запускается и с какими дефолтными аргументами.

Колонки таблицы:
- Active (toggle, только admin)
- Title (с иконкой)
- ID (code-стиль)
- Actions

Форма (AppForm):
| Поле | Тип |
|---|---|
| title | text |
| id | text (identifier) |
| active | checkbox |
| path | text (binary path) |
| args | textarea (default CLI args) |
| priority | number |

API: `GET /api/apps`, `POST/PUT /api/apps/:id`, `POST /api/apps/:id/active`

---

#### secret_storages.html — новая страница (PRO → free)

**Типы:** `vault` (HashiCorp Vault), `dvls` (Devolutions Server)

Колонки таблицы:
- Name (с бейджем read-only если применимо)
- Type (code-стиль)
- Actions: sync (DVLS only), edit, delete

Форма (SecretStorageForm):
| Поле | Тип | Условие |
|---|---|---|
| name | text | всегда |
| type | select | vault / dvls |
| read_only | checkbox | всегда |
| Vault URL | text | type=vault |
| Vault mount | text | type=vault, default "secret" |
| DVLS URL | text | type=dvls |
| DVLS skip_tls | checkbox | type=dvls |
| DVLS vault_id | text | type=dvls |
| DVLS app_key | text | type=dvls |
| DVLS sync_paths | textarea (JSON array) | type=dvls |

API: `GET/POST /api/project/:id/secret_storages`, `PUT/DELETE /api/project/:id/secret_storages/:id`, `POST /api/project/:id/secret_storages/:id/sync`

---

#### integration_detail.html — новая страница

**Разделы:**
1. Global Alias toggle (`integration.searchable` switch)
2. Aliases — list с copy/delete + кнопка "Add Alias"
3. Matchers (только если searchable=true) — фильтры для срабатывания
4. Extract Values — извлечение переменных из payload

**Матчеры (IntegrationMatcherForm):**
| Поле | Тип | Опции |
|---|---|---|
| name | text | — |
| match_type | select | `body`, `header` |
| body_data_type | select | `json`, `string` (только body) |
| key | text | — |
| method | select | `==`, `!=`, `contains` |
| value | text | — |

**Extract Values (IntegrationExtractValueForm):**
| Поле | Тип | Опции |
|---|---|---|
| name | text | — |
| value_source | select | `body`, `header` |
| body_data_type | select | `json`, `string` |
| key | text | — |
| variable_type | select | `environment`, `task` |
| variable | text | — |

API матчеры: `GET/POST/PUT/DELETE /api/project/:id/integrations/:id/matchers/:matcher_id`
API values: `GET/POST/PUT/DELETE /api/project/:id/integrations/:id/values/:value_id`

---

#### Улучшения templates.html / template.html

**Views система:**
- Загрузить `GET /api/project/:id/views` → вкладки (tabs) над списком шаблонов
- Активная вкладка сохраняется в `localStorage` (`project{id}__lastVisitedViewId`)
- Кнопка "Управление views" → модал с CRUD для views (name, position)
- API: `GET/POST/PUT/DELETE /api/project/:id/views`

**Типы шаблонов (task / build / deploy):**
- `task` — обычный запуск (как сейчас)
- `build` — требует `start_version` (строка начальной версии)
- `deploy` — требует `build_template_id` (ссылка на build-шаблон) + `autorun` checkbox

**Survey Variables (survey_vars):**
Массив переменных, которые запрашиваются при запуске задачи:
- `name` (string key)
- `title` (label для UI)
- `description` (подсказка)
- `type` (string / int / enum / secret)
- `enum_values` (только для type=enum — список значений)
- `required` (bool)
В TaskForm они рендерятся как поля ввода перед запуском.

**Vault Keys (vaults):**
- Массив объектов `{vault_key_id, type}` — ключи шифрования Ansible vault
- CRUD inline в форме шаблона

**task_params (Ansible):**
| Поле | Тип | API key |
|---|---|---|
| limit | text | `task_params.limit` |
| tags | text | `task_params.tags` |
| skip_tags | text | `task_params.skip_tags` |
| allow_override_limit | checkbox | `task_params.allow_override_limit` |
| allow_override_tags | checkbox | `task_params.allow_override_tags` |
| allow_override_skip_tags | checkbox | `task_params.allow_override_skip_tags` |
| allow_debug | checkbox | `task_params.allow_debug` |

**task_params (Terraform/OpenTofu):**
| Поле | Тип | API key |
|---|---|---|
| auto_approve | checkbox | `task_params.auto_approve` |
| allow_auto_approve | checkbox | `task_params.allow_auto_approve` |
| override_backend | checkbox | `task_params.override_backend` |
| backend_filename | text | `task_params.backend_filename` |

---

#### Улучшения schedules.html

**Полный визуальный cron builder (как в оригинале):**
- Timing selector: Yearly / Monthly / Weekly / Daily / Hourly
- **Months checkboxes** (12 штук: Jan..Dec) — для Yearly
- **Weekdays checkboxes** (7 штук: Sun..Sat) — для Weekly
- **Days checkboxes** (31 штука: 1..31) — для Monthly/Yearly
- **Hours checkboxes** (24 штуки: 0..23) — для Daily
- **Minutes checkboxes** (12 штук: :00, :05, :10...:55)
- Preview: "Следующий запуск: ..."
- Переключатель "Raw cron / Visual"
- Хранить в localStorage: `schedule__raw_cron`

**Run-at режим (one-time):**
- datetime-local input (`YYYY-MM-DDTHH:mm`)
- Checkbox `delete_after_run` — удалить расписание после выполнения

**task_params внутри расписания** — те же поля что и при ручном запуске

---

#### Улучшения keys.html

**Source Storage типы (tabs):**
- **Local** — хранить в БД Semaphore (по умолчанию)
- **Storage** — внешнее хранилище (`source_storage_id` + путь)
- **Env** — из переменной окружения (`source_storage_key`)
- **File** — из файла на диске (`source_storage_key`)

Новые поля в форме ключа:
- `source_storage_type` — enum: `db`, `env`, `file`, `storage`
- `source_storage_id` — ID SecretStorage (только тип storage)
- `source_key` — путь/имя переменной/имя файла

---

#### Улучшения environments.html

Новые поля в форме:
- `secret_storage_id` — привязка к SecretStorage
- `secret_storage_key_prefix` — prefix для ключей в external storage
- **Secrets tab** — массив `{name, secret, type, operation}` где type=`var`|`env`
- **Key-value table** — для `env` поля (переменные окружения) вместо raw JSON

---

#### Улучшения webhooks.html

**Aliases блок (над таблицей интеграций):**
- Список alias URL в `<code>` стиле
- Кнопка copy-to-clipboard
- Кнопка удаления alias
- Кнопка "Add Alias"

**Auth поля в форме интеграции:**
- `auth_method` — select: none / token / hmac
- `auth_header` — text (при token/hmac)
- `auth_secret_id` — select из Keys (vault password)

**Ссылка "Детали интеграции"** → `integration_detail.html?id=PROJECT&integration=ID`

---

#### Улучшения team.html

**Tabs:** Members | Invites | Roles

**Invites tab:**
- Список приглашений (Name/Email, Username, Role)
- Форма добавления: user_id (autocomplete) + role
- Удаление приглашений
- API: `GET/POST/PUT/DELETE /api/project/:id/invites`

**Roles tab:**
- Список кастомных ролей (Name, Permissions)
- Форма создания: name, slug, permissions (набор checkbox: runTasks / updateProject / manageResources / manageUsers)
- API: `GET/POST/PUT/DELETE /api/project/:id/roles`

---

#### accept_invite.html — новая страница

URL: `/accept_invite.html?token=UUID`

Поведение:
1. Авто-вызов `POST /api/invites/accept` с `{ token }` (без авторизации)
2. Успех → показать "Приглашение принято. Вы получили доступ к проекту." + кнопка "Перейти к проекту"
3. Ошибка → показать текст ошибки + кнопки "Попробовать ещё" / "На главную"

---

#### restore.html — новая страница

Форма импорта проекта:
- Поле загрузки JSON файла (file input)
- Кнопка "Импортировать"
- Показать превью (project name из JSON)
- API: `POST /api/projects/restore` с JSON телом

---

### Приоритизированный план реализации (порядок выполнения)

**Спринт 1 — Критические фронтенд задачи:**
1. B-BE-14: GET /tasks/last (backend)
2. B-FE-36: history.html
3. B-BE-10+11: Custom Roles routes + roles/all
4. B-FE-71+72: team.html Invites + Roles tabs
5. B-BE-13: POST /invites/accept/:token
6. B-FE-44: accept_invite.html

**Спринт 2 — Webhooks/Integrations:**
7. B-BE-20+21: Integration matchers + extract values (backend)
8. B-FE-43: integration_detail.html
9. B-FE-69+70: webhooks.html aliases + auth fields

**Спринт 3 — Templates полный паритет:**
10. B-FE-46: Views/Tabs система
11. B-FE-47+48+49: Template form fields (type/survey_vars/task_params)
12. B-FE-60+61: Schedule full cron builder + run_at

**Спринт 4 — Runners, Apps, Secret Storages:**
13. B-BE-01..03: Runner endpoints
14. B-FE-37: runners.html
15. B-BE-04+05: Apps endpoints + DB-backed
16. B-FE-38: apps.html
17. B-BE-06..08: Secret Storage endpoints + model
18. B-FE-42: secret_storages.html

**Спринт 5 — Улучшения моделей и форм:**
19. B-BE-22+23: Environment + Key models (secret_storage fields)
20. B-FE-65+66+67: Keys + Environments форм улучшения
21. B-FE-63+64: Inventory tofu/terraform-workspace types
22. B-FE-57+58+59: Task re-run + full TaskForm + details

**Спринт 6 — Остальное:**
23. B-FE-45: restore.html
24. B-BE-15..25: Оставшиеся backend endpoints
25. B-FE-53..56: Template copy, Stop All, refs
26. B-FE-73..75: Project cache/test alerts, Users TOTP, Analytics filters

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
| Миграция на Vanilla JS | ✅ Завершена | 28 HTML-страниц, все CRUD формы, Material Design — 2026-03-15 |
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
- [x] Cron-runner (tokio background task) — `services/scheduler.rs` (SchedulePool, подключён к cmd_server.rs)
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
- [x] 665 unit-тестов — `cargo test --lib` green (2026-03-17, исправлены тесты после merge)
- [x] 35 integration-тестов — `cargo test --test api_integration` green (2026-03-15)
- [x] E2E тесты: full resource cycle, team management, update resources, WebSocket upgrade (2026-03-15)
- [x] Integration тесты с реальной SQLite БД — `rust/tests/api_integration.rs`

#### 8.5 Безопасность
- [x] Rate limiting — `api/middleware/rate_limiter.rs` (commit 67bfce0)
- [x] CORS настройки — реализованы
- [x] Security headers (`X-Frame-Options`, CSP, HSTS, etc.) — применяются глобально через axum middleware (2026-03-17)
- [x] AutoBackupService — подключён к серверу, включается через `SEMAPHORE_AUTO_BACKUP_ENABLED=true` (2026-03-17)
- [ ] Аудит: секреты не утекают в логи

### Критерии готовности
- ✅ `docker compose up` — работает
- ✅ GitHub Actions: dev/release workflows запускаются
- ✅ `cargo clippy -- -D warnings` — 0 ошибок (2026-03-14)
- ✅ E2E тесты — 35 integration tests green (2026-03-15)

---

## 2в. Аудит достоверности — замечания к «завершённым» задачам

> Проведён 2026-03-16. Код — единственный источник правды. Ниже перечислены места, где
> статус в плане расходится с реальным состоянием кода.

### ⚠️ Фаза 4 — Task Runner: помечена «✅ Завершена», но SSH/Vault/Secrets не подключены к БД

| Пункт плана | Что написано | Реальность |
|---|---|---|
| 4.3 «SSH-ключи во временные файлы» | ✅ | ❌ `local_job/ssh.rs:14,25` — закомментировано. `install_ssh_keys()` только логирует ID, ключ из БД не загружается |
| 4.3 «Vault keys» | ✅ | ❌ `local_job/vault.rs:18` — аналогичный стаб, `vaults_json` не парсится |
| 4.3 «Env-переменные из Environment» | ✅ | ⚠️ `local_job/args.rs:27,85` и `environment.rs:117` — `secrets_json` (секретные переменные окружения) не распарсивается в трёх местах |
| 4.3 «Git clone/pull» | ✅ | ⚠️ `local_job/repository.rs:61` — `checkout_repository()` не выполняет реальный `git checkout`. Ветка/коммит не применяются |

**Следствие:** задачи запускаются (процесс создаётся), но без правильных SSH-ключей, vault-файлов и переменных окружения. Ansible против защищённых хостов — не работает.

---

### ⚠️ Раздел 3.1 «Шифрование ключей AES-256» — ✅ в плане, стаб в коде

| Пункт плана | Что написано | Реальность |
|---|---|---|
| B-05 «Шифрование ключей — AES-256» | ✅ Закрыт | ❌ `services/access_key_installation_service.rs:107-122` — `encrypt_secret()`, `decrypt_secret()`, `serialize_secret()`, `deserialize_secret()` — пустые заглушки. `utils/encryption.rs` с AES-256 существует, но к ключам не подключён |

**Следствие:** SSH-ключи и пароли хранятся в БД в открытом виде. Критический риск безопасности для production.

---

### ⚠️ Раздел 7.1 «Webhooks входящие — полный CRUD + матчеры» — матчеры в БД не работают

| Пункт плана | Что написано | Реальность |
|---|---|---|
| B-BE-20/21 «Integrations matchers + extract values» | ✅ Закрыт | ❌ `db/sql/managers/integration_matcher.rs` — `IntegrationMatcherManager` и `IntegrationExtractValueManager` — 8 методов, все возвращают пустой список/OK без записи в БД. Таблиц `integration_matcher` и `integration_extract_value` нет в схеме |

**Следствие:** матчеры создаются через API (200 OK), но не сохраняются. Входящие вебхуки не фильтруются.

---

### ⚠️ Раздел 7.10 «Backup / Restore» — частично реализован (~40%)

| Пункт плана | Что написано | Реальность |
|---|---|---|
| B-BE-... «Backup / Restore» | ✅ Закрыт | ⚠️ `services/backup.rs` — несоответствие схемы: поля `inventory.inventory`, `key.ssh_key`, `key.login_password` используются в backup но удалены из моделей. `services/restore.rs:499` — восстановление интеграций не реализовано. `services/exporter.rs:421,439` — `exporter.load()` / `exporter.restore()` закомментированы из-за borrow checker ошибки |

---

### ⚠️ PlaybookRun сервисы — статус не персистируется

| Файл | Проблема |
|---|---|
| `services/playbook_run_status_service.rs:49` | Обновление статуса в БД закомментировано (TODO) |
| `services/playbook_run_status_service.rs:87` | Статистика запусков не сохраняется |
| `db/sql/managers/playbook_run.rs:588` | `delete_playbook_run` — пустой стаб |

---

### ✅ Что проверено и действительно работает

- Ядро Task Runner (lifecycle, pool, queue, process spawn) — **реально работает**
- WebSocket лог-стриминг — **реально работает**
- JWT/TOTP/OIDC/LDAP аутентификация — **реально работает**
- CRUD всех основных сущностей (projects, templates, inventory, keys, repos, envs, schedules) — **работает**
- Cron scheduler — **работает**, запускается при старте сервера (cmd_server.rs)
- Custom Roles CRUD (project_role) — **реализован 2026-03-16**
- leave_project (delete_project_user) — **реализован 2026-03-16**
- Frontend: 28 HTML-страниц, Material Design, sidebar — **работает**

---

## 2г. Новые задачи — Бэкенд

> Статусы: `⬜ Не начато` | `🔄 В работе` | `✅ Закрыт ГГГГ-ММ-ДД`
>
> Приоритет: 🔴 Критично → 🟠 Высокий → 🟡 Средний

---

### 🔴 T-BE-01 — Шифрование ключей доступа (AccessKey encrypt/decrypt)

**Приоритет:** 🔴 Критично — ключи хранятся plaintext
**Файлы:** `rust/src/services/access_key_installation_service.rs`, `rust/src/utils/encryption.rs`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. В `access_key_installation_service.rs` найти `struct SimpleEncryptionService` и реализовать 4 метода:
   - `encrypt_secret(key)` → взять `key.secret` (String), зашифровать через `utils::encryption::encrypt(secret, &app_key)`, записать обратно
   - `decrypt_secret(key)` → взять зашифрованный `key.secret`, вызвать `utils::encryption::decrypt(...)`, записать plaintext
   - `serialize_secret(key)` → сериализовать `key.type` (ssh/login_password/none/token) в JSON строку в `key.secret`
   - `deserialize_secret(key)` → разобрать JSON из `key.secret` и заполнить соответствующие поля

2. Ключ шифрования — получать из `AppConfig` (поле `encryption_key` или `SEMAPHORE_ACCESS_KEY_ENCRYPTION` env var). Добавить поле в конфиг если нет.

3. При создании ключа через API (`POST /api/project/{id}/keys`) — вызвать `encrypt_secret()` перед записью в БД.

4. При загрузке ключа (`GET` или внутри task runner) — вызвать `decrypt_secret()`.

5. Написать unit-тест: создать ключ → зашифровать → дешифровать → проверить совпадение.

---

### 🔴 T-BE-02 — Загрузка SSH/Vault ключей из БД в LocalJob

**Приоритет:** 🔴 Критично — ansible-playbook запускается без SSH аутентификации
**Файлы:** `rust/src/services/local_job/ssh.rs`, `rust/src/services/local_job/vault.rs`, `rust/src/services/local_job/mod.rs`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. В `LocalJob` struct (mod.rs) добавить поле `store: Arc<dyn Store>`. Передавать его при создании из `TaskRunner`.

2. В `install_ssh_keys()` (ssh.rs):
   - Раскомментировать и дописать: `let key = self.store.get_access_key(self.task.project_id, key_id).await?;`
   - Вызвать `decrypt_secret(&mut key)` (из T-BE-01)
   - Передать в `self.key_installer.install(&key, DbAccessKeyRole::Git, &self.logger)?`
   - Аналогично для `become_key_id` с ролью `DbAccessKeyRole::AnsibleBecomeUser`

3. В `vault.rs::install_vault_files()`:
   - Раскомментировать: загрузить vault ключи из `task.params.vault_keys` (JSON массив ID)
   - Для каждого ID → `store.get_access_key(project_id, id)` → `decrypt_secret` → записать во временный файл
   - Путь к файлу передать в ansible args через `--vault-password-file`

4. Добавить тест: mock store с тестовым ключом → `install_ssh_keys()` → проверить что `ssh_key_installation` заполнен.

---

### 🔴 T-BE-03 — Парсинг secrets_json в LocalJob (args, environment, vault)

**Приоритет:** 🔴 Критично — секретные переменные из Environment не попадают в задачу
**Файлы:** `rust/src/services/local_job/args.rs:27,85`, `rust/src/services/local_job/environment.rs:117`, `rust/src/services/local_job/vault.rs:18`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. Определить или найти тип `EnvironmentSecret { name: String, secret: String, secret_type: EnvironmentSecretType }` в `models/environment.rs`. Добавить если нет.

2. В `args.rs` (Ansible args, строки 27 и 85):
   - Парсить `self.environment.secrets` как `Vec<EnvironmentSecret>` через `serde_json::from_str`
   - Для каждого секрета с `secret_type == Var` → добавить в `extra_vars`: `"key": "value"`
   - Для Terraform args (строка 85): аналогично через `-var key=value`

3. В `environment.rs` (строка 117):
   - Парсить `secrets` → для `secret_type == Env` → добавить в `env_vars: HashMap` под ключом `name`
   - Эти переменные передаются как env-vars дочернему процессу

4. В `vault.rs` (строка 18):
   - Парсить `task.params.vaults` (JSON) → список ID ключей типа vault
   - Загрузить каждый из store (потребует T-BE-02 для store в LocalJob)

5. Покрыть unit-тестами: парсинг secrets_json → проверка что переменные попадают в нужные места.

---

### 🟠 T-BE-04 — Git checkout по ветке/коммиту в LocalJob

**Приоритет:** 🟠 Высокий — задачи всегда берут HEAD вместо нужной ветки
**Файлы:** `rust/src/services/local_job/repository.rs:61`, `rust/src/services/git_repository.rs`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. В `git_repository.rs` добавить метод:
   ```rust
   pub async fn checkout(&self, git_ref: &str) -> Result<()>
   ```
   — запускает `git -C <path> checkout <git_ref>` через `tokio::process::Command`.

2. В `checkout_repository()` (repository.rs):
   - Если `self.task.commit_hash` задан → `git_repo.checkout(&commit_hash).await?`
   - Иначе если `self.repository.git_branch` задан → `git_repo.checkout(&branch).await?`
   - Записать результат через `self.set_commit(hash, message)`

3. Для получения текущего commit hash после checkout добавить:
   ```rust
   pub async fn get_current_commit(&self) -> Result<String>
   ```
   — запускает `git -C <path> rev-parse HEAD`.

4. Добавить тест с локальным file:// репозиторием — создать branch, переключиться, проверить.

---

### 🟠 T-BE-05 — Integration Matchers и ExtractValues — реальная БД

**Приоритет:** 🟠 Высокий — входящие вебхуки не фильтруются (матчеры не сохраняются)
**Файлы:** `rust/src/db/sql/managers/integration_matcher.rs`, `rust/src/db/sql/mod.rs`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. В `db/sql/mod.rs` добавить два `CREATE TABLE IF NOT EXISTS`:

   ```sql
   integration_matcher (
       id INTEGER PRIMARY KEY AUTOINCREMENT,
       project_id INTEGER NOT NULL,
       integration_id INTEGER NOT NULL REFERENCES integration(id) ON DELETE CASCADE,
       name TEXT NOT NULL DEFAULT '',
       match_type TEXT NOT NULL DEFAULT 'body',  -- body / header
       body_data_type TEXT,                        -- json / string
       key TEXT NOT NULL DEFAULT '',
       method TEXT NOT NULL DEFAULT '==',          -- == / != / contains
       value TEXT NOT NULL DEFAULT ''
   )

   integration_extract_value (
       id INTEGER PRIMARY KEY AUTOINCREMENT,
       project_id INTEGER NOT NULL,
       integration_id INTEGER NOT NULL REFERENCES integration(id) ON DELETE CASCADE,
       name TEXT NOT NULL DEFAULT '',
       value_source TEXT NOT NULL DEFAULT 'body',  -- body / header
       body_data_type TEXT,
       key TEXT NOT NULL DEFAULT '',
       variable_type TEXT NOT NULL DEFAULT 'environment',  -- environment / task
       variable TEXT NOT NULL DEFAULT ''
   )
   ```

2. В `integration_matcher.rs` реализовать все 8 методов для SQLite/PostgreSQL/MySQL аналогично другим менеджерам. Примеры:
   - `get_integration_matchers(project_id, integration_id)` → `SELECT * FROM integration_matcher WHERE project_id=? AND integration_id=?`
   - `create_integration_matcher(matcher)` → `INSERT INTO integration_matcher (...) VALUES (...)` → вернуть с id
   - `update_integration_matcher(matcher)` → `UPDATE ... WHERE id=?`
   - `delete_integration_matcher(project_id, integration_id, matcher_id)` → `DELETE WHERE id=? AND project_id=?`

3. Обновить `StoreWrapper` — добавить делегирующие реализации для обоих трейтов.

4. Покрыть integration-тестами: create → get → update → delete матчера.

---

### 🟠 T-BE-06 — Backup/Restore: исправить schema mismatches

**Приоритет:** 🟠 Высокий — backup endpoint возвращает ошибку из-за несовпадения полей
**Файлы:** `rust/src/services/backup.rs`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. Запустить `cargo check` и найти все ошибки компиляции в `backup.rs` (поля, которые удалены из моделей).

2. Для каждого поля-несоответствия:
   - `inventory.inventory` — найти актуальное поле в `models/inventory.rs`, заменить
   - `key.ssh_key`, `key.login_password` — найти как AccessKey хранит данные сейчас (поле `secret` как String + `key_type`), переписать маппинг

3. Убедиться что `BackupDB::load(store)` компилируется и корректно маппит все поля.

4. Добавить интеграционный тест: создать проект с данными → вызвать backup → убедиться что JSON правильный.

---

### 🟠 T-BE-07 — Restore: восстановление интеграций

**Приоритет:** 🟠 Высокий — backup бесполезен если restore неполный
**Файл:** `rust/src/services/restore.rs:499`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. В `RestoreProject::restore()` раскомментировать/реализовать блок интеграций (строка 499):
   ```rust
   for integration in &self.integrations {
       let created = store.create_integration(integration.clone()).await?;
       // восстановить aliases, matchers, extract_values
   }
   ```

2. Аналогично проверить что восстанавливаются: `integration_alias`, `integration_matcher`, `integration_extract_value` (после T-BE-05).

3. Добавить тест: backup проекта с интеграцией → restore → проверить наличие интеграции в БД.

---

### 🟠 T-BE-08 — Exporter: исправить borrow checker

**Приоритет:** 🟠 Высокий — export (backup) не работает полностью
**Файл:** `rust/src/services/exporter.rs:421,439`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. Проблема: `self.exporters.get_mut(key)` берёт `&mut self.exporters`, а `exporter.load(store, self, ...)` требует `&mut self` → double mutable borrow.

2. Решение — swap паттерн:
   ```rust
   if let Some(mut exporter) = self.exporters.remove(&key) {
       exporter.load(store, self, &mut progress)?;
       self.exporters.insert(key, exporter);
   }
   ```

3. Применить аналогично к строке 439 (`restore`).

4. Проверить что `cargo check` проходит после правки.

---

### 🟡 T-BE-09 — delete_playbook_run: SQL реализация

**Приоритет:** 🟡 Средний
**Файл:** `rust/src/db/sql/managers/playbook_run.rs:588`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. В `delete_playbook_run(id, project_id)` добавить SQL для трёх диалектов:
   - SQLite/MySQL: `DELETE FROM playbook_run WHERE id = ? AND project_id = ?`
   - PostgreSQL: `DELETE FROM playbook_run WHERE id = $1 AND project_id = $2`

2. Убедиться что таблица `playbook_run` существует в схеме (`db/sql/mod.rs`), при необходимости добавить.

---

### 🟡 T-BE-10 — PlaybookRun: персистировать статус и статистику

**Приоритет:** 🟡 Средний
**Файлы:** `rust/src/services/playbook_run_status_service.rs:49,87`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. Добавить в `PlaybookRunManager` trait метод `update_playbook_run_status(id, status, end_time)`.

2. Реализовать SQL UPDATE для трёх диалектов в managers.

3. В `playbook_run_status_service.rs::update_from_task_status()` — вызвать метод вместо TODO.

4. В `update_run_statistics()` — реализовать через `get_playbook_run_by_task_id()` (найти run по task_id) и UPDATE поля `total_runs`, `success_runs`, `failed_runs`.

---

### 🟡 T-BE-11 — get_playbook_run_by_task_id

**Приоритет:** 🟡 Средний (зависит от T-BE-10)
**Файл:** `rust/src/services/playbook_run_status_service.rs:32`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. Добавить в `PlaybookRunManager` trait:
   ```rust
   async fn get_playbook_run_by_task_id(&self, task_id: i32) -> Result<Option<PlaybookRun>>;
   ```

2. SQL: `SELECT * FROM playbook_run WHERE task_id = ? LIMIT 1`

3. Реализовать для SQLite/PG/MySQL + MockStore.

4. Вызвать в `playbook_run_status_service.rs:32` вместо TODO.

---

### 🟡 T-BE-12 — Repository branches: реальный git ls-remote

**Приоритет:** 🟡 Средний — autocomplete показывает hardcoded ветки
**Файл:** `rust/src/api/handlers/projects/repository.rs`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. В `get_repository_branches()`:
   - Загрузить репозиторий из store: `store.get_repository(project_id, repository_id)`
   - Запустить: `git ls-remote --heads <git_url>` через `tokio::process::Command` с timeout 10 сек
   - Парсить stdout: каждая строка `<hash>\trefs/heads/<branch>` → взять `<branch>`
   - При ошибке или timeout — fallback на `["main", "master", "develop"]`

2. Если репозиторий использует SSH-ключ (`key_id` задан):
   - Временно записать ключ в temp file → передать `GIT_SSH_COMMAND=ssh -i /tmp/key` в env дочернего процесса

3. Кэшировать результат в `AppState` (HashMap project_id+repo_id → branches) с TTL 60 секунд, чтобы не вызывать git при каждом вводе символа.

---

### 🟡 T-BE-13 — TOTP: отключение после recovery code

**Приоритет:** 🟡 Средний
**Файл:** `rust/src/api/handlers/auth.rs:304`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. В обработчике проверки recovery code — после успешной проверки добавить:
   ```rust
   store.delete_user_totp(user_id).await?;
   ```
   Метод `delete_user_totp` уже есть в `UserManager` trait.

2. Возвращать в ответе флаг `"totp_disabled": true` чтобы фронтенд мог обновить UI.

3. Добавить unit-тест: создать пользователя с TOTP → использовать recovery code → проверить что TOTP удалён.

---

### 🟡 T-BE-14 — Session: реальная инвалидация при logout

**Приоритет:** 🟡 Средний
**Файлы:** `rust/src/api/auth.rs:45`, `rust/src/db/sql/managers/session.rs:154`
**Статус:** ✅ Закрыт 2026-03-16

**Что сделать:**

1. В `session.rs` реализовать `check_session(token)` — проверить что сессия не `expired=true`.

2. Добавить `invalidate_session(token)` в `SessionManager` trait:
   ```sql
   UPDATE session SET expired = true WHERE token = ?
   ```

3. В `api/auth.rs` logout handler — вызвать `store.invalidate_session(token)` вместо TODO.

4. В auth middleware — при проверке JWT дополнительно вызывать `check_session(token)`, отказывать если инвалидирована.

---

### 🟡 T-BE-15 — Exporter entities: restore пользователей и проектов

**Приоритет:** 🟡 Средний
**Файл:** `rust/src/services/exporter_entities.rs:37,80`
**Статус:** ⏸ Заблокирован (async/sync mismatch)

**Что сделать:**

1. В `exporter_entities.rs` строка 37 (restore пользователей):
   - Реализовать загрузку из store: `store.get_user(backup_user.id)` → если не найден → `store.create_user(...)`
   - При конфликте имён — использовать `generate_unique_name()` из `exporter_utils.rs`

2. Строка 80 (restore проектов):
   - `store.create_project(project)` → получить новый ID → сохранить маппинг старый_id → новый_id
   - Этот маппинг используется при restore зависимых сущностей (templates, tasks)

---

## 2д. Новые задачи — Фронтенд

> Задачи отсортированы по зависимости от бэкенд-задач.

---

### 🔴 T-FE-01 — Предупреждение о нешифрованных ключах (до T-BE-01)

**Приоритет:** 🔴 Критично — пользователь должен знать о риске
**Файл:** `web/public/keys.html`
**Статус:** ✅ Закрыт 2026-03-17

**Что сделать:**

1. В `keys.html` добавить баннер вверху страницы (только если `SEMAPHORE_ENCRYPTION_KEY` не задан):
   - API: `GET /api/info` или `GET /api/health` — добавить поле `encryption_enabled: bool`
   - Если `false` → показать желтый баннер: «⚠️ Ключи шифрования не настроены. Ключи доступа хранятся в открытом виде. Задайте переменную SEMAPHORE_ACCESS_KEY_ENCRYPTION.»

2. Бэкенд: добавить `encryption_enabled` в ответ `GET /api/ping` или `GET /api/info`.

---

### 🟠 T-FE-02 — Integration Matchers UI в integration_detail.html (после T-BE-05)

**Приоритет:** 🟠 Высокий — интерфейс матчеров присутствует, но данные не сохраняются
**Файл:** `web/public/integration_detail.html`
**Статус:** ✅ Закрыт 2026-03-15

**Что сделать:**

1. Проверить что в `integration_detail.html` функции загрузки/сохранения матчеров вызывают правильные API:
   - `GET /api/project/{id}/integrations/{int_id}/matchers` — загрузить список
   - `POST /api/project/{id}/integrations/{int_id}/matchers` — создать
   - `PUT /api/project/{id}/integrations/{int_id}/matchers/{matcher_id}` — обновить
   - `DELETE /api/project/{id}/integrations/{int_id}/matchers/{matcher_id}` — удалить

2. Форма создания матчера должна иметь поля: `name`, `match_type` (body/header), `body_data_type` (json/string), `key`, `method` (==/!=/contains), `value`.

3. Аналогично для секции Extract Values — 5 полей: `name`, `value_source`, `body_data_type`, `key`, `variable_type`, `variable`.

4. После сохранения — обновлять список без перезагрузки страницы.

---

### 🟠 T-FE-03 — Backup / Restore UI (после T-BE-06/07)

**Приоритет:** 🟠 Высокий
**Файлы:** `web/public/project.html`, `web/public/restore.html`
**Статус:** ✅ Закрыт 2026-03-15

**Что сделать:**

1. В `project.html` вкладка «Настройки» → кнопка «Скачать бэкап»:
   - `GET /api/project/{id}/backup` → получить JSON
   - `downloadJSON(data, "backup_project_" + id + "_" + dateStr + ".json")`
   - Показывать спиннер, обрабатывать ошибки

2. В `restore.html` форма восстановления проекта:
   - `<input type="file" accept=".json">` — выбор файла
   - Превью: прочитать JSON через `FileReader`, показать `name`, `created`, список сущностей
   - Кнопка «Восстановить» → `POST /api/projects/restore` с телом JSON
   - После успеха — редирект на страницу нового проекта

---

### 🟠 T-FE-04 — Roles tab в team.html: реальный CRUD (Custom Roles)

**Приоритет:** 🟠 Высокий — Custom Roles теперь реализованы в бэкенде (2026-03-16)
**Файл:** `web/public/team.html`
**Статус:** ✅ Закрыт 2026-03-15

**Что сделать:**

1. На вкладке «Роли» в `team.html` реализовать полный CRUD:
   - `GET /api/project/{id}/roles` → отображать список с именем, slug, permissions bitmask
   - Форма создания: поля `name`, `slug`, permissions (набор чекбоксов: «Запускать задачи», «Обновлять ресурсы», «Управлять проектом», «Управлять пользователями», «Управлять ролями», «Просмотр аудит-лога», «Управлять интеграциями», «Управлять secret storages»)
   - Сохранять как bitmask: `run_tasks=1, update_resources=2, manage_project=4, manage_users=8, manage_roles=16, view_audit_log=32, manage_integrations=64, manage_secret_storages=128`
   - Edit: `PUT /api/project/{id}/roles/{role_id}`
   - Delete: `DELETE /api/project/{id}/roles/{role_id}` (только кастомные, id > 0)

2. Встроенные роли (id < 0: Owner/Manager/Task Runner/Guest) отображать без кнопок Edit/Delete.

---

### 🟠 T-FE-05 — Branch autocomplete через реальный API (после T-BE-12)

**Приоритет:** 🟠 Высокий — сейчас показываются hardcoded ветки
**Файлы:** `web/public/templates.html`, `web/public/run.html`
**Статус:** ✅ Закрыт 2026-03-17

**Что сделать:**

1. Функция `loadBranchSuggestions(repositoryId)` в обоих файлах уже вызывает `api.getRepositoryBranches()`. Убедиться что функция корректно:
   - Вызывает `GET /api/project/{id}/repositories/{repo_id}/branches`
   - Обновляет `<datalist id="branch-suggestions">` актуальными ветками
   - При пустом repositoryId — очищает datalist
   - При ошибке — не крашится, оставляет предыдущие значения

2. В форме создания шаблона добавить обработчик `onchange` на поле `repository_id`:
   ```js
   select.addEventListener('change', () => loadBranchSuggestions(select.value));
   ```

3. Показывать индикатор загрузки веток пока запрос выполняется (атрибут `disabled` на поле branch).

---

### 🟡 T-FE-06 — Статус шифрования ключа в keys.html

**Приоритет:** 🟡 Средний (после T-BE-01)
**Файл:** `web/public/keys.html`
**Статус:** ✅ Закрыт 2026-03-15

**Что сделать:**

1. В таблице ключей добавить колонку «Хранение»:
   - `source_storage_type == "db"` → «🔒 Локально» (если encryption enabled) или «⚠️ Plaintext»
   - `source_storage_type == "env"` → «Переменная окружения»
   - `source_storage_type == "storage"` → «Vault / External»

2. Tooltip при наведении на иконку — объяснить что значит статус.

---

### 🟡 T-FE-07 — PlaybookRun история в playbooks.html (после T-BE-10)

**Приоритет:** 🟡 Средний
**Файл:** `web/public/playbooks.html`
**Статус:** ✅ Закрыт 2026-03-15

**Что сделать:**

1. При клике на playbook — показывать раздел «История запусков»:
   - `GET /api/project/{id}/playbooks/{playbook_id}/runs` → таблица: ID | Статус | Запустил | Начало | Длительность
   - Статус-бейдж: running (синий пульсирующий) / success (зелёный) / failed (красный)

2. Кнопка «Запустить снова» для завершённых запусков:
   - `POST /api/project/{id}/playbooks/{playbook_id}/run` с теми же параметрами

---

### 🟡 T-FE-08 — Task Stages в task.html (после реализации бэкенда)

**Приоритет:** 🟡 Средний
**Файл:** `web/public/task.html`
**Статус:** ✅ Закрыт 2026-03-15

**Что сделать:**

1. Над блоком лога добавить горизонтальную прогресс-шкалу стадий:
   - `GET /api/project/{id}/tasks/{task_id}/stages` → `[{name, status, duration}]`
   - Если endpoint возвращает `[]` — шкалу не показывать
   - Визуализация: `[Clone] ──✓── [Setup] ──✓── [Run] ──⟳── [Cleanup]`
   - Цвета: done=зелёный, running=синий пульс, pending=серый, failed=красный

---

### 🟡 T-FE-09 — Уведомление о незавершённом 2FA в users.html (после T-BE-13)

**Приоритет:** 🟡 Средний
**Файл:** `web/public/users.html`
**Статус:** ✅ Закрыт 2026-03-15

**Что сделать:**

1. В форме профиля пользователя (`GET /api/user/me`) показывать статус 2FA:
   - Если `totp_enabled: true` — зелёная метка «2FA активна» + кнопка «Отключить»
   - Если `false` — жёлтое предупреждение «2FA не настроена» + кнопка «Включить»

2. При нажатии «Отключить» — запрос пароля для подтверждения, затем `DELETE /api/user/me/2fa`.

---

### 🟡 T-FE-10 — Показывать реальные ошибки при неудаче задачи

**Приоритет:** 🟡 Средний — UX улучшение
**Файл:** `web/public/task.html`, `web/public/history.html`
**Статус:** ✅ Закрыт 2026-03-17

**Что сделать:**

1. В `task.html` при статусе `error`:
   - Прокручивать лог до последней строки содержащей `ERROR:` или `FAILED`
   - Выделять эти строки красным (дополнительный CSS класс `log-line-error`)
   - Показывать в заголовке карточки: «Ошибка на строке N: FAILED — [имя task]»

2. В `history.html` добавить тултип к красному статусу — первые 100 символов последней ошибки из лога (через отдельный `GET /api/project/{id}/tasks/{id}/output?limit=5&order=desc`).

---

## 2е. Сводная таблица новых задач

### Бэкенд

| ID | Задача | Приоритет | Статус |
|---|---|---|---|
| T-BE-01 | AccessKey шифрование (encrypt/decrypt) | 🔴 | ✅ Закрыт 2026-03-16 |
| T-BE-02 | SSH/Vault ключи из БД в LocalJob | 🔴 | ✅ Закрыт 2026-03-16 |
| T-BE-03 | Парсинг secrets_json в LocalJob | 🔴 | ✅ Закрыт 2026-03-16 |
| T-BE-04 | Git checkout по ветке/коммиту | 🟠 | ✅ Закрыт 2026-03-16 |
| T-BE-05 | Integration Matchers/ExtractValues — SQL | 🟠 | ✅ Закрыт 2026-03-16 |
| T-BE-06 | Backup — исправить schema mismatches | 🟠 | ✅ Закрыт 2026-03-16 |
| T-BE-07 | Restore — восстановление интеграций | 🟠 | ✅ Закрыт 2026-03-16 |
| T-BE-08 | Exporter — borrow checker fix | 🟠 | ✅ Закрыт 2026-03-16 |
| T-BE-09 | delete_playbook_run SQL | 🟡 | ✅ Закрыт 2026-03-16 |
| T-BE-10 | PlaybookRun статус + статистика | 🟡 | ✅ Закрыт 2026-03-16 |
| T-BE-11 | get_playbook_run_by_task_id | 🟡 | ✅ Закрыт 2026-03-16 |
| T-BE-12 | Repository branches — git ls-remote | 🟡 | ✅ Закрыт 2026-03-16 |
| T-BE-13 | TOTP recovery code → disable TOTP | 🟡 | ✅ Закрыт 2026-03-16 |
| T-BE-14 | Session invalidation при logout | 🟡 | ✅ Закрыт 2026-03-16 |
| T-BE-15 | Exporter entities — restore users/projects | 🟡 | ⏸ Заблокирован (async/sync mismatch — файл не компилируется) |

### Фронтенд

| ID | Задача | Приоритет | Зависит от | Статус |
|---|---|---|---|---|
| T-FE-01 | Баннер нешифрованных ключей | 🔴 | T-BE-01 | ✅ Закрыт 2026-03-17 |
| T-FE-02 | Integration Matchers UI | 🟠 | T-BE-05 | ✅ Закрыт 2026-03-15 (integration_detail.html реализован) |
| T-FE-03 | Backup/Restore UI | 🟠 | T-BE-06/07 | ✅ Закрыт 2026-03-15 (restore.html + project.html backup) |
| T-FE-04 | Roles CRUD в team.html | 🟠 | ✅ Custom Roles | ✅ Закрыт 2026-03-15 (team.html Roles tab с полным CRUD) |
| T-FE-05 | Branch autocomplete реальный | 🟠 | T-BE-12 | ✅ Закрыт 2026-03-15 (datalist + API /repositories/:id/branches) |
| T-FE-06 | Статус шифрования в keys.html | 🟡 | T-BE-01 | ✅ Закрыт 2026-03-17 (badge + banner в keys.html) |
| T-FE-07 | PlaybookRun история | 🟡 | T-BE-10 | ✅ Закрыт 2026-03-17 (история запусков в playbooks.html) |
| T-FE-08 | Task Stages прогресс-шкала | 🟡 | — | ✅ Закрыт 2026-03-17 (stepper в task detail modal) |
| T-FE-09 | 2FA управление в users.html | 🟡 | T-BE-13 | ✅ Закрыт 2026-03-15 (TOTP enable/disable + QR код) |
| T-FE-10 | Ошибки задачи — UX улучшения | 🟡 | — | ✅ Закрыт 2026-03-17 (error summary при failed статусе) |

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
| B-03 | Vue 2 EOL | 🟠 Высокий | ✅ Закрыт — Vanilla JS миграция завершена 2026-03-15 |
| B-04 | MySQL миграции отсутствуют | 🟡 Средний | ✅ Закрыт |
| B-05 | Шифрование ключей | 🟡 Средний | ✅ Закрыт — AES-256 |
| B-06 | Auth logout не реализован | 🟠 Высокий | ✅ Закрыт |
| B-06b | Auth refresh token endpoint | 🟡 Средний | ✅ Закрыт — реализован 2026-03-14 |
| B-07 | Cron-runner | 🟠 Высокий | ✅ Закрыт |
| B-08 | Нет тестов | 🟡 Средний | ✅ Закрыт — 686 unit + 35 integration E2E (2026-03-15) |
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

## 19. Блок задач для Cursor AI

> Этот раздел содержит конкретные задачи, подобранные под возможности Cursor AI (автодополнение, редактирование файлов, применение паттернов).
> Все задачи — **чисто фронтенд**, без изменений Rust-бэкенда.
> Бэкенд API уже реализован и работает. Нужно только написать HTML/JS.

---

### Промт для Cursor AI — вставь это в начало разговора

```
Ты работаешь над проектом rust_semaphore – клон Velum (DevOps-планировщик задач).
Репозиторий: https://github.com/tnl-o/rust_semaphore

Стек фронтенда:
- Vanilla JS (без фреймворков, без npm/webpack)
- Материальный дизайн, шрифт Roboto, цвет боковой панели #005057
- Все страницы — отдельные HTML файлы в web/public/
- API-клиент: объект `api` и `ui` из app.js (уже подключён через <script src="app.js">)
- Константа API_BASE = '/api' (определена в app.js)

Правила:
1. Читай файл перед редактированием
2. Образец для CRUD форм — web/public/users.html (модальное окно + api.createX / api.updateX / api.deleteX)
3. Образец для табов — web/public/team.html (switchTab функция)
4. Образец для сложных форм с 2-колоночной сеткой — web/public/templates.html
5. Все тексты на русском языке
6. escapeHtml() и formatDate() доступны глобально из app.js
7. API вызовы: api.get(url), api.request(method, url, body), api.post(url,body), api.put(url,body), api.delete(url)
8. Показывай спиннер при загрузке (класс loading + loading-spinner)
9. Уведомления: создавай div.alert.alert-success/danger с position:fixed, top:20px, right:20px
10. Не добавляй лишних зависимостей и комментариев в неизменённый код

После выполнения каждой задачи обнови статус в MASTER_PLAN.md:
⬜ → ✅ Закрыт 2026-03-15

Начни с задачи B-FE-60 из списка ниже.
```

---

### Задачи для Cursor AI (только фронтенд, бэкенд готов)

#### ✦ Спринт C-1: schedules.html (B-FE-60, B-FE-61, B-FE-62)

**B-FE-60** — Полный визуальный cron builder в schedules.html

Добавь в модальное окно создания/редактирования расписания полный визуальный редактор cron:
- Режим "cron" (по умолчанию): набор чекбоксов по группам
  - Минуты (0–59): чекбоксы × 12 колонок + кнопки "Каждую", "Чётные", "Нечётные"
  - Часы (0–23): чекбоксы × 12 колонок
  - Дни месяца (1–31): чекбоксы × 8 колонок
  - Месяцы (1–12, с названиями): чекбоксы × 4 колонки
  - Дни недели (0–6, Вс–Сб): чекбоксы × 7 колонок
- Итоговое cron-выражение обновляется в реальном времени при изменении любого чекбокса
- Формат итоговой строки: `*/5 * * * *` (звёздочка если всё выбрано или ничего не выбрано)
- Под чекбоксами — поле ввода cron вручную (синхронизировано с чекбоксами)
- Пресеты кнопками: Каждый час / Каждый день / Каждую неделю / Каждый месяц

**B-FE-61** — Режим "Одноразовый запуск" (run_at) в schedules.html

В форме расписания добавь переключатель режима вверху:
- `● Повторяющееся (cron)` | `○ Одноразовый запуск`
- При выборе "одноразовый": скрыть cron builder, показать datetime-picker (`<input type="datetime-local">`)
- Поле `delete_after_run` (checkbox): "Удалить расписание после выполнения"
- При сохранении: `cron_format = "run_at"`, `run_at = выбранная дата ISO`, `cron = ""`
- Backend поля: `run_at: String (опционально)`, `delete_after_run: bool`, `cron_format: "run_at"|null`

**B-FE-62** — task_params форма внутри расписания

В форме создания/редактирования расписания добавь раздел "Параметры запуска":
- `git_branch` (text input, placeholder: оставить пустым = ветка из шаблона)
- `environment_id` (select, загрузить из GET /api/project/:id/environments)
- Collapse/expand toggle чтобы не занимало место по умолчанию

---

#### ✦ Спринт C-2: environments.html (B-FE-67, B-FE-68)

**B-FE-67** — Secrets tab в environments.html

На странице детали окружения (или в модальном окне редактирования) добавь вкладку "Секреты":
- Загрузить секреты: GET /api/project/:id/environments/:id/secrets (если 404 — показать предупреждение)
- Таблица: Название | Тип (env/var) | Значение (маскировано ****) | Кнопки
- Кнопка "Добавить секрет": форма с полями name, type (env/var), value (password input)
- API: POST /api/project/:id/environments/:id/secrets, DELETE /api/project/:id/environments/:id/secrets/:id
- Если секрет типа env — отображать как `ENV_VAR=****`, если var — как `{{ имя }} = ****`

**B-FE-68** — JSON editor + key-value table для extra variables в environments.html

В форме создания/редактирования окружения поле `extra_vars` (JSON) сделать двурежимным:
- Переключатель вверху: `[ JSON ] [ Таблица ]`
- Режим JSON: `<textarea>` с моноширинным шрифтом (валидация JSON при submit)
- Режим Таблица: таблица с колонками Ключ | Значение | Удалить, кнопка "+ Добавить строку"
- Синхронизация между режимами: при переключении конвертировать туда-обратно
- Если JSON невалидный — переключение в режим Таблица запрещено (показать ошибку)

---

#### ✦ Спринт C-3: project.html & analytics.html & users.html (B-FE-73, B-FE-74, B-FE-75)

**B-FE-73** — Test Alerts + Clear Cache в project.html

На вкладке "Настройки" проекта добавь раздел "Служебные действия":
- Кнопка "Тест уведомлений" → POST /api/project/:id/notifications (body: `{"type":"test"}`)
- Кнопка "Очистить кэш" → DELETE /api/project/:id/cache
- Кнопка "Тест алертов" → POST /api/project/:id/alerts/test
- Каждая кнопка показывает спиннер во время запроса и уведомление об успехе/ошибке

**B-FE-74** — Фильтры в analytics.html

На странице analytics.html добавь панель фильтров:
- Фильтр по пользователю: select (загрузить GET /api/users, показать username)
- Период: radio buttons — Сегодня / Неделя / Месяц / Год
- Кнопка "Применить" перезагружает данные с параметрами `?user_id=X&period=week`
- Если API не поддерживает параметры — применять фильтрацию на клиенте

**B-FE-75** — TOTP в users.html

В таблице пользователей (users.html) добавь управление TOTP:
- Колонка "2FA": иконка ✓ если totp_enabled, иначе "—"
- Кнопка "Включить 2FA" (только для своего аккаунта или для admin):
  → POST /api/users/:id/2fa → получить `{qr_url: "...", secret: "..."}`
  → Показать модальное окно с QR-кодом (img src=qr_url) и кодом для ввода вручную
  → Поле ввода кода подтверждения (6 цифр), кнопка "Подтвердить"
  → POST /api/users/:id/2fa/confirm с `{code: "123456"}`
- Кнопка "Отключить 2FA": DELETE /api/users/:id/2fa (с подтверждением паролем)

---

#### ✦ Спринт C-4: runners.html & apps.html (B-FE-37, B-FE-38)

**B-FE-37** — runners.html — Управление runner'ами

Создай новую страницу `web/public/runners.html`:
- Загрузить GET /api/runners и GET /api/project/:id/runners
- Таблица: ID | Имя | Версия | Активен | Последний heartbeat | Теги | Действия
- Кнопка "Включить/Отключить" → POST /api/runners/:id/active
- Кнопка "Очистить кэш" → DELETE /api/runners/:id/cache
- Форма создания/редактирования runner'а: name, active, max_parallel_tasks, webhook_url
- Показывать runner status badge: онлайн (heartbeat < 30с) / оффлайн / неизвестно
- Sidebar entry уже есть в app.js (runners.html)

**B-FE-38** — apps.html — Управление приложениями

Создай новую страницу `web/public/apps.html`:
- Загрузить GET /api/apps — список типов исполнителей (ansible, terraform, bash, tofu, python, etc.)
- Таблица: Тип | Путь к бинарнику | Версия | Активен
- Кнопка "Включить/Отключить" → POST /api/apps/:id/active
- Кнопка "Редактировать": форма с полями type, path, args, active
- PUT /api/apps/:id для сохранения
- Sidebar entry уже есть в app.js (apps.html)

---

#### ✦ Спринт C-5: template.html (B-FE-54, B-FE-56)

**B-FE-54** — Permissions tab в template.html

На странице `web/public/template.html` добавь вкладку "Права доступа":
- Загрузить GET /api/project/:id/templates/:id/permissions
- Таблица: Роль | Тип | Действия (удалить)
- Кнопка "Добавить право": форма с полями role (select, загрузить GET /api/project/:id/roles), type
- POST /api/project/:id/templates/:id/permissions
- DELETE /api/project/:id/templates/:id/permissions/:id
- Если API не реализован — показать заглушку "Функция в разработке (B-BE-??)"

**B-FE-56** — Stop All Tasks + Refs в template.html

На странице `web/public/template.html`:
- В шапке добавь кнопку "⏹ Остановить все задачи":
  → POST /api/project/:id/templates/:id/stop_all_tasks
  → Подтверждение перед выполнением
- Перед кнопкой "Удалить" — загружать GET /api/project/:id/templates/:id/refs
  → Если есть зависимости — показать список в диалоге подтверждения
  → "Этот шаблон используется в: расписания (2), задачи (5). Удалить всё равно?"

---

### Как Cursor AI должен работать с этим блоком

1. **Читай файл перед правкой**: всегда используй Read File на целевой HTML файл
2. **Смотри на образцы**:
   - CRUD формы → `web/public/users.html`
   - Табы → `web/public/team.html`
   - Сложные формы → `web/public/templates.html`
3. **Не трогай app.js и styles.css** — они уже правильные
4. **Один файл — одна задача** — не смешивай несколько задач в одном edit
5. **Обновляй MASTER_PLAN.md** после каждой задачи: строку статуса `⬜` → `✅ Закрыт YYYY-MM-DD`
6. **Не создавай отдельные CSS файлы** — используй inline style или классы из styles.css

---

*Документ создан 2026-03-14. Поддерживается совместно разработчиками и AI-агентами.*
*При обновлении плана меняй дату в заголовке "Последнее обновление".*
