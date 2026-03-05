# Детализированный план портирования Semaphore Go → Rust

**Дата:** 2026-03-05  
**Upstream:** https://github.com/semaphoreui/semaphore (Go)  
**Текущий проект:** Rust-порт в `rust/`

---

## 1. Сравнение архитектуры

### 1.1 Go (upstream)

| Компонент | Путь | Описание |
|-----------|------|----------|
| Точка входа | `main.go` | CLI + HTTP сервер |
| API | `api/` | Chi router, handlers |
| Роутер | `api/router.go` | ~33k строк, все маршруты |
| БД | `db/` | Store interface, BoltDB, SQL |
| Модели | `db/*.go` | User, Project, Task, Template и др. |
| Сервисы | `services/` | tasks, projects, schedules, runners |
| db_lib | `db_lib/` | Ansible, Terraform, Git, локальные приложения |
| CLI | `cli/` | setup, user, token, project, migrate |
| Frontend | `web/` | Vue.js SPA |

### 1.2 Rust (текущий)

| Компонент | Путь | Описание |
|-----------|------|----------|
| Точка входа | `rust/src/main.rs`, `lib.rs` | CLI (Clap), FFI |
| API | `rust/src/api/` | Axum, handlers |
| Роутер | `api/routes.rs` | ~80 строк |
| БД | `rust/src/db/` | Store traits, SqlStore (SQLite/MySQL/PostgreSQL) |
| Модели | `rust/src/models/` | User, Project, Task, Template и др. |
| Сервисы | `rust/src/services/` | executor, task_runner, scheduler, backup |
| db_lib | `rust/src/db_lib/` | Ansible, Terraform, Git |
| CLI | `rust/src/cli/` | server, user, migrate, project, token |
| Frontend | `web/public/` | Vanilla JS (HTML/CSS/JS) |

### 1.3 Ключевые отличия

| Аспект | Go | Rust |
|--------|-----|------|
| БД | BoltDB + SQL | Только SQL (BoltDB удалён) |
| HTTP | Chi | Axum |
| Async | goroutines | tokio |
| Конфиг | JSON/YAML | env + JSON |
| Frontend | Vue.js | Vanilla JS |

---

## 2. Сравнение API endpoints

### 2.1 Соответствие (Go basePath: /api)

