# План исправления ошибок сборки Semaphore Rust
Дата: 2026-03-02
Последнее обновление: 2026-03-02 (сессия 2)

## Прогресс

### ✅ Выполнено (Этап 1-3 частично)
- [x] Исправлены все основные модели данных
- [x] Добавлены missing поля в Inventory, Repository, Schedule, View, Environment, Task, TaskStage
- [x] Исправлены TemplateType, AccessKeyOwner, AccessKey
- [x] Добавлены методы в Config
- [x] Исправлен DbDialect::PostgreSQL → Postgres
- [x] Добавлен SecretStorageManager в Store trait
- [x] Реализован SecretStorageManager для SqlStore и BoltStore
- [x] Исправлена инициализация ProjectUser, Project, Template, Task в API handlers
- [x] Убран FromRow из UserTotp/UserEmailOtp

### ⏳ В процессе
- [ ] Исправить инициализацию Task в сервисах (scheduler.rs, task_pool*.rs, local_job/*.rs)
- [ ] Исправить инициализацию Template в db/sql/template_crud.rs
- [ ] Исправить BoltDB update/view методы

### 🔴 Критические ошибки (557 осталось)
1. **BoltDB** - `update`/`view` методы (41 ошибка)
2. **type annotations** - нужно указать типы (105 ошибок)
3. **mismatched types** - несоответствие типов (67 ошибок)
4. **Job::run** - неправильная сигнатура метода
5. **GitClient** - lifetime параметры

---

## Приоритеты исправлений

### 🔴 КРИТИЧЕСКИЙ ПРИОРИТЕТ (Блокируют компиляцию)

#### 1. Исправление моделей данных (Model Structures)
**Файлы:** `src/models/*.rs`

##### 1.1 Добавить отсутствующие поля в структуры

**User (`src/models/user.rs`):**
- [ ] Добавить поле `created: chrono::DateTime<chrono::Utc>` в `User`
- [ ] Добавить поле `created: chrono::DateTime<chrono::Utc>` в `APIToken`

**Task (`src/models/task.rs`):**
- [ ] Добавить поля `repository_id: Option<i32>`, `environment_id: Option<i32>`
- [ ] Исправить тип `params` для совместимости с SQLx
- [ ] Реализовать `Clone` для `RunningTask`

**AccessKey (`src/models/access_key.rs`):**
- [ ] Добавить/исправить поля согласно миграции
- [ ] Проверить enum `AccessKeyType` (SSH вместо Ssh)
- [ ] Проверить enum `AccessKeyOwner`

**Template (`src/models/template.rs`):**
- [ ] Добавить варианты в `TemplateType`: Ansible, Terraform, Shell, Task, Deploy, Build
- [ ] Добавить отсутствующие поля: `arguments`, `build_version`, `start_version`
- [ ] Реализовать `Default` если требуется

**Inventory (`src/models/inventory.rs`):**
- [ ] Исправить поле `inventory_type` → `inventory_data`
- [ ] Добавить `ssh_key_id`, `vaults`, `become_key_id`

**Repository (`src/models/repository.rs`):**
- [ ] Добавить поле `git_branch: String`
- [ ] Добавить поле `ssh_key_id: Option<i32>` или `ssh_key`

**Schedule (`src/models/schedule.rs`):**
- [ ] Добавить поле `cron_format: Option<String>`
- [ ] Добавить поле `last_commit_hash: Option<String>`
- [ ] Добавить поле `repository_id: Option<i32>`

**View (`src/models/view.rs`):**
- [ ] Исправить поле `title` → `name` или наоборот в использовании

**Environment (`src/models/environment.rs`):**
- [ ] Исправить поле `json` → `env` или наоборот
- [ ] Добавить поле `secrets: Vec<?>`

**Project (`src/models/project.rs`):**
- [ ] Добавить поля: `alert`, `alert_chat`, `default_secret_storage_id`, `type`

**ProjectInvite (`src/models/project_invite.rs`):**
- [ ] Добавить поле `token: String`
- [ ] Добавить поле `inviter_user_id: i32`
- [ ] Исправить структуру `ProjectInviteWithUser`

**Integration (`src/models/integration.rs`):**
- [ ] Добавить поля в `IntegrationMatcher`: `project_id`, `matcher_type`, `matcher_value`
- [ ] Добавить поля в `IntegrationExtractValue`: `project_id`, `value_name`, `value_type`

**Role (`src/models/role.rs`):**
- [ ] Добавить поля: `id: i32`, `project_id: i32`

**Runner (`src/models/runner.rs`):**
- [ ] Исправить тип `project_id: Option<i32>` → `project_id: i32`

**TaskOutput/TaskStage (`src/models/task.rs`):**
- [ ] Добавить поле `stage_id: i32` в `TaskOutput`
- [ ] Добавить поле `project_id: i32` в `TaskOutput`
- [ ] Добавить поле `stage_type: String` в `TaskStage`
- [ ] Добавить поле `project_id: i32` в `TaskStage`

##### 1.2 Реализация трейтов SQLx

**UserTotp/UserEmailOtp:**
- [ ] Реализовать `sqlx::Type` и `sqlx::Decode` или убрать из `User`

**Task/TaskWithTpl:**
- [ ] Реализовать совместимость `HashMap<String, JsonValue>` с SQLx
- [ ] Использовать `serde_json::Value` вместо `HashMap`

**TemplateType:**
- [ ] Реализовать `Display` для `Option<TemplateType>` или использовать unwrap/or_else

---

#### 2. Исправление Trait Implementations
**Файлы:** `src/services/*.rs`, `src/db_lib/*.rs`

##### 2.1 Job Trait
**Файл:** `src/services/task_runner/types.rs`, `src/services/job.rs`

- [ ] Исправить сигнатуру `Job::run` во всех реализациях:
  ```rust
  fn run(&mut self, username: &str, incoming_version: Option<&str>, alias: &str) -> Result<(), Error>;
  ```
- [ ] Обновить `LocalJob`, `AnsibleJob`, `TerraformJob`, `ShellJob`

##### 2.2 LocalApp Trait
**Файл:** `src/db_lib/local_app.rs`

- [ ] Реализовать `LocalApp` для `AnsibleApp`
- [ ] Реализовать `LocalApp` для `TerraformApp`
- [ ] Исправить конструкторы `AnsibleApp::new`, `TerraformApp::new`

##### 2.3 Store Trait
**Файл:** `src/db/store.rs`, `src/db/sql/mod.rs`, `src/db/bolt/mod.rs`

- [ ] Добавить методы в трейт `Store`:
  - `get_project_users`
  - `get_secret_storages`
  - `get_secret_storage`
  - `create_secret_storage`
  - `update_secret_storage`
  - `delete_secret_storage`
  - `get_template_users`
  - `get_task_alert_chat`
- [ ] Реализовать методы в `SqlStore` и `BoltStore`
- [ ] Исправить `Clone` для `Box<dyn Store>` - использовать `Arc` вместо клонирования

##### 2.4 Exporter Traits
**Файл:** `src/services/exporter.rs`, `src/services/exporter_main.rs`

- [ ] Реализовать `DataExporter` для `ExporterChain`
- [ ] Реализовать `TypeExporter` для `ValueMap<T>`

---

#### 3. Git Client Fixes
**Файлы:** `src/db_lib/go_git_client.rs`, `src/db_lib/cmd_git_client.rs`

- [ ] Исправить lifetime параметры в `GoGitClient` методах
- [ ] Исправить типы параметров (`&GitRepository` вместо `GitRepository`)
- [ ] Реализовать `Repository::get_full_path` метод
- [ ] Исправить использование `git2::Repository`

---

#### 4. BoltDB Fixes
**Файлы:** `src/db/bolt/*.rs`

- [ ] Реализовать/исправить `Db::update` и `Db::view` методы
- [ ] Исправить `BoltStore::get_project_user`
- [ ] Исправить `BoltStore::get_object_refs`
- [ ] Исправить работу с `sled` transactions
- [ ] Исправить структуры `ProjectInviteWithUser`, `ScheduleWithTpl`, `TemplateWithPerms`, `TaskStageWithResult`

---

#### 5. Config & CLI Fixes
**Файлы:** `src/config/*.rs`, `src/cli/*.rs`

##### 5.1 Config Structure
- [ ] Добавить поле `non_admin_can_create_project: bool` в `Config`
- [ ] Добавить поле/метод `db_dialect` в `Config`
- [ ] Добавить поле/метод `db_path` в `Config`
- [ ] Добавить метод `database_url()` в `Config`
- [ ] Исправить `DbDialect::PostgreSQL` → `DbDialect::Postgres`
- [ ] Реализовать `Config::from_env()`
- [ ] Реализовать `Default` для `HARedisConfig`

##### 5.2 Dependencies
- [ ] Добавить `which = "4"` в `Cargo.toml`
- [ ] Проверить `libc` импорт

---

#### 6. API Handlers Fixes
**Файлы:** `src/api/*.rs`, `src/api/handlers/*.rs`

##### 6.1 State Extractor
- [ ] Исправить использование `axum::extract::State`
- [ ] Заменить `state.store.clone()` на правильное использование `Arc`

##### 6.2 RetrieveQueryParams
- [ ] Унифицировать использование `RetrieveQueryParams`
- [ ] Исправить вызовы методов store с правильными параметрами

##### 6.3 Method Signatures
- [ ] Исправить `get_events` signature
- [ ] Исправить `get_access_keys` - убрать лишние параметры
- [ ] Исправить `get_integrations` - убрать лишние параметры
- [ ] Исправить `get_options` - убрать лишние параметры
- [ ] Исправить `get_template` - добавить `project_id`

---

#### 7. Service Layer Fixes
**Файлы:** `src/services/*.rs`

##### 7.1 Task Runner
- [ ] Исправить `Job::run` вызовы с правильными параметрами
- [ ] Реализовать `Clone` для `RunningTask`
- [ ] Реализовать `Clone` для `TaskLogger` или использовать `Arc`
- [ ] Исправить `AccessKeyInstallerImpl::clone`

##### 7.2 Backup/Restore
- [ ] Исправить структуры `BackupFormat`, `RestoreDB`
- [ ] Сделать `restore()` методы асинхронными
- [ ] Исправить соответствие полей в backup/restore

##### 7.3 Task Pool
- [ ] Исправить `update_task_status` → `update_task`
- [ ] Исправить `get_task_outputs` signature
- [ ] Исправить `max_parallel_tasks` cast

---

#### 8. TemplateType & App Factory
**Файлы:** `src/models/template.rs`, `src/db_lib/app_factory.rs`

- [ ] Добавить все варианты в `TemplateType`
- [ ] Исправить `TemplateApp` - реализовать `Display`
- [ ] Исправить `app_factory.rs` - правильные конструкторы

---

#### 9. SQLx Type Implementations
**Файлы:** `src/models/*.rs`

- [ ] Реализовать `Type` и `Encode/Decode` для кастомных типов
- [ ] Исправить использование `HashMap` → `serde_json::Value`
- [ ] Исправить `Option<usize>` formatting

---

#### 10. FFI Fixes
**Файлы:** `src/ffi/*.rs`

- [ ] Исправить преобразование `Box<dyn Store>` → `Box<dyn Store + Send + Sync>`
- [ ] Исправить преобразование `Arc<dyn Store>` → `Box<dyn Store>`

---

### 🟡 ВЫСОКИЙ ПРИОРИТЕТ

#### 11. Предупреждения компилятора (Warnings)
- [ ] Удалить неиспользуемые импорты (277 warnings)
- [ ] Удалить неиспользуемые переменные
- [ ] Исправить `unused_mut`
- [ ] Исправить `dead_code`

#### 12. Async/Sync Issues
- [ ] Исправить `Send` bound для futures
- [ ] Исправить async вызовы в синхронном контексте

---

### 🟢 СРЕДНИЙ ПРИОРИТЕТ

#### 13. Code Quality
- [ ] Унифицировать обработку ошибок
- [ ] Улучшить обработку `Option` типов
- [ ] Добавить документацию

---

## План работ по этапам

### Этап 1: Модели данных (4-6 часов)
1. Исправить все структуры моделей
2. Добавить отсутствующие поля
3. Реализовать необходимые трейты

### Этап 2: Trait Implementations (3-4 часа)
1. Исправить `Job` trait
2. Исправить `LocalApp` trait
3. Исправить `Store` trait
4. Исправить `Exporter` traits

### Этап 3: DB Layer (3-4 часа)
1. Исправить SQL queries
2. Исправить BoltDB implementation
3. Исправить Git client

### Этап 4: API & Services (3-4 часа)
1. Исправить API handlers
2. Исправить сервисы
3. Исправить CLI

### Этап 5: Финальная сборка (1-2 часа)
1. Исправление оставшихся ошибок
2. Удаление предупреждений
3. Тестирование сборки

---

## Оценка времени
**Общее время:** 14-20 часов

---

## Зависимости
1. Этап 1 должен быть выполнен первым (модели используются везде)
2. Этап 2 зависит от Этапа 1
3. Этап 3 зависит от Этапов 1 и 2
4. Этап 4 зависит от Этапов 1-3
5. Этап 5 выполняется после всех остальных
