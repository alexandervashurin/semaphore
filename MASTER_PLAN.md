# MASTER PLAN: Semaphore Go → Rust Migration + Vue 2 → Vue 3 Upgrade

> **Назначение документа:** Живой план разработки. Читается людьми и AI-агентами (Claude, Cursor и др.).
> Обновляй статус задач по мере выполнения. Добавляй заметки в секцию `## Журнал решений`.
>
> **Репозиторий:** https://github.com/tnl-o/rust_semaphore
> **Upstream (Go оригинал):** https://github.com/semaphoreui/semaphore
> **Последнее обновление:** 2026-03-14 (auto-updated by AI audit)

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

## 2. Текущее состояние

> Обновляй эту таблицу по мере продвижения. Статусы: `✅ Готово` | `🔄 В работе` | `⬜ Не начато` | `❌ Сломано` | `⚠️ Частично`

### Бэкенд (Rust / Axum / SQLx)

> ⚠️ **Таблица обновлена AI-аудитом 2026-03-14.** Многие компоненты реализованы значительно глубже, чем документировалось ранее.

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
| Auth — LDAP | ⚠️ Частично | `rust/src/config/config_ldap.rs` | Конфиг есть, хандлер НЕ подключён |
| Auth — refresh token | ⬜ Не реализован | | JWT single-use, нет endpoint |
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
| Audit Log | ✅ Готово | `rust/src/services/` | Полная схема |
| Хранилище секретов (шифрование) | ✅ Готово | `rust/src/utils/encryption.rs` | AES-256 |
| Secret Storages | ✅ Готово | `rust/src/api/handlers/projects/secret_storages.rs` | |
| Terraform State API | ✅ Готово | `rust/src/models/terraform_inventory.rs` | |
| Уведомления (email / SMTP) | ✅ Готово | `rust/src/utils/mailer.rs`, `services/alert.rs` | lettre + TLS |
| Уведомления (Slack/Telegram) | ⬜ Не начато | | |
| Prometheus Metrics | ✅ Готово | `rust/src/services/metrics.rs` | |
| Backup / Restore | ✅ Готово | `rust/src/services/backup.rs`, `restore.rs` | |
| TOTP (2FA) | ✅ Готово | | |
| HA (High Availability) | ⚠️ Частично | `rust/src/pro/services/ha.rs` | Pro-фича |
| Cargo build — 0 warnings | ✅ Готово | | Исправлено 2026-03-14 |
| cargo test — 524 passed, 0 failed | ✅ Готово | | Исправлено 2026-03-14 |
| CI/CD (GitHub Actions) | ✅ Готово | `.github/workflows/` | dev, release, beta, pro |

### Фронтенд (Vue)