| Go endpoint | Rust endpoint | Статус |
|-------------|---------------|--------|
| GET /ping | GET /api/health | ✅ (разный формат) |
| GET /info | — | ❌ Отсутствует |
| GET /ws | GET /api/ws | ✅ |
| GET/POST /auth/login | GET/POST /api/auth/login | ✅ |
| POST /auth/logout | POST /api/auth/logout | ✅ |
| GET/POST /auth/oidc/{id}/login, redirect | — | ❌ OIDC не реализован |
| GET /user/ | GET /api/user | ✅ Текущий пользователь |
| GET/POST /user/tokens | — | ❌ API токены |
| DELETE /user/tokens/{id} | — | ❌ |
| GET/POST /users | GET/POST /api/users | ✅ |
| GET/PUT/DELETE /users/{id}/ | GET/PUT/DELETE /api/users/{id} | ✅ |
| POST /users/{id}/password | POST /api/users/{id}/password | ✅ Смена пароля |
| GET/POST /projects | GET/POST /api/projects | ✅ |
| POST /projects/restore | POST /api/projects/restore | ✅ Восстановление |
| GET /events | — | ❌ |
| GET /events/last | — | ❌ |
| GET/PUT/DELETE /project/{id}/ | GET/PUT/DELETE /api/projects/{id} | ✅ |
| GET /project/{id}/backup | — | ⚠️ Частично |
| GET /project/{id}/role | — | ❌ Роль пользователя |
| GET /project/{id}/events | — | ❌ |
| GET/POST /project/{id}/users | — | ⚠️ Частично (handlers есть) |
| PUT/DELETE /project/{id}/users/{id} | — | ❌ |
| GET/POST /project/{id}/invites | — | ❌ Приглашения |
| GET/POST/PUT/DELETE /project/{id}/integrations | — | ⚠️ Частично |
| GET/POST/PUT/DELETE /project/{id}/keys | GET/POST/PUT/DELETE /api/projects/{id}/keys | ✅ |
| GET/POST/PUT/DELETE /project/{id}/repositories | GET/POST/PUT/DELETE /api/projects/{id}/repositories | ✅ |
| GET/POST/PUT/DELETE /project/{id}/inventory | GET/POST/PUT/DELETE /api/projects/{id}/inventories | ✅ |
| GET/POST/PUT/DELETE /project/{id}/environment | GET/POST/PUT/DELETE /api/projects/{id}/environments | ✅ |
| GET/POST/PUT/DELETE /project/{id}/templates | ✅ | ✅ |
| POST /project/{id}/templates/{id}/stop_all_tasks | — | ❌ |
| GET/POST/PUT/DELETE /project/{id}/schedules | — | ⚠️ Частично |
| GET/POST/PUT/DELETE /project/{id}/views | — | ⚠️ Частично |
| GET/POST /project/{id}/tasks | ✅ | ✅ |
| GET /project/{id}/tasks/last | — | ❌ |
| GET/DELETE /project/{id}/tasks/{id} | ✅ | ✅ |
| POST /project/{id}/tasks/{id}/stop | — | ❌ Остановка задачи |
| GET /project/{id}/tasks/{id}/output | — | ❌ Вывод задачи |
| GET /project/{id}/tasks/{id}/raw_output | — | ❌ |
| GET /apps | — | ❌ |
| POST /project/{id}/notifications/test | — | ❌ |
| POST /debug/gc | — | ❌ |

### 2.2 Различия в путях

- **Go:** `/api/project/{id}/inventory` (singular)
- **Rust:** `/api/projects/{id}/inventories` (plural)

Rust использует RESTful plural. Для совместимости с Vue frontend из upstream может потребоваться алиас или proxy.

---

## 3. Пошаговый план до полной работоспособности

### Фаза 1: Критичные API (приоритет 1)

| Шаг | Задача | Файлы | Критерий | Статус |
|-----|--------|-------|----------|--------|
| 1.1 | GET /api/user/ — текущий пользователь | handlers/auth.rs, routes.rs | JWT → User | ✅ |
| 1.2 | POST /api/users/{id}/password | handlers/users.rs | Смена пароля | ✅ |
| 1.3 | POST /api/projects/restore | handlers/projects/project.rs, services/backup.rs | Восстановление | ✅ |
| 1.4 | POST /api/projects/{id}/tasks/{id}/stop | handlers/tasks.rs, task_pool | Остановка задачи | ❌ |
| 1.5 | GET /api/projects/{id}/tasks/{id}/output | handlers/tasks.rs, task_output | Вывод задачи | ❌ |

### Фаза 2: Пользователи и проекты (приоритет 2)

| Шаг | Задача | Файлы | Критерий |
|-----|--------|-------|----------|
| 2.1 | GET /api/user/tokens, POST, DELETE | handlers/tokens.rs (новый) | API токены |
| 2.2 | GET/POST/PUT/DELETE /api/projects/{id}/users | handlers/projects/users.rs | Участники проекта |
| 2.3 | GET /api/projects/{id}/role | handlers/projects/project.rs | Роль текущего пользователя |
| 2.4 | GET /api/info | handlers/info.rs | Системная информация |

### Фаза 3: События и WebSocket (приоритет 3)

| Шаг | Задача | Файлы | Критерий |
|-----|--------|-------|----------|
| 3.1 | GET /api/events, /api/events/last | handlers/events.rs | События |
| 3.2 | GET /api/projects/{id}/events | handlers/projects/events.rs | События проекта |
| 3.3 | WebSocket — события в реальном времени | api/websocket, events | Проверка broadcast |

