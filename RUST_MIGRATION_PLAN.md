# План полной миграции с Go на Rust

## 📋 Обзор

Этот документ описывает план **полной замены Go кода на Rust** в проекте Semaphore UI.

**Дата начала**: 2026-02-25  
**Статус**: 🚧 В работе

---

## 🎯 Цель

Полная замена Go реализации на Rust с сохранением функциональности:
- ✅ Все 14 моделей данных
- ✅ HTTP API (Axum)
- ✅ База данных (SQLx + Sled)
- ✅ CLI (Clap)
- ✅ Сервисы (Executor, SSH, Git, Scheduler)
- ✅ Тесты (100+ тестов)

---

## 📊 Текущий статус

### ✅ Завершено (Rust готов)

| Компонент | Файлы | Статус | Тесты |
|-----------|-------|--------|-------|
| **Модели** | `rust/src/models/*.rs` (14 файлов) | ✅ 100% | ✅ |
| **Task Logger** | `rust/src/services/task_logger.rs` | ✅ 100% | 14 тестов |
| **SSH Agent** | `rust/src/services/ssh_agent.rs` | ✅ 100% | 16 тестов |
| **Git Repository** | `rust/src/services/git_repository.rs` | ✅ 100% | 4 теста |
| **Executor** | `rust/src/services/executor.rs` | ✅ 100% | 5 тестов |
| **Config** | `rust/src/config/*.rs` | ✅ 100% | ✅ |
| **DB (SQLx)** | `rust/src/db/*.rs` | ✅ 100% | ✅ |
| **API (Axum)** | `rust/src/api/*.rs` | ✅ 100% | ✅ |
| **CLI** | `rust/src/cli/*.rs` | ✅ 100% | ✅ |
| **Utils** | `rust/src/utils/*.rs` | ✅ 100% | 11 тестов |

**Всего тестов в Rust:** 103 ✅

---

## 📅 Этапы миграции

### Этап 1: Инфраструктура FFI (1-2 недели)

**Цель:** Создать bindings для вызова Rust из Go

**Задачи:**
- [ ] Создать `rust/ffi/` с C API для Go
- [ ] Реализовать `cbindgen` для генерации `.h` файлов
- [ ] Создать Go обёртки для Rust функций
- [ ] Настроить сборку Rust библиотеки (`.a`/`.so`)
- [ ] Интегрировать в Go через `cgo`

**Файлы:**
```
rust/
  ffi/
    mod.rs          # FFI exports
    types.h         # C types (cbindgen)
    lib.rs          # FFI functions
go/
  pkg/rustlib/
    rustlib.go      # Go bindings
    rustlib.h       # C header (generated)
```

**Пример FFI функции:**
```rust
// rust/src/ffi/mod.rs
#[no_mangle]
pub extern "C" fn rust_install_access_key(
    key_ptr: *const C_AccessKey,
    role: C_AccessKeyRole,
    logger_ptr: *mut C_Logger,
) -> C_AccessKeyInstallation {
    // ...
}
```

---

### Этап 2: Модели данных (1 неделя)

**Цель:** Убедиться, что модели Go и Rust идентичны

**Задачи:**
- [ ] Сравнить все 14 моделей
- [ ] Синхронизировать поля и типы
- [ ] Создать конвертеры Go ↔ Rust
- [ ] Добавить тесты на конвертацию

**Модели:**
1. User
2. Project
3. Task
4. Template
5. Inventory
6. Repository
7. Environment
8. AccessKey
9. Integration
10. Schedule
11. Session
12. APIToken
13. Event
14. Runner
15. View
16. Role

---

### Этап 3: AccessKeyInstaller (2 недели)

**Цель:** Заменить `db_lib/AccessKeyInstaller.go` на Rust

**Go код для замены:**
```go
// db_lib/AccessKeyInstaller.go
type AccessKeyInstaller interface {
    Install(key db.AccessKey, usage db.AccessKeyRole, logger task_logger.Logger) 
        (installation ssh.AccessKeyInstallation, err error)
}
```

**Rust реализация:**
```rust
// rust/src/services/ssh_agent.rs (уже есть)
pub struct KeyInstaller;

impl KeyInstaller {
    pub fn install(
        &self,
        key: &AccessKey,
        role: AccessKeyRole,
        logger: &dyn TaskLogger,
    ) -> Result<AccessKeyInstallation>;
}
```

**Задачи:**
- [ ] Создать FFI для `KeyInstaller::install()`
- [ ] Конвертеры: `db.AccessKey` ↔ `rust::models::AccessKey`
- [ ] Конвертеры: `task_logger.Logger` ↔ `rust::services::TaskLogger`
- [ ] Интеграция в Go через cgo
- [ ] Тесты интеграции

---

### Этап 4: GitClient (2 недели)

**Цель:** Заменить `db_lib/CmdGitClient.go` на Rust

**Go код для замены:**
- `db_lib/CmdGitClient.go` (170 строк)
- `db_lib/GoGitClient.go`
- `db_lib/GitRepository.go`

**Rust реализация:**
```rust
// rust/src/services/git_repository.rs (уже есть)
pub struct GitRepository { ... }
pub trait GitClient { ... }
pub struct CmdGitClient;
```