| Компонент | Статус | Примечания |
|---|---|---|
| Vue 2 фронтенд (портирован из upstream) | ✅ Работает | EOL декабрь 2023, требует апгрейда |
| Vue 3 миграция | ⬜ Не начата | **Главная задача** — фаза 6 |
| Vite (замена webpack) | ⬜ Не начато | |
| Pinia (замена Vuex) | ⬜ Не начато | |
| Vue Router 4 | ⬜ Не начато | |
| TypeScript | ⬜ Не начато | |
| Страница логина | ✅ Работает (Vue 2) | |
| Dashboard / Projects | ✅ Работает (Vue 2) | |
| Templates UI | ✅ Работает (Vue 2) | |
| Task Run UI + лог в реальном времени | ⚠️ Частично | Бэкенд WS готов, фронтенд не подключён |
| Mobile-адаптивность | ⚠️ Частично | |

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
Vue 3.4+
Vite 5             — сборщик (замена webpack)
TypeScript 5       — типизация
Pinia              — state management (замена Vuex)
Vue Router 4       — роутинг
Axios / fetch API  — HTTP-запросы
Tailwind CSS 3     — стили (или сохранить текущие)
```

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

**Оценка:** 1–2 дня

### Задачи

- [ ] **1.1** Запустить проект локально (native режим с SQLite), убедиться что `cargo build` проходит без warnings
- [ ] **1.2** Запустить `cargo test` — зафиксировать какие тесты есть и проходят
- [ ] **1.3** Проверить все существующие API-эндпоинты через Postman-коллекцию (`.postman/`)
- [ ] **1.4** Сделать таблицу: каждый Go-пакет из upstream → статус в Rust (используй `UPSTREAM_PORTING_MAP.md`)
- [ ] **1.5** Настроить `tracing` + `tracing-subscriber` (заменить println!/log) — структурированные логи
- [ ] **1.6** Добавить `clippy` в CI, исправить все предупреждения
- [ ] **1.7** Убедиться, что миграции SQLite и PostgreSQL идентичны по схеме
- [ ] **1.8** Написать `CONTRIBUTING.md` с инструкциями по локальному запуску

### Критерии готовности
- `cargo build --release` — success, 0 warnings
- `cargo test` — все тесты green
- Postman: все CRUD-эндпоинты отвечают корректно

---

## Фаза 2 — Auth + Users (завершение)

**Цель:** Полная функциональность аутентификации, паритет с Go-оригиналом.

**Оценка:** 2–3 дня

### Задачи

- [ ] **2.1** Проверить и дополнить `POST /api/auth/login` — возврат `access_token` + `refresh_token`
- [ ] **2.2** Реализовать `POST /api/auth/refresh` — обновление токенов без перелогина
- [ ] **2.3** Реализовать `POST /api/auth/logout` — инвалидация refresh-токена (хранить в БД или Redis)
- [ ] **2.4** Project Users — CRUD ролей в проектах (`GET/POST/PUT/DELETE /api/project/{id}/users`)
- [ ] **2.5** Проверка прав в каждом handler: admin / manager / runner (middleware-уровень)
- [ ] **2.6** `GET /api/user/me` — профиль текущего пользователя
- [ ] **2.7** `PUT /api/user/me/password` — смена пароля
- [ ] **2.8** Написать unit-тесты для auth middleware (проверка невалидного токена, истёкшего, отсутствующего)

### Критерии готовности
- Полный auth flow работает: login → access token → protected route → refresh → logout
- Нельзя обратиться к project другого пользователя с чужим токеном

---

## Фаза 3 — CRUD сущностей (завершение)

**Цель:** Полный паритет CRUD со всеми сущностями Go-оригинала.

**Оценка:** 3–5 дней

### Задачи для каждой сущности

Для каждой из нижеперечисленных сущностей должны работать: `GET /list`, `GET /{id}`, `POST /`, `PUT /{id}`, `DELETE /{id}`. Также нужна валидация входных данных и правильные HTTP-статусы ошибок.

#### 3.1 Keys (ключи доступа)
- [ ] Поддержка типов: `ssh` (приватный ключ), `login_password`, `none`, `token`
- [ ] Шифрование значения ключа в БД (AES-256-GCM или ChaCha20)
- [ ] Никогда не отдавать `secret` в ответе API
- [ ] Эндпоинт: `GET /api/project/{id}/keys`

#### 3.2 Repositories
- [ ] Поддержка типов: `git` (HTTPS/SSH), `local`
- [ ] Валидация URL при создании
- [ ] Привязка к Key для SSH-доступа
- [ ] Эндпоинт: `GET /api/project/{id}/repositories`

#### 3.3 Inventories
- [ ] Поддержка типов: `static` (inline YAML/INI), `file` (путь), `static-yaml`, `terraform-workspace`
- [ ] Проверка формата INI/YAML при сохранении
- [ ] Эндпоинт: `GET /api/project/{id}/inventory`

#### 3.4 Templates
- [ ] Поддержка типов: `ansible`, `terraform`, `tofu`, `bash`, `powershell`
- [ ] Валидация обязательных полей в зависимости от типа
- [ ] Связи: `repository_id`, `inventory_id`, `environment_id`, `vault_key_id`
- [ ] Survey vars (переменные с вопросами при запуске)
- [ ] Эндпоинт: `GET /api/project/{id}/templates`

#### 3.5 Environments
- [ ] Хранение переменных окружения (JSON-объект `{"KEY": "VALUE"}`)
- [ ] Шифрование значений
- [ ] Эндпоинт: `GET /api/project/{id}/environment`

#### 3.6 Tasks (история запусков)
- [ ] `GET /api/project/{id}/tasks` — список с пагинацией
- [ ] `GET /api/project/{id}/tasks/{task_id}` — детали
- [ ] `GET /api/project/{id}/tasks/{task_id}/output` — лог выполнения
- [ ] Статусы: `waiting`, `running`, `success`, `error`, `stopped`

#### 3.7 Schedules
- [ ] Валидация cron-выражения
- [ ] Cron-runner (tokio background task) — запуск по расписанию
- [ ] Включение / выключение расписания
- [ ] `GET /api/project/{id}/schedules`

#### 3.8 Views (категории шаблонов в проекте)
- [ ] CRUD для View
- [ ] Привязка Template к View
- [ ] `GET /api/project/{id}/views`

### Критерии готовности
- Все эндпоинты работают — проверка через Postman
- Невалидные данные возвращают 400 с описанием ошибки, а не 500
- Нет SQL-инъекций (только параметризованные запросы SQLx — это уже гарантировано, но проверить)

---

## Фаза 4 — Task Runner (ключевая фаза)

**Цель:** Реальный запуск ansible-playbook, terraform, bash и других инструментов как дочерних процессов.

**Оценка:** 5–10 дней (самая сложная часть)

> ⚠️ **Это центральная функциональность Semaphore.** Без неё проект — просто CRUD-интерфейс.

### Как это работает в Go-оригинале (контекст)

В Go: `runner/` пакет, `task_runner.go`, запускает `exec.Command(...)` с env-переменными для SSH-ключей, ansible.cfg и т.д. Логи пишутся в БД построчно. Статус задачи обновляется в реальном времени.

### Архитектура Rust Task Runner

```
POST /api/project/{id}/tasks  →  TaskQueue  →  TaskExecutor
                                     ↓               ↓
                                 БД (waiting)    Процесс (ansible-playbook)
                                                     ↓
                                              TaskLog (построчно в БД)
                                                     ↓
                                              WebSocket broadcast
