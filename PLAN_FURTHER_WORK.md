# План дальнейших работ Semaphore Rust

**Дата:** 2026-03-05  
**Последнее обновление:** 2026-03-05 (сессия 14)

---

## ✅ Выполнено (сессия 14) — Фаза 1 API

| Задача | Файлы | Статус |
|--------|-------|--------|
| 1.1 GET /api/user — текущий пользователь | handlers/auth.rs, routes.rs | ✅ |
| 1.2 POST /api/users/{id}/password — смена пароля | handlers/users.rs, routes.rs | ✅ |
| 1.3 POST /api/projects/restore — восстановление | handlers/projects/project.rs, services/backup.rs, routes.rs | ✅ |

---

## ✅ Выполнено (сессия 13)

| Задача | Статус |
|--------|--------|
| Исправление 6 игнорируемых тестов | ✅ 503 passed, 0 failed, 0 ignored |
| Обработка ошибок API | ✅ Error::to_status_code, ErrorResponse::from_crate_error |
| Unit-тесты для handlers | ✅ 5 тестов (health, logout, login, projects) |
| Frontend — кнопки «Добавить» и модальные окна | ✅ Репозитории, окружения, инвентарь, шаблоны, ключи |
| Рекомендуемый порядок выполнения | ✅ Добавлен в план |

---

## 📋 Следующие шаги (Фаза 1 — осталось)

| Шаг | Задача | Файлы | Примечание |
|-----|--------|-------|------------|
| 1.4 | POST /api/projects/{id}/tasks/{id}/stop | handlers/tasks.rs, task_pool_runner | Остановка задачи |
| 1.5 | GET /api/projects/{id}/tasks/{id}/output | handlers/tasks.rs, task_output | Вывод задачи |

---

## 📋 Следующие шаги (общие)

1. **Очистка warnings** (опционально) — убрать `#![allow(...)]`, исправить ~241 предупреждение
2. **Расширить unit-тесты** — task handlers, интеграционные тесты
3. **Технические долги** — SQLx трейты, Exporter, TaskPool, Clone для dyn traits
4. **Реализовать edit/delete** — полноценные модальные окна для редактирования и удаления

---

## Текущее состояние

| Компонент | Статус |
|-----------|--------|
| Сборка lib | ✅ 0 ошибок |
| Тесты | ✅ 503 passed, 0 failed, 0 ignored |
| Сервер | ✅ Запускается |
| API | ✅ Работает |
| Frontend | ✅ Базовая версия + CRUD модалки |

---

## 1. Исправление игнорируемых тестов (приоритет 1) ✅ ВЫПОЛНЕНО

**Цель:** Убрать `#[ignore]` и добиться прохождения всех 6 тестов.

| Тест | Файл | Решение |
|------|------|---------|
| test_validate_config_empty_tmp_path | config/validator.rs | ✅ Добавлена валидная DbConfig в тест |
| test_get_environment_env | services/local_job/environment.rs | ✅ environment.json = "{}" в тесте |
| test_get_template_params | services/local_job/cli.rs | ✅ get_template_params возвращает task.params |
| test_user_add_command | cli/cmd_user.rs | ✅ init_user_table_for_test() + test_sqlite_url |
| test_verify_recovery_code_normalization | config/config_helpers.rs | ✅ Исправлена вставка пробелов (flat_map) |
| test_kill_task | services/task_pool_runner.rs | ✅ MockStore вместо SqlStore |

---

## 2. Frontend — дополнительные страницы (приоритет 2)

**Цель:** Расширить UI для полной работы с проектом.

| Страница | Описание | API endpoints | Статус |
|----------|----------|---------------|--------|
| Задачи (Tasks) | Список задач проекта, статусы, логи | GET /api/projects/{id}/tasks | ✅ |
| Шаблоны (Templates) | CRUD шаблонов, запуск задач | GET/POST /api/projects/{id}/templates | ✅ + Добавить |
| Инвентарь (Inventory) | Управление инвентарём | GET/POST /api/projects/{id}/inventories | ✅ + Добавить |
| Репозитории | Список репозиториев | GET/POST /api/projects/{id}/repositories | ✅ + Добавить |
| Ключи доступа | SSH/другие ключи | GET/POST /api/projects/{id}/keys | ✅ + Добавить |
| Окружения | Environment variables | GET/POST /api/projects/{id}/environments | ✅ + Добавить |

**Файлы:** `web/public/index.html`, `web/public/app.js`, `web/public/styles.css`

---

## 3. Обработка ошибок API (приоритет 3)

**Цель:** Единообразные и информативные ответы об ошибках.

- [x] Стандартизировать формат ErrorResponse (code, message, details)
- [x] Добавить маппинг Error → HTTP status (Error::to_status_code, error_code)
- [x] Логирование ошибок с request_id (ErrorResponse::with_request_id)
- [x] Валидация входных данных (ErrorResponse::validation_error)

**Файлы:** `rust/src/api/middleware.rs`, `rust/src/error.rs`

---

## 4. Unit-тесты для handlers (приоритет 4)

**Цель:** Покрытие API endpoints тестами.

- [x] Тесты для auth handlers (login, logout, health)
- [x] Тесты для project handlers (list)
- [ ] Тесты для task handlers (базовые добавлены)
- [x] Интеграционные тесты с tower::ServiceExt::oneshot

**Файл:** `rust/src/api/handlers/tests.rs`

---

## 5. Очистка warnings (приоритет 5, опционально)

