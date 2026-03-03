# План исправления ошибок сборки Semaphore Rust
Дата: 2026-03-02
Последнее обновление: 2026-03-03 (сессия 3)

## Прогресс

### ✅ Выполнено
- [x] Исправлены все основные модели данных (частично)
- [x] Добавлены missing поля в Inventory, Repository, Schedule, View, Environment, Task, TaskStage
- [x] Исправлены TemplateType, AccessKeyOwner, AccessKey
- [x] Добавлены методы в Config
- [x] Исправлен DbDialect::PostgreSQL → Postgres
- [x] Добавлен SecretStorageManager в Store trait
- [x] Реализован SecretStorageManager для SqlStore и BoltStore
- [x] Исправлена инициализация ProjectUser, Project, Template, Task в API handlers
- [x] Убран FromRow из UserTotp/UserEmailOtp
- [x] **BoltDB API полностью исправлен** (45 ошибок)
- [x] Добавлен метод `get_object_refs()` в BoltStore
- [x] TaskLogger Clone исправлен (Box → Arc)
- [x] AccessKey методы добавлены в ssh_agent.rs
- [x] Добавлены поля в DbConfig (path, connection_string)
- [x] Добавлен Default для Template
- [x] Добавлены поля в Template, Schedule, IntegrationAlias, TerraformTaskParams, TaskOutput

### 🔴 Критические ошибки (237 осталось)
1. **mismatched types** - 31 ошибка
2. **type annotations** - 12 ошибок
3. **missing поля в инициализаторах** - 15 ошибок
4. **missing trait implementations** - 25 ошибок
5. **Git Client** - 8 ошибок
6. **SQLx type compatibility** - 20 ошибок

---

## Приоритеты исправлений

### 🔴 КРИТИЧЕСКИЙ ПРИОРИТЕТ (Блокируют компиляцию)

#### 1. Исправление инициализаторов моделей
**Файлы:** `src/db/bolt/*.rs`, `src/db/sql/*.rs`, `src/services/*.rs`

**Task:**
- [ ] Добавить `environment_id`, `repository_id` в инициализаторы Task

**TemplateFilter:**
- [ ] Добавить `app`, `deleted`, `project_id` в инициализаторы

**Schedule:**
- [ ] Добавить `created` в инициализаторы

**TaskOutput:**
- [ ] Добавить `project_id` в инициализаторы

**ProjectInviteWithUser:**
- [ ] Исправить структуру полей

#### 2. Реализация missing trait methods
**Файлы:** `src/db/store.rs`, `src/db/sql/*.rs`, `src/db/bolt/*.rs`

- [ ] Реализовать `get_project_users` в Store trait
- [ ] Реализовать `get_template_users` в Store trait
- [ ] Реализовать `get_task_alert_chat` в Store trait
- [ ] Реализовать `get_project_schedules` в Store trait
- [ ] Реализовать `create_user_without_password` в Store trait
- [ ] Реализовать `create_task`, `get_task`, `delete_task`, `get_tasks` в Store trait

#### 3. SQLx Type Implementations
**Файлы:** `src/models/*.rs`

- [ ] Реализовать `FromRow` для `SecretStorage`
- [ ] Реализовать `Type`, `Decode` для `ProjectUserRole`
- [ ] Реализовать `FromStr` для `TemplateType`, `AccessKeyOwner`
- [ ] Реализовать `Type`, `Decode` для `Task`

#### 4. Job Trait
**Файлы:** `src/services/job.rs`, `src/services/local_job/*.rs`

- [ ] Реализовать `Job` trait для `LocalJob`
- [ ] Исправить сигнатуру `Job::run`

#### 5. Git Client
**Файлы:** `src/db_lib/go_git_client.rs`, `src/db_lib/cmd_git_client.rs`, `src/services/git_repository.rs`

- [ ] Добавить метод `get_full_path()` для `Repository`
- [ ] Исправить использование `ssh_key` → загрузка через `key_id`
- [ ] Исправить `BuildRepo` → `git2::Repository`

#### 6. TemplateType Display
**Файлы:** `src/models/template.rs`

- [ ] Реализовать `Display` для `Option<TemplateType>`
- [ ] Или использовать `.map(|t| t.to_string()).unwrap_or_default()`

#### 7. Missing поля в моделях
**Файлы:** `src/models/*.rs`, `src/services/backup.rs`

