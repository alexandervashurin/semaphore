# Сравнение semaphore-upstream (Go) и Rust-проекта

Документ сопоставляет файлы upstream (velum/velum, ветка develop) с файлами Rust-проекта, куда перенесены функции. Замечания по каждому сравнению.

**Источник upstream:** `semaphore-upstream/` или https://github.com/velum/velum (develop)

---

## 1. API Handlers

| Upstream (Go) | Rust (куда перенесено) | Статус | Замечания |
|---------------|------------------------|--------|-----------|
| `api/auth.go` | `rust/src/api/handlers/auth.rs` | ✅ Частично | login, logout, get_current_user перенесены. OIDC (oidcLogin, oidcRedirect) — не перенесён |
| `api/login.go` | `rust/src/api/handlers/auth.rs` | ✅ Частично | POST login, GET login metadata. verifySession, recoverySession — в auth.rs или отдельно. OIDC flow — нет |
| `api/user.go` | `rust/src/api/handlers/auth.rs`, `rust/src/api/user.rs` | ✅ | get_current_user в auth.rs. API tokens — в user.rs (get_api_tokens, create_api_token, delete_api_token) |
| `api/users.go` | `rust/src/api/handlers/users.rs` | ✅ | CRUD + update_user_password. AddUser — в users (admin). readonlyUserMiddleware — учтено |
| `api/apps.go` | `rust/src/api/apps.rs` | ✅ Частично | getApps, getApp, deleteApp. setApp, setAppActive — проверить |
| `api/cache.go` | `rust/src/api/cache.rs` | ✅ | clearCache |
| `api/events.go` | `rust/src/api/events.rs` | ✅ | getLastEvents, getAllEvents, getProjectEvents |
| `api/integration.go` | `rust/src/api/handlers/projects/integration.rs` | ✅ Частично | ReceiveIntegration (webhook) — проверить наличие |
| `api/options.go` | `rust/src/api/options.rs` | ✅ | getOptions, setOption (admin) |
| `api/router.go` | `rust/src/api/routes.rs`, `rust/src/api/mod.rs` | ✅ Частично | Маршруты в routes.rs. Go: /api/project/{id}, Rust: /api/projects/{id} (plural) |
| `api/runners.go` | `rust/src/api/runners.rs` | ✅ Частично | get_all_runners, add_global_runner, update, delete. RunnerMiddleware, RegisterRunner — проверить |
| `api/system_info.go` | `rust/src/api/system_info.rs` | ✅ | get_system_info |
| `api/projects/backup_restore.go` | `rust/src/api/handlers/projects/backup_restore.rs` | ✅ | GetBackup, Restore, VerifyBackup |
| `api/projects/environment.go` | `rust/src/api/handlers/projects/environment.rs` | ✅ | CRUD environment |
| `api/projects/integration.go` | `rust/src/api/handlers/projects/integration.rs` | ✅ | CRUD integrations |
| `api/projects/integration_alias.go` | `rust/src/api/handlers/projects/integration_alias.rs` | ✅ | get_aliases, add, delete |
| `api/projects/integration_extract_value.go` | — | ❌ | Не найдён отдельный handler |
| `api/projects/integration_matcher.go` | — | ❌ | Не найдён отдельный handler |
| `api/projects/inventory.go` | `rust/src/api/handlers/projects/inventory.rs` | ✅ | CRUD inventory |
| `api/projects/keys.go` | `rust/src/api/handlers/projects/keys.rs` | ✅ | CRUD keys |
| `api/projects/project.go` | `rust/src/api/handlers/projects/project.rs` | ✅ | GetProject, UpdateProject, DeleteProject, GetUserRole, GetBackup и др. |
| `api/projects/projects.go` | `rust/src/api/handlers/projects/project.rs` | ✅ | GetProjects, AddProject, Restore |
| `api/projects/repository.go` | `rust/src/api/handlers/projects/repository.rs` | ✅ | CRUD repositories |
| `api/projects/schedules.go` | `rust/src/api/handlers/projects/schedules.rs` | ✅ | CRUD schedules |
| `api/projects/secret_storages.go` | `rust/src/api/handlers/projects/secret_storages.rs` | ✅ | CRUD secret_storages |
| `api/projects/tasks.go` | `rust/src/api/handlers/projects/tasks.rs` | ✅ Частично | GetTasks, GetTask, AddTask, DeleteTask. **StopTask, ConfirmTask, RejectTask** — проверить наличие |
| `api/projects/templates.go` | `rust/src/api/handlers/projects/templates.rs` | ✅ | CRUD templates |
| `api/projects/users.go` | `rust/src/api/handlers/projects/users.rs` | ✅ | GetUsers, AddUser, UpdateUserRole, DeleteUser |
| `api/projects/views.go` | `rust/src/api/handlers/projects/views.rs` | ✅ | CRUD views. SetViewPositions — проверить |
| `api/debug/gc.go`, `pprof.go` | — | ❌ | Debug endpoints не перенесены |
| `api/helpers/*.go` | `rust/src/api/extractors.rs`, `middleware.rs` | ✅ Частично | context, route_params, write_response — в extractors/middleware |
| `api/sockets/handler.go`, `pool.go` | `rust/src/api/websocket.rs` | ✅ | WebSocket handler |
| `api/tasks/tasks.go` | `rust/src/api/handlers/tasks.rs` | ✅ | Admin tasks (GetTasks, DeleteTask) |
| `api/runners/runners.go` | `rust/src/api/runners.rs` | ✅ | Runner controller |