**Цель:** Убрать `#![allow(unused_imports, ...)]` и исправить предупреждения вручную.

- [ ] Удалить allow из lib.rs
- [ ] Исправить unused imports (удалить или использовать)
- [ ] Исправить unused variables (префикс _ или использование)
- [ ] Исправить dead_code (удалить или #[allow] локально)

**Оценка:** ~241 предупреждение в 80+ файлах

---

## 6. Технические долги (низкий приоритет)

| Задача | Описание |
|--------|----------|
| SQLx трейты | Глубокая интеграция Type/Decode для Task и др. |
| Exporter traits | Рефакторинг архитектуры экспорта |
| Clone для dyn traits | Изменение архитектуры callback |
| Дублирование TaskPool | Унификация task_pool.rs и task_pool_types.rs |

---

## Рекомендуемый порядок выполнения (от простого к сложному)

### Этап 1. Исправление игнорируемых тестов (1–2 сессии)

**Порядок внутри этапа:**

| № | Тест | Сложность | Почему в таком порядке |
|---|------|-----------|------------------------|
| 1 | test_validate_config_empty_tmp_path | Низкая | Одна проверка в Config::validate или правка теста |
| 2 | test_get_environment_env | Низкая | Скорее всего дефолтные переменные окружения |
| 3 | test_get_template_params | Средняя | Исправить структуру params в тесте или коде |
| 4 | test_user_add_command | Средняя | Использовать sqlite::memory: в тесте |
| 5 | test_verify_recovery_code_normalization | Средняя | Разобраться с хешированием и нормализацией пробелов |
| 6 | test_kill_task | Высокая | Нужен mock для kill, возможны ограничения тестовой среды |

### Этап 2. Обработка ошибок API (~0.5 сессии)

1. Стандартизировать ErrorResponse (code, message, details)
2. Маппинг Error → HTTP status
3. Логирование с request_id
4. Валидация входных данных

### Этап 3. Unit-тесты для handlers (1–2 сессии)

1. Auth handlers (login, logout)
2. Project handlers (list, create, get)
3. Task handlers
4. Интеграционные тесты с axum::test

### Этап 4. Frontend — дополнительные страницы (2–3 сессии)

| № | Страница | Сложность |
|---|----------|-----------|
| 1 | Репозитории | Низкая |
| 2 | Ключи доступа | Низкая |
| 3 | Окружения | Низкая |
| 4 | Задачи (Tasks) | Средняя |
| 5 | Инвентарь | Средняя |
| 6 | Шаблоны | Высокая |

### Этап 5. Очистка warnings (~1 сессия, опционально)

1. Удалить #![allow(...)] из lib.rs
2. unused_imports
3. unused_variables
4. dead_code

### Этап 6. Технические долги (низкий приоритет)

1. Унификация TaskPool
2. SQLx трейты
3. Exporter traits
4. Clone для dyn traits

### Сводная таблица

| Этап | Задача | Оценка | Зависимости |
|------|--------|--------|-------------|
| 1 | 6 игнорируемых тестов | 1–2 сессии | — |
| 2 | Обработка ошибок API | 0.5 сессии | — |
| 3 | Unit-тесты handlers | 1–2 сессии | Этап 2 |
| 4 | Frontend-страницы | 2–3 сессии | Этапы 2, 3 |
| 5 | Очистка warnings | 1 сессия | — |
| 6 | Технические долги | — | После 1–4 |

---

## Порядок выполнения

| Этап | Задача | Оценка |
|------|--------|--------|
| 1 | Исправить 6 #[ignore] тестов | 1–2 сессии |
| 2 | Обработка ошибок API | 0.5 сессии |
| 3 | Unit-тесты для handlers | 1–2 сессии |
| 4 | Frontend: страницы задач, шаблонов, инвентаря | 2–3 сессии |
| 5 | Очистка warnings | 1 сессия (опционально) |

---

## 🔍 Новые вводные (сессия 14)

### Axum 0.8 — extractors

- **AuthUser** и др. extractors должны реализовывать `FromRequestParts<Arc<AppState>>`, а не `FromRequestParts<State<Arc<AppState>>>`. В Axum 0.8 state передаётся как `Arc<AppState>` напрямую.

### BackupFormat — совместимость с api-docs

- API использует `meta` вместо `project`, `keys` вместо `access_keys` — добавлены `#[serde(alias = "meta")]` и `#[serde(alias = "keys")]` в `services/backup.rs`.
- В api-docs поля `type` в шаблонах, инвентаре, ключах — добавлены `#[serde(alias = "type")]` для `template_type`, `inventory_type`, `key_type`.
- Views в api-docs используют `title` — добавлен `#[serde(alias = "title")]` для `BackupView.name`.

### Restore — владелец проекта

- `create_project_user` закомментирован в `handlers/projects/users.rs` — после restore пользователь не добавляется как owner автоматически. Требуется реализация Store::create_project_user.

### Go reference

- RESTORE: `api/projects/backup_restore.go` — `Restore()` читает body, unmarshal в BackupFormat, verify(), backup.Restore(user, store), возвращает 200 + Project.
- Пароль: `api/users.go` — `updateUserPassword()`: только admin или сам пользователь; external users не могут менять пароль.

---

## Чеклист для каждой сессии

1. `cargo build --lib` — сборка без ошибок
2. `cargo test --lib` — все тесты проходят
3. `cargo run -- server` — сервер запускается
4. Обновить BUILD_ERRORS.md или PLAN_FURTHER_WORK.md при изменении статуса
