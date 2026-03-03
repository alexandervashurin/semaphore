# Отчёт об ошибках сборки Semaphore Rust
Дата: 2026-03-02
Последнее обновление: 2026-03-03 (сессия 5)

## Статистика
- **Начальное количество ошибок:** 585
- **Текущее количество ошибок:** 165
- **Исправлено ошибок:** 420 (71.8%)
- **Предупреждений:** ~280

## ✅ Исправленные категории ошибок (сессия 5)

### 1. Удаление BoltDB - ЗАВЕРШЕНО
- ✅ Удалена директория `src/db/bolt/` со всеми файлами BoltDB
- ✅ Удалён `BoltStore` из `src/db/mod.rs`
- ✅ Удалена зависимость от `BoltStore` в CLI (`src/cli/mod.rs`, `src/cli/cmd_server.rs`)
- ✅ Удалён диалект `DbDialect::Bolt` из конфигурации
- ✅ Обновлена валидация конфигурации (удалена проверка Bolt)
- ✅ Изменён CLI: убрана поддержка `--db-dialect bolt`

### 2. Конфигурация - ИСПРАВЛЕНО
- ✅ Исправлен `Config::db_dialect()` - добавлен `.clone()` для unwrap_or
- ✅ Исправлен `Config::non_admin_can_create_project()` - добавлен `.clone()`
- ✅ Исправлены инициализаторы `DbConfig` (добавлены поля `path`, `connection_string`)
- ✅ Исправлен `merge_db_configs()` - добавлены новые поля
- ✅ Исправлен `merge_ha_configs()` - исправлено обращение к `node_id`
- ✅ Исправлено форматирование `[u8; 16]` в hex ( LowerHex/UpperHex trait)

### 3. Модель Repository - ИСПРАВЛЕНО
- ✅ Добавлен метод `Repository::get_full_path()` для получения пути к репозиторию

### 4. Task инициализаторы - ИСПРАВЛЕНО
- ✅ Добавлены поля `environment_id`, `repository_id` в инициализаторы `Task`
- ✅ Исправлен `services/scheduler.rs` - добавлены missing поля

### 5. TaskOutput инициализаторы - ИСПРАВЛЕНО
- ✅ Добавлено поле `project_id` в инициализаторы `TaskOutput`
- ✅ Исправлен `services/task_pool_status.rs`
- ✅ Исправлен `services/task_runner/logging.rs`

### 6. Moved value ошибки - ИСПРАВЛЕНО
- ✅ Исправлен `api/user.rs` - `.clone()` для `current_user`
- ✅ Исправлен `api/users.rs` - `.clone()` для `user_to_update`
- ✅ Исправлен `services/alert.rs` - сохранение `user.email` до move

### 7. RunningTask Clone - ИСПРАВЛЕНО
- ✅ Переписаны методы `get_running_task()` и `get_running_tasks()` без Clone
- ✅ Исправлен `kill_task()` - исправлен порядок операций с lock

### 8. Config sysproc - ИСПРАВЛЕНО
- ✅ Исправлен `nix::unistd::chroot()` - передача `&str` вместо `&String`

## ✅ Исправленные категории ошибок (сессия 1-4)

### 1. System Process
- ✅ Заменён `libc` на `nix` crate для системных вызовов
- ✅ Исправлен `config_sysproc.rs` - использование `nix::unistd` для chroot, setgid, setuid
- ✅ Добавлен `nix = "0.29"` в Cargo.toml с фичами `user`, `fs`

### 2. Default реализации
- ✅ `Repository::default()` - добавлена реализация
- ✅ `Inventory::default()` - добавлена реализация
- ✅ `Environment::default()` - добавлена реализация
- ✅ `HARedisConfig::default()` - добавлена реализация

### 3. ProjectUser модель
- ✅ Добавлены поля: `username: String`, `name: String`
- ✅ Обновлены инициализаторы в `db/sql/mod.rs` и `api/handlers/projects/users.rs`

### 4. TaskStageType
- ✅ Заменён `TaskStageType::InstallRoles` на `TaskStageType::Init`

### 5. Модели данных
- ✅ `TemplateType` - добавлены варианты: Deploy, Task, Ansible, Terraform, Shell
- ✅ `ProjectUser` - добавлены поля: username, name
- ✅ `Task` - добавлены поля: repository_id, environment_id
- ✅ `TaskOutput` - добавлено поле: project_id

## 🔴 Текущие ошибки (165 осталось)

### Топ ошибок по количеству:

