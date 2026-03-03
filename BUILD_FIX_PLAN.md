# План исправления ошибок сборки Semaphore Rust
Дата: 2026-03-02
Последнее обновление: 2026-03-03 (сессия 5)

## Прогресс

### ✅ Сессия 5 - Выполнено
- [x] **Удаление BoltDB** - полностью удалена директория `src/db/bolt/`
- [x] Удалён `BoltStore` из всех импортов и CLI
- [x] Удалён `DbDialect::Bolt` из конфигурации
- [x] Исправлены инициализаторы `DbConfig` (path, connection_string)
- [x] Исправлены инициализаторы `Task` (environment_id, repository_id)
- [x] Исправлены инициализаторы `TaskOutput` (project_id)
- [x] Исправлены moved value ошибки (user.email, current_user)
- [x] Исправлено форматирование `[u8; 16]` в hex
- [x] Исправлен `HAConfig` доступ к `node_id`
- [x] Добавлен `Repository::get_full_path()`
- [x] Исправлен `Config::db_dialect()` и `non_admin_can_create_project()`

### ✅ Сессия 4 - Выполнено
- [x] System Process - libc → nix
- [x] Default реализации для Repository, Inventory, Environment, HARedisConfig
- [x] ProjectUser модель (username, name)
- [x] TaskStageType (InstallRoles → Init)

### ✅ Сессия 1-3 - Выполнено
- [x] BoltDB API (полностью)
- [x] Модели данных (частично)
- [x] Конфигурация
- [x] Store Trait
- [x] TaskLogger Clone
- [x] AccessKey методы

---

## Текущий статус

| Метрика | Значение |
|---------|----------|
| Начальное количество ошибок | 585 |
| Исправлено ошибок | 420 |
| Осталось ошибок | 165 |
| Процент выполнения | 71.8% |

---

## 🔴 Критические ошибки (165 осталось)

### Топ ошибок по количеству