### Фаза 4: Расписания и представления (приоритет 4)

| Шаг | Задача | Файлы | Критерий |
|-----|--------|-------|----------|
| 4.1 | GET/POST/PUT/DELETE /api/projects/{id}/schedules | handlers/projects/schedules.rs | Расписания |
| 4.2 | GET/POST/PUT/DELETE /api/projects/{id}/views | handlers/projects/views.rs | Представления |
| 4.3 | POST /api/projects/{id}/templates/{id}/stop_all_tasks | handlers/templates.rs | Остановка всех задач |

### Фаза 5: Интеграции и приглашения (приоритет 5)

| Шаг | Задача | Файлы | Критерий |
|-----|--------|-------|----------|
| 5.1 | GET/POST/PUT/DELETE /api/projects/{id}/invites | handlers/projects/invites.rs | Приглашения |
| 5.2 | Интеграции — values, matchers, extract | handlers/projects/integration*.rs | Полный CRUD |

### Фаза 6: OIDC и расширения (приоритет 6)

| Шаг | Задача | Файлы | Критерий |
|-----|--------|-------|----------|
| 6.1 | GET/POST /api/auth/oidc/{id}/login, redirect | handlers/auth_oidc.rs | OIDC провайдеры |
| 6.2 | GET /api/apps | handlers/apps.rs | Список приложений |
| 6.3 | POST /api/projects/{id}/notifications/test | handlers/projects/notifications.rs | Тест уведомлений |

### Фаза 7: Качество и стабилизация (приоритет 7)

| Шаг | Задача | Файлы | Критерий |
|-----|--------|-------|----------|
| 7.1 | Убрать #![allow(...)], исправить warnings | lib.rs, все модули | 0 warnings |
| 7.2 | Расширить unit-тесты handlers | api/handlers/tests.rs | >80% покрытие |
| 7.3 | Интеграционные тесты API | tests/ | Полный E2E |
| 7.4 | Совместимость frontend с API | web/public/app.js | Все страницы работают |

---

## 4. Чеклист для каждой сессии

1. `cargo build --lib` — сборка без ошибок
2. `cargo test --lib` — все тесты проходят
3. `cargo run -- server` — сервер запускается
4. Проверить новый endpoint вручную (curl/Postman)
5. Обновить BUILD_ERRORS.md при изменении статуса

---

## 5. Рекомендуемый порядок выполнения

```
Фаза 1 (критичные) → Фаза 2 (пользователи) → Фаза 3 (события) → Фаза 4 (расписания)
→ Фаза 5 (интеграции) → Фаза 6 (OIDC) → Фаза 7 (качество)
```

---

## 6. Технические долги (из BUILD_ERRORS.md)

| Задача | Приоритет |
|--------|-----------|
| Унификация TaskPool (task_pool.rs vs task_pool_types.rs) | Низкий |
| SQLx трейты для Task | Низкий |
| Exporter traits | Низкий |
| Clone для dyn traits (Arc) | Низкий |

---

## 7. Добавление upstream для сравнения

Для локального сравнения с Go-кодом:

```bash
git submodule add https://github.com/semaphoreui/semaphore.git semaphore-upstream
# или
git clone https://github.com/semaphoreui/semaphore.git semaphore-upstream
```

После этого `semaphore-upstream/` будет содержать оригинальный Go-проект.

---

## 8. Ссылки

- **[AI_WORK_PLAN.md](AI_WORK_PLAN.md)** — память AI: порядок работы, таблица Go↔Rust, алгоритм
- [Upstream Go Semaphore](https://github.com/semaphoreui/semaphore)
- [api-docs.yml](api-docs.yml) — полная спецификация API
- [PLAN_FURTHER_WORK.md](PLAN_FURTHER_WORK.md) — текущий план работ
- [BUILD_ERRORS.md](BUILD_ERRORS.md) — история исправлений
