# Отчёт об ошибках сборки Semaphore Rust
Дата: 2026-03-02
Последнее обновление: 2026-03-03 (сессия 3)

## Статистика
- **Начальное количество ошибок:** 585
- **Текущее количество ошибок:** 237
- **Исправлено ошибок:** 348 (59.5%)
- **Предупреждений:** 288

## ✅ Исправленные категории ошибок

### 1. BoltDB API - ИСПРАВЛЕНО ПОЛНОСТЬЮ
- ✅ Заменены все методы `bucket()`, `create_bucket_if_not_exists()`, `delete_bucket()` на sled API
- ✅ Использован `open_tree()` вместо bucket операций
- ✅ Реализован метод `get_object_refs()` для получения ссылок на объекты
- ✅ Исправлены файлы:
  * `rust/src/db/bolt/event.rs` - операции с событиями
  * `rust/src/db/bolt/user.rs` - операции с пользователями
  * `rust/src/db/bolt/task.rs` - операции с задачами
  * `rust/src/db/bolt/template.rs` - операции с шаблонами
  * `rust/src/db/bolt/schedule.rs` - операции с расписанием
  * `rust/src/db/bolt/session.rs` - операции с сессиями
  * `rust/src/db/bolt/project.rs` - операции с проектами
  * `rust/src/db/bolt/access_key.rs` - операции с ключами доступа
  * `rust/src/db/bolt/bolt_db.rs` - добавлен метод get_object_refs

### 2. Модели данных - ИСПРАВЛЕНО ЧАСТИЧНО
- ✅ `TemplateType` - добавлены варианты: Deploy, Task, Ansible, Terraform, Shell
- ✅ `AccessKeyOwner` - добавлен вариант: Shared
- ✅ `Inventory` - исправлено поле: inventory → inventory_type
- ✅ `Repository` - добавлено поле: git_branch, key_id
- ✅ `Schedule` - добавлены поля: cron_format, last_commit_hash, repository_id, created
- ✅ `View` - добавлен алиас name для title
- ✅ `Environment` - добавлено поле: secrets
- ✅ `Task` - добавлены поля: repository_id, environment_id
- ✅ `TaskOutput` - добавлено поле: project_id
- ✅ `TaskStage` - добавлено поле: project_id
- ✅ `IntegrationMatcher` - добавлены поля: project_id, matcher_type, matcher_value
- ✅ `IntegrationExtractValue` - добавлены поля: project_id, value_name, value_type
- ✅ `Role` - добавлены поля: id, project_id
- ✅ `ProjectInvite` - добавлены поля: token, inviter_user_id
- ✅ `AccessKey` - добавлены поля: owner, environment_id
- ✅ `IntegrationAlias` - добавлено поле: project_id
- ✅ `TerraformTaskParams` - добавлены поля: backend_init_required, backend_config, workspace
- ✅ `TemplateFilter` - добавлено поле: view_id
- ✅ `Template` - добавлены поля: vault_key_id, become_key_id, app, deleted, project_id, Default
- ✅ `UserTotp/UserEmailOtp` - убран FromRow (не нужны для SQLx)

### 3. Конфигурация - ИСПРАВЛЕНО
- ✅ `Config` - добавлены методы: from_env(), database_url(), db_path(), db_dialect(), non_admin_can_create_project()
- ✅ `DbDialect` - исправлено: PostgreSQL → Postgres
- ✅ `DbConfig` - добавлены поля: path, connection_string

### 4. Store Trait - ИСПРАВЛЕНО ЧАСТИЧНО
- ✅ Добавлен `SecretStorageManager` trait
- ✅ Реализован для `SqlStore`
- ✅ Реализован для `BoltStore`
- ✅ Добавлен метод `get_object_refs()` в BoltStore

### 5. TaskLogger Clone - ИСПРАВЛЕНО
- ✅ Изменено `Box<dyn TaskLogger>` на `Arc<dyn TaskLogger>` для поддержки Clone
- ✅ Исправлены файлы: ansible_app.rs, terraform_app.rs