| Ошибка | Количество | Файлы |
|--------|------------|-------|
| mismatched types | 24 | Различные |
| type annotations needed | 10 | Различные |
| no field `ssh_key` on `DbRepository` | 4 | go_git_client.rs, cmd_git_client.rs |
| no field `name`/`secret`/`secret_type` on `&String` | 10 | local_job/*.rs |
| dyn Any + Send + Sync: Clone | 4 | local_app.rs |
| no field `key_id` on `DbRepository` | 2 | go_git_client.rs |
| Task: sqlx::Decode/Type | 4 | task_crud.rs |
| SecretStorage: FromRow | 2 | mod.rs |
| ExporterChain: DataExporter | 4 | exporter*.rs |

---

## 📋 План сессии 6

### 🔴 ПРИОРИТЕТ 1: Git Client (6 ошибок)

**Проблема:** Код обращается к несуществующему полю `ssh_key` вместо `key_id`

#### Задачи:
- [ ] **go_git_client.rs** (строки 26, 30, 55, 56)
  - Заменить `repo.repository.ssh_key` на загрузку через `key_id`
  - Использовать `store.get_access_key(key_id)` для получения ключа
  
- [ ] **cmd_git_client.rs** (строки 256, 286)
  - Заменить `r.repository.key_id` на правильную загрузку ключа

#### Пример исправления:
```rust
// Было (неправильно):
let ssh_key = repo.repository.ssh_key;

// Стало (правильно):
let ssh_key = store.get_access_key(repo.repository.key_id).await?;
```

---

### 🔴 ПРИОРИТЕТ 2: Модели данных (10 ошибок)

**Проблема:** Неправильное использование типов - обращение к полям на `&String`

#### Задачи:
- [ ] **local_job/vault.rs** (строки 18, 29)
  - Исправить тип `vault` с `&String` на правильную структуру
  
- [ ] **local_job/environment.rs** (строки 113, 114)
  - Исправить тип `secret` с `&String` на `EnvironmentSecret`
  
- [ ] **local_job/args.rs** (строки 26, 27, 80, 84)
  - Исправить тип `secret` с `&String` на `EnvironmentSecret`

---

### 🔴 ПРИОРИТЕТ 3: mismatched types (24 ошибки)

#### Подзадачи:

**TemplateType match** (3 ошибки):
- [ ] **local_job/run.rs** (строки 71, 76, 81)
  - Исправить `match self.template.template_type` на `match self.template.template_type { Some(t) => ... }`

**TerraformTaskParams** (5 ошибок):
- [ ] **terraform_app.rs** (строки 140, 141, 228, 277, 278, 282)
  - Удалить использование несуществующих полей
  - Использовать доступные поля: `plan`, `auto_approve`

**Backup/Restore модели** (8 ошибок):
- [ ] **restore.rs** (строки 432, 433, 354, 358, 365, 366)
  - Исправить типы полей BackupProject, BackupTemplate
  
- [ ] **backup.rs** (строки 221, 222, 223, 233, 243, 269, 270, 271, 275, 290, 300)
  - Исправить типы полей

**Git Client callback** (2 ошибки):
- [ ] **ansible_app.rs** (строка 398)
  - Исправить тип callback с `FnOnce(u32)` на `Fn(&Child)`
  
- [ ] **ansible_playbook.rs** (строки 71, 96)
  - Исправить использование `tokio::process::Command` vs `std::process::Command`

---

### 🟡 ПРИОРИТЕТ 4: SQLx трейты (4 ошибки)

**Проблема:** `Task` не реализует `sqlx::Decode` и `sqlx::Type`

#### Задачи:
- [ ] **models/task.rs**
  - Добавить реализацию `sqlx::Decode` и `sqlx::Type` для `Task`
  - Или использовать кастомный `FromRow` с ручным парсингом

#### Пример реализации:
```rust
impl<DB: Database> Type<DB> for Task 
where
    TaskStatus: Type<DB>,
    DateTime<Utc>: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        // ...
    }
}
```

---

### 🟡 ПРИОРИТЕТ 5: Job trait (1 ошибка)

**Проблема:** `LocalJob` не реализует трейт `Job`

#### Задачи:
- [ ] **services/local_job/types.rs**
  - Реализовать `impl Job for LocalJob`
  
- [ ] **services/job.rs**
  - Проверить сигнатуру метода `Job::run` (4 параметра)

---

### 🟡 ПРИОРИТЕТ 6: Clone trait (4 ошибки)

**Проблема:** `dyn Any + Send + Sync` и `dyn FnOnce` не реализуют Clone

#### Задачи:
- [ ] **local_app.rs** (строки 22, 25, 28, 51, 54, 57)
  - Убрать `#[derive(Clone)]` с `LocalAppArgs`
  - Использовать `Arc` для shared данных
  - Изменить архитектуру callback

---

### 🟢 ПРИОРИТЕТ 7: Прочие ошибки

#### SecretStorage FromRow (2 ошибки):
- [ ] **models/secret_storage.rs**
  - Добавить `#[derive(FromRow)]` или реализовать вручную

#### ExporterChain DataExporter (4 ошибки):
- [ ] **services/exporter.rs**
  - Реализовать `impl DataExporter for ExporterChain`
  
- [ ] **services/exporter_main.rs**
  - Реализовать `impl DataExporter for ExporterChain`

#### type annotations (10 ошибок):
- [ ] Различные файлы
  - Добавить явные аннотации типов

---

## 📅 Дорожная карта

### Сессия 6 (текущая)
- [ ] Исправить Git Client (6 ошибок)
- [ ] Исправить модели данных (10 ошибок)
- [ ] Исправить mismatched types часть 1 (12 ошибок)
- **Цель: < 140 ошибок**

### Сессия 7
- [ ] Исправить mismatched types часть 2 (12 ошибок)
- [ ] Исправить SQLx трейты (4 ошибки)
- [ ] Исправить Job trait (1 ошибка)
- **Цель: < 120 ошибок**

### Сессия 8
- [ ] Исправить Clone trait (4 ошибки)
- [ ] Исправить SecretStorage (2 ошибки)
- [ ] Исправить ExporterChain (4 ошибки)
- [ ] Исправить type annotations (10 ошибок)
- **Цель: < 100 ошибок**

### Сессия 9-10
- [ ] Исправить оставшиеся ошибки
- [ ] Финальная полировка
- [ ] Первая успешная сборка!
- **Цель: 0 ошибок**

---

## 📝 Заметки

### Архитектурные решения

1. **Удаление BoltDB**
   - BoltDB - Go библиотека, не имеет нативного Rust аналога
   - Sled реализация имела множество проблем
   - SQL БД полностью покрывают потребности

2. **Git Client архитектура**
   - Использовать `key_id` для загрузки ключей из хранилища
   - Не хранить ключи напрямую в Repository

3. **Job trait**
   - Требует 4 параметра в `run()` методе
   - LocalJob должен реализовать этот трейт

### Технические долги

1. **SQLx трейты** - требуют глубокой интеграции с SQLx
2. **Exporter traits** - требуют рефакторинга архитектуры
3. **Clone для dyn traits** - требует изменения архитектуры callback

### Успехи

- ✅ 71.8% ошибок исправлено
- ✅ BoltDB удалён без потери функциональности
- ✅ Конфигурация полностью исправлена
- ✅ Основные модели данных исправлены