| Категория | Количество | Приоритет |
|-----------|------------|-----------|
| mismatched types | 24 | Высокий |
| type annotations needed | 10 | Средний |
| no field `ssh_key` on type `DbRepository` | 4 | Высокий |
| no field `name`/`secret`/`secret_type` on `&String` | 10 | Высокий |
| dyn Any + Send + Sync: Clone | 4 | Средний |
| no field `ha` on type `HAConfig` | 0 | ✅ Исправлено |
| no field `key_id` on type `DbRepository` | 2 | Высокий |
| Task: sqlx::Decode/Type | 2+2 | Высокий |
| SecretStorage: FromRow | 2 | Средний |
| ExporterChain: DataExporter | 2+2 | Низкий |
| and_then for i32 | 3 | Средний |
| is_empty for Option | 2 | Низкий |

### Критические проблемы (требуют исправления):

#### 1. Git Client - использование ssh_key (6 ошибок)
**Проблема:** Код обращается к `repository.ssh_key`, но поле называется `key_id`

**Файлы:**
- `src/db_lib/go_git_client.rs` - 4 ошибки
- `src/db_lib/cmd_git_client.rs` - 2 ошибки

**Решение:** Загружать AccessKey через `key_id` из хранилища

#### 2. Неправильное использование String vs структур (10 ошибок)
**Проблема:** Код обращается к полям `name`, `secret`, `secret_type` на `&String`

**Файлы:**
- `src/services/local_job/vault.rs` - vault_key_id, name
- `src/services/local_job/environment.rs` - secret_type, name, secret
- `src/services/local_job/args.rs` - secret_type, name, secret

**Решение:** Исправить типы данных в моделях

#### 3. mismatched types (24 ошибки)
**Проблема:** Разнородная группа ошибок несоответствия типов

**Основные категории:**
- TemplateType match (Option<TemplateType> vs TemplateType)
- TerraformTaskParams поля
- Backup/Restore модели
- Git client callback типы

#### 4. SQLx трейты для Task (4 ошибки)
**Проблема:** `Task` не реализует `sqlx::Decode` и `sqlx::Type`

**Файлы:**
- `src/db/sql/task_crud.rs`

**Решение:** Добавить реализацию трейтов или использовать кастомный FromRow

#### 5. LocalJob Job trait (1 ошибка)
**Проблема:** `LocalJob` не реализует трейт `Job`

**Файлы:**
- `src/services/local_job/types.rs`
- `src/services/job.rs`

**Решение:** Реализовать трейт `Job` для `LocalJob`

#### 6. AccessKeyInstallerImpl Clone (1 ошибка)
**Проблема:** Требуется Clone для `AccessKeyInstallerImpl`

**Файлы:**
- `src/db_lib/access_key_installer.rs`
- `src/services/task_runner/lifecycle.rs`

**Решение:** Добавить Clone или изменить архитектуру

## 📋 План следующей сессии (сессия 6)

### Приоритет 1: Git Client (6 ошибок)
1. Исправить `go_git_client.rs` - загрузка AccessKey через key_id
2. Исправить `cmd_git_client.rs` - загрузка AccessKey через key_id
3. Обновить сигнатуры методов GitClient

### Приоритет 2: Модели данных (10 ошибок)
1. Исправить `local_job/vault.rs` - правильные типы для vault
2. Исправить `local_job/environment.rs` - правильные типы для secret
3. Исправить `local_job/args.rs` - правильные типы для secret

### Приоритет 3: mismatched types (24 ошибки)
1. Исправить TemplateType match в `local_job/run.rs`
2. Исправить TerraformTaskParams в `terraform_app.rs`
3. Исправить Backup/Restore модели

### Приоритет 4: SQLx трейты (4 ошибки)
1. Реализовать `sqlx::Decode` и `sqlx::Type` для `Task`
2. Или переписать `task_crud.rs` на использование кастомного парсинга

### Приоритет 5: Job trait (1 ошибка)
1. Реализовать `Job` trait для `LocalJob`

## 📝 Заметки

### Удаление BoltDB
В сессии 5 было принято решение удалить BoltDB реализацию, так как:
- BoltDB Go-библиотека, не имеет прямого аналога в Rust
- Существующая реализация через sled имела множество проблем
- SQL базы данных (SQLite, MySQL, PostgreSQL) полностью покрывают потребности

### Прогресс
- Сессия 1-3: Исправлено ~200 ошибок (модели, трейты, конфигурация)
- Сессия 4: Исправлено ~159 ошибок (System Process, Default, ProjectUser)
- Сессия 5: Исправлено ~61 ошибка (удаление BoltDB, конфигурация, инициализаторы)
- **Всего исправлено: 420 из 585 (71.8%)**

### Следующие вехи
- **< 100 ошибок:** Реализация SQLx трейтов для всех моделей
- **< 50 ошибок:** Завершение работы с Git Client
- **< 10 ошибок:** Финальная полировка и тесты
- **0 ошибок:** Первая успешная сборка!