---

## 2. Модели (db → models)

| Upstream (Go) | Rust | Статус | Замечания |
|---------------|------|--------|-----------|
| `db/User.go` | `rust/src/models/user.rs` | ✅ | |
| `db/Project.go` | `rust/src/models/project.rs` | ✅ | |
| `db/ProjectUser.go` | `rust/src/models/project_user.rs` | ✅ | |
| `db/ProjectInvite.go` | `rust/src/models/project_invite.rs` | ✅ | |
| `db/Task.go` | `rust/src/models/task.rs` | ✅ | |
| `db/TaskParams.go` | `rust/src/models/task_params.rs` | ✅ | |
| `db/Template.go` | `rust/src/models/template.rs` | ✅ | |
| `db/TemplateVault.go` | `rust/src/models/template_vault.rs` | ✅ | |
| `db/Inventory.go` | `rust/src/models/inventory.rs` | ✅ | |
| `db/Repository.go` | `rust/src/models/repository.rs` | ✅ | |
| `db/Environment.go` | `rust/src/models/environment.rs` | ✅ | |
| `db/AccessKey.go` | `rust/src/models/access_key.rs` | ✅ | |
| `db/Schedule.go` | `rust/src/models/schedule.rs` | ✅ | |
| `db/Event.go` | `rust/src/models/event.rs` | ✅ | |
| `db/APIToken.go` | `rust/src/models/token.rs` | ✅ | |
| `db/Integration.go` | `rust/src/models/integration.rs` | ✅ | |
| `db/View.go` | `rust/src/models/view.rs` | ✅ | |
| `db/Session.go` | `rust/src/models/session.rs` | ✅ | |
| `db/Runner.go` | `rust/src/models/runner.rs` | ✅ | |
| `db/SecretStorage.go` | `rust/src/models/secret_storage.rs` | ✅ | |
| `db/Alias.go` | `rust/src/models/alias.rs` | ✅ | |
| `db/Role.go` | `rust/src/models/role.rs` | ✅ | |
| `db/Option.go` | `rust/src/models/option.rs` | ✅ | |
| `db/BackupEntity.go` | `rust/src/models/backup_entity.rs` | ✅ | |
| `db/ExportEntityType.go` | `rust/src/models/export_entity_type.rs` | ✅ | |
| `db/ansible.go` | `rust/src/models/ansible.rs` | ✅ | |
| `db/TerraformInventoryAlias.go` | `rust/src/models/terraform_inventory.rs` | ✅ | |
| `db/Store.go` | `rust/src/db/store.rs` | ✅ | Интерфейс Store |
| `db/config.go` | `rust/src/config/` | ✅ | |

---

## 3. Store / SQL (db/sql)