```

### Задачи

#### 4.1 Структуры данных
- [ ] `Task` модель: id, template_id, project_id, status, created_by, started_at, finished_at, message
- [ ] `TaskOutput` модель: task_id, task_order (u32), output (String), time (datetime)
- [ ] `TaskStatus` enum: `Waiting`, `Running`, `Success`, `Error`, `Stopped`

#### 4.2 Очередь задач
- [ ] `TaskQueue` — tokio `mpsc::channel` или `Arc<Mutex<VecDeque<TaskId>>>`
- [ ] Worker pool: N воркеров (конфигурируется, по умолчанию 1 на проект)
- [ ] Глобальный `AppState` содержит `Arc<TaskQueue>`
- [ ] Инициализация воркеров при старте сервера (`tokio::spawn`)

#### 4.3 Подготовка окружения перед запуском
- [ ] Клонировать/обновить репозиторий (`git clone` / `git pull`) через `tokio::process::Command`
- [ ] Написать SSH-ключ во временный файл (mktemp), добавить в `SSH_AUTH_SOCK` или `-i` параметр
- [ ] Сгенерировать `ansible.cfg` / `inventory` файл во временную директорию
- [ ] Собрать env-переменные из Environment сущности

#### 4.4 Запуск процессов
```rust
// Пример для Ansible:
let mut cmd = tokio::process::Command::new("ansible-playbook");
cmd.arg(&playbook_path)
   .arg("-i").arg(&inventory_path)
   .env("ANSIBLE_FORCE_COLOR", "true")
   .env_clear()
   .envs(&task_env)
   .stdout(Stdio::piped())
   .stderr(Stdio::piped())
   .kill_on_drop(true);