### 6. AccessKey методы - ИСПРАВЛЕНО
- ✅ Добавлены методы в ssh_agent::AccessKey: get_type(), get_ssh_key_data(), get_login_password_data()
- ✅ Добавлены helper методы в models::AccessKey: new_ssh(), new_login_password()

### 7. Инициализация моделей - ЧАСТИЧНО ИСПРАВЛЕНО
- ✅ `ProjectUser` - добавлено поле: created
- ✅ `Project` - добавлены поля в инициализацию
- ✅ `Template` - добавлены поля в инициализацию (templates.rs, restore.rs)
- ✅ `Task` - добавлены поля в инициализацию (tasks.rs, task_pool_status.rs)
- ✅ `APIToken` - добавлено поле: created
- ✅ `TaskWithTpl` - добавлено поле: build_task
- ⚠️ `Runner` - `project_id` is `Option<i32>` вместо `i32`

## 🔴 Текущие ошибки (237 осталось)

### 1. mismatched types (31 ошибка)
Проблемы с несоответствием типов в различных частях кода.

### 2. type annotations needed (12 ошибок)
Требуется явное указание типов для closure и generic параметров.

### 3. missing поля в моделях и инициализаторах
- ❌ `Task` - missing `environment_id`, `repository_id` в инициализаторах
- ❌ `TemplateFilter` - missing `app`, `deleted`, `project_id`
- ❌ `Schedule` - missing `created` в инициализаторах
- ❌ `TaskOutput` - missing `project_id` в инициализаторах
- ❌ `ProjectInviteWithUser` - неправильная структура полей

### 4. missing trait implementations
- ❌ `get_project_users` - метод не реализован в Store trait
- ❌ `Task` - не реализованы `sqlx::Decode`, `sqlx::Type`
- ❌ `SecretStorage` - не реализован `FromRow`
- ❌ `ProjectUserRole` - не реализованы `Type`, `Decode` для SQLx
- ❌ `TemplateType`, `AccessKeyOwner` - не реализован `FromStr`
- ❌ `LocalJob` - не реализует `Job` trait
- ❌ `ExporterChain`, `ValueMap<T>` - не реализуют `DataExporter`, `TypeExporter`

### 5. Git Client проблемы
- ❌ `DbRepository::ssh_key` - поле отсутствует (нужно загружать через key_id)
- ❌ `Repository::get_full_path` - метод не найден
- ❌ `BuildRepo` - тип не найден

### 6. TemplateType Display
- ❌ `Option<TemplateType>` не реализует `Display`

### 7. Config поля
- ❌ `HAConfig` - missing поле `ha`
- ❌ `AccessKey` - missing поля `secret_type`, `secret`, `login_password`, `key_type`, `override_secret`
- ❌ `Inventory` - missing поле `variables`
- ❌ `Template` - missing поля `hooks`, `params`
- ❌ `ProjectUser` - missing поля `username`, `name`
- ❌ `BackupProject` - missing поля `type`, `default_secret_storage_id`
- ❌ `BackupTemplate` - missing поля `description`, `build_version`, `start_version`
- ❌ `BackupRepository` - missing поле `git_type`

### 8. method argument mismatches
- ❌ "this method takes 4 arguments but 3 were supplied" (5 ошибок)
- ❌ "this method takes 1 argument but 2 arguments were supplied" (4 ошибки)
- ❌ "this method takes 2 arguments but 1 argument was supplied" (3 ошибки)

### 9. missing методы
- ❌ `Template::validate`, `Template::extract_params`
- ❌ `AccessKey::validate`
- ❌ `get_project_user`, `get_template_users`, `get_task_alert_chat`
- ❌ `get_project_schedules`
- ❌ `create_user_without_password`, `create_task`, `delete_task`, `get_tasks`, `get_task`
- ❌ `update_task_status`
- ❌ `get_url` (для Task)
- ❌ `destroy` (для AccessKeyInstallation)
- ❌ `get_full_path` (для Repository)
- ❌ `as_str` (для i64)
- ❌ `Default::default()` для `Repository`, `Inventory`, `Environment`, `HARedisConfig`