| Upstream (Go) | Rust | Статус | Замечания |
|---------------|------|--------|-----------|
| `db/sql/SqlDb.go` | `rust/src/db/sql/mod.rs` | ✅ | Инициализация, пул соединений |
| `db/sql/user.go` | `rust/src/db/sql/user_crud.rs`, `user_auth.rs` | ✅ | |
| `db/sql/project.go` | `rust/src/db/sql/mod.rs` (project_*) | ✅ | |
| `db/sql/task.go` | `rust/src/db/sql/mod.rs` | ✅ | |
| `db/sql/template.go` | `rust/src/db/sql/mod.rs` | ✅ | |
| `db/sql/inventory.go` | `rust/src/db/sql/mod.rs` | ✅ | |
| `db/sql/repository.go` | `rust/src/db/sql/mod.rs` | ✅ | |
| `db/sql/environment.go` | `rust/src/db/sql/environment.rs` | ✅ | |
| `db/sql/access_key.go` | `rust/src/db/sql/mod.rs` | ✅ | |
| `db/sql/schedule.go` | `rust/src/db/sql/schedule.rs` | ✅ | |
| `db/sql/event.go` | `rust/src/db/sql/event.rs` | ✅ | |
| `db/sql/session.go` | `rust/src/db/sql/session.rs` | ✅ | |
| `db/sql/integration*.go` | `rust/src/db/sql/integration_crud.rs`, `integration_extract.rs` | ✅ | |
| `db/sql/view.go` | `rust/src/db/sql/view.rs` | ✅ | |
| `db/sql/secret_storage.go` | `rust/src/db/sql/mod.rs` | ✅ | |
| `db/sql/runner.go` | `rust/src/db/sql/runner.rs` | ✅ | |
| `db/bolt/*` | — | ❌ | BoltDB удалён в Rust. Только SQL (SQLite, MySQL, PostgreSQL) |

---

## 4. Сервисы

| Upstream (Go) | Rust | Статус | Замечания |
|---------------|------|--------|-----------|
| `services/session_svc.go` | (в auth, JWT) | ✅ | Session через JWT |
| `services/export/*` | `rust/src/services/exporter*.rs`, `export_*.rs` | ✅ | Export сущностей |
| `services/project/backup.go` | `rust/src/services/backup.rs` | ✅ | |
| `services/project/restore.go` | `rust/src/services/restore.rs` | ✅ | |
| `services/tasks/TaskPool.go` | `rust/src/services/task_pool*.rs` | ✅ | |
| `services/tasks/TaskRunner.go` | `rust/src/services/task_runner/` | ✅ | |
| `services/tasks/LocalJob.go` | `rust/src/services/local_job/` | ✅ | |
| `services/tasks/alert.go` | `rust/src/services/alert.rs` | ✅ | |
| `services/schedules/SchedulePool.go` | `rust/src/services/scheduler.rs` | ✅ | |
| `services/server/*` | `rust/src/services/server/` | ✅ | access_key_svc, environment_svc, project_svc и др. |
| `services/runners/*` | `rust/src/services/` | ✅ Частично | job_pool, running_job — проверить |

---

## 5. Util / Config

| Upstream (Go) | Rust | Статус | Замечания |
|---------------|------|--------|-----------|
| `util/config.go` | `rust/src/config/` | ✅ | |
| `util/config_auth.go` | `rust/src/config/config_auth.rs` | ✅ | |
| `util/config_sysproc.go` | `rust/src/config/config_sysproc.rs` | ✅ | |
| `util/ansi.go` | `rust/src/utils/ansi.rs` | ✅ | |
| `util/version.go` | `rust/src/utils/version.rs` | ✅ | |
| `util/App.go` | `rust/src/utils/app.rs` | ✅ | |
| `util/OdbcProvider.go` | `rust/src/utils/oidc_provider.rs` | ✅ | OIDC |
| `util/encryption.go` | (в services) | ✅ | |
| `util/mailer/*` | — | ❌ | Mailer не перенесён |

---

## 6. CLI

