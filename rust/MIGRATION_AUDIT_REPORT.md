# 📊 Аудит Миграции: Go → Rust

**Дата**: 2026-02-28
**Статус**: 🚧 В ПРОЦЕССЕ

---

## 📈 Общая Статистика

| Категория | Go Файлов | Rust Файлов | Прогресс |
|-----------|-----------|-------------|----------|
| **api/** | 47 | 22 | ~47% |
| **db/** | 122 | 36 | ~30% |
| **db_lib/** | 12 | 6 | ~50% |
| **services/** | 79 | 40 | ~51% |
| **cli/** | 27 | 2 | ~7% |
| **pkg/** | 3 | 0 | 0% |
| **pro/** | 18 | 0 | 0% |
| **util/** | 17 | 6 | ~35% |
| **ВСЕГО** | **334** | **156** | **~47%** |

---

## 🔍 Детальный Анализ по Категориям

### 1. API (47 Go → 22 Rust, ~47%)

#### ✅ Мигрировано:

| Go Файл | Rust Файл | Статус |
|---------|-----------|--------|
| `api/auth.go` | `rust/src/api/auth.rs` | ✅ |
| `api/login.go` (частично) | `rust/src/api/auth_local.rs` | ✅ |
| `api/user.go` | `rust/src/api/user.rs` | ✅ |
| `api/users.go` | `rust/src/api/users.rs` | ✅ |
| `api/integration.go` | `rust/src/api/integration.rs` | ✅ |
| - | `rust/src/api/handlers/*.rs` (10 файлов) | ✅ |
| - | `rust/src/api/extractors.rs` | ✅ |
| - | `rust/src/api/middleware.rs` | ✅ |
| - | `rust/src/api/routes.rs` | ✅ |
| - | `rust/src/api/state.rs` | ✅ |
| - | `rust/src/api/websocket.rs` | ✅ |

#### ⏳ В Процессе:

| Go Файл | Rust Аналог | Прогресс |
|---------|-------------|----------|
| `api/login.go` (LDAP/OIDC) | `rust/src/api/auth_ldap.rs`, `auth_oidc.rs` | 0% |
| `api/router.go` | `rust/src/api/routes.rs` (частично) | 50% |
| `api/apps.go` | - | 0% |
| `api/options.go` | - | 0% |
| `api/runners.go` | - | 0% |
| `api/system_info.go` | - | 0% |
| `api/events.go` | - | 0% |
| `api/cache.go` | - | 0% |

#### 📁 Projects API (подпапка):

| Go Файл | Rust Файл | Статус |
|---------|-----------|--------|
| `api/projects/*.go` (33 файла) | `rust/src/api/handlers/*.rs` | ✅ Частично |

---

### 2. DB (122 Go → 36 Rust, ~30%)

#### ✅ Мигрировано:

| Go Файл | Rust Файл | Статус |
|---------|-----------|--------|
| `db/Store.go` (частично) | `rust/src/db/store.rs` | ✅ |
| `db/sql/SqlDb.go` (частично) | `rust/src/db/sql/mod.rs` + модули | ✅ |
| `db/bolt/BoltDb.go` (частично) | `rust/src/db/bolt/mod.rs` + модули | ✅ |

**SQL Модули**:
- `rust/src/db/sql/runner.rs` ✅
- `rust/src/db/sql/project_invite.rs` ✅
- `rust/src/db/sql/terraform_inventory.rs` ✅
- `rust/src/db/sql/utils.rs` ✅
- `rust/src/db/sql/user_totp.rs` ✅
- `rust/src/db/sql/task_crud.rs` ✅
- `rust/src/db/sql/task_output.rs` ✅
- `rust/src/db/sql/task_stage.rs` ✅
- `rust/src/db/sql/template_*.rs` (4 файла) ✅
- `rust/src/db/sql/user_*.rs` (3 файла) ✅
- `rust/src/db/sql/integration_*.rs` (3 файла) ✅

**Bolt Модули**:
- `rust/src/db/bolt/event.rs` ✅
- `rust/src/db/bolt/user.rs` ✅
- `rust/src/db/bolt/project_invite.rs` ✅
- `rust/src/db/bolt/task.rs` ✅
- `rust/src/db/bolt/template.rs` ✅
- `rust/src/db/bolt/project.rs` ✅
- `rust/src/db/bolt/schedule.rs` ✅
- `rust/src/db/bolt/session.rs` ✅
- `rust/src/db/bolt/inventory_repository_environment.rs` ✅
- `rust/src/db/bolt/access_key.rs` ✅
- `rust/src/db/bolt/view_option.rs` ✅

#### ⏳ Осталось:

| Go Файл | Rust Аналог | Прогресс |
|---------|-------------|----------|
| `db/Task.go` | - | 0% |
| `db/User.go` | - | 0% |
| `db/Project.go` | - | 0% |
| `db/Template.go` | - | 0% |
| `db/Inventory.go` | - | 0% |
| `db/Repository.go` | - | 0% |
| `db/Environment.go` | - | 0% |
| `db/AccessKey.go` | - | 0% |
| `db/Integration.go` | - | 0% |
| `db/Schedule.go` | - | 0% |
| `db/Session.go` | - | 0% |
| `db/APIToken.go` | - | 0% |
| `db/Event.go` | - | 0% |
| `db/Runner.go` | - | 0% |
| `db/View.go` | - | 0% |
| `db/Role.go` | - | 0% |
| `db/SecretStorage.go` | - | 0% |
| `db/sql/migration.go` | - | 0% |
| `db/sql/migrations/*.go` (15 файлов) | - | 0% |
| `db/bolt/migration*.go` (10 файлов) | - | 0% |

---

### 3. DB Lib (12 Go → 6 Rust, ~50%)

#### ✅ Мигрировано:

| Go Файл | Rust Файл | Статус |
|---------|-----------|--------|
| `db_lib/AnsibleApp.go` | `rust/src/db_lib/ansible_app.rs` | ✅ |
| `db_lib/TerraformApp.go` | `rust/src/db_lib/terraform_app.rs` | ✅ |
| `db_lib/AccessKeyInstaller.go` | `rust/src/db_lib/access_key_installer.rs` | ✅ |
| `db_lib/CmdGitClient.go` | `rust/src/db_lib/cmd_git_client.rs` | ✅ |
| `db_lib/GitRepository.go` | `rust/src/services/git_repository.rs` | ✅ |
| `db_lib/types.go` (частично) | `rust/src/db_lib/types.rs` | ✅ |

#### ⏳ Осталось:

| Go Файл | Rust Аналог | Прогресс |
|---------|-------------|----------|
| `db_lib/AppFactory.go` | - | 0% |
| `db_lib/GoGitClient.go` | - | 0% |
| `db_lib/GitClientFactory.go` | - | 0% |
| `db_lib/LocalApp.go` | - | 0% |
| `db_lib/LocalApp_test.go` | - | 0% |
| `db_lib/ShellApp.go` | - | 0% |

---

### 4. Services (79 Go → 40 Rust, ~51%)

#### ✅ Мигрировано:

| Go Файл | Rust Файл | Статус |
|---------|-----------|--------|
| `services/tasks/TaskPool.go` (частично) | `rust/src/services/task_pool*.rs` (5 файлов) | ✅ |
| `services/tasks/TaskRunner.go` (частично) | `rust/src/services/task_runner/` (7 файлов) | ✅ |
| `services/tasks/LocalJob.go` (частично) | `rust/src/services/local_job/` (8 файлов) | ✅ |
| `services/tasks/alert.go` | `rust/src/services/alert.rs` | ✅ |
| `services/project/backup.go` | `rust/src/services/backup.rs` | ✅ |
| `services/project/restore.go` | `rust/src/services/restore.rs` | ✅ |
| `services/export/Exporter.go` (частично) | `rust/src/services/exporter*.rs` (3 файла) | ✅ |
| `services/schedules/SchedulePool.go` | `rust/src/services/scheduler.rs` | ✅ |
| `services/runners/job_pool.go` | `rust/src/services/job.rs` | ✅ |
| `services/server/*.go` (10 файлов) | `rust/src/services/access_key_*.rs` | ✅ |

#### ⏳ Осталось:

| Go Файл | Rust Аналог | Прогресс |
|---------|-------------|----------|
| `services/tasks/*.go` (40 файлов) | - | ~60% |
| `services/export/*.go` (10 файлов) | - | ~30% |
| `services/schedules/*.go` (5 файлов) | - | ~80% |
| `services/runners/*.go` (10 файлов) | - | ~80% |
| `services/server/*.go` (10 файлов) | - | ~50% |

---

### 5. CLI (27 Go → 2 Rust, ~7%)

#### ✅ Мигрировано:

| Go Файл | Rust Файл | Статус |
|---------|-----------|--------|
| - | `rust/src/main.rs` | ✅ |
| - | `rust/src/cli/mod.rs` | ✅ |

#### ⏳ Осталось:

| Go Файл | Rust Аналог | Прогресс |
|---------|-------------|----------|
| `cli/cmd/*.go` (15 файлов) | - | 0% |
| `cli/setup/*.go` (10 файлов) | - | 0% |

---

### 6. PKG (3 Go → 0 Rust, 0%)

#### ⏳ Осталось:

| Go Файл | Rust Аналог | Прогресс |
|---------|-------------|----------|
| `pkg/ssh/agent.go` | `rust/src/services/ssh_agent.rs` (готов ✅) | 0% |
| `pkg/ssh/agent_test.go` | - | 0% |
| `pkg/task_logger/task_logger.go` | `rust/src/services/task_logger.rs` (готов ✅) | 0% |

**Примечание**: Rust аналоги готовы, нужно только удалить Go файлы

---

### 7. PRO (18 Go → 0 Rust, 0%)

#### ⏳ В Процессе:

| Go Файл | Rust Файл | Статус |
|---------|-----------|--------|
| `pro/db/sql/terraform_inventory.go` | `rust/src/db/sql/terraform_inventory.rs` | ✅ |
| `pro/api/*.go` (5 файлов) | - | 0% |
| `pro/pkg/*.go` (4 файла) | - | 0% |
| `pro/services/*.go` (5 файлов) | - | 0% |
| `pro/db/factory/*.go` (3 файла) | - | 0% |

---

### 8. Util (17 Go → 6 Rust, ~35%)

#### ✅ Мигрировано:

| Go Файл | Rust Файл | Статус |
|---------|-----------|--------|
| `util/mailer/mailer.go` | `rust/src/utils/mailer.rs` | ✅ |
| `util/mailer/auth.go` | `rust/src/utils/mailer.rs` | ✅ |
| `util/encryption.go` | `rust/src/utils/encryption.rs` | ✅ |
| `util/shell.go` | `rust/src/utils/shell.rs` | ✅ |
| `util/config.go` (частично) | `rust/src/config/*.rs` (8 файлов) | ✅ |
| `util/version.go` | - | ✅ |

#### ⏳ Осталось:

| Go Файл | Rust Аналог | Прогресс |
|---------|-------------|----------|
| `util/config.go` (1407 строк) | `rust/src/config/` (частично) | ~80% |
| `util/config_test.go` | - | 0% |
| `util/config_assign_test.go` | - | 0% |
| `util/debug.go` | - | 0% |
| `util/errorLogging.go` | - | 0% |
| `util/test_helpers.go` | - | 0% |
| `util/ansi.go` | - | 0% |

---

## 🎯 Приоритеты Миграции

### Приоритет 1 (Критично):

1. **pkg/task_logger** - Rust готов, удалить Go ✅
2. **pkg/ssh** - Rust готов, удалить Go ✅

### Приоритет 2 (Важно):

3. **CLI** - только 2 файла из 27 мигрировано (~7%)
4. **PRO модули** - 1 файл из 18 мигрировано (~6%)
5. **DB модели** - основные модели не мигрированы

### Приоритет 3 (Средне):

6. **API** - 47% готово, осталось ~25 файлов
7. **Util** - 35% готово, осталось ~11 файлов
8. **Services** - 51% готово, осталось ~39 файлов

---

## 📋 План Завершения

### Этап 1: Удаление Готовых Модулей (1 день)

- [ ] Удалить `pkg/task_logger/`
- [ ] Удалить `pkg/ssh/`
- [ ] Обновить зависимости в Go (если нужно)

### Этап 2: Завершение PRO Модулей (2-3 дня)

- [ ] Мигрировать `pro/api/*.go` (5 файлов)
- [ ] Мигрировать `pro/services/*.go` (5 файлов)
- [ ] Мигрировать `pro/pkg/*.go` (4 файла)

### Этап 3: Завершение CLI (3-4 дня)

- [ ] Мигрировать `cli/cmd/*.go` (15 файлов)
- [ ] Мигрировать `cli/setup/*.go` (10 файлов)

### Этап 4: Финализация (2-3 дня)

- [ ] Удалить `go.mod`, `go.sum`
- [ ] Обновить документацию
- [ ] cargo-audit
- [ ] Релиз v1.0.0

---

**Ответственный**: Alexander Vashurin
**Следующий шаг**: Удаление pkg/task_logger и pkg/ssh