### 10. SQLx type compatibility
- ❌ `TaskWithTpl::fetch_all` - trait bounds не satisfied
- ❌ `SecretStorage::fetch_optional`, `fetch_all` - trait bounds не satisfied
- ❌ `String: Type<DB>`, `String: Decode<DB>` - не реализованы
- ❌ `ProjectUserRole: Type<Sqlite>`, `ProjectUserRole: Decode<Sqlite>` - не реализованы

### 11. Clone trait
- ❌ `RunningTask` не реализует `Clone`
- ❌ `dyn Any + Send + Sync: Clone` не реализован
- ❌ `AccessKeyInstallerImpl` не реализует `Clone`
- ❌ `dyn FnOnce(u32) + Send: Clone` не реализован

### 12. Async/Sync issues
- ❌ future cannot be sent between threads safely
- ❌ Async методы в синхронном контексте

### 13. Formatting errors
- ❌ `Option<usize>` не реализует `Display`
- ❌ `Option<String>` не реализует `Display`
- ❌ `[u8; 16]` не реализует `UpperHex`/`LowerHex`

### 14. Crate dependencies
- ❌ `which` crate - unresolved import
- ❌ `libc` crate - unresolved import

### 15. Прочие ошибки
- ❌ `ConflictableTransactionError::TransactionError` - variant not found
- ❌ `EventType::Task` - variant not found
- ❌ `AccessKeyType::Ssh` - variant not found
- ❌ Cast `Option<i32>` as `usize`
- ❌ Move out of shared reference (`config.database.dialect`)
- ❌ Use of moved value
- ❌ Cannot assign to behind `&` reference
- ❌ Use of unstable library feature `str_as_str`

#### Несоответствие типов SQLx Decode/Encode (E0277)
Проблемы с реализацией трейтов для SQLx:
- `UserTotp` - не реализованы `sqlx::Decode`, `sqlx::Type`
- `UserEmailOtp` - не реализованы `sqlx::Decode`, `sqlx::Type`
- `Task` - не реализованы `sqlx::Decode`, `sqlx::Type` (из-за `HashMap<String, JsonValue>`)
- `TaskWithTpl` - не реализован `FromRow`
- `ProjectInvite` - не реализованы `sqlx::Decode`, `sqlx::Type`
- `TemplateType` - `Option<TemplateType>` не реализует `Display`

#### Неправильные типы полей
- `params.offset` - `usize` вместо `Option<usize>`
- `template.template_type` - `Option<TemplateType>` вместо `TemplateType`
- `task.repository_id`, `task.environment_id` - поля отсутствуют
- `inventory.inventory_type` - поле отсутствует, есть `inventory_data`
- `repository.git_branch` - поле отсутствует
- `environment.env` - поле отсутствует, есть `json`
- `access_key.owner` - поле отсутствует
- `schedule.cron_format` - поле отсутствует

### 2. Проблемы trait implementation (Trait Errors)
#### Job trait (E0050)
Метод `Job::run` требует 4 параметра, но реализации имеют 1:
- `LocalJob::run`
- `AnsibleJob::run`
- `TerraformJob::run`
- `ShellJob::run`

#### Store trait (E0599)
- `Box<dyn Store>` не реализует `Clone`
- Отсутствуют методы: `get_project_users`, `get_secret_storages`, `get_secret_storage`, `create_secret_storage`, `update_secret_storage`, `delete_secret_storage`, `get_template_users`, `get_task_alert_chat`

#### LocalApp trait (E0277)
- `AnsibleApp` не реализует `LocalApp`
- `TerraformApp` не реализует `LocalApp`

#### Exporter traits (E0277)
- `ExporterChain` не реализует `DataExporter`
- `ValueMap<T>` не реализует `TypeExporter`