| Upstream (Go) | Rust | Статус | Замечания |
|---------------|------|--------|-----------|
| `cli/main.go` | `rust/src/main.rs`, `rust/src/cli/` | ✅ | |
| `cli/cmd/server.go` | `rust/src/cli/cmd_server.rs` | ✅ | |
| `cli/cmd/user*.go` | `rust/src/cli/` | ✅ | user add, change, delete, list, totp |
| `cli/cmd/token.go` | `rust/src/cli/cmd_token.rs` | ✅ | |
| `cli/cmd/vault*.go` | `rust/src/cli/cmd_vault.rs` | ✅ | |
| `cli/cmd/version.go` | `rust/src/utils/version.rs` | ✅ | |
| `cli/cmd/project*.go` | `rust/src/cli/` | ✅ | project export/import |
| `cli/cmd/runner*.go` | `rust/src/cli/` | ✅ Частично | runner register, setup, start, unregister |
| `cli/cmd/migrate.go` | — | ❓ | Миграции через db/sql |
| `cli/cmd/setup.go` | `rust/src/cli/` | ✅ | |
| `cli/cmd/syslog*.go` | — | ❓ | |

---

## 7. pkg

| Upstream (Go) | Rust | Статус | Замечания |
|---------------|------|--------|-----------|
| `pkg/ssh/agent.go` | `rust/src/services/ssh_agent.rs` | ✅ | |
| `pkg/task_logger/*` | `rust/src/services/task_runner/logging.rs` | ✅ | |
| `pkg/conv/*` | (встроено в Rust) | ✅ | |
| `pkg/random/*` | (rand/uuid) | ✅ | |
| `pkg/tz/*` | chrono | ✅ | |

---

## 8. Критичные отсутствующие endpoints (по api-docs)

| Endpoint | Go | Rust handler | В routes.rs? | Примечание |
|----------|-----|--------------|--------------|------------|
| POST /api/projects/{id}/tasks/{id}/stop | projects.StopTask | stop_task есть | ✅ Да | Добавлен в routes.rs |
| GET /api/projects/{id}/tasks/{id}/output | — | — | ❌ | Фаза 1.5 |
| GET /api/auth/login (metadata) | login (GET) | — | ❌ | OIDC providers, login_with_password |
| POST /api/auth/verify | verifySession | — | ❌ | TOTP verify |
| POST /api/auth/recovery | recoverySession | — | ❌ | Recovery code |
| GET/POST/DELETE /api/user/tokens | user.go | user.rs | ✅ Да | Добавлены в routes.rs |
| GET /api/projects/{id}/role | GetUserRole | get_user_role | ✅ Да | Добавлен в routes.rs |
| GET /api/info | GetSystemInfo | system_info.rs | ✅ Да | Добавлен в routes.rs |
| POST /api/projects/{id}/notifications/test | SendTestNotification | — | ❌ | |
| POST /api/projects/{id}/tasks/{id}/confirm | ConfirmTask | — | ❌ | |
| POST /api/projects/{id}/tasks/{id}/reject | RejectTask | — | ❌ | |
| POST /api/projects/{id}/views/positions | SetViewPositions | — | ❌ | |
| POST /api/projects/{id}/schedules/validate | ValidateScheduleCronFormat | — | ❌ | |
| GET/POST /api/projects/{id}/schedules | — | get_project_schedules, add_schedule | ✅ Да | Добавлены в routes.rs |
| GET/POST /api/projects/{id}/views | — | get_views, add_view | ✅ Да | Добавлены в routes.rs |
| GET/POST /api/projects/{id}/integrations | — | get_integrations, add_integration | ✅ Да | Добавлены в routes.rs |
| GET/POST /api/projects/{id}/secret_storages | — | get_secret_storages, add_secret_storage | ✅ Да | Добавлены в routes.rs |
| GET/POST /api/projects/{id}/users | — | get_users, add_user (project) | ✅ Да | Добавлены в routes.rs |
| GET /api/events, /api/events/last | — | get_last_events, get_all_events | ✅ Да | Добавлены в routes.rs |
| GET /api/apps | — | get_apps | ✅ Да | Добавлен в routes.rs |
| GET/POST /api/options | — | get_options, set_option | ✅ Да | Добавлены в routes.rs |
| GET/POST /api/runners | — | get_all_runners, add_global_runner | ✅ Да | Добавлены в routes.rs |
| DELETE /api/cache | — | clear_cache | ✅ Да | Добавлен в routes.rs |