```

- [ ] **ansible-playbook** — передача: playbook-файл, inventory, extra-vars, vault-password
- [ ] **terraform** — `init` → `plan` → `apply` (или только apply с -auto-approve)
- [ ] **opentofu** — аналогично terraform
- [ ] **bash / sh** — произвольный скрипт
- [ ] **powershell** — `pwsh -File script.ps1`

#### 4.5 Сбор и сохранение логов
```rust
// Читать stdout и stderr построчно асинхронно
let stdout = cmd.stdout.take().unwrap();
let reader = BufReader::new(stdout).lines();
while let Some(line) = reader.next_line().await? {
    // сохранить в БД + broadcast в WebSocket
    save_output_line(&db, task_id, order, &line).await?;
    ws_broadcaster.send(line).await;
    order += 1;
}
```
- [ ] Сохранение каждой строки в `task_output` таблицу
- [ ] Broadcast через `tokio::sync::broadcast::Sender`
- [ ] ANSI-escape коды: сохранять как есть (фронтенд рендерит цвета)

#### 4.6 Управление задачами
- [ ] `POST /api/project/{id}/tasks` — создать и поставить в очередь
- [ ] `POST /api/project/{id}/tasks/{task_id}/stop` — послать SIGTERM дочернему процессу
- [ ] Корректная обработка SIGTERM/SIGKILL с таймаутом (10 сек на graceful, потом SIGKILL)
- [ ] Обновление статуса в БД: `running` → `success` | `error` | `stopped`
- [ ] Cleanup временных файлов после завершения

#### 4.7 Тесты
- [ ] Unit-тест: запуск `/bin/echo "hello"` и проверка что лог содержит "hello"
- [ ] Unit-тест: отмена задачи
- [ ] Тест: задача с несуществующим playbook → статус `error`

### Критерии готовности
- Запустить `ansible-playbook` на реальном тестовом inventory и получить успешный статус
- Лог виден построчно в БД сразу во время выполнения
- `stop` endpoint реально останавливает процесс

---

## Фаза 5 — WebSocket и реалтайм

**Цель:** Стриминг логов выполнения задачи в браузер в реальном времени.

**Оценка:** 2–3 дня

### Задачи

- [ ] **5.1** Добавить зависимость `axum` с feature `ws`
- [ ] **5.2** Реализовать handler: `GET /api/project/{id}/tasks/{task_id}/ws` — upgrade to WebSocket
- [ ] **5.3** При подключении: отдать существующий лог из БД, затем подписаться на broadcast
- [ ] **5.4** `broadcast::Sender<String>` в `AppState` — одна шина на задачу или глобальная с фильтрацией по task_id
- [ ] **5.5** Heartbeat ping/pong каждые 30 сек (не закрывать idle соединение)
- [ ] **5.6** Корректное закрытие WS при завершении задачи (послать специальный `{"type":"done"}`)
- [ ] **5.7** Фронтенд: подключить WebSocket на странице задачи, рендерить ANSI-цвета (библиотека `ansi_up` или `xterm.js`)

### API WebSocket

```
ws://host/api/project/{id}/tasks/{task_id}/ws
  → авторизация через ?token=JWT или cookie
  → сервер шлёт: {"type":"output","line":"...","order":1}
  → сервер шлёт: {"type":"status","status":"running"}
  → сервер шлёт: {"type":"done","status":"success"}