- [ ] `HAConfig` - добавить поле `ha`
- [ ] `AccessKey` - добавить поля `secret_type`, `secret`, `login_password`, `key_type`, `override_secret`
- [ ] `Inventory` - добавить поле `variables`
- [ ] `Template` - добавить поля `hooks`, `params`
- [ ] `ProjectUser` - добавить поля `username`, `name`
- [ ] `BackupProject` - добавить поля `type`, `default_secret_storage_id`
- [ ] `BackupTemplate` - добавить поля `description`, `build_version`, `start_version`
- [ ] `BackupRepository` - добавить поле `git_type`

#### 8. Missing методы
**Файлы:** `src/models/template.rs`, `src/models/access_key.rs`, `src/db/store.rs`

- [ ] `Template::validate`, `Template::extract_params`
- [ ] `AccessKey::validate`
- [ ] `get_project_user`, `get_template_users`, `get_task_alert_chat`
- [ ] `update_task_status`
- [ ] `get_url` (для Task)
- [ ] `destroy` (для AccessKeyInstallation)
- [ ] `as_str` (для i64)

#### 9. Default implementations
**Файлы:** `src/models/*.rs`, `src/config/*.rs`

- [ ] `Default` для `Repository`
- [ ] `Default` для `Inventory`
- [ ] `Default` для `Environment`
- [ ] `Default` для `HARedisConfig`

#### 10. Clone implementations
**Файлы:** `src/services/*.rs`

- [ ] `Clone` для `RunningTask`
- [ ] `Clone` для `AccessKeyInstallerImpl`

#### 11. Crate dependencies
**Файлы:** `Cargo.toml`, `src/config/config_sysproc.rs`

- [ ] Проверить `which` crate в зависимостях
- [ ] Проверить `libc` crate в зависимостях

#### 12. Async/Sync issues
**Файлы:** `src/services/*.rs`, `src/db/bolt/*.rs`

- [ ] Исправить `Send` bound для futures
- [ ] Исправить async вызовы в синхронном контексте

#### 13. Formatting errors
**Файлы:** `src/models/*.rs`

- [ ] Реализовать `Display` для `Option<usize>`
- [ ] Реализовать `Display` для `Option<String>`
- [ ] Реализовать `UpperHex`/`LowerHex` для `[u8; 16]`

#### 14. Прочие ошибки
- [ ] `ConflictableTransactionError::TransactionError` - исправить на `TransactionError(String)`
- [ ] `EventType::Task` - добавить вариант
- [ ] `AccessKeyType::Ssh` - добавить вариант
- [ ] Cast `Option<i32>` as `usize` - исправить
- [ ] Move out of shared reference - исправить
- [ ] Use of moved value - исправить
- [ ] Cannot assign to behind `&` reference - исправить
- [ ] Use of unstable library feature `str_as_str` - исправить

---

## План работ по этапам

### Этап 1: Инициализаторы моделей (2-3 часа)
1. Исправить все инициализаторы Task, TemplateFilter, Schedule, TaskOutput
2. Исправить ProjectInviteWithUser структуру

### Этап 2: Trait Implementations (4-5 часов)
1. Реализовать missing методы в Store trait
2. Реализовать SQLx трейты для моделей
3. Реализовать Job trait для LocalJob

### Этап 3: Git Client (2-3 часа)
1. Добавить get_full_path() метод
2. Исправить загрузку SSH ключей
3. Исправить BuildRepo → git2

### Этап 4: Missing поля и методы (3-4 часа)
1. Добавить missing поля в модели
2. Добавить missing методы
3. Реализовать Default для моделей

### Этап 5: Финальная сборка (2-3 часа)
1. Исправление оставшихся ошибок
2. Удаление предупреждений
3. Тестирование сборки

---

## Оценка времени
**Общее время:** 13-18 часов

---

## Зависимости
1. Этап 1 должен быть выполнен первым (инициализаторы используются везде)
2. Этап 2 зависит от Этапа 1
3. Этап 3 зависит от Этапов 1 и 2
4. Этап 4 зависит от Этапов 1-3
5. Этап 5 выполняется после всех остальных

---

## Достижения сессии 3

### Исправлено ошибок: 118 (356 → 238)
- BoltDB API: 45 ошибок
- TaskLogger Clone: 3 ошибки
- AccessKey методы: 4 ошибки
- Модели поля: 20 ошибок
- Config: 10 ошибок
- Прочие: 36 ошибок

### Коммиты:
- `a45ea575` - fix: замена BoltDB API на sled API во всех файлах
- `4895ad30` - fix: исправление моделей, TaskLogger Clone, и различных ошибок