---

## 9. Различия путей API

| Go (upstream) | Rust | Примечание |
|---------------|------|------------|
| `/api/project/{id}/inventory` | `/api/projects/{id}/inventories` | singular → plural |
| `/api/project/{id}/keys` | `/api/projects/{id}/keys` | |
| `/api/project/{id}/environment` | `/api/projects/{id}/environments` | |
| `/api/project/{id}/repositories` | `/api/projects/{id}/repositories` | |
| `/api/project/{id}/templates` | `/api/projects/{id}/templates` | |
| `/api/project/{id}/tasks` | `/api/projects/{id}/tasks` | |
| `/api/project/{id}/users` | `/api/projects/{id}/users` | (project users) |
| `/api/project/{id}/views` | `/api/projects/{id}/views` | |
| `/api/project/{id}/schedules` | `/api/projects/{id}/schedules` | |
| `/api/project/{id}/integrations` | `/api/projects/{id}/integrations` | |
| `/api/project/{id}/secret_storages` | `/api/projects/{id}/secret_storages` | |

Frontend в `web/public/` и Vue ожидают Rust-пути (plural). Go frontend — singular.

---

## 10. Сводка

- **API handlers:** ~95% кода перенесено. **Важно:** большинство handlers реализованы и **зарегистрированы в routes.rs** (schedules, views, integrations, secret_storages, project users, events, apps, options, runners, cache, info, stop_task, user tokens, get_user_role). Отсутствуют: OIDC, auth/verify, auth/recovery, confirm/reject task, notifications/test, views/positions, schedules/validate, task output.
- **Модели:** Полностью перенесены.
- **Store/SQL:** Перенесено. BoltDB удалён.
- **Сервисы:** Основные перенесены.
- **CLI:** Основные команды перенесены.
- **Util/Config:** Перенесено. Mailer — нет.

---

## 11. Декомпозиция DB/SQL ✅

**Статус:** 100% завершено

В ходе рефакторинга монолитный файл `db/sql/mod.rs` (~4500 строк) был декомпозирован на модули:

### Созданные модули:

| Диалект | user | template | project | inventory | repository | environment |
|---------|------|----------|---------|-----------|------------|-------------|
| **SQLite** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **PostgreSQL** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **MySQL** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

**Всего:** 18 модулей (~2000 строк)

### Адаптеры:

| Файл | Строк | Сокращение |
|------|-------|------------|
| template_crud.rs | ~100 | -63% |
| user_crud.rs | ~150 | -73% |
| inventory.rs | ~100 | -70% |
| repository.rs | ~100 | -70% |
| environment.rs | ~100 | -70% |
| project.rs | ~100 | -70% |

**Документация:** `rust/src/db/sql/README.md`

### Преимущества:

- ✅ Читаемость: файлы по 100-150 строк вместо 4500
- ✅ Поддержка: легко найти код для конкретного БД
- ✅ Тестирование: можно тестировать каждый диалект отдельно
- ✅ Расширяемость: легко добавить новый БД
- ✅ Параллельная работа: разные разработчики могут работать с разными БД

---

## 12. Рекомендуемые действия

1. ✅ **Выполнено:** Добавить маршруты в routes.rs для существующих handlers: schedules, views, integrations, secret_storages, project users, events, apps, options, runners, cache, info, stop_task, user tokens, get_user_role.
2. ✅ **Выполнено:** Декомпозиция DB/SQL модуля (100% завершено).
3. Реализовать отсутствующие: auth/verify, auth/recovery, GET /api/auth/login (metadata), confirm/reject task, notifications/test, views/positions, schedules/validate.
4. Реализовать GET /api/projects/{id}/tasks/{id}/output (Фаза 1.5).
5. Реализовать OIDC flow (login, verify, recovery).
6. Реализовать mailer (если нужен).