```

### Критерии готовности
- Открываем страницу задачи, запускаем playbook — лог появляется строчка за строчкой без reload

---

## Фаза 6 — Фронтенд: Vue 2 → Vue 3

**Цель:** Обновить фронтенд до актуального стека, убрать EOL зависимости.

**Оценка:** 7–14 дней

> Vue 2 достиг End-of-Life в декабре 2023. Нет обновлений безопасности.

### Стратегия миграции

Рекомендуется **постепенная миграция компонент за компонентом** через промежуточный слой, а не "большой взрыв" переписывания всего сразу.

**Порядок миграции:**
1. Настроить новый Vite + Vue 3 проект в `web/` рядом со старым
2. Перенести routing, stores, API-клиент
3. Переписывать компоненты по одному, начиная с листовых (без зависимостей)
4. Проверять каждый компонент вручную перед следующим

### 6.1 Подготовка

- [ ] Зафиксировать полный список компонентов Vue 2 (`find web/src -name "*.vue" | sort`)
- [ ] Изучить Vue Migration Guide: https://v3-migration.vuejs.org/
- [ ] Установить `@vue/compat` для режима совместимости (опционально)
- [ ] Настроить Vite: `web/vite.config.ts`
- [ ] Настроить TypeScript: `web/tsconfig.json`

### 6.2 Архитектура нового фронтенда

```
web/src/
├── main.ts                    # createApp() вместо new Vue()
├── App.vue
├── router/
│   └── index.ts               # createRouter() (Vue Router 4)
├── stores/
│   ├── auth.ts                # Pinia store (вместо Vuex)
│   ├── projects.ts
│   ├── tasks.ts
│   └── ...
├── api/
│   └── client.ts              # axios instance с interceptors
├── components/
│   ├── common/
│   └── ...
├── views/
│   ├── LoginView.vue
│   ├── ProjectsView.vue
│   ├── TasksView.vue
│   └── ...
└── types/
    └── api.ts                 # TypeScript типы из API
```

### 6.3 Breaking changes Vue 2 → Vue 3

| Vue 2 | Vue 3 | Действие |
|---|---|---|
| `new Vue()` | `createApp()` | Обновить `main.js` |
| `Vue.use(Router)` | `app.use(router)` | Обновить |
| Vuex | Pinia | Переписать stores |
| `this.$store` | `useStore()` | Во всех компонентах |
| `Vue.set()` | Нет (реактивность автоматическая) | Удалить |
| `v-model` (событие `input`) | `v-model` (событие `update:modelValue`) | Проверить кастомные компоненты |
| `$listeners` | Слитось с `$attrs` | Проверить |
| Filters `{{ value \| filter }}` | Убраны, заменить computed | Найти и заменить |
| `beforeDestroy` | `onBeforeUnmount` | Переименовать |
| `destroyed` | `onUnmounted` | Переименовать |
| `<template slot="...">` | `<template #...>` | Обновить |

### 6.4 Компоненты для переписывания (приоритет)

- [ ] `App.vue` — главный компонент, layout
- [ ] `LoginView.vue` — страница входа
- [ ] `ProjectsView.vue` — список проектов
- [ ] `ProjectView.vue` — страница проекта
- [ ] `TasksView.vue` — список задач, создание задачи
- [ ] `TaskView.vue` — детали задачи + **WebSocket лог** (новый компонент!)
- [ ] `TemplatesView.vue`
- [ ] `InventoryView.vue`
- [ ] `KeysView.vue`
- [ ] `RepositoriesView.vue`
- [ ] `SchedulesView.vue`
- [ ] `UsersView.vue`
- [ ] `SettingsView.vue`

### 6.5 Новые компоненты (которых нет в Vue 2 версии)

- [ ] `TaskLogViewer.vue` — рендеринг ANSI-логов через WebSocket
  - Использовать `xterm.js` или `ansi_up`
  - Автоскролл вниз
  - Кнопка "стоп" с подтверждением
- [ ] `CronEditor.vue` — редактор cron-выражений с превью следующих запусков
- [ ] `SecretInput.vue` — инпут для секретов (маскировка, "показать/скрыть")
- [ ] `StatusBadge.vue` — бейдж статуса задачи с анимацией для `running`

### Критерии готовности фазы 6
- `npm run build` — success, 0 ошибок TypeScript
- Все страницы работают
- Лог задачи стримится в реальном времени через WebSocket
- Lighthouse: accessibility ≥ 90, performance ≥ 80

---

## Фаза 7 — Интеграции и дополнительные возможности

**Оценка:** 3–5 дней

### Задачи