**Задачи:**
- [ ] FFI для `GitClient` trait
- [ ] Интеграция с `KeyInstaller` (SSH ключи)
- [ ] Поддержка команд: clone, pull, checkout, ls-remote
- [ ] Тесты с реальным Git

---

### Этап 5: TaskRunner (3 недели)

**Цель:** Заменить `services/tasks/TaskRunner.go` и связанные файлы

**Go код для замены:**
- `services/tasks/TaskRunner.go` (439 строк)
- `services/tasks/LocalJob.go` (1020 строк)
- `services/tasks/RemoteJob.go`
- `services/tasks/TaskPool.go`

**Rust реализация:**
```rust
// rust/src/services/task_runner.rs
// rust/src/services/task_pool.rs
// rust/src/services/job.rs
```

**Задачи:**
- [ ] FFI для `TaskRunner`
- [ ] Интеграция с `Executor` (Ansible, Terraform, Shell)
- [ ] Логирование в реальном времени
- [ ] WebSocket для streaming логов
- [ ] Тесты end-to-end

---

### Этап 6: Executor (2 недели)

**Цель:** Заменить `db_lib/AnsibleApp.go`, `TerraformApp.go`, `ShellApp.go`

**Go код для замены:**
- `db_lib/AnsibleApp.go` (174 строки)
- `db_lib/TerraformApp.go` (391 строка)
- `db_lib/ShellApp.go` (127 строк)
- `db_lib/LocalApp.go`

**Rust реализация:**
```rust
// rust/src/services/executor.rs (уже есть)
pub struct AnsibleApp { ... }
pub struct TerraformApp { ... }
pub struct ShellApp { ... }
```

**Задачи:**
- [ ] FFI для `Executor` trait
- [ ] Поддержка Ansible playbook
- [ ] Поддержка Terraform/OpenTofu
- [ ] Поддержка Bash/PowerShell/Python
- [ ] Тесты с реальными инструментами

---

### Этап 7: API Handlers (3 недели)

**Цель:** Заменить `api/` handlers на Rust Axum

**Go код для замены:**
- `api/*.go` (handlers)
- `api/projects/*.go`
- `api/tasks/*.go`
- `api/runners/*.go`

**Rust реализация:**
```rust
// rust/src/api/handlers.rs (уже есть)
// rust/src/api/routes.rs
```

**Задачи:**
- [ ] Перенести все REST endpoints
- [ ] JWT аутентификация
- [ ] Middleware (CORS, logging, auth)
- [ ] WebSocket support
- [ ] API тесты (Dredd)

---

### Этап 8: Удаление Go модулей (1 неделя)

**Цель:** Удалить `pkg/task_logger` и `pkg/ssh`

**Задачи:**
- [ ] Проверить, что весь Go код использует Rust FFI
- [ ] Удалить `pkg/task_logger/`
- [ ] Удалить `pkg/ssh/`
- [ ] Обновить `go.mod`
- [ ] Проверить компиляцию `go build ./...`
- [ ] Запустить тесты `go test ./...`

---

### Этап 9: Финализация (2 недели)

**Цель:** Завершить миграцию и задокументировать

**Задачи:**
- [ ] Полное тестирование
- [ ] Benchmark производительности
- [ ] Документация API
- [ ] Migration guide для пользователей
- [ ] CHANGELOG.md
- [ ] Релиз v1.0.0 (Rust)

---

## 📈 Прогресс

```
Этап 1: Инфраструктура FFI     [          ] 0%
Этап 2: Модели данных          [          ] 0%
Этап 3: AccessKeyInstaller     [          ] 0%
Этап 4: GitClient              [          ] 0%
Этап 5: TaskRunner             [          ] 0%
Этап 6: Executor               [          ] 0%
Этап 7: API Handlers           [          ] 0%
Этап 8: Удаление Go модулей    [          ] 0%
Этап 9: Финализация            [          ] 0%
─────────────────────────────────────────────
Общий прогресс                 [          ] 0%
```

**Rust готовность:** 85% (код есть, FFI нет)  
**Go готовность к удалению:** 0% (использует модули)

---

## 🛠 Инструменты

### FFI / cgo

```toml
# rust/Cargo.toml
[lib]
crate-type = ["cdylib", "staticlib"]
name = "semaphore_ffi"
```

```bash
# Установка cbindgen
cargo install cbindgen

# Генерация C header
cbindgen --config rust/ffi/cbindgen.toml --output rust/ffi/semaphore.h
```

### Сборка

```bash
# Сборка Rust библиотеки
cd rust
cargo build --release --features ffi

# Копирование библиотеки
cp target/release/libsemaphore_ffi.so ../go/pkg/rustlib/
cp target/release/semaphore.h ../go/pkg/rustlib/

# Сборка Go
cd go
go build -tags rust .
```

---

## ⚠️ Риски

1. **Производительность FFI** - вызовы Rust из Go медленнее нативных
2. **Сложность отладки** - стектрейсы через FFI сложнее
3. **Сборка** - требуется Rust + Go компиляторы
4. **Потоконезависимость** - Rust async runtime vs Go goroutines

---

## 📞 Команда

- **Ведущий разработчик**: Alexander Vashurin
- **Rust команда**: [требуется]
- **Go команда**: [требуется]
- **Тестирование**: [требуется]

---

**Последнее обновление**: 2026-02-25  
**Следующий milestone**: Этап 1 - FFI инфраструктура