### 3. Проблемы Git клиента (Git Client Errors)
#### GoGitClient implementation (E0195, E0053)
Несоответствие сигнатур методов трейту `GitClient`:
- `clone` - lifetime параметры не совпадают
- `pull` - lifetime параметры не совпадают
- `checkout` - lifetime параметры не совпадают
- `can_be_pulled` - тип параметра `GitRepository` вместо `&GitRepository`
- `get_last_commit_message` - lifetime параметры не совпадают
- `get_last_commit_hash` - lifetime параметры не совпадают
- `get_last_remote_commit_hash` - lifetime параметры не совпадают
- `get_remote_branches` - lifetime параметры не совпадают

#### Missing methods
- `Repository::get_full_path` - метод не найден
- `Template::extract_params` - метод не найден
- `Template::validate` - метод не найден
- `AccessKey::validate` - метод не найден

### 4. Проблемы BoltDB (BoltDB Errors)
#### Missing methods
- `Db::update` - метод не найден
- `Db::view` - метод не найден
- `BoltStore::get_project_user` - метод не найден
- `BoltStore::get_object_refs` - метод не найден

#### Type errors
- `Sized` не реализован для `[u8]` в контексте BoltDB transactions
- `ProjectInviteWithUser` - неправильная структура полей
- `ScheduleWithTpl` - missing `template_name`
- `TemplateWithPerms` - missing `permissions`
- `TaskStageWithResult` - неправильная структура полей

### 5. Проблемы CLI и конфигурации (CLI/Config Errors)
#### Config fields
- `Config::non_admin_can_create_project` - поле отсутствует
- `Config::db_dialect` - поле отсутствует
- `Config::db_path` - поле отсутствует
- `Config::database_url()` - метод не найден
- `DbDialect::PostgreSQL` - варианта нет (есть `Postgres`)

#### Missing dependencies
- `which` crate - не добавлена в Cargo.toml
- `libc` crate - используется но не импортирована

#### Config methods
- `Config::from_env` - метод не найден
- `HARedisConfig::default` - не реализован

### 6. Проблемы API handlers (API Errors)
#### State extractor (E0308)
- `axum::extract::State` используется неправильно
- `state.store.clone()` - `Box<dyn Store>` не реализует `Clone`

#### RetrieveQueryParams (E0308)
- Неправильное использование в методах store
- `api::users::RetrieveQueryParams` vs `store::RetrieveQueryParams`

#### Method signature mismatches
- `get_events` - неправильные параметры (limit: usize вместо RetrieveQueryParams)
- `get_access_keys` - лишние параметры
- `get_integrations` - лишние параметры
- `get_options` - лишние параметры
- `get_template` - missing `project_id` параметр

### 7. Проблемы сервисов (Service Errors)
#### Task Runner
- `Job` trait требует 4 параметра в `run`
- `LocalJob` не реализует `Job`
- `RunningTask` не реализует `Clone`
- `TaskLogger` не реализует `Clone`

#### Backup/Restore
- `BackupFormat` поля не соответствуют моделям
- `RestoreDB` поля не соответствуют моделям
- Асинхронные методы вызываются в синхронном контексте

#### Exporter
- `ExporterChain` не реализует требуемые трейты
- `ValueMap<T>` не реализует `TypeExporter`

### 8. Проблемы TemplateType (TemplateType Errors)
#### Missing variants
- `TemplateType::Ansible` - варианта нет
- `TemplateType::Terraform` - варианта нет
- `TemplateType::Shell` - варианта нет
- `TemplateType::Task` - варианта нет
- `TemplateType::Deploy` - варианта нет
- `TemplateType::Build` - варианта нет

### 9. Проблемы AccessKey (AccessKey Errors)
#### Missing variants
- `AccessKeyType::Ssh` - варианта нет (есть `SSH`)
- `AccessKeyOwner::Shared` - варианта нет

#### Missing fields
- `key_type` - поле отсутствует
- `login_password` - поле отсутствует
- `access_key` - поле отсутствует
- `environment_id` - поле отсутствует
- `owner` - поле отсутствует
- `override_secret` - поле отсутствует
- `created` - поле отсутствует