- [ ] **7.1 Webhooks входящие** — `POST /api/project/{id}/integrations/{integration_id}` запускает задачу
- [ ] **7.2 Webhooks исходящие** — HTTP POST при смене статуса задачи (success/fail)
- [ ] **7.3 Уведомления Email** — SMTP конфигурация, шаблон письма при завершении задачи
- [ ] **7.4 Уведомления Slack** — incoming webhook URL в настройках проекта
- [ ] **7.5 Уведомления Telegram** — Bot API токен + chat_id
- [ ] **7.6 MySQL поддержка** — дописать миграции под MySQL 8, протестировать
- [ ] **7.7 Terraform State API** — `GET/POST /api/project/{id}/terraform/state/{serial}` (HTTP backend для tfstate)
- [ ] **7.8 LDAP Auth** — опциональная интеграция с LDAP/AD для SSO

---

## Фаза 8 — Prod-готовность

**Оценка:** 3–4 дня

### Задачи

#### 8.1 Docker
- [ ] Multi-stage `Dockerfile`: build stage (Rust + Node) → final stage (distroless/alpine)
- [ ] `docker-compose.yml` — полный стек: backend + frontend + postgres
- [ ] `docker-compose.dev.yml` — dev режим с hot-reload
- [ ] Образ: цель < 50MB

#### 8.2 CI/CD (GitHub Actions)
- [ ] `.github/workflows/ci.yml` — build + test + clippy на каждый PR
- [ ] `.github/workflows/release.yml` — сборка binaries для Linux/macOS/Windows
- [ ] Кросс-компиляция: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`
- [ ] Docker image push в GitHub Container Registry

#### 8.3 Конфигурация
- [ ] Проверить все env-переменные задокументированы в `CONFIG.md`
- [ ] `--config` флаг для загрузки из YAML-файла (как в оригинале)
- [ ] Health check endpoint: `GET /api/ping` → `{"status":"ok"}`

#### 8.4 Тесты
- [ ] Integration тесты с реальной БД (SQLite in-memory) через `sqlx::test`
- [ ] E2E тесты API через `reqwest` (или сохранить Postman-коллекцию актуальной)
- [ ] Покрытие ≥ 60% критических путей

#### 8.5 Безопасность
- [ ] Rate limiting для `/api/auth/login` (axum-governor или custom middleware)
- [ ] CORS настройки — только разрешённые origins
- [ ] Заголовки безопасности: `X-Frame-Options`, `Content-Security-Policy`, etc.
- [ ] Проверить что секреты не утекают в логи

### Критерии готовности
- `docker compose up` и всё работает без дополнительных действий
- GitHub Actions: все шаги green на main ветке

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
| `api/auth.go` | `src/api/handlers/auth.rs` | ⚠️ | Нет `/api/auth/refresh` endpoint |
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
| B-01 | Task Runner не реализован | 🔴 Критично | ✅ Закрыт — реализован в `services/task_runner/` |
| B-02 | WebSocket не реализован | 🔴 Критично | ✅ Закрыт — реализован в `api/websocket.rs` |
| B-03 | Vue 2 EOL, нет security patches | 🟠 Высокий | ❌ Открыт — главный текущий блокер |
| B-04 | MySQL миграции отсутствуют | 🟡 Средний | ✅ Закрыт — MySQL CRUD реализован |
| B-05 | Шифрование ключей — неясна схема | 🟡 Средний | ✅ Закрыт — AES-256 в `utils/encryption.rs` |
| B-06 | Auth logout не реализован | 🟠 Высокий | ✅ Закрыт — logout через cookie clear |
| B-06b | Auth refresh token endpoint | 🟡 Средний | ❌ Открыт — нет `/api/auth/refresh` |
| B-07 | Cron-runner (фоновый scheduler) | 🟠 Высокий | ✅ Закрыт — реализован в `services/scheduler.rs` |
| B-08 | Нет тестов | 🟡 Средний | ✅ Частично — 524 unit-теста, нет E2E |
| B-09 | LDAP auth не подключён к auth flow | 🟡 Средний | ❌ Открыт — конфиг есть, хандлер нет |
| B-10 | Фронтенд Vue 2 не использует WS для логов | 🟠 Высокий | ❌ Открыт |
| B-11 | Slack/Telegram уведомления | 🟡 Средний | ❌ Открыт |

---

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