### 10. Проблемы Task (Task Errors)
#### Missing fields
- `repository_id` - поле отсутствует
- `environment_id` - поле отсутствует
- `params` - тип `Option<HashMap<String, Value>>` вместо ожидаемого

### 11. Проблемы FFI (FFI Errors)
#### Store boxing (E0277)
- `Box<dyn Store>` не может быть преобразован в `Box<dyn Store + Send + Sync>`
- `Arc<dyn Store>` не может быть преобразован в `Box<dyn Store>`

### 12. Проблемы SQLx типов (SQLx Type Errors)
#### HashMap encoding
- `HashMap<String, JsonValue>` не реализует `sqlx::Encode`, `sqlx::Type`

#### Option<usize> formatting
- `Option<usize>` не реализует `Display` для format!()

### 13. Проблемы Ansible/Terraform (Ansible/Terraform Errors)
#### Missing fields
- `TerraformTaskParams::backend_init_required` - поле отсутствует
- `TerraformTaskParams::backend_config` - поле отсутствует
- `TerraformTaskParams::workspace` - поле отсутствует
- `Inventory::variables` - поле отсутствует
- `Template::hooks` - поле отсутствует
- `Template::params` - поле отсутствует
- `Repository::ssh_key` - поле отсутствует

#### Type mismatches
- `tokio::process::Command` vs `std::process::Command`
- Callback типы не совпадают

### 14. Проблемы Project Invite (Project Invite Errors)
#### Missing fields
- `ProjectInvite::token` - поле отсутствует
- `ProjectInvite::inviter_user_id` - поле отсутствует
- `ProjectInviteWithUser` - неправильная структура

### 15. Проблемы Schedule (Schedule Errors)
#### Missing fields
- `Schedule::cron_format` - поле отсутствует
- `Schedule::last_commit_hash` - поле отсутствует
- `Schedule::repository_id` - поле отсутствует

### 16. Проблемы View (View Errors)
#### Missing fields
- `View::name` - поле отсутствует (есть `title`)

### 17. Проблемы Environment (Environment Errors)
#### Missing fields
- `Environment::env` - поле отсутствует (есть `json`)
- `Environment::secrets` - поле отсутствует

### 18. Проблемы Integration (Integration Errors)
#### Missing fields
- `IntegrationMatcher::project_id` - поле отсутствует
- `IntegrationMatcher::matcher_type` - поле отсутствует
- `IntegrationMatcher::matcher_value` - поле отсутствует
- `IntegrationExtractValue::project_id` - поле отсутствует
- `IntegrationExtractValue::value_name` - поле отсутствует
- `IntegrationExtractValue::value_type` - поле отсутствует

### 19. Проблемы Role (Role Errors)
#### Missing fields
- `Role::id` - поле отсутствует
- `Role::project_id` - поле отсутствует

### 20. Проблемы Runner (Runner Errors)
#### Option type
- `Runner::project_id` - `Option<i32>` вместо `i32`

### 21. Проблемы Async/Sync (Async/Sync Errors)
#### Sync bound
- future cannot be sent between threads safely (BoltDB filter)

#### Async in sync context
- `restore()` методы синхронные, но вызывают асинхронные store методы

### 22. Проблемы Clone trait (Clone Errors)
- `RunningTask` не реализует `Clone`
- `TaskLogger` не реализует `Clone`
- `Box<dyn Store>` не реализует `Clone`
- `Box<dyn TaskLogger>` не реализует `Clone`
- `AccessKeyInstallerImpl` не реализует `Clone`

### 23. Проблемы форматирования (Formatting Errors)
#### Display trait
- `Option<TemplateType>` не реализует `Display`
- `[u8; 16]` не реализует `UpperHex`/`LowerHex`
- `Option<usize>` не реализует `Display`

### 24. Missing crate dependencies
- `which` - не добавлена
- `libc` - используется но не импортирована явно

### 25. Проблемы пропущенных полей в инициализаторах
Множественные структуры инициализированы с неправильным набором полей.
